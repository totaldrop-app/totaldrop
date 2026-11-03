package app.totaldrop.totaldrop

import android.Manifest
import android.content.pm.PackageManager
import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel

class MainActivity : FlutterActivity() {
    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)
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

    override fun cleanUpFlutterEngine(flutterEngine: FlutterEngine) {
        TransferJob.onStop = null
        super.cleanUpFlutterEngine(flutterEngine)
    }

    /** The transfer runs without it too; the notification is then hidden. */
    private fun askForNotifications() {
        if (checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS) !=
            PackageManager.PERMISSION_GRANTED
        ) {
            requestPermissions(arrayOf(Manifest.permission.POST_NOTIFICATIONS), 0)
        }
    }

    companion object {
        const val CHANNEL = "app.totaldrop/transfer_job"
    }
}
