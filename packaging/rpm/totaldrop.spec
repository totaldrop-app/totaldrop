# RPM package for TotalDrop on Fedora: the Qt 6 desktop app and the CLI.
# openSUSE names its Qt packages differently and needs a spec of its own.
# The Rust toolchain comes from rustup: rust-toolchain.toml pins the newest
# stable Rust, which distributions do not ship yet.

%global upstream_version 0.3.0-beta.4
%global srcname totaldrop-linux-v%{upstream_version}
%global debug_package %{nil}

Name:           totaldrop
Version:        0.3.0~beta.4
Release:        1%{?dist}
Summary:        Send files straight to another device, end-to-end encrypted
License:        GPL-3.0-or-later AND (MIT OR Apache-2.0)
URL:            https://totaldrop.app
Source0:        https://github.com/totaldrop-app/totaldrop/archive/refs/tags/linux-v%{upstream_version}.tar.gz#/%{srcname}.tar.gz

BuildRequires:  gcc-c++
BuildRequires:  lld
BuildRequires:  pkgconf
BuildRequires:  cmake(Qt6Core)
BuildRequires:  cmake(Qt6Gui)
BuildRequires:  cmake(Qt6Qml)
BuildRequires:  cmake(Qt6Quick)
BuildRequires:  cmake(Qt6QuickControls2)
BuildRequires:  desktop-file-utils
BuildRequires:  libappstream-glib

Requires:       qt6-qtbase
Requires:       qt6-qtdeclarative
Requires:       qt6-qtwayland
Requires:       qt6-qtsvg
Requires:       xdg-desktop-portal
Requires:       hicolor-icon-theme
Requires:       (google-noto-sans-fonts or dejavu-sans-fonts or font(:lang=en))
Recommends:     google-noto-sans-arabic-fonts
Recommends:     google-noto-sans-hebrew-fonts
Recommends:     google-noto-sans-devanagari-fonts
Recommends:     google-noto-sans-bengali-fonts
Recommends:     google-noto-sans-thai-fonts
Suggests:       google-noto-sans-cjk-fonts
Recommends:     (kf6-qqc2-desktop-style if plasma-workspace)
Recommends:     (xdg-desktop-portal-kde if plasma-workspace)
Recommends:     (xdg-desktop-portal-gtk if gnome-shell)

%description
TotalDrop sends files and folders from one device to another, end to end
encrypted. Pick the files, share a short code or a QR code, and the other
device receives them. No account, no email, no phone number.

This package contains the desktop app, which follows the desktop's theme
(Plasma, Hyprland, Omarchy, GNOME), and the totaldrop command.

%prep
%autosetup -n %{srcname}

%build
export QMAKE=%{_qt6_bindir}/qmake
cargo build --release --locked -p totaldrop-linux -p totaldrop-cli

%install
packaging/linux/install.sh %{buildroot} %{_prefix} target/release

%check
desktop-file-validate %{buildroot}%{_datadir}/applications/app.totaldrop.TotalDrop.desktop
appstream-util validate-relax --nonet %{buildroot}%{_metainfodir}/app.totaldrop.TotalDrop.metainfo.xml
cargo test --release --locked -p totaldrop-linux -p totaldrop-proto

%files
%license LICENSES/GPL-3.0-or-later.txt LICENSES/MIT.txt LICENSES/Apache-2.0.txt LICENSING.md
%doc README.md CHANGELOG.md
%{_bindir}/totaldrop
%{_bindir}/totaldrop-desktop
%{_datadir}/applications/app.totaldrop.TotalDrop.desktop
%{_metainfodir}/app.totaldrop.TotalDrop.metainfo.xml
%{_datadir}/icons/hicolor/scalable/apps/app.totaldrop.TotalDrop.svg
%{_datadir}/icons/hicolor/symbolic/apps/app.totaldrop.TotalDrop-symbolic.svg

%changelog
* Wed Nov 04 2026 TotalDrop <hello@totaldrop.app> - 0.3.0~beta.4-1
- A transfer that started on a direct connection no longer stops with "timed
  out" when that path goes quiet, for example when Wi-Fi drops: after 8
  seconds without anything from the other device it moves onto the relay and
  goes on. Both devices must use the same relay.
- A transfer through the relay survives a connection that dies without
  either end noticing, for example after switching Wi-Fi networks: the
  device connects again within about 10 seconds.
- The sending device's progress bar shows what has reached the other device,
  instead of running ahead and showing a file as sent while it is still on
  the way.
- When a transfer fails, the reason is now given in your language for many
  failures, under Details. The technical message stays in English for
  problem reports and is labelled as such.

* Wed Nov 04 2026 TotalDrop <hello@totaldrop.app> - 0.3.0~beta.3-1
- Opening files with TotalDrop while it is already open, for example with
  "Open with" in a file manager, adds them to the open window instead of
  starting a second one, and the window comes to the front.
- A transfer through the relay connects even when the relay is slow to get
  going, instead of failing after 20 seconds, and reconnects when the
  relay closes the connection. It is also faster when the relay is far
  away.
- A transfer no longer sometimes stops right after the code is entered,
  with "the other side left" on the sending device.
- Stop on the sending device now cuts the transfer at once; files still
  being sent when you press Stop are no longer completed and kept on the
  receiving device.

* Tue Nov 03 2026 TotalDrop <hello@totaldrop.app> - 0.3.0~beta.2-1
- On GNOME and on Hyprland without a Qt theme, the window takes the
  colours of your GTK theme and follows a theme change at once; buttons,
  fields and switches take the desktop's shape. Omarchy keeps its own
  theme.
- Clearer messages: a code nobody is using, a code that expired and a
  server address that is not valid are each named as such.
- Pasting the whole share message into the receive field finds the code
  in it, invisible characters such as direction marks are dropped from a
  pasted code, and codes typed with Arabic, Persian or Urdu digits are
  accepted. In a right-to-left language the shared message shows the
  code in order.
- The screens stay centred in a wide window and files in the Send list
  get an icon for their kind. Escape goes back from Send, Receive and
  Settings, Enter presses the focused button on the result screen, and
  Tab reaches controls it used to skip, scrolling them into view.
- With some Mesa drivers (seen on Intel graphics) the window no longer
  falls back to 16-bit colour, so its colours are no longer a shade off.
- totaldrop-desktop --help and --version print to the terminal instead
  of opening the window.

* Fri Oct 30 2026 TotalDrop <hello@totaldrop.app> - 0.3.0~beta.1-1
- New mark, Köşe, and a new icon: a circle with one square corner. It
  turns while a file is on its way.
- The interface in 31 more languages, including Arabic, Chinese, Hindi,
  Japanese and Spanish. They are drafts being reviewed, so the security
  texts stay in English until a language is checked.
- QR codes are read more reliably.

* Thu Oct 29 2026 TotalDrop <hello@totaldrop.app> - 0.2.0~beta.1-1
- Privacy mode in Settings: everything goes through the relay, so the
  other device never learns your IP address.
- Stop now works at every stage, tells the other side, and leaves no
  unfinished files behind.
- Adding a large folder no longer freezes the window.
- .deb and .rpm packages for ARM64 (Snapdragon X, NVIDIA RTX Spark).

* Wed Oct 28 2026 TotalDrop <hello@totaldrop.app> - 0.1.0~beta.4-1
- Transfers across the internet: devices connect directly through most
  routers, or through the encrypted relay when they cannot.

* Wed Oct 28 2026 TotalDrop <hello@totaldrop.app> - 0.1.0~beta.3-1
- Sending to another device now works even when this computer's firewall
  blocks incoming connections (as Omarchy's ufw does by default).

* Tue Oct 27 2026 TotalDrop <hello@totaldrop.app> - 0.1.0~beta.2-1
- First Linux release: send and receive with a code or a QR code,
  following the desktop's theme.
