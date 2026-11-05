- **Relay (Cloudflare):** The relay now bounds what it holds for a device
  that does not read: before, a modified client could make it keep any
  amount in memory. Devices from before 0.3.0-beta.4 lose a transfer
  through this relay after 32 MiB reaches them: a receiver after 32 MiB
  of the file, a sender later (an estimated 1 GiB of file, since only
  acknowledgements reach it). Update both devices (#190).
