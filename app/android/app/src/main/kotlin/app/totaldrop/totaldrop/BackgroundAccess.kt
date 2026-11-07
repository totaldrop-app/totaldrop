// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

package app.totaldrop.totaldrop

import android.app.Activity
import android.content.ActivityNotFoundException
import android.content.ComponentName
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.os.PowerManager
import android.provider.Settings

/**
 * Whether a transfer may keep going in the background, and the system
 * pages that change it (#338).
 *
 * The user-initiated data transfer job (#194) keeps the process and its
 * network up on stock Android. Some makers stop an app's traffic anyway
 * once it leaves the screen, unless the person lifts the app's battery
 * restrictions: on Xiaomi's HyperOS a transfer stopped within seconds with
 * TotalDrop's battery saver on its default and kept going with "No
 * restrictions".
 *
 * The app opens the settings pages; it never asks for
 * REQUEST_IGNORE_BATTERY_OPTIMIZATIONS, which Google Play allows only for
 * a few kinds of app, and the person decides there.
 */
object BackgroundAccess {
    /** Xiaomi's phones (MIUI, HyperOS) add a battery saver of their own,
     * and an Autostart switch, on top of Android's battery optimisation;
     * both have pages the app can open. Other makers' extra savers have
     * no page to open, so only Android's own setting is offered there. */
    private val XIAOMI = setOf("xiaomi", "redmi", "poco")

    private fun maker(): String = Build.MANUFACTURER.orEmpty().trim().lowercase()

    /** What the Dart side shows: whether Android's own battery
     * optimisation is off for TotalDrop, and the maker when it has a saver
     * of its own with a page to open (null otherwise). The maker's saver
     * cannot be read. */
    fun state(activity: Activity): Map<String, Any?> {
        val power = activity.getSystemService(PowerManager::class.java)
        val exempt = power?.isIgnoringBatteryOptimizations(activity.packageName) == true
        val xiaomi = maker() in XIAOMI
        return mapOf(
            "exempt" to exempt,
            "maker" to if (xiaomi) "Xiaomi" else null,
            "autostart" to xiaomi,
        )
    }

    /** Opens [page]: `battery` (TotalDrop's app info, where App battery
     * usage is one tap away; Android's list of battery optimisation if
     * that does not open),
     * `maker` (the maker's battery saver for TotalDrop, else its app
     * info) or `autostart` (Xiaomi). False when nothing could open. */
    fun open(activity: Activity, page: String): Boolean {
        val pkg = activity.packageName
        val details = Intent(Settings.ACTION_APPLICATION_DETAILS_SETTINGS, Uri.parse("package:$pkg"))
        val candidates: List<Intent> = when (page) {
            // The list opens filtered to apps already exempt, so TotalDrop is
            // not in it until the filter is changed: app info first.
            "battery" -> listOf(details, Intent(Settings.ACTION_IGNORE_BATTERY_OPTIMIZATION_SETTINGS))
            "maker" -> if (maker() in XIAOMI) {
                listOf(
                    Intent().setComponent(
                        ComponentName("com.miui.powerkeeper", "com.miui.powerkeeper.ui.HiddenAppsConfigActivity"),
                    )
                        .putExtra("package_name", pkg)
                        .putExtra("package_label", activity.applicationInfo.loadLabel(activity.packageManager).toString()),
                    details,
                )
            } else {
                listOf(details)
            }
            "autostart" -> listOf(
                Intent().setComponent(
                    ComponentName("com.miui.securitycenter", "com.miui.permcenter.autostart.AutoStartManagementActivity"),
                ),
                details,
            )
            else -> emptyList()
        }
        for (intent in candidates) {
            try {
                activity.startActivity(intent)
                return true
            } catch (_: ActivityNotFoundException) {
                // Not on this phone: the next page.
            } catch (_: SecurityException) {
                // Not exported on this version: the next page.
            }
        }
        return false
    }
}
