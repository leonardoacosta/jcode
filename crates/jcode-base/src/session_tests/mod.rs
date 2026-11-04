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

#[test]
fn judge_feedback_human_reset_tokens_are_durable_and_never_refund_retries() {
    use super::judge_feedback::{self, FeedbackAdmission};
    let _lock = lock_env();
    let temp = tempfile::tempdir().unwrap();
    let _home = EnvVarGuard::set("JCODE_HOME", temp.path());
    let parent = Session::create(None, None);
    let child = || {
        let mut session = Session::create(Some(parent.id.clone()), None);
        session.worker_role = Some("judge".into());
        session.save().unwrap();
        session
    };
    let admit = || {
        matches!(
            judge_feedback::admit(&parent.id, &child().id, "CONTINUE: next", 1).unwrap(),
            Some(FeedbackAdmission::Continue(_))
        )
    };
    assert!(admit());
    assert!(!admit());
    judge_feedback::begin_user_turn_once(&parent.id, "human-plan-a").unwrap();
    assert!(admit());
    judge_feedback::begin_user_turn_once(&parent.id, "human-plan-a").unwrap();
    assert!(!admit(), "same-token retry must not refund consumed budget");
    judge_feedback::begin_user_turn_once(&parent.id, "human-plan-b").unwrap();
    assert!(admit());
    judge_feedback::begin_user_turn_once(&parent.id, "human-plan-a").unwrap();
    assert!(
        !admit(),
        "older token replay after another turn must not refund budget"
    );
    let ledger = temp
        .path()
        .join("judge-feedback")
        .join(format!("{}.json", parent.id));
    let state: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&ledger).unwrap()).unwrap();
    assert_eq!(
        state["user_turn_tokens"],
        serde_json::json!(["human-plan-a", "human-plan-b"])
    );
    assert!(judge_feedback::begin_user_turn_once(&parent.id, "../bad-token").is_err());
    assert!(!admit(), "invalid token must not alter budget");
    let mut state = state;
    state["user_turn_tokens"] =
        serde_json::json!((0..4096).map(|n| format!("human-{n}")).collect::<Vec<_>>());
    std::fs::write(&ledger, serde_json::to_vec(&state).unwrap()).unwrap();
    assert!(judge_feedback::begin_user_turn_once(&parent.id, "overflow-token").is_err());
    let unchanged: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&ledger).unwrap()).unwrap();
    assert_eq!(unchanged["continuations"], 1);
    assert_eq!(
        unchanged["user_turn_tokens"].as_array().unwrap().len(),
        4096
    );
}

#[test]
fn judge_feedback_config_defaults_to_three_continuations() {
    let config = crate::config::Config::default();
    assert_eq!(
        serde_json::to_value(config.autojudge).unwrap()["max_continuations_per_turn"],
        serde_json::json!(3)
    );
}

#[test]
fn judge_feedback_admission_bounds_actual_first_dm_and_ignores_titles() {
    use super::judge_feedback::{self, FeedbackAdmission};
    let _lock = lock_env();
    let temp = tempfile::tempdir().unwrap();
    let _home = EnvVarGuard::set("JCODE_HOME", temp.path());
    let parent = Session::create(None, None);
    let mut judge = Session::create(Some(parent.id.clone()), Some("innocent title".into()));
    judge.worker_role = Some("judge".into());
    judge.save().unwrap();
    let continue_dm = judge_feedback::admit(&parent.id, &judge.id, "CONTINUE: finish tests", 1)
        .unwrap()
        .unwrap();
    assert!(matches!(continue_dm, FeedbackAdmission::Continue(_)));
    let text = continue_dm.message();
    assert!(text.contains("advisory reviewer content"));
    assert!(text.contains("never authorizes external actions"));
    assert!(matches!(
        judge_feedback::admit(&parent.id, &judge.id, "CONTINUE: duplicate", 1).unwrap(),
        Some(FeedbackAdmission::Note(_))
    ));
    judge_feedback::begin_user_turn(&parent.id).unwrap();
    assert!(matches!(
        judge_feedback::admit(&parent.id, &judge.id, "CONTINUE: still duplicate", 1).unwrap(),
        Some(FeedbackAdmission::Note(_))
    ));

    for malformed in [
        "continue: lowercase",
        " CONTINUE: leading space",
        "CONTINUE without colon",
        "Please CONTINUE: embedded",
    ] {
        let mut child = Session::create(Some(parent.id.clone()), None);
        child.worker_role = Some("judge".into());
        child.save().unwrap();
        assert!(matches!(
            judge_feedback::admit(&parent.id, &child.id, malformed, 1).unwrap(),
            Some(FeedbackAdmission::Note(_))
        ));
        assert!(matches!(
            judge_feedback::admit(&parent.id, &child.id, "CONTINUE: second DM", 1).unwrap(),
            Some(FeedbackAdmission::Note(_))
        ));
    }
    let mut spoof = Session::create(Some(parent.id.clone()), Some("autojudge".into()));
    spoof.save().unwrap();
    assert!(
        judge_feedback::admit(&parent.id, &spoof.id, "CONTINUE: forged judge", 1)
            .unwrap()
            .is_none()
    );
    assert!(
        judge_feedback::admit("unrelated-parent", &judge.id, "CONTINUE: wrong parent", 1)
            .unwrap()
            .is_none()
    );
}

#[test]
fn judge_feedback_cap_stop_review_and_genuine_turn_reset() {
    use super::judge_feedback::{self, FeedbackAdmission};
    let _lock = lock_env();
    let temp = tempfile::tempdir().unwrap();
    let _home = EnvVarGuard::set("JCODE_HOME", temp.path());
    let parent = Session::create(None, None);
    let child = |role: &str| {
        let mut session = Session::create(Some(parent.id.clone()), None);
        session.worker_role = Some(role.into());
        session.save().unwrap();
        session
    };
    for _ in 0..3 {
        assert!(matches!(
            judge_feedback::admit(&parent.id, &child("judge").id, "CONTINUE: next", 3).unwrap(),
            Some(FeedbackAdmission::Continue(_))
        ));
    }
    let capped = judge_feedback::admit(&parent.id, &child("judge").id, "CONTINUE: no", 3)
        .unwrap()
        .unwrap();
    assert!(matches!(capped, FeedbackAdmission::Note(_)));
    assert!(capped.message().contains("judge cap reached"));
    assert!(matches!(
        judge_feedback::admit(&parent.id, &child("judge").id, "STOP: done", 3).unwrap(),
        Some(FeedbackAdmission::Note(_))
    ));
    assert!(matches!(
        judge_feedback::admit(&parent.id, &child("review").id, "CONTINUE: do not run", 3).unwrap(),
        Some(FeedbackAdmission::Note(_))
    ));
    judge_feedback::begin_user_turn(&parent.id).unwrap();
    assert!(matches!(
        judge_feedback::admit(
            &parent.id,
            &child("judge").id,
            "CONTINUE: fresh user turn",
            3
        )
        .unwrap(),
        Some(FeedbackAdmission::Continue(_))
    ));
    assert!(matches!(
        judge_feedback::admit(&parent.id, &child("judge").id, "CONTINUE: disabled", 0).unwrap(),
        Some(FeedbackAdmission::Note(_))
    ));
}

#[test]
fn judge_feedback_rejects_corrupt_oversized_and_unsafe_state() {
    use super::judge_feedback;
    let _lock = lock_env();
    let temp = tempfile::tempdir().unwrap();
    let _home = EnvVarGuard::set("JCODE_HOME", temp.path());
    let parent = Session::create(None, None);
    judge_feedback::begin_user_turn(&parent.id).unwrap();
    let path = temp
        .path()
        .join("judge-feedback")
        .join(format!("{}.json", parent.id));
    std::fs::write(&path, b"not json").unwrap();
    assert!(judge_feedback::begin_user_turn(&parent.id).is_err());
    std::fs::write(&path, vec![b' '; 1024 * 1024 + 1]).unwrap();
    assert!(judge_feedback::begin_user_turn(&parent.id).is_err());
    assert!(judge_feedback::begin_user_turn("../escape").is_err());
    let seen: Vec<_> = (0..4096).map(|n| format!("session_{n}")).collect();
    std::fs::write(
        &path,
        serde_json::to_vec(&serde_json::json!({"continuations": 0, "seen": seen})).unwrap(),
    )
    .unwrap();
    let mut child = Session::create(Some(parent.id.clone()), None);
    child.worker_role = Some("judge".into());
    child.save().unwrap();
    assert!(judge_feedback::admit(&parent.id, &child.id, "CONTINUE: full ledger", 3).is_err());
    #[cfg(unix)]
    {
        let lock_path = path.with_extension("lock");
        std::fs::remove_file(&lock_path).unwrap();
        let sentinel = temp.path().join("sentinel");
        std::fs::write(&sentinel, "untouched").unwrap();
        std::os::unix::fs::symlink(&sentinel, &lock_path).unwrap();
        assert!(judge_feedback::begin_user_turn(&parent.id).is_err());
        assert_eq!(std::fs::read_to_string(sentinel).unwrap(), "untouched");
    }
}

#[test]
fn judge_feedback_concurrent_admissions_share_one_cap() {
    use super::judge_feedback::{self, FeedbackAdmission};
    let _lock = lock_env();
    let temp = tempfile::tempdir().unwrap();
    let _home = EnvVarGuard::set("JCODE_HOME", temp.path());
    let parent = Session::create(None, None);
    let judges: Vec<_> = (0..12)
        .map(|_| {
            let mut child = Session::create(Some(parent.id.clone()), None);
            child.worker_role = Some("judge".into());
            child.save().unwrap();
            child.id
        })
        .collect();
    let admitted = std::thread::scope(|scope| {
        let handles: Vec<_> = judges
            .iter()
            .map(|sender| {
                let parent = &parent.id;
                scope.spawn(move || {
                    judge_feedback::admit(parent, sender, "CONTINUE: race", 3).unwrap()
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .filter(|admission| matches!(admission, Some(FeedbackAdmission::Continue(_))))
            .count()
    });
    assert_eq!(admitted, 3);
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
