- **Linux, CLI:** closing the window during a transfer now stops the transfer
  and asks the router to close the port it opened before the app exits,
  in at most a few seconds; the command line does the same after an
  internal error. Before, the port stayed open until the router's lease
  ran out (two minutes asked) (#433).
