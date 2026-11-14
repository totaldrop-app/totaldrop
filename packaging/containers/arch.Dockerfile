# Build environment for the Arch package.
FROM archlinux:latest@sha256:d4594bb2300cba11f41c2f27923eb56f63d9988909c402f9c95517d1e362c899
RUN pacman -Syu --noconfirm --needed base-devel git rustup qt6-base qt6-declarative qt6-wayland qt6-svg \
      xdg-desktop-portal hicolor-icon-theme && pacman -Scc --noconfirm
# The same Rust as every other build (rust-toolchain.toml), not Arch's.
ENV RUSTUP_HOME=/opt/rustup CARGO_HOME=/opt/cargo PATH=/opt/cargo/bin:$PATH
RUN mkdir -p /opt/rustup /opt/cargo && rustup toolchain install 1.99.1 --profile minimal && rustup default 1.99.1 \
    && chmod -R a+rwX /opt/rustup /opt/cargo
RUN useradd -m builder && echo 'builder ALL=(ALL) NOPASSWD: ALL' > /etc/sudoers.d/builder
