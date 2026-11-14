// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

package app.totaldrop.totaldrop

import android.app.PendingIntent
import android.content.Intent
import android.service.quicksettings.Tile
import android.service.quicksettings.TileService
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test
import org.w3c.dom.Element
import java.io.File
import javax.xml.parsers.DocumentBuilderFactory

/** Quick Settings tile target, state, and manifest registration (#53). */
class ReceiveTileServiceTest {
    @Test
    fun tileStaysInActionIdleState() {
        assertEquals(Tile.STATE_INACTIVE, ReceiveTileService.IDLE_STATE)
    }

    @Test
    fun tapTargetsMainActivityReceiveShortcut() {
        val target = ReceiveTileService.targetFor("app.totaldrop")
        assertEquals(MainActivity.ACTION_SHORTCUT_RECEIVE, target.action)
        assertEquals("app.totaldrop", target.packageName)
        assertEquals("app.totaldrop.totaldrop.MainActivity", target.className)
        assertTrue(target.flags and Intent.FLAG_ACTIVITY_NEW_TASK != 0)
        assertTrue(target.flags and Intent.FLAG_ACTIVITY_CLEAR_TOP != 0)
        assertTrue(target.pendingFlags and PendingIntent.FLAG_IMMUTABLE != 0)
        assertTrue(target.pendingFlags and PendingIntent.FLAG_UPDATE_CURRENT != 0)
    }

    @Test
    fun manifestProtectsTileWithBindQuickSettingsPermission() {
        val androidNs = "http://schemas.android.com/apk/res/android"
        val appDir = listOf(File("."), File("app")).firstOrNull {
            File(it, "src/main/AndroidManifest.xml").isFile
        }
        assertNotNull("Could not locate src/main/AndroidManifest.xml", appDir)
        val manifest = File(appDir!!, "src/main/AndroidManifest.xml")

        val factory = DocumentBuilderFactory.newInstance().apply { isNamespaceAware = true }
        val doc = factory.newDocumentBuilder().parse(manifest)
        val services = doc.getElementsByTagName("service")
        var tileService: Element? = null
        for (i in 0 until services.length) {
            val el = services.item(i) as Element
            if (el.getAttributeNS(androidNs, "name") == ".ReceiveTileService") {
                tileService = el
                break
            }
        }
        assertNotNull(".ReceiveTileService missing from AndroidManifest.xml", tileService)
        val service = tileService!!
        assertEquals("true", service.getAttributeNS(androidNs, "exported"))
        assertEquals(
            "android.permission.BIND_QUICK_SETTINGS_TILE",
            service.getAttributeNS(androidNs, "permission"),
        )
        assertEquals("@drawable/ic_tile_receive", service.getAttributeNS(androidNs, "icon"))
        assertEquals("@string/tile_receive", service.getAttributeNS(androidNs, "label"))

        val actions = service.getElementsByTagName("action")
        assertEquals(1, actions.length)
        assertEquals(
            TileService.ACTION_QS_TILE,
            (actions.item(0) as Element).getAttributeNS(androidNs, "name"),
        )

        assertTrue(File(appDir, "src/main/res/drawable/ic_tile_receive.xml").isFile)
        val defaultStrings = File(appDir, "src/main/res/values/l10n.xml").readText()
        assertTrue(defaultStrings.contains("<string name=\"tile_receive\">"))
    }
}
