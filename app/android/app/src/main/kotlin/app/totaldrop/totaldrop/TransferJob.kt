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
import android.os.Handler
import android.os.Looper

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

    private var notice: Notice? = null
    private var running: Pair<TransferJobService, JobParameters>? = null

    /** Called when the person or the system stops the transfer. */
    var onStop: (() -> Unit)? = null

    /** Start holding the process up. Must be called while the app is visible. */
    fun begin(context: Context, notice: Notice): Boolean {
        // A job still running from the transfer before ends quietly:
        // replacing it would stop it, and that reads as a stop of this one.
        running?.let { (service, params) -> service.jobFinished(params, false) }
        running = null
        this.notice = notice
        val app = context.applicationContext
        val scheduler = app.getSystemService(JobScheduler::class.java) ?: return false
        val job = JobInfo.Builder(JOB_ID, ComponentName(app, TransferJobService::class.java))
            .setUserInitiated(true)
            .setRequiredNetworkType(JobInfo.NETWORK_TYPE_ANY)
            .build()
        return scheduler.schedule(job) == JobScheduler.RESULT_SUCCESS
    }

    /** The transfer ended: drop the job and its notification. */
    fun end(context: Context) {
        notice = null
        val r = running
        running = null
        if (r != null) {
            r.first.jobFinished(r.second, false)
        } else {
            context.applicationContext.getSystemService(JobScheduler::class.java)?.cancel(JOB_ID)
        }
    }

    internal fun started(service: TransferJobService, params: JobParameters): Boolean {
        val n = notice ?: return false
        running = service to params
        service.setNotification(
            params,
            NOTIFICATION_ID,
            build(service, n),
            JobService.JOB_END_NOTIFICATION_POLICY_REMOVE,
        )
        return true
    }

    internal fun stoppedBySystem(params: JobParameters) {
        // Only the job of the transfer under way; not one that was replaced.
        if (running?.second !== params) return
        running = null
        notice = null
        onStop?.invoke()
    }

    internal fun stoppedByPerson() {
        onStop?.invoke()
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
            .setSmallIcon(R.mipmap.ic_launcher_monochrome)
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

    override fun onStopJob(params: JobParameters): Boolean {
        // The person stopped it from the system's task manager, or the
        // system ended it: the transfer cannot go on in the background.
        Handler(Looper.getMainLooper()).post { TransferJob.stoppedBySystem(params) }
        return false
    }
}

/** The Stop action on the notification. Not exported: only our own
 * PendingIntent reaches it. */
class StopTransferReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        if (intent.action == TransferJob.ACTION_STOP) TransferJob.stoppedByPerson()
    }
}
