- **Android, Linux, CLI:** A transfer that started on a direct connection
  no longer stops with "timed out" when that path goes quiet, for example
  when a phone's Wi-Fi drops: after 8 seconds without anything from the
  other device, it moves onto the relay and goes on (#185).
