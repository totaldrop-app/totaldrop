// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

package app.totaldrop.totaldrop

import android.app.job.JobInfo
import android.app.job.JobParameters
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

/** What a stop of the transfer job by the system or the person does to the
 * transfer (#260, #593). The job's own Android calls are not touched here. */
class TransferJobTest {
    private val notice = TransferJob.Notice("channel", "title", "text", "stop")
    private var stops = 0

    @Before
    fun listen() {
        stops = 0
        TransferJob.onStop = { stops += 1 }
    }

    @After
    fun forget() {
        TransferJob.onStop = null
        TransferJob.clearActive()
    }

    /** Airplane mode, a tunnel, a Wi-Fi that drops: the job loses its
     * network constraint and the system stops it. The transfer goes on and
     * rides out the gap; the job runs again when the network is back. */
    @Test
    fun losingTheNetworkDoesNotStopTheTransfer() {
        val generation = TransferJob.prepare(notice)

        val again = TransferJob.stoppedBySystem(
            generation,
            JobParameters.STOP_REASON_CONSTRAINT_CONNECTIVITY,
        )

        assertEquals("the transfer was stopped", 0, stops)
        assertTrue("the job was not asked to run again", again)
        assertTrue("the job that runs again was refused", TransferJob.accepts(generation))
    }

    /** Every other stop still ends the transfer, such as the system's own
     * limits. (Stop in the system's task manager kills the process without
     * calling onStopJob.) */
    @Test
    fun anyOtherStopStillStopsTheTransfer() {
        val reasons = listOf(
            JobParameters.STOP_REASON_USER,
            JobParameters.STOP_REASON_TIMEOUT,
            JobParameters.STOP_REASON_QUOTA,
            JobParameters.STOP_REASON_SYSTEM_PROCESSING,
            JobParameters.STOP_REASON_UNDEFINED,
        )
        for (reason in reasons) {
            val generation = TransferJob.prepare(notice)

            val again = TransferJob.stoppedBySystem(generation, reason)

            assertFalse("reason $reason: the job runs again", again)
            assertFalse("reason $reason: the job is still accepted", TransferJob.accepts(generation))
        }
        assertEquals(reasons.size, stops)
    }

    /** A job of a transfer that was replaced stops nothing. */
    @Test
    fun aStaleJobStopsNothing() {
        val old = TransferJob.prepare(notice)
        TransferJob.prepare(notice)

        val again = TransferJob.stoppedBySystem(old, JobParameters.STOP_REASON_USER)

        assertFalse(again)
        assertEquals(0, stops)
    }

    /** Known non-negative sizes pass through; missing or negative values map
     * to JobInfo.NETWORK_BYTES_UNKNOWN (#593). */
    @Test
    fun networkBytesMapsKnownAndUnknownValues() {
        val unknown = JobInfo.NETWORK_BYTES_UNKNOWN.toLong()
        assertEquals(unknown, TransferJob.networkBytes(null))
        assertEquals(unknown, TransferJob.networkBytes(-1L))
        assertEquals(0L, TransferJob.networkBytes(0L))
        assertEquals(5_368_709_120L, TransferJob.networkBytes(5_368_709_120L))
    }

    /** Tapping Stop on the notification dismisses the job immediately and
     * still notifies Dart so a stalled socket cannot leave the notice up (#593).
     *
     * This host-JVM test verifies the dismiss-before-stop ordering via the
     * lambda overload; `end(context)`'s `JobScheduler` and `NotificationManager`
     * calls belong to the Android framework and are not executed here. */
    @Test
    fun stoppedByPersonDismissesImmediatelyThenNotifiesDart() {
        val generation = TransferJob.prepare(notice)
        val events = mutableListOf<String>()
        TransferJob.onStop = { events += "stop" }

        TransferJob.stoppedByPerson {
            TransferJob.clearActive()
            events += "dismiss"
        }

        assertEquals(listOf("dismiss", "stop"), events)
        assertFalse(TransferJob.accepts(generation))

        // Without an attached engine, dismissing still clears the active job.
        val orphan = TransferJob.prepare(notice)
        TransferJob.onStop = null
        TransferJob.stoppedByPerson {
            TransferJob.clearActive()
            events += "dismiss-orphan"
        }
        assertEquals(listOf("dismiss", "stop", "dismiss-orphan"), events)
        assertFalse(TransferJob.accepts(orphan))
    }
}
