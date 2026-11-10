- **Transfers:** When a direct connection that had gone quiet comes back,
  the transfer picks up within a second instead of standing still for 10
  seconds or more. The side that had moved onto the relay sends its last
  few packets once more on the direct path, which makes QUIC resend what
  was lost at once (#435).
