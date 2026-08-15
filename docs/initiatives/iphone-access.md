# Throwaway iPhone access initiative

## Current state

- Host: Arch Linux server
- Device: Leonardo’s iPhone
- UDID: `00008120-000224683AEB401E`
- iOS: `26.6` (Mac CoreDevice report)
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
- Two consecutive userspace tunnel create/close cycles completed cleanly, and
  the device remained discoverable afterward. Physical cable disconnect and
  reconnect remain untested.
- The Mac target `mac` sees the same device UDID
  `00008120-000224683AEB401E` as the Linux host, so the cross-host device
  identity matches.
- Mac prerequisites are present: macOS 26.5.2, Xcode 26.6, XcodeGen, an Apple
  Development certificate, and provisioning profiles. The signed runner build
  completed successfully after the Mac login keychain was authorized.
- The signed XCUITest runner launched on the physical iPhone and exposed its
  on-device HTTP server on port 8100.
- Homebrew `libimobiledevice` and `libusbmuxd` were installed on the Mac, and
  Mac `iproxy` forwarded port 8100 successfully. `GET /health` returned `ok`.
- `GET /screenshot` through the USB forward returned a valid 1290x2796 JPEG
  (173,617 bytes).

## Blocked checks

The original `pymobiledevice3` tunnel path is blocked, but Linux RSD transport is available through the Rust userspace tunnel bridge. Screenshot capture, HID discovery, and accessibility-settings reads are verified. The specific CoreDevice display-stream remote-control action is version-gated by iOS 26.4.2, not RSD as a whole. A separate iOS 26-compatible route is to deploy a signed XCUITest/WebDriverAgent-style runner to the phone and expose its tap/screenshot endpoints over usbmuxd; Linux hosting of that route is documented as experimental and requires an Apple-built or otherwise validly signed runner.

## Safe operating procedure

1. Connect the iPhone with a data-capable USB cable.
2. Unlock it and accept Trust This Computer.
3. Confirm `usbmuxd` is active and repair the temporary USB-node ACL if needed.
4. Run `idevicepair validate` and read only non-sensitive metadata.
5. Enable Developer Mode on the phone.
6. Run `pymobiledevice3 mounter auto-mount` while the phone is unlocked.
7. For CoreDevice display streaming, use an iOS 27+ device or supported Mac/Xcode path. For iOS 26 control, use the signed XCUITest runner on the Mac and Mac `iproxy` over the USB connection.
8. Do not jailbreak, extract passcodes, export cookies, or run MFA actions until screenshot/accessibility/input tests and emergency-stop behavior pass.

## Closeout criteria

The initiative can close only after either the CoreDevice route or the signed XCUITest/WebDriverAgent route proves screenshot, accessibility-tree, harmless tap/text-entry, disconnect/reconnect, emergency-stop, and one explicitly approved MFA boundary test. The Linux RSD transport prerequisite is satisfied; iOS 26.4.2 blocks only the tested CoreDevice display-control action.
