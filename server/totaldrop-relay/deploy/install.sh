#!/bin/sh
# Set up or update the relay on a fresh Debian 13 server. Run as root:
#   scp target/release/totaldrop-relay deploy/* root@HOST:/tmp/relay/
#   ssh root@HOST sh /tmp/relay/install.sh relay.example.org
set -eu
here=$(cd "$(dirname "$0")" && pwd)

# The name clients use for this relay; Caddy gets its certificate for it.
usage() {
  echo "usage: install.sh HOST   (a lower-case DNS name such as relay.example.org; punycode for IDN)" >&2
  exit 2
}
[ $# -eq 1 ] || usage
host=$1
[ ${#host} -le 253 ] || usage
case "$host" in
  *[!a-z0-9.-]*) usage ;;
  *.*) ;;
  *) usage ;;
esac
# Each label: 1-63 of a-z, 0-9 and -, not starting or ending with -.
for label in $(echo "$host" | tr . ' '); do
  case "$label" in
    "" | -* | *- | *[!a-z0-9-]*) usage ;;
  esac
  [ ${#label} -le 63 ] || usage
done
case "$host" in .* | *. | *..*) usage ;; esac

# Write and check the Caddy configuration before anything is installed.
tmp=$(mktemp /etc/caddy/Caddyfile.XXXXXX 2>/dev/null || mktemp)
trap 'rm -f "$tmp"' EXIT
sed "s/^RELAY_HOST {/$host {/" "$here/Caddyfile" > "$tmp"
grep -q "^$host {" "$tmp" || { echo "RELAY_HOST placeholder not found in Caddyfile" >&2; exit 1; }

if ! command -v caddy >/dev/null; then
  apt-get update -q
  apt-get install -yq caddy nftables unattended-upgrades
fi

caddy validate --config "$tmp" --adapter caddyfile
chmod 0644 "$tmp"

install -m 0755 "$here/totaldrop-relay" /usr/local/bin/totaldrop-relay
install -m 0644 "$here/totaldrop-relay.service" /etc/systemd/system/totaldrop-relay.service
install -m 0644 "$here/nftables.conf" /etc/nftables.conf
mv "$tmp" /etc/caddy/Caddyfile

systemctl daemon-reload
systemctl enable --now nftables
nft -f /etc/nftables.conf
systemctl enable totaldrop-relay caddy
systemctl restart totaldrop-relay
systemctl reload caddy || systemctl restart caddy
systemctl --no-pager --lines=3 status totaldrop-relay caddy
