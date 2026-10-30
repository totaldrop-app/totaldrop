- **Core, CLI (tests):** Two tests failed now and then on slow CI runners.
  The end-to-end test of a receiver stopped while it connects now stops it
  before any reply from the server can arrive, so it always ends in
  "stopped" and no longer sometimes reads the departed sender's mailbox
  and ends in "wrong code". The privacy-mode test now runs on a paused
  clock and checks that the relay connection comes up before the sender's
  own dial would start, instead of timing the connection on the wall clock
  (#126).
