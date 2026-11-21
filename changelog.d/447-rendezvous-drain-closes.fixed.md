- **Server:** When the Rust rendezvous server ends a mailbox connection
  itself, such as when a code expires or a side sends too many messages,
  Windows clients now receive the reason instead of "connection aborted"
  when messages or keep-alive pings are still in flight: every close the
  server starts now waits up to 2 seconds for the client to close (#447).
