- **Linux (Flatpak):** `totaldrop receive` without `-o` saved into the
  sandbox's memory-only home folder and said "Saved", and the file was
  gone when the command ended. It now saves into `~/Downloads/TotalDrop`,
  the same folder as the window, and refuses a folder that Flatpak would
  not keep before the code is used up (#350).
