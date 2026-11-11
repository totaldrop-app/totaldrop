- **Server:** A client over its connection rate is told "too many
  requests" on Windows too. The Rust rendezvous server closed a refused
  connection with the client's first message unread, which reset the
  connection, and Windows dropped the refusal before the client read it
  ("connection aborted"). The server now waits up to 2 seconds for the
  client to close (#442).
