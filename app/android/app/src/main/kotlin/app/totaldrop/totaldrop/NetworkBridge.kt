// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

package app.totaldrop.totaldrop

import android.content.Context
import android.net.ConnectivityManager
import android.net.LinkProperties
import android.net.Network
import android.os.Handler
import android.os.Looper
import java.net.Inet4Address
import java.net.InetAddress

/** Picks the first unicast IPv4 default gateway from `(isDefaultRoute, gateway)`
 * pairs (#406, decision 0018). Point-to-point cellular routes (`0.0.0.0`),
 * loopback, multicast, broadcast, and IPv6 gateways are ignored. */
internal fun pickIpv4Gateway(routes: Iterable<Pair<Boolean, InetAddress?>>): String? {
    for ((isDefault, gateway) in routes) {
        if (!isDefault || gateway !is Inet4Address) continue
        if (gateway.isAnyLocalAddress ||
            gateway.isLoopbackAddress ||
            gateway.isMulticastAddress
        ) {
            continue
        }
        val host = gateway.hostAddress ?: continue
        if (host == "255.255.255.255") continue
        return host
    }
    return null
}

/** Pure state machine for Android's default-network callbacks (#406, #32).
 *
 * Kept free of Android framework calls so unit tests exercise every handoff
 * ordering directly on the JVM:
 * - Initial seed / sticky attach reports `changed = false`.
 * - Make-before-break (`onAvailable(netB)` -> `onLinkProperties(netB)` ->
 *   late `onLost(netA)`) emits `changed = true` once on `netB`, updates the
 *   gateway when `netB`'s properties arrive without firing a duplicate
 *   `changed = true`, and ignores the stale `onLost(netA)`.
 * - Break-before-make (`onLost(netA)` -> `onAvailable(netB)`) emits
 *   `changed = true` on both transitions so a direct transfer moves onto the
 *   relay immediately and the relay link wakes from backoff as soon as the
 *   new network comes up.
 * - Periodic IPv6 RA / DNS refreshes on the same network with unchanged
 *   gateway and unicast addresses emit nothing. */
internal class NetworkTracker<N> {
    data class Event(val gateway: String?, val changed: Boolean)

    private var initialized = false
    private var active: N? = null
    private var gateway: String? = null
    private var addresses: Set<String> = emptySet()
    private var awaitingInitialProps = false

    @Synchronized
    fun seed(network: N?, gateway: String?, addresses: Set<String>): String? {
        initialized = true
        active = network
        this.gateway = if (network != null) gateway else null
        this.addresses = if (network != null) addresses else emptySet()
        awaitingInitialProps = false
        return this.gateway
    }

    @Synchronized
    fun onAvailable(network: N): Event? {
        if (!initialized) {
            initialized = true
            active = network
            gateway = null
            addresses = emptySet()
            awaitingInitialProps = true
            return null
        }
        if (active == network) return null
        active = network
        gateway = null
        addresses = emptySet()
        awaitingInitialProps = true
        return Event(gateway = null, changed = true)
    }

    @Synchronized
    fun onLinkProperties(
        network: N,
        gateway: String?,
        addresses: Set<String>,
    ): Event? {
        if (!initialized) {
            initialized = true
            active = network
            this.gateway = gateway
            this.addresses = addresses
            awaitingInitialProps = false
            return Event(gateway = gateway, changed = false)
        }
        if (active != network) return null
        if (awaitingInitialProps) {
            awaitingInitialProps = false
            val gwChanged = this.gateway != gateway
            this.gateway = gateway
            this.addresses = addresses
            return if (gwChanged) Event(gateway = gateway, changed = false) else null
        }
        if (this.gateway == gateway && this.addresses == addresses) return null
        this.gateway = gateway
        this.addresses = addresses
        return Event(gateway = gateway, changed = true)
    }

    @Synchronized
    fun onLost(network: N): Event? {
        if (active != network) return null
        val wasInitialized = initialized
        initialized = true
        active = null
        gateway = null
        addresses = emptySet()
        awaitingInitialProps = false
        return Event(gateway = null, changed = wasInitialized)
    }
}

/** Bridges `ConnectivityManager` default-network updates to Flutter (#406, #32). */
internal class NetworkBridge(
    context: Context,
    private val onEvent: (NetworkTracker.Event) -> Unit,
) {
    private val appContext = context.applicationContext
    private val main = Handler(Looper.getMainLooper())
    private val tracker = NetworkTracker<Network>()
    private var callback: ConnectivityManager.NetworkCallback? = null

    /** Snapshot the current default network's IPv4 gateway and begin watching
     * default-network changes. Safe to call more than once. */
    fun start(): String? {
        val cm = appContext.getSystemService(ConnectivityManager::class.java)
            ?: return tracker.seed(null, null, emptySet())
        val current = runCatching { cm.activeNetwork }.getOrNull()
        val (gw, addrs) = snapshot(runCatching { current?.let(cm::getLinkProperties) }.getOrNull())
        val seeded = tracker.seed(current, gw, addrs)
        if (callback == null) {
            val cb = object : ConnectivityManager.NetworkCallback() {
                override fun onAvailable(network: Network) {
                    tracker.onAvailable(network)?.let(::emit)
                }

                override fun onLinkPropertiesChanged(
                    network: Network,
                    linkProperties: LinkProperties,
                ) {
                    val (nextGw, nextAddrs) = snapshot(linkProperties)
                    tracker.onLinkProperties(network, nextGw, nextAddrs)?.let(::emit)
                }

                override fun onLost(network: Network) {
                    tracker.onLost(network)?.let(::emit)
                }
            }
            if (runCatching { cm.registerDefaultNetworkCallback(cb) }.isSuccess) {
                callback = cb
            }
        }
        return seeded
    }

    fun stop() {
        val cb = callback ?: return
        callback = null
        val cm = appContext.getSystemService(ConnectivityManager::class.java) ?: return
        runCatching { cm.unregisterNetworkCallback(cb) }
    }

    private fun emit(event: NetworkTracker.Event) {
        main.post {
            if (callback != null) onEvent(event)
        }
    }

    companion object {
        private fun snapshot(lp: LinkProperties?): Pair<String?, Set<String>> {
            if (lp == null) return null to emptySet()
            val routes = lp.routes.map { it.isDefaultRoute to it.gateway }
            val gateway = pickIpv4Gateway(routes)
            val addresses = lp.linkAddresses
                .mapNotNull { it.address?.hostAddress }
                .toSet()
            return gateway to addresses
        }
    }
}
