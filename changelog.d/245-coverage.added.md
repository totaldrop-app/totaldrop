- **CI:** Line coverage of the protocol crates, the command line and the
  Rust servers is measured with cargo-llvm-cov, and CI fails below 90%.
  New tests run the `totaldrop` program itself: sending a folder,
  answering the question, declining, and its error exits (#245).
