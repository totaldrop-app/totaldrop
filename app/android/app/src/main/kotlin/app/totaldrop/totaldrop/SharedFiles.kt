// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

package app.totaldrop.totaldrop

import android.content.ContentResolver
import android.content.Intent
import android.net.Uri
import android.provider.OpenableColumns
import java.io.File
import java.io.IOException

/**
 * Files shared to TotalDrop from another app (#53): Gallery, Files, any app
 * with a Share button.
 *
 * Another app hands over `content://` URIs with a read grant that lasts as
 * long as this activity. The core reads files by path, so each one is copied
 * into the app cache first, as the system picker's plugin does for picked
 * files (#269). The grant is never made persistent.
 *
 * Only `content://` URIs from other apps are taken. A `file://` URI, or a
 * content URI of this app's own provider, could name TotalDrop's private
 * files (its device key among them), and sending them would leak them: any
 * app can put any URI into a share intent.
 */
object SharedFiles {
    /** A copied file, ready for the core. */
    data class Item(val name: String, val path: String, val size: Long)

    /** Longest file name the copy keeps, in UTF-8 bytes (ext4's limit). */
    const val MAX_NAME_BYTES = 255

    /** At most this many files from one share. */
    const val MAX_FILES = 10_000

    /** The streams of a share intent, in order, each once. */
    fun urisOf(intent: Intent): List<Uri> {
        val out = LinkedHashSet<Uri>()
        when (intent.action) {
            Intent.ACTION_SEND ->
                intent.getParcelableExtra(Intent.EXTRA_STREAM, Uri::class.java)?.let(out::add)
            Intent.ACTION_SEND_MULTIPLE ->
                intent.getParcelableArrayListExtra(Intent.EXTRA_STREAM, Uri::class.java)
                    ?.filterNotNull()?.let(out::addAll)
            else -> return emptyList()
        }
        // Some apps put the files only in ClipData, which carries the grant.
        intent.clipData?.let { clip ->
            for (i in 0 until clip.itemCount) clip.getItemAt(i).uri?.let(out::add)
        }
        return out.take(MAX_FILES)
    }

    /** Whether a share intent carries text and no file. */
    fun isTextOnly(intent: Intent, uris: List<Uri>): Boolean =
        uris.isEmpty() && intent.action == Intent.ACTION_SEND &&
            !intent.getCharSequenceExtra(Intent.EXTRA_TEXT).isNullOrEmpty()

    /** Whether a URI may be read for sending: see the class comment. */
    fun acceptable(scheme: String?, authority: String?, ownAuthorityPrefix: String): Boolean =
        scheme == ContentResolver.SCHEME_CONTENT &&
            !authority.isNullOrEmpty() &&
            authority != ownAuthorityPrefix &&
            !authority.startsWith("$ownAuthorityPrefix.")

    /**
     * A file name that is safe to create in a folder: no folder separators,
     * no control or hidden format characters, not `.` or `..`, not hidden,
     * at most [MAX_NAME_BYTES] UTF-8 bytes with the extension kept. The
     * receiver checks names again (path rules of the protocol); this only
     * keeps the copy inside its folder.
     */
    fun safeName(displayName: String?, index: Int): String {
        val fallback = "file-${index + 1}"
        var name = (displayName ?: "").substringAfterLast('/').substringAfterLast('\\')
        name = buildString {
            name.codePoints().forEach { cp ->
                val type = Character.getType(cp)
                val hidden = type == Character.CONTROL.toInt() ||
                    type == Character.FORMAT.toInt() ||
                    type == Character.LINE_SEPARATOR.toInt() ||
                    type == Character.PARAGRAPH_SEPARATOR.toInt()
                if (!hidden) appendCodePoint(cp)
            }
        }.trim().trimStart('.')
        if (name.isEmpty()) return fallback
        if (name.toByteArray(Charsets.UTF_8).size <= MAX_NAME_BYTES) return name
        val dot = name.lastIndexOf('.')
        val ext = if (dot > 0 && name.length - dot <= 16) name.substring(dot) else ""
        var stem = name.substring(0, name.length - ext.length)
        while ((stem + ext).toByteArray(Charsets.UTF_8).size > MAX_NAME_BYTES && stem.isNotEmpty()) {
            stem = stem.dropLast(if (Character.isLowSurrogate(stem.last())) 2 else 1)
        }
        return (stem + ext).ifEmpty { fallback }
    }

    /**
     * Names for a share's files, made unique the way a file manager does:
     * `a.jpg`, `a (2).jpg`. Case is ignored, since the receiver may be on
     * a file system that ignores it.
     */
    fun uniqueNames(names: List<String>): List<String> {
        val taken = HashSet<String>()
        return names.map { name ->
            var candidate = name
            var n = 2
            while (!taken.add(candidate.lowercase())) {
                val dot = name.lastIndexOf('.')
                candidate = if (dot > 0) {
                    "${name.substring(0, dot)} ($n)${name.substring(dot)}"
                } else {
                    "$name ($n)"
                }
                n += 1
            }
            candidate
        }
    }

    /** The folder the copies go into. Emptied by [clear]. */
    fun folder(cacheDir: File): File = File(cacheDir, "shared")

    /**
     * Copies the shared files into a fresh folder under [folder]. Throws
     * [IOException] (or [SecurityException], for a grant that is gone)
     * when a file cannot be read or written; nothing is kept then.
     */
    fun copy(resolver: ContentResolver, uris: List<Uri>, cacheDir: File, ownAuthorityPrefix: String): List<Item> {
        val usable = uris.filter { acceptable(it.scheme, it.authority, ownAuthorityPrefix) }
        if (usable.isEmpty()) throw IOException("no file that can be read")
        val names = uniqueNames(usable.mapIndexed { i, uri -> safeName(displayName(resolver, uri), i) })
        val dir = File(folder(cacheDir), System.nanoTime().toString())
        if (!dir.mkdirs()) throw IOException("cannot create the share folder")
        try {
            return usable.zip(names).map { (uri, name) ->
                val target = File(dir, name)
                val input = resolver.openInputStream(uri) ?: throw IOException("no stream")
                input.use { i -> target.outputStream().use { o -> i.copyTo(o, 1 shl 20) } }
                Item(name, target.path, target.length())
            }
        } catch (e: Exception) {
            dir.deleteRecursively()
            throw e
        }
    }

    /** Deletes every copy. */
    fun clear(cacheDir: File) {
        folder(cacheDir).deleteRecursively()
    }

    private fun displayName(resolver: ContentResolver, uri: Uri): String? = try {
        resolver.query(uri, arrayOf(OpenableColumns.DISPLAY_NAME), null, null, null)?.use { c ->
            if (c.moveToFirst() && !c.isNull(0)) c.getString(0) else null
        }
    } catch (_: Exception) {
        null
    } ?: uri.lastPathSegment
}
