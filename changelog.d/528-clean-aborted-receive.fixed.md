- **Android, Linux, CLI:** Closing the app or aborting a receive while a
  file is arriving removes partial `.part` files and empty created folders
  right away without placing any file after the stop (#528).
