- **Server:** The rendezvous server, on Cloudflare and in Rust, now limits
  each client (an IPv4 address or an IPv6 /64) to 60 new codes and
  mailboxes at once and one more every second. A client over the limit is
  told "too many requests". The counts are kept in memory only, under a
  randomly keyed hash rather than the address, and are never written to
  disk or a log (#46).
