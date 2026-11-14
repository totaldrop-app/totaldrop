A code-level architecture map (docs/ARCHITECTURE.md) and CI checks that
keep it true: the crates' dependency direction must match both the table
in tools/check-layers.mjs and the map's layer diagram, and every
`path:line` in the map is checked against the text it was written for
(warning when lines move, failing when a cited line changes); nothing
changes for people who use TotalDrop.
