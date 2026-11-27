# Build environment for the Arch package.
FROM archlinux:latest@sha256:375a401bb3c6376b37f40b6dfa25067db37ec78d0b7af67ae6315f43a0184a9d
RUN pacman -Syu --noconfirm --needed base-devel git rustup qt6-base qt6-declarative qt6-wayland qt6-svg \
      xdg-desktop-portal hicolor-icon-theme && pacman -Scc --noconfirm
# The same Rust as every other build (rust-toolchain.toml), not Arch's.
ENV RUSTUP_HOME=/opt/rustup CARGO_HOME=/opt/cargo PATH=/opt/cargo/bin:$PATH
RUN mkdir -p /opt/rustup /opt/cargo && rustup toolchain install 1.99.1 --profile minimal && rustup default 1.99.1 \
    && chmod -R a+rwX /opt/rustup /opt/cargo
RUN useradd -m builder && echo 'builder ALL=(ALL) NOPASSWD: ALL' > /etc/sudoers.d/builder
