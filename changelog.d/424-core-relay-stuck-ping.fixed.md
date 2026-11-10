- **All platforms:** A transfer that falls back to the relay no longer hangs for
  minutes when the network drops while the connection to the relay is
  full: the keep-alive ping and the closing of the connection now give up
  after 10 seconds, as every other write to the relay already did, and
  the app connects again (#424).
