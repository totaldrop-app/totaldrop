- **Android:** The transfer percent is written the way the device's
  region writes it, as the sizes beside it already are: "%50" with a
  Turkish region, "50 %" with a German one. It is rounded down, so it no
  longer reads 100% while the last bytes are still arriving, and a screen
  reader now hears the same number (#483).
