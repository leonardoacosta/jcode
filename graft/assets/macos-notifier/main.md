# assets/macos-notifier/main.swift

- residentSecondsAfterPost · variable · L22-L22 — private let residentSecondsAfterPost: TimeInterval = 600
- exitDelayAfterClick · variable · L26-L26 — private let exitDelayAfterClick: TimeInterval = 0.7
- PostRequest · struct · L28-L36 — struct PostRequest
- Delegate · class · L38-L138 — final class Delegate: NSObject, NSApplicationDelegate, UNUserNotificationCenterDelegate
- Delegate · method · L41-L43 — init(post: PostRequest?)
- applicationDidFinishLaunching · method · L45-L60 — func applicationDidFinishLaunching(_ notification: Notification)
- deliver · method · L62-L91 — private func deliver(_ post: PostRequest, via center: UNUserNotificationCenter)
- userNotificationCenter · method · L93-L112 — func userNotificationCenter( _ center: UNUserNotificationCenter, didReceive response: UNNotificationResponse, withCompletionHandler completionHandler: @escaping () -> Void )
- activate · method · L114-L130 — private func activate(bundleID: String)
- runShell · method · L132-L137 — private func runShell(_ command: String)
- log · function · L140-L143 — func log(_ message: String)
- append · function · L145-L155 — func append(_ line: String, to path: String)
- parsePost · function · L157-L176 — func parsePost() -> PostRequest?
- app · variable · L178-L178 — let app = NSApplication.shared
- delegate · variable · L179-L179 — let delegate = Delegate(post: parsePost())
