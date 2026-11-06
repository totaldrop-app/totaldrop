// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

package app.totaldrop.totaldrop

import org.junit.Assert.assertEquals
import org.junit.Test

/** A share that starts the app arrives before Flutter listens (#53). */
class LaunchesTest {
    @Test
    fun eventsBeforeListeningAreKeptInOrder() {
        val l = Launches()
        l.send(mapOf("screen" to "send", "copying" to true))
        l.send(mapOf("files" to emptyList<Any>()))
        val got = ArrayList<Map<String, Any>>()
        val before = l.attach { got.add(it) }
        assertEquals(listOf(mapOf("screen" to "send", "copying" to true), mapOf("files" to emptyList<Any>())), before)
        l.send(mapOf("screen" to "receive"))
        assertEquals(listOf(mapOf("screen" to "receive")), got)
    }

    @Test
    fun onlyTheLatestEventsAreKept() {
        val l = Launches()
        repeat(Launches.MAX_WAITING + 4) { l.send(mapOf("n" to it)) }
        val before = l.attach {}
        assertEquals(Launches.MAX_WAITING, before.size)
        assertEquals(mapOf("n" to 4), before.first())
    }

    @Test
    fun afterDetachEventsWaitAgain() {
        val l = Launches()
        l.attach {}
        l.detach()
        l.send(mapOf("screen" to "send"))
        assertEquals(listOf(mapOf("screen" to "send")), l.attach {})
    }
}
