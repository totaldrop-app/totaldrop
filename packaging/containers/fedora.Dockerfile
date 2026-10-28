# Build environment for the .rpm: Fedora 43 (Qt 6.10).
FROM fedora:43@sha256:a651ddf48ea28a06ed4e1e6519f51c9f47e7a5a138722ade87369b8fbb7e5b42
RUN dnf install -y rpm-build rpmdevtools gcc-c++ lld pkgconf curl git \
      qt6-qtbase-devel qt6-qtdeclarative-devel desktop-file-utils libappstream-glib \
    && dnf clean all
ENV RUSTUP_HOME=/opt/rustup CARGO_HOME=/opt/cargo PATH=/opt/cargo/bin:$PATH
RUN curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain 1.99.1 && chmod -R a+rwX /opt/rustup /opt/cargo
