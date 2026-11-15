// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

package app.totaldrop.totaldrop

import android.app.Activity
import android.content.ActivityNotFoundException
import android.content.Intent
import android.net.Uri

/**
 * Opens a confirmed http(s) URL from a received text in the person's
 * browser (#53).
 *
 * Kept outside [MainActivity] so its string checks run in plain JVM unit
 * tests without loading Android's stub Activity class.
 */
object WebLinks {
    /** Defense-in-depth check on the Kotlin side before [Uri.parse]; the
     * Rust core (`totaldrop_proto::single_web_url`) validates first (#53). */
    internal fun isSafeWebUrlString(raw: String?): Boolean =
        raw != null &&
            raw.length in 1..2048 &&
            (raw.startsWith("https://") || raw.startsWith("http://")) &&
            raw.none { it.isWhitespace() || it.isISOControl() || it == '\\' }

    /** Empty-host scheme URI for [Intent.setSelector] so [Intent.ACTION_VIEW]
     * resolves to web browsers rather than domain-specific App Links (#53). */
    internal fun browserSelectorUri(raw: String?): String? {
        if (!isSafeWebUrlString(raw)) return null
        return when {
            raw!!.startsWith("https://") -> "https://"
            raw.startsWith("http://") -> "http://"
            else -> null
        }
    }

    /** Opens a confirmed http(s) URL in the person's browser (#53). */
    fun open(activity: Activity, raw: String?): Boolean {
        val selectorUri = browserSelectorUri(raw) ?: return false
        val uri = Uri.parse(raw)
        val scheme = uri.scheme?.lowercase()
        if ((scheme != "https" && scheme != "http") || uri.host.isNullOrEmpty()) return false
        val intent = Intent(Intent.ACTION_VIEW, uri)
            .addCategory(Intent.CATEGORY_BROWSABLE)
            .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        // A generic http(s) selector targets browsers rather than App Link
        // handlers, including TotalDrop's own https://totaldrop.app/r filter.
        intent.selector = Intent(Intent.ACTION_VIEW, Uri.parse(selectorUri))
            .addCategory(Intent.CATEGORY_BROWSABLE)
        return try {
            activity.startActivity(intent)
            true
        } catch (_: ActivityNotFoundException) {
            false
        }
    }
}
