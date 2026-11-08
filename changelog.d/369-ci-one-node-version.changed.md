- **CI:** Every workflow runs the same fixed Node major (24) instead of
  following the newest LTS, `@types/node` keeps that major, and a check
  fails a pull request where they disagree. Dependabot no longer proposes
  `@types/node` majors on their own; a Node upgrade is one pull request
  (#369).
