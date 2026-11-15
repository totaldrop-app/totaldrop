// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

package app.totaldrop.totaldrop

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.job.JobInfo
import android.app.job.JobParameters
import android.app.job.JobScheduler
import android.app.job.JobService
import android.content.BroadcastReceiver
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.net.wifi.WifiManager
import android.os.PersistableBundle
import android.os.PowerManager
import android.util.Log

/**
 * Keeps a send or a receive alive while TotalDrop is in the background
 * (#194, #340). Android cuts the network of an app that is not visible,
 * which drops the code at the rendezvous server, and puts the Wi-Fi radio
 * into power-save sleep seconds after the screen locks. A user-initiated
 * data transfer job (Android 14+) tells the system the person started this
 * transfer so the process keeps its network and shows an ongoing
 * notification with a Stop action; alongside it, [Locks] holds a Wi-Fi
 * high-performance lock and a bounded partial CPU wake lock until [end] or
 * a terminal system stop. The transfer itself still runs in the Rust core.
 *
 * Everything here runs on the main thread.
 */
object TransferJob {
    private const val JOB_ID = 194
    private const val NOTIFICATION_ID = 194
    private const val CHANNEL_ID = "transfers"
    private const val LOCK_TAG = "TotalDrop:transfer"
    const val ACTION_STOP = "app.totaldrop.totaldrop.STOP_TRANSFER"

    /** Upper bound on the partial wake lock (6 hours, matching Android 15's
     * daily background data-transfer window) so a wedged session never holds
     * the CPU awake indefinitely (#340). */
    internal const val MAX_LOCK_MS = 6L * 60 * 60 * 1000

    internal interface Locks {
        fun acquire()
        fun release()
    }

    /** What the notification says, in the person's language (from Dart). */
    data class Notice(
        val channel: String,
        val title: String,
        val text: String,
        val stop: String,
    )

    private const val GENERATION = "generation"

    private var notice: Notice? = null
    private var running: Pair<TransferJobService, JobParameters>? = null
    internal var locks: Locks? = null

    /** Which transfer a job belongs to. The system hands each callback its
     * own JobParameters, so they cannot be compared by identity. */
    private var generation = 0L

    /** Called when the person or the system stops the transfer. */
    var onStop: (() -> Unit)? = null

    /** Start holding the process up. Must be called while the app is visible. */
    fun begin(context: Context, notice: Notice): Boolean {
        val app = context.applicationContext
        if (locks == null) locks = AndroidLocks(app)
        val generation = prepare(notice)
        val scheduler = app.getSystemService(JobScheduler::class.java) ?: return false
        val job = JobInfo.Builder(JOB_ID, ComponentName(app, TransferJobService::class.java))
            .setUserInitiated(true)
            .setRequiredNetworkType(JobInfo.NETWORK_TYPE_ANY)
            .setExtras(PersistableBundle().apply { putLong(GENERATION, generation) })
            .build()
        return scheduler.schedule(job) == JobScheduler.RESULT_SUCCESS
    }

    /** Make [notice]'s transfer the one under way, acquire its radio/CPU
     * locks, and return its generation. */
    internal fun prepare(notice: Notice): Long {
        // A job still running from the transfer before ends quietly:
        // replacing it would stop it, and that reads as a stop of this one.
        running?.let { (service, params) -> service.jobFinished(params, false) }
        running = null
        this.notice = notice
        generation += 1
        locks?.acquire()
        return generation
    }

    /** Whether a job of [generation] belongs to the transfer under way. */
    internal fun accepts(generation: Long): Boolean =
        notice != null && generation == this.generation

    /** Clear the active transfer state and release radio/CPU locks, returning
     * any running job so [end] can finish it. */
    internal fun finishTransfer(): Pair<TransferJobService, JobParameters>? {
        notice = null
        generation += 1
        val r = running
        running = null
        locks?.release()
        return r
    }

    /** The transfer ended: drop the job, its locks, and its notification. */
    fun end(context: Context) {
        val r = finishTransfer()
        val app = context.applicationContext
        if (r != null) {
            r.first.jobFinished(r.second, false)
        } else {
            // Also a job waiting to run again after the network went away.
            app.getSystemService(JobScheduler::class.java)?.cancel(JOB_ID)
        }
        // The job leaves its notification behind (see [started]).
        app.getSystemService(NotificationManager::class.java)?.cancel(NOTIFICATION_ID)
    }

    internal fun started(service: TransferJobService, params: JobParameters): Boolean {
        val n = notice ?: return false
        if (!accepts(params.extras.getLong(GENERATION))) return false
        running = service to params
        service.setNotification(
            params,
            NOTIFICATION_ID,
            build(service, n),
            // Kept when the job stops, so that while the network is away
            // the transfer stays visible and can still be stopped (#260);
            // [end] and [stoppedBySystem] remove it.
            JobService.JOB_END_NOTIFICATION_POLICY_DETACH,
        )
        return true
    }

    /** The system stopped the job; see [stoppedBySystem]. */
    internal fun stoppedBySystem(context: Context, params: JobParameters): Boolean {
        val again = stoppedBySystem(params.extras.getLong(GENERATION), params.stopReason)
        if (!again && notice == null) {
            context.getSystemService(NotificationManager::class.java)?.cancel(NOTIFICATION_ID)
        }
        return again
    }

    /** The system stopped a job of [generation] for [stopReason] (one of
     * `JobParameters.STOP_REASON_*`). Returns whether the system should
     * run the job again. */
    internal fun stoppedBySystem(generation: Long, stopReason: Int): Boolean {
        // Only the job of the transfer under way; not one that was replaced
        // or ended.
        if (!accepts(generation)) return false
        running = null
        if (stopReason == JobParameters.STOP_REASON_CONSTRAINT_CONNECTIVITY) {
            // The network went away (airplane mode, a tunnel, a Wi-Fi that
            // drops): the transfer goes on, and the core can ride out a gap
            // shorter than QUIC's 30-second idle timeout (#185). Keep the
            // CPU wake lock held across the gap while the same job waits to
            // run again once there is a network (#260, #340).
            return true
        }
        notice = null
        this.generation += 1
        locks?.release()
        onStop?.invoke()
        return false
    }

    internal fun stoppedByPerson(context: Context) {
        val stop = onStop
        if (stop != null) {
            stop()
        } else {
            // No app left to stop the transfer: at least end the job, so
            // the notification does not stay behind.
            end(context)
        }
    }

    private class AndroidLocks(app: Context) : Locks {
        private val wakeLock: PowerManager.WakeLock? =
            app.getSystemService(PowerManager::class.java)
                ?.newWakeLock(PowerManager.PARTIAL_WAKE_LOCK, LOCK_TAG)
                ?.apply { setReferenceCounted(false) }

        // WIFI_MODE_FULL_LOW_LATENCY is deactivated by Android's
        // WifiLockManager as soon as the screen turns off or the app leaves
        // the foreground; WIFI_MODE_FULL_HIGH_PERF keeps Wi-Fi out of
        // power-save sleep while the screen is locked (#340).
        @Suppress("DEPRECATION")
        private val wifiLock: WifiManager.WifiLock? =
            app.getSystemService(WifiManager::class.java)
                ?.createWifiLock(WifiManager.WIFI_MODE_FULL_HIGH_PERF, LOCK_TAG)
                ?.apply { setReferenceCounted(false) }

        override fun acquire() {
            tryLock("wake acquire") { wakeLock?.acquire(MAX_LOCK_MS) }
            tryLock("wifi acquire") {
                if (wifiLock?.isHeld == false) {
                    wifiLock.acquire()
                }
            }
        }

        override fun release() {
            tryLock("wifi release") {
                if (wifiLock?.isHeld == true) {
                    wifiLock.release()
                }
            }
            tryLock("wake release") {
                if (wakeLock?.isHeld == true) {
                    wakeLock.release()
                }
            }
        }

        private inline fun tryLock(action: String, block: () -> Unit) {
            try {
                block()
            } catch (e: Exception) {
                Log.w("TotalDrop", "transfer lock $action failed: ${e.javaClass.simpleName}")
            }
        }
    }

    private fun build(context: Context, n: Notice): Notification {
        val manager = context.getSystemService(NotificationManager::class.java)
        manager?.createNotificationChannel(
            NotificationChannel(CHANNEL_ID, n.channel, NotificationManager.IMPORTANCE_LOW),
        )
        val open = PendingIntent.getActivity(
            context,
            0,
            Intent(context, MainActivity::class.java)
                .setFlags(Intent.FLAG_ACTIVITY_SINGLE_TOP),
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
        )
        val stop = PendingIntent.getBroadcast(
            context,
            1,
            Intent(context, StopTransferReceiver::class.java).setAction(ACTION_STOP),
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
        )
        return Notification.Builder(context, CHANNEL_ID)
            .setSmallIcon(R.drawable.ic_stat_transfer)
            .setContentTitle(n.title)
            .setContentText(n.text)
            .setContentIntent(open)
            .setOngoing(true)
            .setOnlyAlertOnce(true)
            .setCategory(Notification.CATEGORY_PROGRESS)
            .addAction(Notification.Action.Builder(null, n.stop, stop).build())
            .build()
    }
}

/** Runs while a transfer is under way; see [TransferJob]. */
class TransferJobService : JobService() {
    override fun onStartJob(params: JobParameters): Boolean = TransferJob.started(this, params)

    override fun onStopJob(params: JobParameters): Boolean =
        TransferJob.stoppedBySystem(this, params)
}

/** The Stop action on the notification. Not exported: only our own
 * PendingIntent reaches it. */
class StopTransferReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        if (intent.action == TransferJob.ACTION_STOP) TransferJob.stoppedByPerson(context)
    }
}
