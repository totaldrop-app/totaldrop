- **Command line:** Folder paths in receive errors and the save prompt,
  and arguments in usage errors, now name control and direction-override
  characters (`<U+001B>`, `<U+202E>`) instead of passing them raw to the
  terminal, and a compile-time Clippy rule requires every terminal line to
  go through the sanitiser (#399).
