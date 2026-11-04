- **CI:** A workflow builds each Linux package twice, from two clean
  checkouts, and fails if the two differ. It runs weekly, by hand, and on
  pull requests that change the packaging (#252).
