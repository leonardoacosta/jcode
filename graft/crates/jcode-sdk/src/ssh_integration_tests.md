# crates/jcode-sdk/src/ssh_integration_tests.rs

- Process · struct · L11-L11 — struct Process(Child);
- drop · function · L13-L19 — fn drop(&mut self)
- spawn · function · L22-L26 — fn spawn(mut command: Command) -> Process
- wait_until · function · L28-L34 — fn wait_until(mut ready: impl FnMut() -> bool, label: &str)
- connect · function · L36-L44 — fn connect(options: &SshConnectOptions, config: &Path) -> Result<JcodeClient>
- localhost_native_harness · function · L48-L247 — fn localhost_native_harness()
