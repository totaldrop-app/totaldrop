# 0019: The repository is written in English

- **Status:** Accepted, 11 November 2026 (Ozan, #490)
- **Context:** Code, comments, commit messages, `README.md`,
  `CONTRIBUTING.md` and the protocol notes were already in English, but the
  process and design documents in `docs/`, the decision records, the agent
  instructions (`CLAUDE.md`) and parts of the pull request and issue
  templates were in Turkish (`CLAUDE.md`, "Belgeler ve kullanıcıya yazılan
  metinler Türkçe"; `CONTRIBUTING.md`, "Process and design documents in
  `docs/` are in Turkish"). The maintainers will invite people into the
  process through GitHub, and most of them read English, not Turkish.
- **Options:**
  1. Keep the documents in Turkish and translate on request. Every new
     contributor depends on a translation that can be out of date.
  2. Keep both languages side by side. Every change has to be made twice,
     and the two versions drift apart.
  3. Write the repository in English only.
- **Decision:** Option 3. Everything in the repository is written in
  English: documents, decision records, templates, agent instructions and
  skills, changelog entries, and from now on every issue, pull request,
  review comment and commit that the maintainers and their assistants
  write. File and folder names follow, in one change
  at the end of the move (#490).
- **Why:** The repository is where new contributors learn how the project
  works and why it made its choices. One language that most of them read
  lets them take part in reviews and decisions without a translator, and
  keeps a single version of every rule.
- **Not affected:**
  - Text that users read in the apps, on the site, in the store listings
    and in the packages stays in every supported language
    ([0008](0008-diller-ve-bolgeler.md)), and so do the translated READMEs
    under `docs/readme/`.
  - Anyone may still open an issue in their own language; the maintainers
    answer in English.
  - Conversations between the maintainers and their assistants may be in
    any language; what reaches the repository is in English.
  - Old issues, pull requests and commits are not rewritten.
- **Consequences:**
  - The Turkish documents are translated in small pull requests (#490),
    so that the open pull requests that touch `docs/` are not rebased over
    one large change. Until that is done, a Turkish document is still
    binding; its English translation replaces it, word for word in what it
    requires.
  - The definition of done in [surec.md](../surec.md) and the pull request
    template ask for English documents instead of Turkish ones.
