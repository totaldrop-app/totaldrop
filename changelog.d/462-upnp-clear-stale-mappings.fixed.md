- **Core:** Before opening a UPnP port on the router, TotalDrop checks up
  to 64 existing mappings and removes leftover `TotalDrop` UDP rules for
  this device's address whose local UDP port is no longer in use, leaving
  other programs' rules, other devices' rules, and any live transfer
  untouched (#462).
