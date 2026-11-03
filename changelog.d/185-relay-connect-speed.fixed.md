- **Android, Linux, CLI:** A transfer through the relay connects even when
  the relay is slow to get going, instead of failing after 20 seconds, and
  the connection to the relay is made again when the relay closes it. It
  is also faster when the round trip to the relay is long, because more
  data may be on the way at once (#185).
