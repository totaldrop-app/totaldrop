- **Linux:** closing the window during a transfer now stops the transfer
  and closes the port it opened on the router before the app exits,
  waiting at most 3 seconds; the command line does the same after an
  internal error. Before, the port stayed open until the router's lease
  ran out (two minutes asked) (#433).
