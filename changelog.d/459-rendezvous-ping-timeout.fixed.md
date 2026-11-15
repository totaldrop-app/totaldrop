- **All platforms:** Waiting for a code no longer hangs for minutes when
  the connection to the rendezvous server stalls with a full send buffer:
  client writes, the keep-alive ping included, now give up after 10
  seconds (#459).
