use super::*;
use std::ffi::OsString;

#[test]
fn input_compression_requires_durable_edit_or_review_identity() {
    use jcode_provider_core::InputCompressionProfile::{Conservative, Exact, Optimized};
    let mut session = Session::create(None, None);
    assert_eq!(session.input_compression_profile(), Conservative);
    session.worker_role = Some("edit".to_string());
    assert_eq!(session.input_compression_profile(), Conservative);
    session.parent_id = Some("synthetic-parent".to_string());
    assert_eq!(session.input_compression_profile(), Optimized);
    session.worker_role = Some("research".to_string());
    assert_eq!(session.input_compression_profile(), Conservative);
    session.worker_role = None;
    assert_eq!(session.input_compression_profile(), Conservative);
    session.council_id = Some("synthetic-review".to_string());
    assert_eq!(session.input_compression_profile(), Exact);
    session.worker_role = Some("edit".to_string());
    assert_eq!(session.input_compression_profile(), Exact);
}

fn lock_env() -> std::sync::MutexGuard<'static, ()> {
    crate::storage::lock_test_env()
}

struct EnvVarGuard {
    key: &'static str,
    prev: Option<OsString>,
}

impl EnvVarGuard {
    fn set(key: &'static str, value: impl AsRef<std::ffi::OsStr>) -> Self {
        let prev = std::env::var_os(key);
        crate::env::set_var(key, value);
        Self { key, prev }
    }

    fn remove(key: &'static str) -> Self {
        let prev = std::env::var_os(key);
        crate::env::remove_var(key);
        Self { key, prev }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        if let Some(prev) = &self.prev {
            crate::env::set_var(self.key, prev);
        } else {
            crate::env::remove_var(self.key);
        }
    }
}

#[path = "cases.rs"]
mod cases;

#[path = "tool_image_decay.rs"]
mod tool_image_decay;
