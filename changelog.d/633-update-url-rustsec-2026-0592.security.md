- **Core:** Update the transitive `url` crate to 2.5.9 (with
  `form_urlencoded` 1.2.3) to resolve RUSTSEC-2026-0592, a stack overflow in
  `Url::origin()` on nested `blob:` URLs (#633).
