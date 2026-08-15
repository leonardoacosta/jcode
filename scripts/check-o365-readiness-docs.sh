#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
main_doc="$repo_root/docs/design/local-socks-routed-chrome-agent-targets.md"
readiness_doc="$repo_root/docs/design/o365-local-chrome-manual-sign-in-readiness.md"

fail() {
  printf 'o365 readiness check failed: %s\n' "$1" >&2
  exit 1
}

[[ -f "$main_doc" ]] || fail "missing local Chrome target design doc"
[[ -f "$readiness_doc" ]] || fail "missing O365 manual sign-in readiness doc"

grep -Fq 'O365 Local Chrome Manual Sign-In Readiness' "$readiness_doc" || fail "readiness doc title missing"
grep -Fq '"browser":"chrome_o365"' "$readiness_doc" || fail "normalized chrome_o365 browser examples missing"
grep -Fq 'Do not pass a separate `profile` value' "$readiness_doc" || fail "profile override prohibition missing"
grep -Fq 'No agent step types into email, password, OTP, MFA, passkey, recovery, or consent fields.' "$readiness_doc" || fail "credential and MFA automation prohibition missing"
grep -Fq 'No agent step reads cookies, local storage, browser profile files, credential stores, token caches, request authorization headers, or password-manager data.' "$readiness_doc" || fail "hidden credential-state prohibition missing"
grep -Fq 'account-affecting action after sign-in requires explicit user confirmation' "$readiness_doc" || fail "post-sign-in confirmation requirement missing"
grep -Fq 'auth-required' "$readiness_doc" || fail "auth-required failure state missing"
grep -Fq 'manual-mfa-required' "$readiness_doc" || fail "manual MFA failure state missing"
grep -Fq 'visible `Sign out` control' "$readiness_doc" || fail "authenticated visible-state guidance missing"
grep -Fq 'CREATE_USER_CONTEXT_FAILED_GENERIC' "$readiness_doc" || fail "application-context failure guidance missing"
grep -Fq 'initial page is not a routing control' "$readiness_doc" || fail "process-wide navigation routing guidance missing"
grep -Fq './o365-local-chrome-manual-sign-in-readiness.md' "$main_doc" || fail "main design doc does not link O365 readiness doc"

if grep -RInE 'password=|refresh[_-]?token|bearer [A-Za-z0-9._~+/=-]{20,}|Set-Cookie:|Cookie:' "$readiness_doc" "$main_doc"; then
  fail "possible credential or token material found in O365 readiness docs"
fi

printf 'o365 readiness docs check passed\n'
