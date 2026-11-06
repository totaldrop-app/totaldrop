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

    private val ownUid = 10_123
    private val media = "com.android.providers.media.module"

    private fun ok(
        scheme: String?,
        authority: String?,
        pkg: String? = media,
        uid: Int? = 10_050,
    ) = SharedFiles.acceptable(scheme, authority, pkg, uid, own, ownUid)

    @Test
    fun otherAppsContentUrisAreRead() {
        assertTrue(ok("content", "com.android.providers.media.documents"))
        assertTrue(ok("content", "media"))
        // Another app whose name merely starts like ours.
        assertTrue(ok("content", "app.totaldropper.files", "app.totaldropper"))
    }

    /** Any app can put any URI into a share intent: one naming TotalDrop's
     * own files (its device key among them) must not be sent. */
    @Test
    fun ownFilesAndFileUrisAreNot() {
        assertFalse(ok("file", ""))
        assertFalse(ok("file", null))
        assertFalse(ok("content", null))
        assertFalse(ok("CONTENT", "media"))
        assertFalse(ok("android.resource", "com.x"))
        // Served by TotalDrop itself, whatever the authority is called.
        assertFalse(ok("content", "$own.flutter.share_provider", own, ownUid))
        assertFalse(ok("content", "$own.fileprovider", own, ownUid))
        assertFalse(ok("content", "dev.fluttercommunity.plus.share", own, ownUid))
        // Same process, another package name (a shared UID).
        assertFalse(ok("content", "x.files", "x", ownUid))
        // Nothing serves it: nothing to read.
        assertFalse(ok("content", "gone.provider", null, null))
    }

    /** content://0@authority reaches the provider of authority for user 0:
     * the prefix must not hide TotalDrop's own provider. */
    @Test
    fun aUserPrefixIsRefused() {
        assertFalse(ok("content", "0@$own.fileprovider"))
        assertFalse(ok("content", "0@media"))
        assertFalse(ok("content", "@media"))
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
    fun numbersNeverMakeANameTooLong() {
        val long = SharedFiles.safeName("a".repeat(400) + ".txt", 0)
        val names = SharedFiles.uniqueNames(listOf(long, long, long))
        assertEquals(3, names.toSet().size)
        for (n in names) {
            assertTrue(n.endsWith(".txt"))
            assertTrue(n.toByteArray(Charsets.UTF_8).size <= SharedFiles.MAX_NAME_BYTES)
        }
        assertTrue(names[1].endsWith(" (2).txt"))
    }

    @Test
    fun sameNamesGetNumbers() {
        assertEquals(
            listOf("a.jpg", "a (2).jpg", "A (3).JPG", "b", "b (2)", "a (2) (2).jpg"),
            SharedFiles.uniqueNames(listOf("a.jpg", "a.jpg", "A.JPG", "b", "b", "a (2).jpg")),
        )
    }
}
