#!/usr/bin/env bash
set -Eeuo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
installer="$repo_root/install-command-center.sh"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

fail() { echo "FAIL: $*" >&2; exit 1; }
assert_file() { [[ -f "$1" ]] || fail "missing file: $1"; }
assert_contains() { grep -Fq -- "$2" "$1" || fail "$1 does not contain: $2"; }
assert_not_contains() { ! grep -Fq -- "$2" "$1" || fail "$1 unexpectedly contains: $2"; }

make_mocks() {
  local bin="$1"
  mkdir -p "$bin"
  cat >"$bin/pnpm" <<'MOCK'
#!/usr/bin/env bash
set -Eeuo pipefail
app=''
while (($#)); do [[ $1 == --dir ]] && { app=$2; shift 2; continue; }; shift; done
[[ -n "$app" ]] || exit 2
mkdir -p "$app/.output/public"
printf '<!doctype html>\n' > "$app/.output/public/index.html"
MOCK
  cat >"$bin/curl" <<'MOCK'
#!/usr/bin/env bash
set -Eeuo pipefail
if [[ ${MOCK_CURL_FAIL:-0} == 1 ]]; then exit 22; fi
printf '%s\n' '{"status":"ok"}'
MOCK
  cat >"$bin/systemctl" <<'MOCK'
#!/usr/bin/env bash
set -Eeuo pipefail
printf 'systemctl %s\n' "$*" >> "${MOCK_LOG:?}"
if [[ ${MOCK_SYSTEMCTL_FAIL:-0} == 1 ]]; then exit 1; fi
MOCK
  cat >"$bin/loginctl" <<'MOCK'
#!/usr/bin/env bash
set -Eeuo pipefail
printf 'loginctl %s\n' "$*" >> "${MOCK_LOG:?}"
if [[ ${MOCK_LOGINCTL_FAIL:-0} == 1 ]]; then exit 1; fi
MOCK
  chmod +x "$bin"/*
  mkdir -p "$tmp/app"
  printf 'export default {}\n' > "$tmp/app/server.mjs"
}

run_installer() {
  local home=$1
  HOME="$home" PATH="$tmp/bin:$PATH" MOCK_LOG="$tmp/mock.log" \
    JCODE_COMMAND_CENTER_SKIP_PREREQS=1 \
    JCODE_COMMAND_CENTER_APP_DIR="$tmp/app" \
    JCODE_COMMAND_CENTER_HEALTH_URL=http://127.0.0.1:43119/healthz \
    bash "$installer"
}

home="$tmp/home with spaces"
mkdir -p "$home"
make_mocks "$tmp/bin"
run_installer "$home"

root="$home/.local/lib/jcode-command-center"
unit="$home/.config/systemd/user/jcode-command-center.service"
env="$home/.config/jcode-command-center.env"
assert_file "$root/current/server.mjs"
assert_file "$root/current/public/index.html"
assert_file "$unit"
assert_file "$env"
assert_contains "$unit" 'Restart=on-failure'
assert_contains "$unit" 'RestartSec=2'
assert_contains "$unit" 'WantedBy=default.target'
assert_not_contains "$unit" 'WorkingDirectory='
escaped_env=${env// /\\x20}
assert_contains "$unit" "EnvironmentFile=$escaped_env"
assert_not_contains "$unit" 'EnvironmentFile="'
assert_contains "$unit" 'ExecStart="'
assert_contains "$env" 'JCODE_COMMAND_CENTER_UI_BIND=0.0.0.0:43119'
assert_contains "$env" 'JCODE_COMMAND_CENTER_API_URL=http://127.0.0.1:43118'
assert_contains "$tmp/mock.log" 'systemctl --user daemon-reload'
assert_contains "$tmp/mock.log" 'systemctl --user enable jcode-command-center.service'
assert_contains "$tmp/mock.log" 'systemctl --user restart jcode-command-center.service'
assert_contains "$tmp/mock.log" 'loginctl enable-linger '

first=$(readlink "$root/current")
# An operator-edited env file must survive a repeat install.
printf 'JCODE_COMMAND_CENTER_UI_BIND=0.0.0.0:9999\n' > "$env"
run_installer "$home"
second=$(readlink "$root/current")
[[ "$first" != "$second" ]] || fail 'repeat install did not create a new release'
assert_contains "$env" 'JCODE_COMMAND_CENTER_UI_BIND=0.0.0.0:9999'

MOCK_CURL_FAIL=1 HOME="$home" PATH="$tmp/bin:$PATH" MOCK_LOG="$tmp/mock.log" \
  JCODE_COMMAND_CENTER_SKIP_PREREQS=1 JCODE_COMMAND_CENTER_APP_DIR="$tmp/app" JCODE_COMMAND_CENTER_HEALTH_ATTEMPTS=1 bash "$installer" >/dev/null 2>&1 && fail 'health failure unexpectedly passed'
[[ "$(readlink "$root/current")" == "$second" ]] || fail 'failed health check replaced current release'

echo 'installer contract tests passed'
