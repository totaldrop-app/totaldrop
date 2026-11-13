- **Server:** The relay at relay.totaldrop.app turns away an IP address
  that opens more than 30 connections within 10 seconds, for 10 seconds,
  with HTTP 429. A device holds one relay connection per transfer at a
  time, but everyone behind one address (a large office, carrier-grade
  NAT) shares the limit; a blocked device tries again a few times before
  the transfer gives up (#510).
