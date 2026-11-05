- **Relay:** The Cloudflare relay now bounds what it holds for a device
  that does not read: before, a modified client could make it keep any
  amount in memory. Through this relay, devices from before 0.3.0-beta.4
  now receive at most 32 MiB per transfer; update to send larger files
  (#190).
