- **Server:** The relay at relay.totaldrop.app turns away an IP address
  that opens more than 30 connections within 10 seconds, for 10 seconds,
  with HTTP 429. A transfer opens one per device, so ordinary use never
  reaches it; the apps try again by themselves (#510).
