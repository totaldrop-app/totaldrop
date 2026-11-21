// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

package app.totaldrop.totaldrop

import java.nio.file.Files
import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Test

class DownloadsStorageTest {
    @Test
    fun scannablePathsCollectsFilesAndExpandsDirectories() {
        val root = Files.createTempDirectory("totaldrop-scan-test").toFile()
        try {
            val single = root.resolve("photo.jpg").apply { writeText("img") }
            val folder = root.resolve("album").apply { mkdirs() }
            val nestedA = folder.resolve("a.png").apply { writeText("a") }
            val nestedB = folder.resolve("sub").apply { mkdirs() }
                .resolve("b.mp4").apply { writeText("b") }
            val missing = root.resolve("missing.pdf")

            val result = DownloadsStorage.scannablePaths(
                listOf(
                    single.path,
                    single.path, // duplicate ignored
                    folder.path,
                    missing.path,
                    "relative/path.txt",
                    "   ",
                    null,
                    42,
                ),
            )

            assertEquals(
                setOf(single.path, nestedA.path, nestedB.path),
                result.toSet(),
            )
            assertEquals(3, result.size)
        } finally {
            root.deleteRecursively()
        }
    }

    @Test
    fun scannablePathsHonorsLimitAndEmptyInput() {
        assertArrayEquals(emptyArray<String>(), DownloadsStorage.scannablePaths(null))
        assertArrayEquals(emptyArray<String>(), DownloadsStorage.scannablePaths(emptyList<String>()))

        val root = Files.createTempDirectory("totaldrop-scan-limit").toFile()
        try {
            val files = (1..5).map { idx ->
                root.resolve("f$idx.txt").apply { writeText("$idx") }.path
            }
            val capped = DownloadsStorage.scannablePaths(files, limit = 2)
            assertEquals(2, capped.size)
        } finally {
            root.deleteRecursively()
        }
    }
}
