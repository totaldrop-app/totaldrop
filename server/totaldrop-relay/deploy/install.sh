#!/bin/sh
# Set up or update the relay on a fresh Debian 13 server. Run as root:
#   scp target/release/totaldrop-relay deploy/* root@HOST:/tmp/relay/
#   ssh root@HOST sh /tmp/relay/install.sh
set -eu
here=$(cd "$(dirname "$0")" && pwd)

if ! command -v caddy >/dev/null; then
  apt-get update -q
  apt-get install -yq caddy nftables unattended-upgrades
fi

install -m 0755 "$here/totaldrop-relay" /usr/local/bin/totaldrop-relay
install -m 0644 "$here/totaldrop-relay.service" /etc/systemd/system/totaldrop-relay.service
install -m 0644 "$here/Caddyfile" /etc/caddy/Caddyfile
install -m 0644 "$here/nftables.conf" /etc/nftables.conf

systemctl daemon-reload
systemctl enable --now nftables
nft -f /etc/nftables.conf
systemctl enable totaldrop-relay caddy
systemctl restart totaldrop-relay
systemctl reload caddy || systemctl restart caddy
systemctl --no-pager --lines=3 status totaldrop-relay caddy
