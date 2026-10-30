- **CLI, translations:** The translation check rejected only the English
  no-words (n, no, not) among a language's yes answers (`cliYes`), so a
  draft could make a word for no, such as нет, nein or لا, mean yes. Each
  language now lists its words for no in `l10n/status.json` (`noWords`),
  and the check rejects a yes answer that is one of them, or the answer
  for no in a save prompt it can be shown with (`[д/Н]`, `[ن/ل]`). A save
  prompt must show ASCII `[yes/no]` answers, and its answer for yes must
  be in `cliYes` or be y or yes. Answers are compared exactly as the
  command line compares them: `fold_answer`, `is_yes` and `ALWAYS_YES` are
  now public in `totaldrop-l10n`, and the command line uses them (#129).
