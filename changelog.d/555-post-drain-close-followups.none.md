Follow-up cleanups on post-drain `CONNECTION_CLOSE` replay: pass
`Fallback` directly instead of a process-wide socket table, rank closing
destination connection IDs by pre-close activity so a dual-dial loser
cannot shadow the winning connection, move the post-drain wait off the
caller's stop path and tie lingering stops into shutdown release, pin the
8-byte QUIC connection ID length explicitly, and tighten the packet-loss
tests (#555).
