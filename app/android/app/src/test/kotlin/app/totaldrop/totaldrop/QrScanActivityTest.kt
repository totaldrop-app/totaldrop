// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

package app.totaldrop.totaldrop

import android.content.pm.PackageManager
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import org.w3c.dom.Element
import java.io.File
import javax.xml.parsers.DocumentBuilderFactory

/** JVM unit tests for QR scanner candidate filtering and manifest safety (#29). */
class QrScanActivityTest {
    @Test
    fun picksFirstValidReceiveLinkAndSkipsUnrelatedQrPayloads() {
        val chosen = QrScanActivity.firstQrReceiveLink(
            listOf(
                null,
                "",
                "WIFI:S:Home;T:WPA;P:secret;;",
                "https://example.com/r#7-harbor-rocket",
                "http://totaldrop.app/r#7-harbor-rocket",
                "7-harbor-rocket",
                " \u00AD\u2068https://totaldrop.app/r#7-harbor-rocket\u2069 ",
                "totaldrop://r/8-other-code",
            ),
        )
        assertEquals("https://totaldrop.app/r#7-harbor-rocket", chosen)
    }

    @Test
    fun acceptsCaseInsensitiveTotalDropHttpsAndCustomSchemeLinks() {
        assertEquals(
            "totaldrop://r/7-harbor-rocket",
            QrScanActivity.firstQrReceiveLink(listOf("totaldrop://r/7-harbor-rocket")),
        )
        assertEquals(
            "https://totaldrop.app/r/#7-harbor-rocket",
            QrScanActivity.firstQrReceiveLink(listOf("https://totaldrop.app/r/#7-harbor-rocket")),
        )
        assertEquals(
            "HTTPS://TOTALDROP.APP/R#7-HARBOR-ROCKET",
            QrScanActivity.firstQrReceiveLink(listOf("HTTPS://TOTALDROP.APP/R#7-HARBOR-ROCKET")),
        )
        assertNull(QrScanActivity.firstQrReceiveLink(listOf("https://totaldrop.app/r#")))
        assertNull(QrScanActivity.firstQrReceiveLink(listOf("https://totaldrop.app/r/#")))
        assertNull(QrScanActivity.firstQrReceiveLink(listOf("totaldrop://r/")))
        assertFalse(QrScanActivity.hasQrReceivePrefix("totaldrop:7-harbor-rocket"))
        assertNull(
            QrScanActivity.firstQrReceiveLink(
                listOf("https://totaldrop.app/r#" + "a".repeat(1100)),
            ),
        )
        // 400 three-byte UTF-8 chars (< 1024 UTF-16 chars, > 1024 UTF-8 bytes).
        assertNull(
            QrScanActivity.firstQrReceiveLink(
                listOf("https://totaldrop.app/r#" + "あ".repeat(400)),
            ),
        )
    }

    @Test
    fun classifiesPermissionResultsProperly() {
        assertEquals(
            QrScanActivity.OUTCOME_CANCELLED,
            QrScanActivity.permissionOutcome(intArrayOf()),
        )
        assertNull(
            QrScanActivity.permissionOutcome(intArrayOf(PackageManager.PERMISSION_GRANTED)),
        )
        assertEquals(
            QrScanActivity.OUTCOME_DENIED,
            QrScanActivity.permissionOutcome(intArrayOf(PackageManager.PERMISSION_DENIED)),
        )
    }

    @Test
    fun invisibleMarkSetMatchesRustProtocolTable() {
        val protoCandidates = listOf(
            File("../../crates/totaldrop-proto/src/lib.rs"),
            File("../../../crates/totaldrop-proto/src/lib.rs"),
        )
        val protoFile = protoCandidates.firstOrNull { it.isFile }
        assertNotNull("Could not locate crates/totaldrop-proto/src/lib.rs", protoFile)
        val block = Regex(
            """pub const INVISIBLE_FORMAT: &\s*\[\(u32, u32\)\] = &\[(.*?)];""",
            RegexOption.DOT_MATCHES_ALL,
        ).find(protoFile!!.readText())
        assertNotNull("INVISIBLE_FORMAT table missing from lib.rs", block)

        val expected = mutableSetOf<Int>()
        val pairRegex = Regex("""\(0x([0-9A-Fa-f]+),\s*0x([0-9A-Fa-f]+)\)""")
        for (match in pairRegex.findAll(block!!.groupValues[1])) {
            val start = match.groupValues[1].toInt(16)
            val end = match.groupValues[2].toInt(16)
            for (cp in start..end) expected.add(cp)
        }
        assertFalse(expected.isEmpty())

        val actual = mutableSetOf<Int>()
        for (cp in 0..0xFFFF) {
            if (QrScanActivity.isInvisibleMark(cp.toChar())) actual.add(cp)
        }
        assertEquals(expected, actual)
    }

    @Test
    fun manifestKeepsCameraOptionalSupportsRtlAndLeavesQrScanActivityUnexported() {
        val androidNs = "http://schemas.android.com/apk/res/android"
        val appDir = listOf(File("."), File("app")).firstOrNull {
            File(it, "src/main/AndroidManifest.xml").isFile
        }
        assertNotNull("Could not locate src/main/AndroidManifest.xml", appDir)
        val manifest = File(appDir!!, "src/main/AndroidManifest.xml")

        val factory = DocumentBuilderFactory.newInstance().apply { isNamespaceAware = true }
        val doc = factory.newDocumentBuilder().parse(manifest)

        val appElement = doc.getElementsByTagName("application").item(0) as Element
        assertEquals("true", appElement.getAttributeNS(androidNs, "supportsRtl"))

        val permissions = doc.getElementsByTagName("uses-permission")
        var hasCameraPerm = false
        for (i in 0 until permissions.length) {
            val el = permissions.item(i) as Element
            if (el.getAttributeNS(androidNs, "name") == "android.permission.CAMERA") {
                hasCameraPerm = true
            }
        }
        assertTrue("CAMERA permission missing from AndroidManifest.xml", hasCameraPerm)

        val features = doc.getElementsByTagName("uses-feature")
        val featureRequired = mutableMapOf<String, String>()
        for (i in 0 until features.length) {
            val el = features.item(i) as Element
            featureRequired[el.getAttributeNS(androidNs, "name")] =
                el.getAttributeNS(androidNs, "required")
        }
        assertEquals("false", featureRequired["android.hardware.camera"])
        assertEquals("false", featureRequired["android.hardware.camera.any"])

        val activities = doc.getElementsByTagName("activity")
        var scanActivity: Element? = null
        for (i in 0 until activities.length) {
            val el = activities.item(i) as Element
            if (el.getAttributeNS(androidNs, "name") == ".QrScanActivity") {
                scanActivity = el
                break
            }
        }
        assertNotNull(".QrScanActivity missing from AndroidManifest.xml", scanActivity)
        assertEquals("false", scanActivity!!.getAttributeNS(androidNs, "exported"))
        assertEquals("@string/qr_scan_title", scanActivity.getAttributeNS(androidNs, "label"))

        val defaultStrings = File(appDir, "src/main/res/values/l10n.xml").readText()
        assertTrue(defaultStrings.contains("<string name=\"qr_scan_title\">"))
        assertTrue(defaultStrings.contains("<string name=\"qr_scan_hint\">"))
        assertTrue(defaultStrings.contains("<string name=\"qr_scan_close\">"))
    }
}
