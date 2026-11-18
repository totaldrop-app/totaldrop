// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

package app.totaldrop.totaldrop

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Test
import org.w3c.dom.Element
import java.io.File
import javax.xml.parsers.DocumentBuilderFactory

/** Android 15+ manifest configuration checks (#590). */
class ManifestConfigTest {
    @Test
    fun applicationEnablesPredictiveBackCallback() {
        val androidNs = "http://schemas.android.com/apk/res/android"
        val appDir = listOf(File("."), File("app")).firstOrNull {
            File(it, "src/main/AndroidManifest.xml").isFile
        }
        assertNotNull("Could not locate src/main/AndroidManifest.xml", appDir)
        val manifest = File(appDir!!, "src/main/AndroidManifest.xml")

        val factory = DocumentBuilderFactory.newInstance().apply { isNamespaceAware = true }
        val doc = factory.newDocumentBuilder().parse(manifest)
        val application = doc.getElementsByTagName("application").item(0) as? Element
        assertNotNull("<application> missing from AndroidManifest.xml", application)
        assertEquals(
            "true",
            application!!.getAttributeNS(androidNs, "enableOnBackInvokedCallback"),
        )
    }
}
