#!/bin/bash
# Setup script for the Claude cloud environment (claude.ai/code, environment
# settings > Setup script). This copy is for reference: the environment runs
# what is pasted there, not this file.
#
# The environment snapshots the disk after this script and reuses it for new
# sessions, as long as the script finishes in about five minutes. So it only
# installs and downloads; the dependency build comes prebuilt from the
# build-cache workflow (.github/workflows/build-cache.yml).
#
# Needs these environment variables in the same settings dialog:
#   CARGO_TARGET_DIR=/opt/arena-target
#   CARGO_BUILD_JOBS=4
set -u

rustup toolchain install 1.95 --profile minimal -c clippy -c rustfmt \
  && rustup default 1.95

apt-get update -qq \
  && DEBIAN_FRONTEND=noninteractive apt-get install -y -qq \
    pkg-config libx11-dev libwayland-dev libxkbcommon-dev libudev-dev

# Prebuilt dependencies. If the download fails, sessions just build cold.
for profile in debug release; do
  curl -fsSL "https://github.com/Maginis00/Arena-flow/releases/download/build-cache/arena-target-$profile.tgz" \
    | tar -xz -C /opt || true
done

exit 0
