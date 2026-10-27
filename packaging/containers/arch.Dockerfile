# Build environment for the Arch package.
FROM archlinux:latest
RUN pacman -Syu --noconfirm --needed base-devel git rust qt6-base qt6-declarative qt6-wayland qt6-svg \
      xdg-desktop-portal hicolor-icon-theme && pacman -Scc --noconfirm
RUN useradd -m builder && echo 'builder ALL=(ALL) NOPASSWD: ALL' > /etc/sudoers.d/builder
