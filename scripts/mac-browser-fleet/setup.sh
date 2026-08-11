#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
manifest="$repo_root/crates/jcode-mac-browser-setup/Cargo.toml"

cargo run --quiet --manifest-path "$manifest" -- "$@"
