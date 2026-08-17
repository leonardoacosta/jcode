#!/bin/bash
# Template: Authenticated Session Workflow
# Purpose: Reuse saved browser state, or create it through the agent-browser auth vault.
# Usage: ./authenticated-session.sh <login-url> <username> <password-file> [profile] [state-file]
#
# Security:
#   - The password file must contain only the password and should be mode 600.
#   - Do not pass a password as an argument or environment variable.
#   - The generated state file contains session credentials. Keep it out of version control.

set -euo pipefail

LOGIN_URL="${1:?Usage: $0 <login-url> <username> <password-file> [profile] [state-file]}"
USERNAME="${2:?Missing username}"
PASSWORD_FILE="${3:?Missing password-file}"
PROFILE="${4:-app-login}"
STATE_FILE="${5:-./auth-state.json}"

[[ -f "$PASSWORD_FILE" ]] || { echo "Password file not found: $PASSWORD_FILE" >&2; exit 1; }

cleanup() {
    agent-browser close 2>/dev/null || true
}
trap cleanup EXIT

# Reuse existing state when it still reaches an authenticated page.
if [[ -f "$STATE_FILE" ]]; then
    if agent-browser --state "$STATE_FILE" open "$LOGIN_URL" 2>/dev/null; then
        agent-browser wait --load networkidle
        CURRENT_URL=$(agent-browser get url)
        if [[ "$CURRENT_URL" != *"login"* ]] && [[ "$CURRENT_URL" != *"signin"* ]]; then
            echo "Session restored successfully"
            agent-browser snapshot -i
            trap - EXIT
            exit 0
        fi
    fi
    agent-browser close 2>/dev/null || true
    rm -f "$STATE_FILE"
fi

# Store the credential through stdin so it never appears in argv or shell history.
agent-browser auth save "$PROFILE" \
    --url "$LOGIN_URL" \
    --username "$USERNAME" \
    --password-stdin < "$PASSWORD_FILE"

agent-browser auth login "$PROFILE"
agent-browser wait --load networkidle

FINAL_URL=$(agent-browser get url)
if [[ "$FINAL_URL" == *"login"* ]] || [[ "$FINAL_URL" == *"signin"* ]]; then
    echo "Login may have failed: still on a login-like URL" >&2
    exit 1
fi

agent-browser state save "$STATE_FILE"
chmod 600 "$STATE_FILE"
echo "Login successful; state saved to $STATE_FILE"
agent-browser snapshot -i
trap - EXIT
