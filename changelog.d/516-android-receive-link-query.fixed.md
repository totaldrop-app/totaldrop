- **Android:** Receive-link classification now runs strictly in
  `totaldrop-proto` instead of Dart, so opening a `https://totaldrop.app/r`
  or `totaldrop://r` link with a query string shows the invalid-code
  message on Receive instead of an empty field (#516).
