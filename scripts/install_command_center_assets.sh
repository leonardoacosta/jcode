#!/usr/bin/env bash
set -Eeuo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
app_dir="$repo_root/apps/command-center"
destination=${JCODE_COMMAND_CENTER_ASSET_DEST:-"$HOME/.jcode/command-center/public"}

if ! command -v pnpm >/dev/null 2>&1; then
  echo "pnpm is required to build Command Center assets" >&2
  exit 1
fi

pnpm --dir "$app_dir" install --frozen-lockfile
pnpm --dir "$app_dir" build

parent=$(dirname "$destination")
mkdir -p "$parent"
stage=$(mktemp -d "$parent/.public-stage.XXXXXX")
cleanup() { rm -rf "$stage"; }
trap cleanup EXIT

cp -a "$app_dir/.output/public/." "$stage/"
rm -rf "$destination"
mv "$stage" "$destination"
trap - EXIT

test -f "$destination/index.html"
echo "Installed Command Center assets at $destination"
