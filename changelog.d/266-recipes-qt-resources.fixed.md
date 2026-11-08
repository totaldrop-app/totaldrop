- **Linux:** The Debian, RPM and Arch recipes keep build times out of the
  desktop app's Qt resources themselves, so a package built straight from
  `dpkg-buildpackage`, `rpmbuild` or `makepkg` is reproducible too, not
  only one built by `packaging/build.sh` (#252, #266).
- **Linux:** The AppStream metainfo names the same license as the
  packages, GPL-3.0-or-later AND (MIT OR Apache-2.0) (#249, #266).
