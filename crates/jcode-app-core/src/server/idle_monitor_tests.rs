use super::idle_monitor_should_start;

#[test]
fn shared_idle_monitor_preserves_live_headless_worker() {
    assert!(!idle_monitor_should_start(0, true));
}

#[test]
fn temporary_idle_monitor_preserves_live_headless_worker() {
    assert!(!idle_monitor_should_start(0, true));
}

#[test]
fn idle_monitor_starts_only_without_clients_or_headless_workers() {
    assert!(idle_monitor_should_start(0, false));
    assert!(!idle_monitor_should_start(1, false));
}

mod resident_lifecycle {
    use super::super::lifecycle::{
        Admission, IdleExit, TemporaryServerPolicy, WorkPresence, owner_exited,
    };

    fn temp_policy(owner_pid: Option<u32>) -> TemporaryServerPolicy {
        TemporaryServerPolicy {
            owner_pid,
            idle_timeout_secs: 60,
        }
    }

    #[test]
    fn shared_scope_never_idle_exits_and_temporary_scope_does() {
        // The shared daemon has no temporary policy, so it has no idle exit
        // at any idle duration; there is no timeout to pass.
        assert!(matches!(IdleExit::for_scope(None), IdleExit::Never));
        assert!(matches!(
            IdleExit::for_scope(Some(temp_policy(None))),
            IdleExit::After(policy) if policy.idle_timeout_secs == 60
        ));
    }

    #[test]
    fn temporary_idle_exit_requires_no_work_of_any_kind() {
        assert!(WorkPresence::default().idle());
        for busy in [
            WorkPresence {
                clients: 1,
                ..Default::default()
            },
            WorkPresence {
                live_headless_worker: true,
                ..Default::default()
            },
            WorkPresence {
                ambient_pending: true,
                ..Default::default()
            },
            WorkPresence {
                background_running: true,
                ..Default::default()
            },
        ] {
            assert!(!busy.idle(), "{busy:?} must keep the server alive");
        }
    }

    #[test]
    fn dead_owner_pid_shuts_a_temporary_server_down() {
        assert!(owner_exited(&temp_policy(Some(999_001)), |_| false));
        assert!(!owner_exited(&temp_policy(Some(999_001)), |_| true));
        assert!(
            !owner_exited(&temp_policy(None), |_| false),
            "no owner recorded"
        );
        assert!(
            !owner_exited(&temp_policy(Some(std::process::id())), |_| false),
            "a server is never its own dead owner"
        );
    }

    #[test]
    fn admission_flag_closes_on_shutdown_and_can_reopen() {
        let admission = Admission::default();
        assert!(!admission.stopping());
        admission.set_stopping(true);
        assert!(admission.stopping(), "no new work once shutdown begins");
        admission.set_stopping(true);
        assert!(admission.stopping(), "idempotent");
        admission.set_stopping(false);
        assert!(
            !admission.stopping(),
            "a refused reload leaves the daemon serving"
        );
    }
}
