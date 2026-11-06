// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

package app.totaldrop.totaldrop

/**
 * Events for the Dart side (a share, a shortcut), kept until it listens:
 * the share that starts the app arrives before Flutter has drawn anything.
 * Used on the main thread only.
 */
class Launches {
    private val waiting = ArrayList<Map<String, Any>>()
    private var listener: ((Map<String, Any>) -> Unit)? = null

    fun send(event: Map<String, Any>) {
        val l = listener
        if (l != null) {
            l(event)
        } else {
            if (waiting.size == MAX_WAITING) waiting.removeAt(0)
            waiting.add(event)
        }
    }

    /** Starts delivering to [l]; returns what came before, oldest first. */
    fun attach(l: (Map<String, Any>) -> Unit): List<Map<String, Any>> {
        listener = l
        val before = ArrayList(waiting)
        waiting.clear()
        return before
    }

    fun detach() {
        listener = null
    }

    companion object {
        const val MAX_WAITING = 16
    }
}
