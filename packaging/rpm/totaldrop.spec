# RPM package for TotalDrop on Fedora: the Qt 6 desktop app and the CLI.
# openSUSE names its Qt packages differently and needs a spec of its own.
# The Rust toolchain comes from rustup: rust-toolchain.toml pins the newest
# stable Rust, which distributions do not ship yet.

%global upstream_version 0.2.0-beta.1
%global srcname totaldrop-linux-v%{upstream_version}
%global debug_package %{nil}

Name:           totaldrop
Version:        0.2.0~beta.1
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
%license LICENSE-GPL-3.0 LICENSE-MIT LICENSE-APACHE LICENSING.md
%doc README.md CHANGELOG.md
%{_bindir}/totaldrop
%{_bindir}/totaldrop-desktop
%{_datadir}/applications/app.totaldrop.TotalDrop.desktop
%{_metainfodir}/app.totaldrop.TotalDrop.metainfo.xml
%{_datadir}/icons/hicolor/scalable/apps/app.totaldrop.TotalDrop.svg
%{_datadir}/icons/hicolor/symbolic/apps/app.totaldrop.TotalDrop-symbolic.svg

%changelog
* Thu Oct 29 2026 TotalDrop <hello@totaldrop.app> - 0.2.0~beta.1-1
- Privacy mode in Settings: everything goes through the relay, so the
  other device never learns your IP address.
- Stop now works at every stage, tells the other side, and leaves no
  unfinished files behind.
- Adding a large folder no longer freezes the window.
- Packages for ARM64 (Snapdragon X, NVIDIA RTX Spark).

* Wed Oct 28 2026 TotalDrop <hello@totaldrop.app> - 0.1.0~beta.4-1
- Transfers across the internet: devices connect directly through most
  routers, or through the encrypted relay when they cannot.

* Wed Oct 28 2026 TotalDrop <hello@totaldrop.app> - 0.1.0~beta.3-1
- Sending to another device now works even when this computer's firewall
  blocks incoming connections (as Omarchy's ufw does by default).

* Tue Oct 27 2026 TotalDrop <hello@totaldrop.app> - 0.1.0~beta.2-1
- First Linux release: send and receive with a code or a QR code,
  following the desktop's theme.
