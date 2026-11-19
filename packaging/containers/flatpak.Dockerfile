# SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Build container for the Flatpak job (.github/workflows/linux.yml).
# Dependabot's github-actions ecosystem does not update container.image
# digests in workflows, so this file sits in packaging/containers/ for
# weekly Dependabot docker updates (#601). packaging/check-versions.sh
# requires .github/workflows/linux.yml to use this exact image and digest,
# and requires the kde-<version> tag to match runtime-version in
# packaging/flatpak/app.totaldrop.TotalDrop.yml.
FROM ghcr.io/flathub-infra/flatpak-github-actions:kde-6.11@sha256:b0a8a2acb9e420dd0e3e083ff552bea5a817e17e41532f1d2287ff49ef9aae0e
