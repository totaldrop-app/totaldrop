// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

package app.totaldrop.totaldrop

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/** Which shared URIs are read, and the names their copies get (#53). */
class SharedFilesTest {
    private val own = "app.totaldrop"

    @Test
    fun otherAppsContentUrisAreRead() {
        assertTrue(SharedFiles.acceptable("content", "com.android.providers.media.documents", own))
        assertTrue(SharedFiles.acceptable("content", "media", own))
        // Another app whose name merely starts like ours.
        assertTrue(SharedFiles.acceptable("content", "app.totaldropper.files", own))
    }

    /** Any app can put any URI into a share intent: one naming TotalDrop's
     * own files (its device key among them) must not be sent. */
    @Test
    fun ownFilesAndFileUrisAreNot() {
        assertFalse(SharedFiles.acceptable("file", "", own))
        assertFalse(SharedFiles.acceptable("file", null, own))
        assertFalse(SharedFiles.acceptable("content", own, own))
        assertFalse(SharedFiles.acceptable("content", "$own.flutter.share_provider", own))
        assertFalse(SharedFiles.acceptable("content", "$own.fileprovider", own))
        assertFalse(SharedFiles.acceptable("content", null, own))
        assertFalse(SharedFiles.acceptable("CONTENT", "media", own))
        assertFalse(SharedFiles.acceptable("android.resource", "com.x", own))
    }

    @Test
    fun namesStayInsideTheirFolder() {
        assertEquals("passwd", SharedFiles.safeName("../../etc/passwd", 0))
        assertEquals("b.txt", SharedFiles.safeName("a\\b.txt", 0))
        assertEquals("file-1", SharedFiles.safeName("..", 0))
        assertEquals("file-3", SharedFiles.safeName("/", 2))
        assertEquals("file-1", SharedFiles.safeName(null, 0))
        assertEquals("file-2", SharedFiles.safeName("   ", 1))
        // A hidden name would hide the copy, and the received file.
        assertEquals("bashrc", SharedFiles.safeName(".bashrc", 0))
    }

    @Test
    fun hiddenCharactersAreDropped() {
        assertEquals("ab.txt", SharedFiles.safeName("a\u0000b.txt", 0))
        assertEquals("line.txt", SharedFiles.safeName("li\nne.txt", 0))
        // Right-to-left override: "photo‮gpj.exe" would read as photoexe.jpg.
        assertEquals("photogpj.exe", SharedFiles.safeName("photo‮gpj.exe", 0))
        assertEquals("çiçek 🌸.jpg", SharedFiles.safeName("çiçek 🌸.jpg", 0))
    }

    @Test
    fun longNamesAreCutKeepingTheExtension() {
        val name = SharedFiles.safeName("ğ".repeat(300) + ".jpeg", 0)
        assertTrue(name.endsWith(".jpeg"))
        assertTrue(name.toByteArray(Charsets.UTF_8).size <= SharedFiles.MAX_NAME_BYTES)
        // No half of a surrogate pair is left at the cut.
        val emoji = SharedFiles.safeName("😀".repeat(100), 0)
        assertTrue(emoji.toByteArray(Charsets.UTF_8).size <= SharedFiles.MAX_NAME_BYTES)
        assertEquals(emoji, String(emoji.toByteArray(Charsets.UTF_8), Charsets.UTF_8))
    }

    @Test
    fun sameNamesGetNumbers() {
        assertEquals(
            listOf("a.jpg", "a (2).jpg", "A (3).JPG", "b", "b (2)", "a (2) (2).jpg"),
            SharedFiles.uniqueNames(listOf("a.jpg", "a.jpg", "A.JPG", "b", "b", "a (2).jpg")),
        )
    }
}
