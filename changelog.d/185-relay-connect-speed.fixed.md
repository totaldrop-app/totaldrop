- **Android, Linux, CLI:** A transfer through the relay connects even when
  the relay is slow to get going, instead of failing after 20 seconds, and
  carries on if the connection to the relay drops for a moment. It is also
  several times faster when the round trip to the relay is long (#185).
