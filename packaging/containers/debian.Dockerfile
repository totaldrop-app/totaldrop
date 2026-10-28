# Build environment for the .deb: Debian 13 (Qt 6.8) and newer.
FROM debian:trixie@sha256:8c4733a92aacd0bcff3779eadc35a4a5147f36cb20a5fcb56504cbc740212259
RUN apt-get update && apt-get install -y --no-install-recommends \
      build-essential debhelper devscripts fakeroot lld pkgconf ca-certificates curl git \
      qt6-base-dev qt6-base-dev-tools qt6-declarative-dev lintian \
    && rm -rf /var/lib/apt/lists/*
ENV RUSTUP_HOME=/opt/rustup CARGO_HOME=/opt/cargo PATH=/opt/cargo/bin:$PATH
RUN curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain 1.99.1 && chmod -R a+rwX /opt/rustup /opt/cargo
