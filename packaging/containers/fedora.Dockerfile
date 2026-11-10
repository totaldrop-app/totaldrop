# Build environment for the .rpm: Fedora 44 (Qt 6.11) and newer.
FROM fedora:44@sha256:15da97ed9aa94dd3d92c3925eb89520f6c80e07ea7ee24cba848d1f8b6912b3e
RUN dnf install -y rpm-build rpmdevtools gcc-c++ lld pkgconf curl git \
      qt6-qtbase-devel qt6-qtdeclarative-devel desktop-file-utils libappstream-glib \
    && dnf clean all
ENV RUSTUP_HOME=/opt/rustup CARGO_HOME=/opt/cargo PATH=/opt/cargo/bin:$PATH
# rustup-init from a fixed rustup release, checked against its sha256
# before it runs (no curl | sh).
ARG RUSTUP_VERSION=1.29.3
ARG RUSTUP_SHA256_X86_64=b1c8cb14e30889e7e2999e6b69e62d9e9b9ffc0f4df8180564d306d9680e6c55
ARG RUSTUP_SHA256_AARCH64=9f608c142e0e38a78957d0661ba1dfaf0b6fb37184cfaa4b9a0aa57af6184282
RUN set -eu; arch=$(uname -m); \
    case "$arch" in \
      x86_64) sum=$RUSTUP_SHA256_X86_64 ;; \
      aarch64) sum=$RUSTUP_SHA256_AARCH64 ;; \
      *) echo "no rustup-init checksum for $arch" >&2; exit 1 ;; \
    esac; \
    curl -sSfo /tmp/rustup-init "https://static.rust-lang.org/rustup/archive/$RUSTUP_VERSION/$arch-unknown-linux-gnu/rustup-init"; \
    echo "$sum  /tmp/rustup-init" | sha256sum -c -; \
    chmod +x /tmp/rustup-init; \
    /tmp/rustup-init -y --no-modify-path --profile minimal --default-toolchain 1.99.1; \
    rm /tmp/rustup-init; \
    chmod -R a+rwX /opt/rustup /opt/cargo
