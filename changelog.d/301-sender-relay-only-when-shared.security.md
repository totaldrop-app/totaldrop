- **Android, Linux, CLI:** A sending device no longer connects to its relay
  when the receiving device uses a different relay or none, so that relay
  no longer sees the sender's IP address for a transfer it could never
  carry. A receiver on 0.1 set to another relay, which used to fall back
  to the sender's relay anyway, now needs a direct path (#301).
