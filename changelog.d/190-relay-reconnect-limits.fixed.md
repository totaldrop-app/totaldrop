- **Server:** The relay, on Cloudflare and in Rust, now counts the 32 GiB
  byte limit per key and the 6-hour time limit per pair across reconnects,
  instead of starting over on each connection (#190).
