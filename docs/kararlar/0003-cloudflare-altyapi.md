# 0003: Infrastructure on Cloudflare

- **Status:** Accepted, 28 October 2026
- **Context:** Devices on different networks need address discovery (STUN)
  and a relay server that carries the encrypted traffic when no direct path
  opens. Options: our own server (Hetzner), Cloudflare Workers, Cloudflare
  Realtime TURN.
- **Decision:** The site, rendezvous and the relay run on Cloudflare.
  Cloudflare's free STUN for address discovery; for the relay, one Durable
  Object per transfer, over WebSocket. For those who want to run it on
  their own server, a Rust relay server that speaks the same protocol stays
  in the repository.
- **Why:** Everything is managed in one place; there is no new machine to
  update and protect. Worker egress is free, so the cost is close to zero.
  Measured locally, 100 MB over the relay alone took 8 seconds; on the real
  internet 28 seconds (both ends were on the same home connection).
- **Review condition:** If the relay is too slow, or Cloudflare's terms of
  use cause a problem, Cloudflare Realtime TURN is added as a second path
  (the first 1,000 GB a month free, then USD 0.05 per GB). Its cost is
  covered by donations.
- **Consequences:** Cloudflare sees IP addresses; the privacy policy says
  so. Privacy mode (relay only, Tor later) is for those who also want to
  hide their IP.
