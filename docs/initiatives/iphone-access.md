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
- `rust-ios-device-tunnel` 0.1.8 discovers the device and starts a working
  userspace CoreDevice/RSD tunnel on Linux.
- The Rust tunnel exposes the iOS 26 RSD service catalog, including
  `com.apple.coredevice.displayservice`, `com.apple.accessibility.axAuditDaemon.remoteAXService`,
  `com.apple.coredevice.hid.universalhid`, and
  `com.apple.coredevice.screencaptureservice`.
- The upstream `pymobiledevice3` bridge works over that userspace tunnel:
  app enumeration completed successfully on iOS 26.4.2.
- A direct CoreDevice screenshot probe succeeded, producing a 305,773-byte PNG.
- Universal HID enumeration succeeded and exposed the authenticated touchscreen,
  keyboard, and main-screen-button services.
- The live screen-stream server can read accessibility settings, but display
  stream startup is rejected with CoreDevice error 9021: `Remote control
  requires iOS 27.0 or later on this device.`

## Blocked checks

The original `pymobiledevice3` tunnel path is blocked, but the Linux capability is now available through the Rust userspace tunnel bridge. Screenshot capture, HID discovery, and accessibility-settings reads are verified. Full live remote control is explicitly version-gated by iOS 26.4.2, so accessibility-tree, tap, text-entry, recovery, and MFA tests cannot be completed on this build. The upstream bridge's basic probes showed that some older pymobiledevice3 actions are not implemented on this iOS 26 build, while app enumeration works.

## Safe operating procedure

1. Connect the iPhone with a data-capable USB cable.
2. Unlock it and accept Trust This Computer.
3. Confirm `usbmuxd` is active and repair the temporary USB-node ACL if needed.
4. Run `idevicepair validate` and read only non-sensitive metadata.
5. Enable Developer Mode on the phone.
6. Run `pymobiledevice3 mounter auto-mount` while the phone is unlocked.
7. Start `rust-ios-device-tunnel` 0.1.8 in userspace mode and run the upstream `pymobiledevice3_coredevice_bridge.py` transport bridge for CoreDevice work. For live remote control, use an iOS 27+ device or a supported Mac/Xcode path.
8. Do not jailbreak, extract passcodes, export cookies, or run MFA actions until screenshot/accessibility/input tests and emergency-stop behavior pass.

## Closeout criteria

The initiative can close only after a device/build that permits remote control proves screenshot, accessibility-tree, harmless tap/text-entry, disconnect/reconnect, emergency-stop, and one explicitly approved MFA boundary test. The Linux RSD transport prerequisite is satisfied, but iOS 26.4.2 itself blocks live control with an explicit iOS 27.0 requirement.
