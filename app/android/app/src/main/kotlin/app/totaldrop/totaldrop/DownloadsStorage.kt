// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

package app.totaldrop.totaldrop

import android.content.Context
import android.media.MediaScannerConnection
import android.os.Environment
import java.io.File
import java.util.concurrent.Executors

/**
 * Public `Downloads` path lookup and `MediaStore` indexing for received files
 * (#592).
 *
 * Secondary users and Android Work Profiles mount shared storage under
 * `/storage/emulated/<userId>/Download` rather than user `0`. Querying
 * [Environment.getExternalStoragePublicDirectory] returns the active user's
 * public `Downloads` directory, and [scanSaved] notifies `MediaStore` off the
 * UI thread so files written by the Rust core appear immediately in Gallery
 * and the system Files app's Recent view.
 */
object DownloadsStorage {
    private val scanner = Executors.newSingleThreadExecutor { runnable ->
        Thread(runnable, "totaldrop-media-scan").apply { isDaemon = true }
    }

    /** The active Android user's public `Downloads` folder, or null if unavailable. */
    fun publicDownloadsDir(): String? =
        Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_DOWNLOADS)
            ?.path
            ?.takeIf { it.isNotEmpty() }

    /**
     * Collects existing regular file paths from [paths] (expanding received
     * directories up to [limit] files) so `MediaScannerConnection` receives
     * concrete file paths.
     */
    internal fun scannablePaths(
        paths: List<*>?,
        limit: Int = SharedFiles.MAX_FILES,
    ): Array<String> {
        if (paths.isNullOrEmpty() || limit <= 0) return emptyArray()
        val out = LinkedHashSet<String>()
        for (raw in paths) {
            if (out.size >= limit) break
            val p = (raw as? String)?.trim().orEmpty()
            if (p.isEmpty()) continue
            val file = File(p)
            if (!file.isAbsolute || !file.exists()) continue
            if (file.isFile) {
                out.add(file.path)
            } else if (file.isDirectory) {
                for (child in file.walkTopDown()) {
                    if (out.size >= limit) break
                    if (child.isFile) out.add(child.path)
                }
            }
        }
        return out.toTypedArray()
    }

    /** Indexes [paths] in `MediaStore` on a background thread. */
    fun scanSaved(context: Context, paths: List<*>?) {
        if (paths.isNullOrEmpty()) return
        val app = context.applicationContext
        val snapshot = paths.toList()
        scanner.execute {
            val files = scannablePaths(snapshot)
            if (files.isNotEmpty()) {
                MediaScannerConnection.scanFile(app, files, null, null)
            }
        }
    }
}
