- **Linux, CLI:** `SIGTERM` and `SIGHUP` now stop active transfers cleanly
  and release held router port mappings before exiting; on Linux an
  inherited ignored `SIGHUP` (as under `nohup`) is preserved (#527).
