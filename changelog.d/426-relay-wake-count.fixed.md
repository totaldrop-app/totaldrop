- **Server:** After the Cloudflare relay wakes from hibernation, it no
  longer counts up to 4 MiB too little for a device that had not answered
  a probe yet, so its 32 MiB and 64 MiB limits on unread data hold there
  too (#426).
