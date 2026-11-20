- **Server:** Both relays (Cloudflare and Rust) now accept an
  `X-TotalDrop-Auth` Ed25519 signature on WebSocket Upgrade, bound to the
  pair name, so an outsider who knows a pair name cannot hold waiting
  slots and block the real peer with HTTP 409; invalid headers get HTTP
  403, and clients now send the header while older beta clients without it
  keep working (#78).
