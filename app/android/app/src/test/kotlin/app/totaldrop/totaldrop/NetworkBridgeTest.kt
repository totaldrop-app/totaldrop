// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

package app.totaldrop.totaldrop

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import java.net.InetAddress

class NetworkBridgeTest {
    @Test
    fun pickIpv4GatewaySelectsUnicastDefaultRouteOnly() {
        val nonDefault = false to InetAddress.getByName("192.168.1.254")
        val v6Default = true to InetAddress.getByName("fe80::1")
        val unspecified = true to InetAddress.getByName("0.0.0.0")
        val loopback = true to InetAddress.getByName("127.0.0.1")
        val multicast = true to InetAddress.getByName("224.0.0.1")
        val broadcast = true to InetAddress.getByName("255.255.255.255")
        val wifiRouter = true to InetAddress.getByName("192.168.1.1")

        assertNull(
            pickIpv4Gateway(
                listOf(nonDefault, v6Default, unspecified, loopback, multicast, broadcast),
            ),
        )
        assertEquals(
            "192.168.1.1",
            pickIpv4Gateway(
                listOf(nonDefault, v6Default, unspecified, wifiRouter),
            ),
        )
    }

    @Test
    fun initialStickyCallbacksAfterSeedEmitNoSpuriousChange() {
        val tracker = NetworkTracker<String>()
        assertEquals(
            "192.168.1.1",
            tracker.seed("wifi", "192.168.1.1", setOf("192.168.1.42", "fe80::2")),
        )
        assertNull(tracker.onAvailable("wifi"))
        assertNull(
            tracker.onLinkProperties("wifi", "192.168.1.1", setOf("192.168.1.42", "fe80::2")),
        )
    }

    @Test
    fun makeBeforeBreakWifiToCellularFiresOnceAndIgnoresLateWifiLost() {
        val tracker = NetworkTracker<String>()
        tracker.seed("wifi", "192.168.1.1", setOf("192.168.1.42"))

        // Cellular becomes the default network before Wi-Fi's onLost arrives.
        assertEquals(
            NetworkTracker.Event(gateway = null, changed = true),
            tracker.onAvailable("cell"),
        )
        // Cellular has no IPv4 router gateway; should not double-fire changed.
        assertNull(tracker.onLinkProperties("cell", null, setOf("100.64.0.9")))
        // Late onLost for the old Wi-Fi must not clobber cellular.
        assertNull(tracker.onLost("wifi"))
    }

    @Test
    fun switchingToAnotherWifiUpdatesGatewayWithoutDoubleChange() {
        val tracker = NetworkTracker<String>()
        tracker.seed("cell", null, setOf("100.64.0.9"))

        assertEquals(
            NetworkTracker.Event(gateway = null, changed = true),
            tracker.onAvailable("office-wifi"),
        )
        assertEquals(
            NetworkTracker.Event(gateway = "10.0.0.1", changed = false),
            tracker.onLinkProperties("office-wifi", "10.0.0.1", setOf("10.0.0.50")),
        )
        // Unchanged RA refresh is ignored.
        assertNull(
            tracker.onLinkProperties("office-wifi", "10.0.0.1", setOf("10.0.0.50")),
        )
        // DHCP address change on the same Wi-Fi triggers re-probing.
        assertEquals(
            NetworkTracker.Event(gateway = "10.0.0.1", changed = true),
            tracker.onLinkProperties("office-wifi", "10.0.0.1", setOf("10.0.0.99")),
        )
    }

    @Test
    fun breakBeforeMakeSignalsBothLossAndRecovery() {
        val tracker = NetworkTracker<String>()
        tracker.seed("wifi", "192.168.1.1", setOf("192.168.1.42"))

        assertEquals(
            NetworkTracker.Event(gateway = null, changed = true),
            tracker.onLost("wifi"),
        )
        assertEquals(
            NetworkTracker.Event(gateway = null, changed = true),
            tracker.onAvailable("cell"),
        )
        assertNull(tracker.onLinkProperties("cell", null, setOf("100.64.0.9")))
    }
}
