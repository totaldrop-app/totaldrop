- **CI:** The site and the rendezvous and relay Workers deploy from CI, from
  main only. The commit's checks run again, then the owner approves the run
  in the `cloudflare` environment. Only what changed is deployed. A probe
  then opens a real WebSocket to each Worker, reaches the rendezvous
  directory without taking a number, and loads the site's pages.
  docs/surec.md describes the one-time setup and how to roll back (#94).
