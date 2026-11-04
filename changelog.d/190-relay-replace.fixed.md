- **Android, Linux, CLI, relay:** A transfer through the relay survives a
  connection that dies without either end noticing, for example after
  switching Wi-Fi networks: the device notices within 10 seconds and
  connects again, and the relay now accepts the new connection at once
  instead of refusing it until the transfer times out. Both relays now end
  the other side in the same cases (#190).
