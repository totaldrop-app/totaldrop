- **Server:** The rendezvous server and the relay, on Cloudflare and in
  Rust, refuse a WebSocket that a web page opens (one that carries an
  `Origin` header). The apps and the command line never send one, so a
  page can no longer open connections from its visitors' browsers and
  spend their per-client limits (#450).
