// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

package app.totaldrop.totaldrop

import android.Manifest
import android.app.Activity
import android.content.Intent
import android.content.pm.PackageManager
import android.graphics.Color
import android.graphics.drawable.GradientDrawable
import android.os.Bundle
import android.util.TypedValue
import android.view.Gravity
import android.view.View
import android.view.WindowInsets
import android.widget.FrameLayout
import android.widget.ImageButton
import android.widget.LinearLayout
import android.widget.TextView
import androidx.camera.core.CameraSelector
import androidx.camera.core.ImageAnalysis
import androidx.camera.core.Preview
import androidx.camera.lifecycle.ProcessCameraProvider
import androidx.camera.view.PreviewView
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleOwner
import androidx.lifecycle.LifecycleRegistry
import zxingcpp.BarcodeReader
import java.util.concurrent.ExecutorService
import java.util.concurrent.Executors
import java.util.concurrent.atomic.AtomicBoolean

/**
 * Full-screen CameraX preview that decodes TotalDrop receive QR codes with
 * zxing-cpp (#29, ADR 0023).
 *
 * Camera permission is asked only when this screen opens. Unrelated QR codes
 * in view are ignored; the first `https://totaldrop.app/r#…`,
 * `https://totaldrop.app/r/#…` or `totaldrop://r/…` symbol ends the activity
 * and hands the raw link back to Dart, where the Rust core
 * (`TransferCode::from_qr`) validates the code.
 */
class QrScanActivity : Activity(), LifecycleOwner {
    private val lifecycleRegistry = LifecycleRegistry(this)
    private val completed = AtomicBoolean(false)
    private var permissionInFlight = false
    private var analysisExecutor: ExecutorService? = null
    private var cameraProvider: ProcessCameraProvider? = null
    private lateinit var previewView: PreviewView

    override val lifecycle: Lifecycle
        get() = lifecycleRegistry

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        lifecycleRegistry.handleLifecycleEvent(Lifecycle.Event.ON_CREATE)
        permissionInFlight = savedInstanceState?.getBoolean(KEY_PERMISSION_IN_FLIGHT) ?: false

        if (!packageManager.hasSystemFeature(PackageManager.FEATURE_CAMERA_ANY)) {
            finishWithOutcome(OUTCOME_UNAVAILABLE)
            return
        }

        window.insetsController?.setSystemBarsAppearance(
            0,
            android.view.WindowInsetsController.APPEARANCE_LIGHT_STATUS_BARS or
                android.view.WindowInsetsController.APPEARANCE_LIGHT_NAVIGATION_BARS,
        )
        setContentView(buildContentView())

        if (checkSelfPermission(Manifest.permission.CAMERA) == PackageManager.PERMISSION_GRANTED) {
            permissionInFlight = false
            startCamera()
        } else if (!permissionInFlight) {
            permissionInFlight = true
            requestPermissions(arrayOf(Manifest.permission.CAMERA), REQUEST_CAMERA)
        }
    }

    override fun onSaveInstanceState(outState: Bundle) {
        super.onSaveInstanceState(outState)
        outState.putBoolean(KEY_PERMISSION_IN_FLIGHT, permissionInFlight)
    }

    override fun onStart() {
        super.onStart()
        lifecycleRegistry.handleLifecycleEvent(Lifecycle.Event.ON_START)
    }

    override fun onResume() {
        super.onResume()
        lifecycleRegistry.handleLifecycleEvent(Lifecycle.Event.ON_RESUME)
        if (checkSelfPermission(Manifest.permission.CAMERA) == PackageManager.PERMISSION_GRANTED) {
            permissionInFlight = false
            startCamera()
        } else if (!permissionInFlight && !isFinishing) {
            // Recreated after a system permission prompt was dismissed or
            // interrupted without delivering a result: request once cleanly
            // rather than stranding on an empty preview.
            permissionInFlight = true
            requestPermissions(arrayOf(Manifest.permission.CAMERA), REQUEST_CAMERA)
        }
    }

    override fun onPause() {
        lifecycleRegistry.handleLifecycleEvent(Lifecycle.Event.ON_PAUSE)
        super.onPause()
    }

    override fun onStop() {
        lifecycleRegistry.handleLifecycleEvent(Lifecycle.Event.ON_STOP)
        super.onStop()
    }

    override fun onDestroy() {
        lifecycleRegistry.handleLifecycleEvent(Lifecycle.Event.ON_DESTROY)
        cameraProvider?.unbindAll()
        analysisExecutor?.shutdownNow()
        analysisExecutor = null
        super.onDestroy()
    }

    override fun onRequestPermissionsResult(
        requestCode: Int,
        permissions: Array<out String>,
        grantResults: IntArray,
    ) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults)
        if (requestCode != REQUEST_CAMERA) return
        permissionInFlight = false
        val outcome = permissionOutcome(grantResults)
        if (outcome == null) {
            startCamera()
        } else if (completed.compareAndSet(false, true)) {
            finishWithOutcome(outcome)
        }
    }

    private fun startCamera() {
        if (analysisExecutor != null) return
        val executor = Executors.newSingleThreadExecutor()
        analysisExecutor = executor
        val future = ProcessCameraProvider.getInstance(this)
        future.addListener(
            {
                if (isFinishing || isDestroyed) return@addListener
                val provider = try {
                    future.get()
                } catch (_: Throwable) {
                    if (completed.compareAndSet(false, true)) {
                        finishWithOutcome(OUTCOME_UNAVAILABLE)
                    }
                    return@addListener
                }
                cameraProvider = provider
                val selector = when {
                    hasCamera(provider, CameraSelector.DEFAULT_BACK_CAMERA) ->
                        CameraSelector.DEFAULT_BACK_CAMERA
                    hasCamera(provider, CameraSelector.DEFAULT_FRONT_CAMERA) ->
                        CameraSelector.DEFAULT_FRONT_CAMERA
                    else -> {
                        if (completed.compareAndSet(false, true)) {
                            finishWithOutcome(OUTCOME_UNAVAILABLE)
                        }
                        return@addListener
                    }
                }
                val preview = Preview.Builder().build().also {
                    it.surfaceProvider = previewView.surfaceProvider
                }
                val reader by lazy {
                    BarcodeReader(
                        BarcodeReader.Options(
                            formats = setOf(BarcodeReader.Format.QR_CODE),
                            tryHarder = true,
                            tryRotate = true,
                            tryInvert = true,
                            tryDownscale = true,
                            maxNumberOfSymbols = 4,
                            textMode = BarcodeReader.TextMode.PLAIN,
                        ),
                    )
                }
                val analysis = ImageAnalysis.Builder()
                    .setBackpressureStrategy(ImageAnalysis.STRATEGY_KEEP_ONLY_LATEST)
                    .build()
                analysis.setAnalyzer(executor) { image ->
                    val texts = try {
                        image.use { frame -> reader.read(frame).map { it.text } }
                    } catch (_: Throwable) {
                        emptyList()
                    }
                    val link = firstQrReceiveLink(texts)
                    if (link != null && completed.compareAndSet(false, true)) {
                        runOnUiThread { finishWithOutcome(OUTCOME_SCANNED, link) }
                    }
                }
                try {
                    provider.unbindAll()
                    provider.bindToLifecycle(this, selector, preview, analysis)
                } catch (_: Throwable) {
                    if (completed.compareAndSet(false, true)) {
                        finishWithOutcome(OUTCOME_UNAVAILABLE)
                    }
                }
            },
            mainExecutor,
        )
    }

    private fun hasCamera(provider: ProcessCameraProvider, selector: CameraSelector): Boolean =
        try {
            provider.hasCamera(selector)
        } catch (_: Throwable) {
            false
        }

    private fun finishWithOutcome(outcome: String, link: String? = null) {
        val data = Intent().putExtra(EXTRA_OUTCOME, outcome)
        if (link != null) data.putExtra(EXTRA_LINK, link)
        setResult(RESULT_OK, data)
        finish()
    }

    private fun dp(value: Int): Int = TypedValue.applyDimension(
        TypedValue.COMPLEX_UNIT_DIP,
        value.toFloat(),
        resources.displayMetrics,
    ).toInt()

    private fun buildContentView(): FrameLayout {
        val root = FrameLayout(this).apply {
            setBackgroundColor(Color.parseColor("#0B0F14"))
        }
        previewView = PreviewView(this).apply {
            implementationMode = PreviewView.ImplementationMode.COMPATIBLE
            scaleType = PreviewView.ScaleType.FILL_CENTER
        }
        root.addView(
            previewView,
            FrameLayout.LayoutParams(
                FrameLayout.LayoutParams.MATCH_PARENT,
                FrameLayout.LayoutParams.MATCH_PARENT,
            ),
        )

        val reticleSize = dp(248)
        val reticle = FrameLayout(this).apply {
            background = GradientDrawable().apply {
                shape = GradientDrawable.RECTANGLE
                cornerRadius = dp(28).toFloat()
                setColor(Color.TRANSPARENT)
                setStroke(dp(2), Color.argb(210, 255, 255, 255))
            }
        }
        root.addView(
            reticle,
            FrameLayout.LayoutParams(reticleSize, reticleSize, Gravity.CENTER),
        )

        val topBar = LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            setPaddingRelative(dp(12), dp(12), dp(20), dp(12))
            setBackgroundColor(Color.argb(148, 11, 15, 20))
        }
        val closeButton = ImageButton(this).apply {
            setImageResource(android.R.drawable.ic_menu_close_clear_cancel)
            setColorFilter(Color.WHITE)
            background = GradientDrawable().apply {
                shape = GradientDrawable.OVAL
                setColor(Color.argb(64, 255, 255, 255))
            }
            contentDescription = getString(R.string.qr_scan_close)
            val size = dp(48)
            layoutParams = LinearLayout.LayoutParams(size, size)
            setOnClickListener {
                if (completed.compareAndSet(false, true)) {
                    finishWithOutcome(OUTCOME_CANCELLED)
                }
            }
        }
        val titleView = TextView(this).apply {
            text = getString(R.string.qr_scan_title)
            setTextColor(Color.WHITE)
            setTextSize(TypedValue.COMPLEX_UNIT_SP, 20f)
            textAlignment = View.TEXT_ALIGNMENT_VIEW_START
            setPaddingRelative(dp(12), 0, dp(12), 0)
        }
        topBar.addView(closeButton)
        topBar.addView(
            titleView,
            LinearLayout.LayoutParams(0, LinearLayout.LayoutParams.WRAP_CONTENT, 1f),
        )
        root.addView(
            topBar,
            FrameLayout.LayoutParams(
                FrameLayout.LayoutParams.MATCH_PARENT,
                FrameLayout.LayoutParams.WRAP_CONTENT,
                Gravity.TOP,
            ),
        )

        val hintView = TextView(this).apply {
            text = getString(R.string.qr_scan_hint)
            setTextColor(Color.WHITE)
            setTextSize(TypedValue.COMPLEX_UNIT_SP, 15f)
            gravity = Gravity.CENTER
            setPaddingRelative(dp(20), dp(14), dp(20), dp(14))
            background = GradientDrawable().apply {
                shape = GradientDrawable.RECTANGLE
                cornerRadius = dp(20).toFloat()
                setColor(Color.argb(176, 11, 15, 20))
            }
        }
        val hintParams = FrameLayout.LayoutParams(
            FrameLayout.LayoutParams.MATCH_PARENT,
            FrameLayout.LayoutParams.WRAP_CONTENT,
            Gravity.BOTTOM,
        ).apply {
            val margin = dp(24)
            setMargins(margin, margin, margin, margin)
        }
        root.addView(hintView, hintParams)

        root.setOnApplyWindowInsetsListener { view, insets ->
            val safe = insets.getInsets(
                WindowInsets.Type.systemBars() or WindowInsets.Type.displayCutout(),
            )
            val isRtl = view.layoutDirection == View.LAYOUT_DIRECTION_RTL
            val leftBase = if (isRtl) dp(20) else dp(12)
            val rightBase = if (isRtl) dp(12) else dp(20)
            topBar.setPadding(
                leftBase + safe.left,
                dp(12) + safe.top,
                rightBase + safe.right,
                dp(12),
            )
            hintParams.setMargins(
                dp(24) + safe.left,
                dp(24),
                dp(24) + safe.right,
                dp(24) + safe.bottom,
            )
            hintView.layoutParams = hintParams
            insets
        }
        return root
    }

    companion object {
        private const val REQUEST_CAMERA = 29
        private const val KEY_PERMISSION_IN_FLIGHT = "permission_in_flight"
        private const val MAX_QR_LINK_BYTES = 1024

        const val EXTRA_OUTCOME = "outcome"
        const val EXTRA_LINK = "link"

        const val OUTCOME_SCANNED = "scanned"
        const val OUTCOME_DENIED = "denied"
        const val OUTCOME_UNAVAILABLE = "unavailable"
        const val OUTCOME_CANCELLED = "cancelled"

        /**
         * Classifies Android's `onRequestPermissionsResult` array: an empty
         * `grantResults` means the permission request was interrupted/cancelled
         * by the system rather than denied by the person. Returns `null` when
         * camera permission was granted.
         */
        internal fun permissionOutcome(grantResults: IntArray): String? = when {
            grantResults.isEmpty() -> OUTCOME_CANCELLED
            grantResults[0] == PackageManager.PERMISSION_GRANTED -> null
            else -> OUTCOME_DENIED
        }

        /**
         * Returns the first candidate QR payload that carries a TotalDrop
         * receive link (`https://totaldrop.app/r#…`,
         * `https://totaldrop.app/r/#…` or `totaldrop://r/…`), or `null` so
         * unrelated QR codes in view do not close the camera (#29). Full code
         * validation happens in Rust (`TransferCode::from_qr`).
         */
        internal fun firstQrReceiveLink(candidates: Iterable<String?>): String? {
            for (raw in candidates) {
                if (raw == null || raw.toByteArray(Charsets.UTF_8).size > MAX_QR_LINK_BYTES) {
                    continue
                }
                val cleaned = stripInvisibleMarks(raw).trim()
                if (hasQrReceivePrefix(cleaned)) return cleaned
            }
            return null
        }

        internal fun hasQrReceivePrefix(text: String): Boolean {
            for (prefix in QR_PREFIXES) {
                if (text.length > prefix.length &&
                    text.regionMatches(0, prefix, 0, prefix.length, ignoreCase = true)
                ) {
                    return true
                }
            }
            return false
        }

        private fun stripInvisibleMarks(text: String): String = buildString(text.length) {
            for (ch in text) {
                if (!isInvisibleMark(ch)) append(ch)
            }
        }

        /** Matches `INVISIBLE_FORMAT` in `crates/totaldrop-proto/src/lib.rs`. */
        internal fun isInvisibleMark(ch: Char): Boolean =
            ch == '\u00AD' ||
                ch == '\u061C' ||
                ch in '\u200B'..'\u200F' ||
                ch in '\u202A'..'\u202E' ||
                ch == '\u2060' ||
                ch in '\u2066'..'\u2069' ||
                ch == '\uFEFF'

        private val QR_PREFIXES = arrayOf(
            "https://totaldrop.app/r#",
            "https://totaldrop.app/r/#",
            "totaldrop://r/",
        )
    }
}
