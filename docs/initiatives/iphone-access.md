# Throwaway iPhone access initiative

## Current state

- Host: Arch Linux server
- Device: Leonardo’s iPhone
- UDID: `00008120-000224683AEB401E`
- iOS: `26.4.2`, build `23E261`
- USB pairing and trust: verified
- `usbmuxd`: installed and active
- Developer Mode: enabled
- Personalized Developer Disk Image: mounted read-only at `/System/Developer`

## Verified checks

- `lsusb` detects Apple device `05ac:12a8`.
- `idevice_id -l` returns the device UDID.
- `idevicepair validate` succeeds.
- `ideviceinfo` returns safe metadata.
- `pymobiledevice3 mounter auto-mount` succeeds after unlocking the phone.
- SpringBoard orientation and home-screen icon metrics are readable.
- Static accessibility capability discovery is available.

## Blocked checks

Live screenshot, accessibility, tap, text-entry, recovery, and MFA tests remain blocked on this Arch host. The current `pymobiledevice3` release and upstream main both fail to discover or establish an iOS 26 RSD tunnel. Privileged USB tunneling, the userspace tunnel, remote pairing, and Web Inspector fallback were tested. Web Inspector is disabled on the phone.

## Safe operating procedure

1. Connect the iPhone with a data-capable USB cable.
2. Unlock it and accept Trust This Computer.
3. Confirm `usbmuxd` is active and repair the temporary USB-node ACL if needed.
4. Run `idevicepair validate` and read only non-sensitive metadata.
5. Enable Developer Mode on the phone.
6. Run `pymobiledevice3 mounter auto-mount` while the phone is unlocked.
7. Use a Mac with Xcode/CoreDevice, or a Linux host with working iOS 26 RSD support, for live UI-control tests.
8. Do not jailbreak, extract passcodes, export cookies, or run MFA actions until screenshot/accessibility/input tests and emergency-stop behavior pass.

## Closeout criteria

The initiative can close only after a supported host proves screenshot, accessibility-tree, harmless tap/text-entry, disconnect/reconnect, emergency-stop, and one explicitly approved MFA boundary test. Until then, the initiative is verified through device setup but blocked at live UI control.
