// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

package app.totaldrop.totaldrop

import android.Manifest
import android.app.Activity
import android.content.Intent
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel
import java.io.File
import java.util.concurrent.atomic.AtomicLong

class MainActivity : FlutterActivity() {
    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)
        val share = MethodChannel(flutterEngine.dartExecutor.binaryMessenger, SHARE_CHANNEL)
        share.setMethodCallHandler { call, result ->
            when (call.method) {
                // The Dart side listens: everything that came before it.
                "ready" -> result.success(launches.attach { share.invokeMethod("launch", it) })
                // Shares the person cannot take, or cleared: their copies go.
                "drop" -> {
                    val ids = (call.arguments as? List<*>)?.mapNotNull { (it as? Number)?.toLong() }
                    copier.drop(ids.orEmpty())
                    result.success(null)
                }
                else -> result.notImplemented()
            }
        }
        // Battery restrictions that stop a transfer in the background (#338).
        MethodChannel(flutterEngine.dartExecutor.binaryMessenger, BACKGROUND_CHANNEL)
            .setMethodCallHandler { call, result ->
                when (call.method) {
                    "state" -> result.success(BackgroundAccess.state(this))
                    "open" -> result.success(BackgroundAccess.open(this, call.arguments as? String ?: ""))
                    else -> result.notImplemented()
                }
            }
        val channel = MethodChannel(flutterEngine.dartExecutor.binaryMessenger, CHANNEL)
        TransferJob.onStop = { channel.invokeMethod("stop", null) }
        channel.setMethodCallHandler { call, result ->
            when (call.method) {
                "begin" -> {
                    val notice = TransferJob.Notice(
                        channel = call.argument<String>("channel") ?: "TotalDrop",
                        title = call.argument<String>("title") ?: "TotalDrop",
                        text = call.argument<String>("text") ?: "",
                        stop = call.argument<String>("stop") ?: "Stop",
                    )
                    askForNotifications()
                    result.success(TransferJob.begin(this, notice))
                }
                "end" -> {
                    TransferJob.end(this)
                    result.success(null)
                }
                else -> result.notImplemented()
            }
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        // A new activity has a new Flutter engine, which knows no copy from
        // before: they all go, before any new one is written.
        copier.clearAll()
        // A share or a shortcut that started the app. Not when the activity
        // comes back after the system ended it: that intent was handled.
        if (savedInstanceState == null) take(intent)
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
        take(intent)
    }

    /** What the person asked for by sharing to TotalDrop, with a launcher
     * shortcut (#53) or with a link (#445), as events for the Dart side
     * ([launches]). */
    private fun take(intent: Intent) {
        // Reopened from Recents: the share was handled when it came.
        if (intent.flags and Intent.FLAG_ACTIVITY_LAUNCHED_FROM_HISTORY != 0) return
        when (intent.action) {
            ACTION_SHORTCUT_SEND -> launches.send(mapOf("screen" to "send"))
            ACTION_SHORTCUT_RECEIVE -> launches.send(mapOf("screen" to "receive"))
            ACTION_SHORTCUT_CLIPBOARD -> launches.send(mapOf("screen" to "clipboard"))
            // A receive link, totaldrop://r/ or https://totaldrop.app/r
            // (#445): the Dart side reads the code out of it, strictly,
            // through the Rust core. The activity is exported, so an
            // explicit intent can skip the filters: only their links are
            // taken.
            Intent.ACTION_VIEW -> intent.data
                ?.takeIf { isReceiveLink(it) }
                ?.let { launches.send(mapOf("link" to it.toString())) }
            Intent.ACTION_SEND, Intent.ACTION_SEND_MULTIPLE -> takeShare(intent)
            // Text selected in another app, sent from its selection menu
            // (#326). Nothing is given back to that app.
            Intent.ACTION_PROCESS_TEXT -> {
                SharedFiles.processedText(intent)?.let { launches.send(mapOf("screen" to "send", "text" to it)) }
                forgetText(intent)
            }
        }
    }

    /** The activity keeps its intent: the text, now handed over, need not
     * stay in it. */
    /** The two forms of a receive link (#445); the manifest's filters let
     *  only these in, and an explicit intent is checked here as well. The
     *  code itself is read in Rust (TransferCode::from_link). */
    private fun isReceiveLink(uri: Uri): Boolean =
        (uri.scheme == "totaldrop" && uri.host == "r") ||
            (uri.scheme == "https" && uri.host == "totaldrop.app" && uri.path == "/r")

    private fun forgetText(intent: Intent) {
        intent.removeExtra(Intent.EXTRA_TEXT)
        intent.removeExtra(Intent.EXTRA_PROCESS_TEXT)
    }

    private fun takeShare(intent: Intent) {
        val uris = SharedFiles.urisOf(intent)
        if (SharedFiles.isTextOnly(intent, uris)) {
            // A text shared from another app (#326): sent as a text.
            val text = intent.getCharSequenceExtra(Intent.EXTRA_TEXT)?.toString().orEmpty()
            launches.send(mapOf("screen" to "send", "text" to text))
            forgetText(intent)
            return
        }
        val id = nextShare.incrementAndGet()
        launches.send(mapOf("id" to id, "screen" to "send", "copying" to true))
        // The read grant lasts while this activity does, so the copy starts
        // at once, off the main thread.
        copier.copy(id, uris) { copied ->
            val event: Map<String, Any> = if (copied == null) {
                mapOf("id" to id, "failed" to true)
            } else {
                mapOf(
                    "id" to id,
                    "files" to copied.items.map { mapOf("name" to it.name, "path" to it.path, "size" to it.size) },
                    "skipped" to copied.skipped,
                )
            }
            main.post { launches.send(event) }
        }
    }

    override fun cleanUpFlutterEngine(flutterEngine: FlutterEngine) {
        launches.detach()
        // Nothing would be left to end the job: end it, and its
        // notification, with the engine.
        TransferJob.onStop = null
        TransferJob.end(this)
        super.cleanUpFlutterEngine(flutterEngine)
    }

    /** Asked once per launch at most. The transfer runs without it too; the
     * notification is then hidden. */
    private fun askForNotifications() {
        if (askedForNotifications) return
        askedForNotifications = true
        if (checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS) !=
            PackageManager.PERMISSION_GRANTED
        ) {
            requestPermissions(arrayOf(Manifest.permission.POST_NOTIFICATIONS), 0)
        }
    }

    private var askedForNotifications = false
    private val launches = Launches()
    private val main = Handler(Looper.getMainLooper())
    private val copier by lazy { copierFor(this) }

    companion object {
        // One for the process: a copy of an activity that went away still
        // runs on its thread, before the next activity's clear-up.
        private var sharedCopier: ShareCopier? = null
        private val nextShare = AtomicLong()

        @Synchronized
        private fun copierFor(activity: Activity): ShareCopier = sharedCopier ?: ShareCopier(
            File(activity.noBackupFilesDir, "shared"),
            activity.applicationContext.contentResolver,
            activity.applicationContext.packageManager,
            activity.packageName,
        ).also { sharedCopier = it }

        const val CHANNEL = "app.totaldrop/transfer_job"
        const val SHARE_CHANNEL = "app.totaldrop/launch"
        const val BACKGROUND_CHANNEL = "app.totaldrop/background"
        // The launcher shortcuts (res/xml/shortcuts.xml).
        const val ACTION_SHORTCUT_SEND = "app.totaldrop.action.SEND"
        const val ACTION_SHORTCUT_RECEIVE = "app.totaldrop.action.RECEIVE"
        const val ACTION_SHORTCUT_CLIPBOARD = "app.totaldrop.action.SEND_CLIPBOARD"
    }
}
