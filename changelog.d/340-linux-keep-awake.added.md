- **Linux:** While a send or a receive is under way, TotalDrop keeps the
  computer from going to sleep, so that a transfer is not cut when the
  computer would suspend on idle. The screen may still dim and lock. It
  uses systemd-logind outside the Flatpak and the desktop portal inside
  it (#340).
