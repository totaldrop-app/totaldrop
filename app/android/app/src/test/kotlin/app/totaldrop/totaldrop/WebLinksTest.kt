// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

package app.totaldrop.totaldrop

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

/** Kotlin-side defense-in-depth checks before opening a browser URL (#53). */
class WebLinksTest {
    @Test
    fun acceptsNormalizedHttpAndHttpsStrings() {
        assertTrue(WebLinks.isSafeWebUrlString("https://totaldrop.app/docs"))
        assertTrue(WebLinks.isSafeWebUrlString("http://192.168.1.10:8080/"))
        assertEquals(
            "https://",
            WebLinks.browserSelectorUri("https://totaldrop.app/r#7-harbor-rocket"),
        )
        assertEquals(
            "http://",
            WebLinks.browserSelectorUri("http://192.168.1.10:8080/"),
        )
    }

    @Test
    fun rejectsNonWebSchemesWhitespaceControlsAndOversizedStrings() {
        assertFalse(WebLinks.isSafeWebUrlString(null))
        assertFalse(WebLinks.isSafeWebUrlString(""))
        assertFalse(WebLinks.isSafeWebUrlString("javascript:alert(1)"))
        assertFalse(WebLinks.isSafeWebUrlString("intent://scan/#Intent;end"))
        assertFalse(WebLinks.isSafeWebUrlString("file:///etc/passwd"))
        assertFalse(WebLinks.isSafeWebUrlString("HTTPS://totaldrop.app/"))
        assertFalse(WebLinks.isSafeWebUrlString("https://a.com https://b.com"))
        assertFalse(WebLinks.isSafeWebUrlString("https://a.com\n"))
        assertFalse(WebLinks.isSafeWebUrlString("https://a.com\\@evil.com"))
        assertFalse(WebLinks.isSafeWebUrlString("https://example.com/" + "a".repeat(2040)))
        assertNull(WebLinks.browserSelectorUri("intent://scan/#Intent;end"))
    }
}
