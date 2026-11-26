- **Core:** Cap the post-stop QUIC endpoint drain wait after
  `Error::Stopped` so a silent or stalled peer does not stack another
  2 s–5 s timeout on top of `STOP_NOTICE` (#654).
