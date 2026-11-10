- **Linux:** During a transfer, TotalDrop asks the home router to open a
  port for it, using PCP or NAT-PMP where the router offers them, so the
  other device can connect directly more often instead of through the
  relay. It asks for two minutes at a time, renews the port while the
  transfer runs and closes it when the transfer ends; if TotalDrop stops
  unexpectedly, the router closes it once the time it granted runs out.
  Privacy mode asks the router nothing (#406).
