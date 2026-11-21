- **Linux:** Closing the window during a transfer now finishes telling the
  other device before the app exits, even if the window tears down its
  transfer view first, so the other side shows that the transfer was
  stopped rather than a connection error (#359).
- **Android, Linux, CLI:** Stopping a transfer no longer waits extra
  seconds after the other device has already closed the connection (#359).
