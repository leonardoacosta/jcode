---
name: home-router-network
description: Reference for Leo's home router access flow, safe-access rules, and the resolved 2026-07-29 Tailscale relay diagnosis. Load this whenever a task needs router-admin access or needs to diagnose home-network/Tailscale/LAN connectivity between the main machines. Also load it before guessing router credentials — it documents the brute-force lockout behavior so you don't trip it again.
---

# Home Router & Network Reference

## Router

- **Admin UI**: use the router's current LAN gateway address. Auth is plain **HTTP Basic Auth** (`WWW-Authenticate: Basic realm=""`) — a bare `curl -u user:pass http://<router-lan-host>/` works, no session cookie or CSRF dance needed for the root page.
- **Authentication**: use the approved secret-injection mechanism for the environment. Never read, print, copy, guess, or place authentication material in commands, logs, prompts, or source files. If authenticated access is unavailable, stop and request operator assistance.

## Known gotcha: brute-force lockout

The router's login page enforces a lockout on repeated bad attempts from the same IP: **"Your Device is disabled, please try again in 1 minute"** (escalates to 5 minutes on repeat offenses, and likely longer after that — untested past 5). This fires per-IP, not per-credential-attempt, so **one wrong guess mixed into a batch of otherwise-correct calls still locks out the whole batch**, even the correct one gets bounced.

**Rule: never loop-guess authentication material against this router.** If approved access is unavailable, do not retry candidates. If you hit the lockout page, wait it out; repeated retries can escalate the lockout tier.

## RESOLVED 2026-07-29: DERP-only relay was the homelab's firewall, not the router

**The router was never the problem. Do not go looking for AP/client isolation in the router admin
UI** — that was this file's standing hypothesis from 2026-07-19 and it was wrong. Two real causes,
both off-router, and BOTH were required:

1. **`ufw` on the homelab had no rule for UDP 41641.** `tailscaled` listens there
   (`ss -ulnp | grep tailscale`), ufw defaults to `deny (incoming)`, so every inbound
   hole-punching packet was dropped. `tailscale netcheck` reported **`UDP: false`** and
   `IPv4: (no addr found)` — the tell. This relayed EVERY peer, not just the Mac; cpc too.
   Fix: `sudo ufw allow 41641/udp`. Telling detail that made it easy to miss — `51820/udp`
   (WireGuard's default port) WAS already allowed, so the ruleset looked Tailscale-aware.
2. **The Mac's netmap was stale.** Even with UDP working on the homelab, the Mac showed
   `Endpoints: None, Addrs: None` for the homelab peer — it never received the endpoint list.
   A `tailscaled` restart on the homelab did not fix it. Cleared by toggling
   **System Settings -> Privacy & Security -> Network Extensions -> Tailscale off/on**, the same
   root-owned `io.tailscale.ipn.macsys.network-extension` that survives `tailscale down/up` and
   `killall Tailscale` (see the 2026-07-21 MagicDNS incident — same extension, same lever).

Result: direct LAN traffic resumed and the measured transfer/ping results improved materially.

### Diagnostic order for "why is this peer relayed?"

1. `tailscale netcheck` on BOTH ends — `UDP: false` means a local firewall is dropping the
   listener's port. That is a host problem, never a router problem.
2. `tailscale status --json` on the peer — `Endpoints: None` means the netmap never arrived;
   restart/toggle the peer's Tailscale, not the network.
3. Only if both are clean does router-level isolation become worth investigating.

The bandwidth stakes remain as originally noted: DERP is intentionally capped shared
infrastructure, so anything moving bytes Mac<->homelab (`platform/raycast-scripts/paste-image.sh`,
the `/Volumes/dev` NFS mount) pays it. That cost is now removed.

## Machine network reference

Do not trust any previously recorded LAN or Tailscale IPs here; they drift. Resolve live state at
runtime instead:

- Current Tailscale peers and addresses: `tailscale status` or `tailscale status --json`
- Current local Wi-Fi/LAN address on macOS: `ipconfig getifaddr en0`
- Current mesh/topology notes: `<installfest-repo>/ssh-mesh/README.md`

Use placeholders such as `<router-lan-host>`, `<mac-lan-ip>`, `<homelab-tailnet-ip>`, and
`<cloudpc-tailnet-ip>` in any copied commands or notes until you have re-resolved the values.

## Naming these three machines — two valid names, different purposes (2026-07-20)

Each machine has both a MagicDNS FQDN and a prettier local-domain alias. Use the MagicDNS name
directly for anything boot-critical or machine-to-machine; use the friendlier alias for
human-facing flows. Full writeup: `<homelab-repo>/docs/dns/README.md` § Device names
resolve through AdGuard too.

**Known live blocker**: SSH mesh auth from homelab to `mac` currently fails
(`hl-bma.18` in the homelab repo) — `~/.ssh/authorized_keys` on homelab still carries a
pre-rotation mesh pubkey, so a `chezmoi apply` never landed the new one after the mesh key
rotation script last ran. Affects anything that needs homelab to reach `mac`
over SSH (including `mac-open`'s primary "show this on my Mac" path, which falls back to an
iPhone push when this is broken).
