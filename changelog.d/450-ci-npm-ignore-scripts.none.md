CI installs the Workers' packages without running their install scripts,
so none can change wrangler before the deploy step gets the Cloudflare
token; packaging/check-versions.sh keeps it so. Nothing in the apps or
servers changes (#450).
