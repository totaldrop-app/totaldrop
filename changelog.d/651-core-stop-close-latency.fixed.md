- **Core:** Prioritize the QUIC control stream over bulk file streams,
  overlap sender stream resets with stop cancellation, halt abandoned
  receive streams before `fsync`, and close cancelled connections before
  local disk cleanup so stopped relay transfers stay within the 1.5 s
  window-close budget (#651).
