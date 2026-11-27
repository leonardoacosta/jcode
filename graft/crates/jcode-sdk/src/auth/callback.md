# crates/jcode-sdk/src/auth/callback.rs

- CallbackListener · struct · L5-L9 — pub(super) struct CallbackListener
- bind · function · L12-L52 — pub(super) fn bind(auth_url: &url::Url) -> Option<Self>
- close · function · L54-L56 — pub(super) fn close(&self)
- interrupted · function · L58-L60 — fn interrupted(flow: &FlowInner) -> bool
- wait · function · L62-L100 — pub(super) fn wait(&self, flow: &FlowInner) -> Result<String>
- receive · function · L102-L158 — fn receive(
- parse_request · function · L160-L194 — fn parse_request(&self, request: &[u8]) -> Option<String>
