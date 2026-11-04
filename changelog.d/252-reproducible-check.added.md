- **CI:** A workflow builds each Linux package twice, from two clean
  checkouts, and fails if the two differ. It runs weekly, by hand, and on
  pull requests that change the packaging. The one known difference, the
  order cxx-qt links the Qt libraries, is reported as a warning
  (#252, #256).
