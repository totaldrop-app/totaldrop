CI checks the relay edge rate-limit table in server/relay-worker/README.md
and compares it daily and after relay deploys against the live Cloudflare
zone ruleset; nothing in the apps or servers changes (#526).
