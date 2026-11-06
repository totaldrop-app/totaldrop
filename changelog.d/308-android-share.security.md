- **Android:** A share reads only files that other apps offer
  (`content://`), never `file://` addresses or TotalDrop's own files, so
  another app cannot make TotalDrop send its private data (#308).
