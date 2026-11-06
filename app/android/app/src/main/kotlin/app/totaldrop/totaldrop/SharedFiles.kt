// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

package app.totaldrop.totaldrop

import android.content.ContentResolver
import android.content.Intent
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Process
import android.provider.OpenableColumns
import java.io.File
import java.io.IOException
import java.io.InputStream
import java.io.OutputStream
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.Executors

/**
 * Files shared to TotalDrop from another app (#53, #308): Gallery, Files,
 * any app with a Share button.
 *
 * Another app hands over `content://` URIs with a read grant that lasts as
 * long as this activity. The core reads files by path, so each one is copied
 * first, as the system picker's plugin does for picked files (#269). The
 * grant is never made persistent.
 *
 * Only `content://` URIs served by another app are read. Any app can put any
 * URI into a share intent; a `file://` URI, or a content URI of a provider in
 * TotalDrop's own process (share_plus has one), could name TotalDrop's
 * private files, its device key among them, and sending them would leak
 * them. The provider is looked up the way the system resolves it, so a
 * `user@` prefix in the authority does not hide it.
 */
object SharedFiles {
    /** A copied file, ready for the core. */
    data class Item(val name: String, val path: String, val size: Long)

    /** What a share became: its files, and how many of its items were left
     * out because they may not or cannot be read. */
    data class Copied(val items: List<Item>, val skipped: Int)

    /** Longest file name a copy keeps, in UTF-8 bytes (ext4's limit). */
    const val MAX_NAME_BYTES = 255

    /** At most this many files from one share; the rest count as
     * skipped. */
    const val MAX_FILES = 10_000

    /** Room left on the disk after the copy, so the phone does not fill up. */
    const val SPARE_BYTES = 256L shl 20

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
        return out.toList()
    }

    /** Whether a share intent carries text and no file. */
    fun isTextOnly(intent: Intent, uris: List<Uri>): Boolean =
        uris.isEmpty() && intent.action == Intent.ACTION_SEND &&
            !intent.getCharSequenceExtra(Intent.EXTRA_TEXT).isNullOrEmpty()

    /**
     * Whether a URI may be read for sending: see the class comment.
     * [providerPackage] and [providerUid] are what the system resolves the
     * authority to, null when nothing serves it.
     */
    fun acceptable(
        scheme: String?,
        authority: String?,
        providerPackage: String?,
        providerUid: Int?,
        ownPackage: String,
        ownUid: Int,
    ): Boolean =
        scheme == ContentResolver.SCHEME_CONTENT &&
            !authority.isNullOrEmpty() &&
            // content://0@authority/… reaches the same provider.
            '@' !in authority &&
            providerPackage != null && providerUid != null &&
            providerPackage != ownPackage &&
            providerUid != ownUid

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
        return fit(name, "").ifEmpty { fallback }
    }

    /** [name] with [suffix] before its extension, cut to [MAX_NAME_BYTES]. */
    private fun fit(name: String, suffix: String): String {
        val dot = name.lastIndexOf('.')
        val ext = if (dot > 0 && name.length - dot <= 16) name.substring(dot) else ""
        var stem = name.substring(0, name.length - ext.length)
        fun bytes() = (stem + suffix + ext).toByteArray(Charsets.UTF_8).size
        while (bytes() > MAX_NAME_BYTES && stem.isNotEmpty()) {
            stem = stem.dropLast(if (stem.length > 1 && Character.isLowSurrogate(stem.last())) 2 else 1)
        }
        return stem + suffix + ext
    }

    /**
     * Names for a share's files, made unique the way a file manager does:
     * `a.jpg`, `a (2).jpg`. Case is ignored, since the receiver may be on
     * a file system that ignores it. A number never makes a name too long.
     */
    fun uniqueNames(names: List<String>): List<String> {
        val taken = HashSet<String>()
        // The next number to try for a name, so many copies of one name
        // do not try every number again.
        val next = HashMap<String, Int>()
        return names.map { name ->
            var candidate = name
            val key = name.lowercase()
            var n = next[key] ?: 2
            while (!taken.add(candidate.lowercase())) {
                candidate = fit(name, " ($n)")
                n += 1
            }
            if (candidate != name) next[key] = n
            candidate
        }
    }

    /** Copying stopped: the share was dropped, or the disk is full. */
    class Stopped(val noSpace: Boolean) : IOException(if (noSpace) "no space" else "dropped")

    /**
     * Copies [input] to [output] in blocks of [buffer]. Before each block it
     * asks [dropped] whether to stop, and [spaceLeft] how much room the disk
     * has: below [SPARE_BYTES] it stops, so a size a provider got wrong or
     * did not say cannot fill the phone. Returns the bytes copied.
     */
    fun copyStream(
        input: InputStream,
        output: OutputStream,
        buffer: ByteArray,
        dropped: () -> Boolean,
        spaceLeft: () -> Long,
    ): Long {
        var total = 0L
        while (true) {
            if (dropped()) throw Stopped(noSpace = false)
            if (spaceLeft() < SPARE_BYTES + buffer.size) throw Stopped(noSpace = true)
            val n = input.read(buffer)
            if (n < 0) return total
            output.write(buffer, 0, n)
            total += n
        }
    }
}

/**
 * Copies shares into their own folders under [root], one at a time, and
 * deletes them: every copy and delete runs on one thread, so a delete never
 * meets a copy half way. A dropped share stops copying at the next block;
 * its open stream is closed too, so a provider that stops sending cannot
 * hold the thread.
 */
class ShareCopier(
    private val root: File,
    private val resolver: ContentResolver,
    private val packages: PackageManager,
    private val ownPackage: String,
) {
    private val worker = Executors.newSingleThreadExecutor { r -> Thread(r, "share-copy") }
    private val dropped = ConcurrentHashMap.newKeySet<Long>()
    // The stream each share is reading now, closed from outside to end a
    // read that waits.
    private val reading = ConcurrentHashMap<Long, InputStream>()

    /** Copies [uris] as share [id]; [done] gets the result, or null when
     * nothing could be copied, on the worker. A dropped share gets no
     * result: no one waits for it. */
    fun copy(id: Long, uris: List<Uri>, done: (SharedFiles.Copied?) -> Unit) {
        worker.execute {
            val dir = File(root, id.toString())
            val result = try {
                copyNow(id, uris, dir)
            } catch (e: Exception) {
                android.util.Log.w("TotalDrop", "shared files could not be copied: ${e.javaClass.simpleName}")
                dir.deleteRecursively()
                null
            }
            if (id in dropped) {
                dir.deleteRecursively()
            } else {
                done(result)
            }
        }
    }

    /** The person cannot take these shares, or cleared them: stop and
     * delete their copies. */
    fun drop(ids: Collection<Long>) {
        if (ids.isEmpty()) return
        dropped.addAll(ids)
        for (id in ids) reading[id]?.let(::closeQuietly)
        worker.execute {
            for (id in ids) File(root, id.toString()).deleteRecursively()
        }
    }

    /** Deletes every copy, left from an earlier run; one still copying
     * stops. */
    fun clearAll() {
        for ((id, stream) in reading) {
            dropped.add(id)
            closeQuietly(stream)
        }
        worker.execute { root.deleteRecursively() }
    }

    private fun closeQuietly(stream: InputStream) {
        try {
            stream.close()
        } catch (_: Exception) {
            // Already closed, or the provider is gone: either way it ends.
        }
    }

    private fun readable(uri: Uri): Boolean {
        val authority = uri.authority
        val info = authority?.takeIf { '@' !in it }?.let {
            try {
                packages.resolveContentProvider(it, PackageManager.ComponentInfoFlags.of(0))
            } catch (_: Exception) {
                null
            }
        }
        return SharedFiles.acceptable(
            uri.scheme, authority, info?.packageName, info?.applicationInfo?.uid,
            ownPackage, Process.myUid(),
        )
    }

    private fun copyNow(id: Long, uris: List<Uri>, dir: File): SharedFiles.Copied? {
        if (id in dropped) return null
        val usable = uris.take(SharedFiles.MAX_FILES).filter(::readable)
        if (usable.isEmpty()) return null
        val meta = usable.mapIndexed { i, uri -> query(uri, i) }
        val names = SharedFiles.uniqueNames(meta.map { it.first })
        // A folder of the same number from an earlier run goes first.
        dir.deleteRecursively()
        if (!dir.mkdirs()) throw IOException("cannot create the share folder")
        // Known sizes must fit with room to spare; an unknown size is
        // found out while copying, and a full disk fails the copy.
        val known = meta.sumOf { it.second ?: 0L }
        if (known > dir.usableSpace - SharedFiles.SPARE_BYTES) throw IOException("no space")
        val items = ArrayList<SharedFiles.Item>()
        var skipped = uris.size - usable.size
        val buffer = ByteArray(1 shl 20)
        for ((uri, name) in usable.zip(names)) {
            val target = File(dir, name)
            val input = try {
                resolver.openInputStream(uri)
            } catch (_: Exception) {
                null
            }
            if (input == null) {
                skipped += 1
                continue
            }
            reading[id] = input
            try {
                input.use { i ->
                    target.outputStream().use { o ->
                        SharedFiles.copyStream(i, o, buffer, { id in dropped }, { dir.usableSpace })
                    }
                }
            } catch (e: SharedFiles.Stopped) {
                // Dropped, or the disk is full: the whole share ends.
                throw e
            } catch (e: Exception) {
                // This file could not be read to the end: it is left out.
                if (id in dropped) throw e
                target.delete()
                skipped += 1
                continue
            } finally {
                reading.remove(id)
            }
            items.add(SharedFiles.Item(name, target.path, target.length()))
        }
        if (items.isEmpty()) {
            dir.deleteRecursively()
            return null
        }
        return SharedFiles.Copied(items, skipped)
    }

    /** The display name (made safe) and the size, if the provider says. */
    private fun query(uri: Uri, index: Int): Pair<String, Long?> {
        var name: String? = null
        var size: Long? = null
        try {
            resolver.query(uri, arrayOf(OpenableColumns.DISPLAY_NAME, OpenableColumns.SIZE), null, null, null)
                ?.use { c ->
                    if (c.moveToFirst()) {
                        if (!c.isNull(0)) name = c.getString(0)
                        if (!c.isNull(1)) size = c.getLong(1).takeIf { it >= 0 }
                    }
                }
        } catch (_: Exception) {
            // Name and size stay unknown; the copy finds out the size.
        }
        return SharedFiles.safeName(name ?: uri.lastPathSegment, index) to size
    }
}
