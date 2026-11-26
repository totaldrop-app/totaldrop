- **Linux:** On X11 (KDE Plasma on X11, GNOME on Xorg, Xfce, Cinnamon),
  opening files or TotalDrop again while its window is already open now
  finishes the desktop's startup notification so the busy cursor stops at
  once, and passes the launch's timestamp so focus-stealing prevention lets
  the window come to the front (#184).
