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
import android.os.PersistableBundle

/**
 * Keeps a send or a receive alive while TotalDrop is in the background
 * (#194). Android cuts the network of an app that is not visible, which
 * drops the code at the rendezvous server. A user-initiated data transfer
 * job (Android 14+) tells the system the person started this transfer: the
 * process keeps its network and shows an ongoing notification with a Stop
 * action. The transfer itself still runs in the Rust core; the job only
 * holds the process up until [end] is called.
 *
 * Everything here runs on the main thread.
 */
object TransferJob {
    private const val JOB_ID = 194
    private const val NOTIFICATION_ID = 194
    private const val CHANNEL_ID = "transfers"
    const val ACTION_STOP = "app.totaldrop.totaldrop.STOP_TRANSFER"

    /** What the notification says, in the person's language (from Dart). */
    data class Notice(
        val channel: String,
        val title: String,
        val text: String,
        val stop: String,
    )

    private const val GENERATION = "generation"

    /** Thin handle over the active [JobService] execution so JVM unit tests
     * can verify in-place estimate updates never finish the running job (#593). */
    internal interface RunningJob {
        fun finish()
        fun updateEstimatedNetworkBytes(downloadBytes: Long, uploadBytes: Long)
    }

    private class AndroidRunningJob(
        private val service: TransferJobService,
        private val params: JobParameters,
    ) : RunningJob {
        override fun finish() {
            service.jobFinished(params, false)
        }

        override fun updateEstimatedNetworkBytes(downloadBytes: Long, uploadBytes: Long) {
            service.updateEstimatedNetworkBytes(params, downloadBytes, uploadBytes)
        }
    }

    private val UNKNOWN_BYTES: Long = JobInfo.NETWORK_BYTES_UNKNOWN.toLong()

    private var notice: Notice? = null
    private var running: RunningJob? = null
    private var pendingDownloadBytes: Long = UNKNOWN_BYTES
    private var pendingUploadBytes: Long = UNKNOWN_BYTES

    /** Which transfer a job belongs to. The system hands each callback its
     * own JobParameters, so they cannot be compared by identity. */
    private var generation = 0L

    /** Called when the person or the system stops the transfer. */
    var onStop: (() -> Unit)? = null

    /** Maps a byte count to a `JobInfo` network byte estimate. */
    internal fun networkBytes(value: Long?): Long =
        if (value != null && value >= 0L) value else UNKNOWN_BYTES

    /** Start holding the process up. Must be called while the app is visible. */
    fun begin(
        context: Context,
        notice: Notice,
        downloadBytes: Long? = null,
        uploadBytes: Long? = null,
    ): Boolean {
        val generation = prepare(notice)
        val app = context.applicationContext
        val scheduler = app.getSystemService(JobScheduler::class.java) ?: return false
        val dl = networkBytes(downloadBytes)
        val ul = networkBytes(uploadBytes)
        val builder = JobInfo.Builder(JOB_ID, ComponentName(app, TransferJobService::class.java))
            .setUserInitiated(true)
            .setRequiredNetworkType(JobInfo.NETWORK_TYPE_ANY)
            .setExtras(PersistableBundle().apply { putLong(GENERATION, generation) })
        if (dl != UNKNOWN_BYTES || ul != UNKNOWN_BYTES) {
            builder.setEstimatedNetworkBytes(dl, ul)
        }
        return scheduler.schedule(builder.build()) == JobScheduler.RESULT_SUCCESS
    }

    /**
     * Update the estimated network bytes on the running user-initiated job
     * in place (API 34+ [JobService.updateEstimatedNetworkBytes]) without
     * finishing or re-scheduling it (#593). If the job has not entered
     * [started] yet (or is awaiting a network reconnect), the estimate is
     * remembered and applied as soon as it starts.
     */
    fun update(
        downloadBytes: Long? = null,
        uploadBytes: Long? = null,
    ): Boolean {
        if (notice == null) return false
        val dl = networkBytes(downloadBytes)
        val ul = networkBytes(uploadBytes)
        if (dl == UNKNOWN_BYTES && ul == UNKNOWN_BYTES) return false
        pendingDownloadBytes = dl
        pendingUploadBytes = ul
        running?.updateEstimatedNetworkBytes(dl, ul)
        return true
    }

    /** Make [notice]'s transfer the one under way, and return its
     * generation. */
    internal fun prepare(notice: Notice): Long {
        // A job still running from the transfer before ends quietly:
        // replacing it would stop it, and that reads as a stop of this one.
        running?.finish()
        running = null
        pendingDownloadBytes = UNKNOWN_BYTES
        pendingUploadBytes = UNKNOWN_BYTES
        this.notice = notice
        generation += 1
        return generation
    }

    /** Whether a job of [generation] belongs to the transfer under way. */
    internal fun accepts(generation: Long): Boolean =
        notice != null && generation == this.generation

    /** Clears the active transfer state and returns the running job, if any. */
    internal fun clearActive(): RunningJob? {
        notice = null
        generation += 1
        pendingDownloadBytes = UNKNOWN_BYTES
        pendingUploadBytes = UNKNOWN_BYTES
        val r = running
        running = null
        return r
    }

    /** The transfer ended: drop the job and its notification. */
    fun end(context: Context) {
        val r = clearActive()
        val app = context.applicationContext
        if (r != null) {
            r.finish()
        } else {
            // Also a job waiting to run again after the network went away.
            app.getSystemService(JobScheduler::class.java)?.cancel(JOB_ID)
        }
        // The job leaves its notification behind (see [started]).
        app.getSystemService(NotificationManager::class.java)?.cancel(NOTIFICATION_ID)
    }

    /** Records a started job for [generation] and applies any estimate update
     * that arrived before `onStartJob`. */
    internal fun attachStarted(generation: Long, job: RunningJob): Boolean {
        if (!accepts(generation)) return false
        running = job
        if (pendingDownloadBytes != UNKNOWN_BYTES || pendingUploadBytes != UNKNOWN_BYTES) {
            job.updateEstimatedNetworkBytes(pendingDownloadBytes, pendingUploadBytes)
        }
        return true
    }

    internal fun started(service: TransferJobService, params: JobParameters): Boolean {
        val n = notice ?: return false
        val jobGeneration = params.extras.getLong(GENERATION)
        if (!accepts(jobGeneration)) return false
        service.setNotification(
            params,
            NOTIFICATION_ID,
            build(service, n),
            // Kept when the job stops, so that while the network is away
            // the transfer stays visible and can still be stopped (#260);
            // [end] and [stoppedBySystem] remove it.
            JobService.JOB_END_NOTIFICATION_POLICY_DETACH,
        )
        return attachStarted(jobGeneration, AndroidRunningJob(service, params))
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
            // shorter than QUIC's 30-second idle timeout (#185). The same
            // job runs again once there is a network (#260).
            return true
        }
        notice = null
        pendingDownloadBytes = UNKNOWN_BYTES
        pendingUploadBytes = UNKNOWN_BYTES
        this.generation += 1
        onStop?.invoke()
        return false
    }

    internal fun stoppedByPerson(context: Context) {
        stoppedByPerson { end(context) }
    }

    /**
     * Dismisses the job and its notification at once via [dismiss], then
     * notifies [onStop] so the notification never lingers while a stalled
     * socket closes (#593).
     *
     * Kotlin cannot cancel the Rust `Stopper` directly: that handle lives in
     * the Dart isolate and is triggered through [onStop] (wired to the
     * `MethodChannel` in `MainActivity`). Dismissing first immediately drops
     * Android's UIDT background network exemption and removes the notification
     * even if the Flutter engine is no longer attached (`MainActivity` clears
     * [onStop] and calls [end] in `cleanUpFlutterEngine`, and everything here
     * runs on the main thread).
     */
    internal fun stoppedByPerson(dismiss: () -> Unit) {
        val stop = onStop
        dismiss()
        stop?.invoke()
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
