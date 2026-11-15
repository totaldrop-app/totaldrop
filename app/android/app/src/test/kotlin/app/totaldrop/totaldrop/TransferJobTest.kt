// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

package app.totaldrop.totaldrop

import android.app.job.JobParameters
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

/** What a stop of the transfer job by the system does to the transfer
 * (#260) and how radio/CPU locks follow its lifecycle (#340). The job's own
 * Android framework calls are replaced with [FakeLocks] here. */
class TransferJobTest {
    private class FakeLocks : TransferJob.Locks {
        var acquires = 0
        var releases = 0
        var held = false

        override fun acquire() {
            acquires += 1
            held = true
        }

        override fun release() {
            releases += 1
            held = false
        }
    }

    private val notice = TransferJob.Notice("channel", "title", "text", "stop")
    private var stops = 0
    private lateinit var locks: FakeLocks

    @Before
    fun listen() {
        stops = 0
        locks = FakeLocks()
        TransferJob.locks = locks
        TransferJob.onStop = { stops += 1 }
    }

    @After
    fun forget() {
        TransferJob.finishTransfer()
        TransferJob.locks = null
        TransferJob.onStop = null
    }

    /** Airplane mode, a tunnel, a Wi-Fi that drops: the job loses its
     * network constraint and the system stops it. The transfer goes on and
     * rides out the gap; the job runs again when the network is back, and
     * the locks stay held across the gap (#260, #340). */
    @Test
    fun losingTheNetworkDoesNotStopTheTransferOrReleaseLocks() {
        val generation = TransferJob.prepare(notice)
        assertTrue(locks.held)
        assertEquals(1, locks.acquires)

        val again = TransferJob.stoppedBySystem(
            generation,
            JobParameters.STOP_REASON_CONSTRAINT_CONNECTIVITY,
        )

        assertEquals("the transfer was stopped", 0, stops)
        assertTrue("the job was not asked to run again", again)
        assertTrue("the job that runs again was refused", TransferJob.accepts(generation))
        assertTrue("locks were dropped during connectivity gap", locks.held)
        assertEquals(0, locks.releases)
    }

    /** Every other stop still ends the transfer and releases the locks, such
     * as the system's own limits. (Stop in the system's task manager kills
     * the process without calling onStopJob.) */
    @Test
    fun anyOtherStopStillStopsTheTransferAndReleasesLocks() {
        val reasons = listOf(
            JobParameters.STOP_REASON_USER,
            JobParameters.STOP_REASON_TIMEOUT,
            JobParameters.STOP_REASON_QUOTA,
            JobParameters.STOP_REASON_SYSTEM_PROCESSING,
            JobParameters.STOP_REASON_UNDEFINED,
        )
        for (reason in reasons) {
            val generation = TransferJob.prepare(notice)
            assertTrue("reason $reason: locks were not acquired", locks.held)

            val again = TransferJob.stoppedBySystem(generation, reason)

            assertFalse("reason $reason: the job runs again", again)
            assertFalse("reason $reason: the job is still accepted", TransferJob.accepts(generation))
            assertFalse("reason $reason: locks were left held", locks.held)
        }
        assertEquals(reasons.size, stops)
        assertEquals(reasons.size, locks.acquires)
        assertEquals(reasons.size, locks.releases)
    }

    /** Ending a transfer normally releases the locks and rejects the finished
     * generation. */
    @Test
    fun finishingTheTransferReleasesLocks() {
        val generation = TransferJob.prepare(notice)
        assertTrue(locks.held)

        TransferJob.finishTransfer()

        assertFalse(locks.held)
        assertEquals(1, locks.releases)
        assertFalse(TransferJob.accepts(generation))
    }

    /** A job of a transfer that was replaced stops nothing and leaves the
     * replacement transfer's locks held. */
    @Test
    fun aStaleJobStopsNothingAndKeepsReplacementLocks() {
        val old = TransferJob.prepare(notice)
        val current = TransferJob.prepare(notice)

        val again = TransferJob.stoppedBySystem(old, JobParameters.STOP_REASON_USER)

        assertFalse(again)
        assertEquals(0, stops)
        assertTrue(TransferJob.accepts(current))
        assertTrue(locks.held)
        assertEquals(0, locks.releases)
    }

    /** The partial wake lock is bounded to 6 hours and the manifest declares
     * both install-time lock permissions (#340). */
    @Test
    fun wakeLockCapAndManifestPermissionsAreConfigured() {
        assertEquals(6L * 60 * 60 * 1000, TransferJob.MAX_LOCK_MS)
        val candidates = listOf(
            java.io.File("src/main/AndroidManifest.xml"),
            java.io.File("app/src/main/AndroidManifest.xml"),
        )
        val manifest = candidates.first { it.isFile }.readText()
        assertTrue(
            manifest.contains("""android:name="android.permission.WAKE_LOCK""""),
        )
        assertTrue(
            manifest.contains("""android:name="android.permission.ACCESS_WIFI_STATE""""),
        )
    }
}
