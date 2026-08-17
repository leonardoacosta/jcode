#!/usr/bin/env bash
set -Eeuo pipefail
IFS=$'\n\t'

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
app_dir=${JCODE_COMMAND_CENTER_APP_DIR:-"$repo_root/apps/command-center"}
install_root=${JCODE_COMMAND_CENTER_INSTALL_ROOT:-"$HOME/.local/lib/jcode-command-center"}
config_dir=${JCODE_COMMAND_CENTER_CONFIG_DIR:-"$HOME/.config"}
unit_dir="$config_dir/systemd/user"
unit_path="$unit_dir/jcode-command-center.service"
env_path=${JCODE_COMMAND_CENTER_ENV_FILE:-"$config_dir/jcode-command-center.env"}
service_name=jcode-command-center.service
ui_bind=${JCODE_COMMAND_CENTER_UI_BIND:-127.0.0.1:43119}
api_url=${JCODE_COMMAND_CENTER_API_URL:-http://127.0.0.1:43118}
health_url=${JCODE_COMMAND_CENTER_HEALTH_URL:-"http://$ui_bind/healthz"}

need() {
  command -v "$1" >/dev/null 2>&1 || { echo "required command not found: $1" >&2; exit 1; }
}
if [[ ${JCODE_COMMAND_CENTER_SKIP_PREREQS:-0} != 1 ]]; then
  need node; need pnpm; need curl; need systemctl
fi

[[ -f "$app_dir/server.mjs" ]] || { echo "missing Command Center server: $app_dir/server.mjs" >&2; exit 1; }

pnpm --dir "$app_dir" install --frozen-lockfile
pnpm --dir "$app_dir" build
[[ -f "$app_dir/.output/public/index.html" ]] || { echo "build did not produce .output/public/index.html" >&2; exit 1; }

mkdir -p "$install_root/releases" "$unit_dir" "$(dirname "$env_path")"
release="$install_root/releases/$(date -u +%Y%m%dT%H%M%S.%NZ)-$$"
stage=$(mktemp -d "$install_root/.release-stage.XXXXXX")
previous=''
if [[ -L "$install_root/current" ]]; then previous=$(readlink "$install_root/current"); fi
cleanup() { rm -rf "$stage"; }
rollback() {
  local status=${1:-$?}
  if [[ -n "$previous" ]]; then
    ln -sfn "$previous" "$install_root/current"
    systemctl --user restart "$service_name" >/dev/null 2>&1 || true
  else
    rm -f "$install_root/current"
  fi
  echo "Command Center installation failed; previous release restored" >&2
  exit "$status"
}
trap cleanup EXIT

mkdir -p "$stage/public"
cp -a "$app_dir/.output/public/." "$stage/public/"
cp -a "$app_dir/server.mjs" "$stage/server.mjs"
find "$stage" -type f -exec chmod a-w {} +
mv "$stage" "$release"
ln -sfn "releases/$(basename "$release")" "$install_root/current"

cat >"$env_path" <<EOF
JCODE_COMMAND_CENTER_UI_BIND=$ui_bind
JCODE_COMMAND_CENTER_API_URL=$api_url
EOF
chmod 600 "$env_path"
systemd_env_path=${env_path// /\\x20}
cat >"$unit_path" <<EOF
[Unit]
Description=Jcode Command Center UI
After=default.target
StartLimitIntervalSec=60
StartLimitBurst=5

[Service]
Type=simple
EnvironmentFile=$systemd_env_path
ExecStart="$(command -v node)" "$install_root/current/server.mjs"
Restart=on-failure
RestartSec=2

[Install]
WantedBy=default.target
EOF

trap rollback ERR
systemctl --user daemon-reload
systemctl --user enable "$service_name"
systemctl --user restart "$service_name"
if ! loginctl enable-linger "$USER" >/dev/null 2>&1; then
  echo "Could not enable lingering automatically. Run: loginctl enable-linger $USER" >&2
fi
for _ in $(seq 1 "${JCODE_COMMAND_CENTER_HEALTH_ATTEMPTS:-20}"); do
  if curl --fail --silent --show-error --max-time 2 "$health_url" >/dev/null; then
    trap - ERR
    echo "Installed Command Center release $(basename "$release")"
    exit 0
  fi
  sleep 1
done
rollback 1
