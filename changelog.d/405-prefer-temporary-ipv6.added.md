- **Core:** On Linux, prefer a usable temporary IPv6 address (RFC 8981,
  `IFA_F_TEMPORARY`) from `/proc/net/if_inet6` over a stable-privacy or
  EUI-64 source on the same interface and `/64` prefix for `HINTS`, IPv6
  STUN probes, and outbound QUIC datagrams via `IPV6_PKTINFO`, falling
  back safely when no temporary address is available. On all platforms,
  outbound QUIC datagrams now route through an `IPV6_PKTINFO`-aware UDP
  wrapper and both direct QUIC sends and IPv6 STUN probes refuse to fall
  back to a modified EUI-64 default IPv6 source (#405).
