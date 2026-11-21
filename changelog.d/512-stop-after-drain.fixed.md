- **Android, Linux, CLI:** When one side stops a transfer and its closing
  packets are delayed or lost until the QUIC connection finishes draining,
  the other side now still learns that the transfer was stopped instead of
  reporting a lost connection ("reset by peer") (#512).
