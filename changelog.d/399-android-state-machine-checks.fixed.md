- **Android:** Disposing the send or receive view model while a transfer,
  file pick or clipboard read is still awaiting no longer skips declining a
  late offer or releasing a reserved code, and no longer notifies listeners
  after disposal; resetting and re-entering text mode while a clipboard
  read waits for window focus no longer lets the earlier read overwrite the
  new box (#342, #399).
