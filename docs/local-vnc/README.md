# Local WayVNC homelab configuration

This directory records the non-secret local WayVNC systemd/PAM configuration for this workstation.
It intentionally omits credentials, usernames in the runtime config, certificate contents, and private key material.

## Installed local files

- `/etc/systemd/system/wayvnc-homelab.service` should match `wayvnc-homelab.service`.
- `/etc/pam.d/wayvnc` should match `pam-wayvnc`.
- `/home/nyaptor/.config/wayvnc/config` should follow `wayvnc-config.example` with the real local username.
- `/home/nyaptor/.config/wayvnc/server.key` must stay mode `0600` and must never be committed.
- `/home/nyaptor/.config/wayvnc/server.crt` may be world-readable locally, but should not be committed unless intentionally publishing the certificate.

## Security posture

The service enables PAM-backed WayVNC authentication and TLS key files. It currently listens on `0.0.0.0:5900`, so network exposure must be controlled by firewall, VPN, or LAN trust boundaries. Prefer Tailscale or SSH tunneling and consider changing `address=127.0.0.1` if direct LAN VNC is not required.

The service is persisted to run as `nyaptor`, not `root`. Running WayVNC as root is unnecessary for this user desktop and widens impact if WayVNC or VNC authentication is compromised.

## Verification commands

```bash
systemd-analyze verify /etc/systemd/system/wayvnc-homelab.service
systemctl is-enabled wayvnc-homelab.service
systemctl is-active wayvnc-homelab.service
stat -c '%a %U:%G %n' /home/nyaptor/.config/wayvnc/config /home/nyaptor/.config/wayvnc/server.key /etc/pam.d/wayvnc /etc/systemd/system/wayvnc-homelab.service
ss -ltnp | awk 'NR==1 || /:5900/'
```
