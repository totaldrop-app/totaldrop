- **All platforms:** A relay that accepts sign-in and then stalls is no
  longer retried for hours: the link now gives up after 10 sessions in a
  row that ended without carrying a datagram from the other device (#460).
