- **Linux:** While a send or a receive is under way, TotalDrop asks the
  system not to suspend, so that a transfer is not cut when the computer
  would go to sleep on idle. The screen may still dim and lock. A suspend
  you ask for yourself may be refused or ask for confirmation while a
  transfer runs. It uses systemd-logind in the system packages and the
  desktop portal in the Flatpak, where on Hyprland and Omarchy the portal
  may not hold sleep (#340).
