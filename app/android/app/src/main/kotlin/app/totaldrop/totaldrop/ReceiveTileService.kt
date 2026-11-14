// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

package app.totaldrop.totaldrop

import android.app.PendingIntent
import android.content.Intent
import android.service.quicksettings.Tile
import android.service.quicksettings.TileService

/** Quick Settings tile (#53): "Receive with TotalDrop" collapses the shade
 * and opens the Receive screen directly through [MainActivity]. */
class ReceiveTileService : TileService() {
    override fun onStartListening() {
        super.onStartListening()
        qsTile?.apply {
            state = IDLE_STATE
            updateTile()
        }
    }

    override fun onClick() {
        super.onClick()
        val launch = Runnable {
            val target = targetFor(packageName)
            val intent = Intent(target.action)
                .setClassName(target.packageName, target.className)
                .addFlags(target.flags)
            val pending = PendingIntent.getActivity(
                this,
                0,
                intent,
                target.pendingFlags,
            )
            startActivityAndCollapse(pending)
        }
        if (isLocked) {
            unlockAndRun(launch)
        } else {
            launch.run()
        }
    }

    internal data class LaunchTarget(
        val action: String,
        val packageName: String,
        val className: String,
        val flags: Int,
        val pendingFlags: Int,
    )

    companion object {
        /** Action tile: clickable without appearing permanently toggled on. */
        const val IDLE_STATE: Int = Tile.STATE_INACTIVE

        internal fun targetFor(packageName: String): LaunchTarget = LaunchTarget(
            action = MainActivity.ACTION_SHORTCUT_RECEIVE,
            packageName = packageName,
            className = MainActivity::class.java.name,
            flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TOP,
            pendingFlags = PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
        )
    }
}
