- **Command line:** When the command exits right after a transfer, the
  request asking the router to close the port now goes out twice, as it
  does while the app keeps running, so one lost packet no longer leaves
  the port open until the router's lease runs out (#502).
