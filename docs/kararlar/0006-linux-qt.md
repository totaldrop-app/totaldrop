# 0006: The Linux app uses Qt 6 and QML

- **Status:** Accepted, 27 October 2026
- **Context:** The app must match the system theme one hundred percent on
  KDE Plasma, Hyprland and Omarchy; native Wayland and the latest
  technology are wanted.
- **Decision:** Qt 6 and QML, bound to the Rust core with CXX-Qt. The theme
  source is chosen by desktop: on KDE the system palette and Breeze, on
  Omarchy `colors.toml` (watched live), on Hyprland the corner rounding,
  elsewhere the XDG portal. Packages for Arch, Debian and Fedora are built
  from separate recipes, in separate containers.
- **Why:** Qt is both Plasma's own toolkit and the only mature option that
  works well on Hyprland and GNOME; with QML a theme change applies without
  a restart.
- **Consequences:** Every interface change is tried on the three desktops,
  in the light and the dark theme ([tasarim.md](../tasarim.md)).
