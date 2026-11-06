- **Android:** A share reads only files that another app serves
  (`content://`). Addresses served by TotalDrop itself, `file://`
  addresses and addresses with a user prefix are refused, so another app
  cannot make TotalDrop send its private data, such as its device key.
  Copies of shared files are kept out of backups and deleted when the
  selection is cleared, or at the latest when the app starts again
  (#308).
