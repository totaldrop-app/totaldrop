- **Command line:** When a send or receive fails, the detail of the error
  names control characters (`<U+001B>`) instead of passing them to the
  terminal. The reason the other device gives for cancelling, or for
  closing the connection, could otherwise move the cursor, rewrite earlier
  lines or set the window title (GHSA-c6jh-7p7q-8gg4).
