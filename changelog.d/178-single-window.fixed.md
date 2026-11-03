- **Linux:** Opening files with TotalDrop while it is already open, for
  example with "Open with" in a file manager, adds them to the open
  window instead of starting a second one; a link or `--screen` does the
  same, and the window comes to the front. During a transfer, or while
  its result is shown, the request waits until the window is back to
  idle; a screen with files listed, a code typed or a server address not
  saved stays under the new one. The window owns the D-Bus name
  `app.totaldrop.TotalDrop` and serves `org.freedesktop.Application`. A
  relative path given on the command line now opens too (#178).
