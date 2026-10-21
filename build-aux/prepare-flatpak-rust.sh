#!/bin/bash
# Run before flatpak-builder: its build sandbox has no network access.
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p .flatpak-rust/cargo
soldr cargo vendor --locked .flatpak-rust/vendor > .flatpak-rust/cargo/config.toml
# Cargo configuration must refer to the sandbox path, not the host checkout.
sed -i 's|directory = ".*"|directory = "/run/build/obs/.flatpak-rust/vendor"|' .flatpak-rust/cargo/config.toml
