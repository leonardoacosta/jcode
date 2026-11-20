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
    fn idle_exit_follows_who_owns_the_servers_lifetime() {
        // Supervised (the shared daemon) and debug-controlled servers stay.
        assert!(matches!(
            IdleExit::for_scope(None, true, false),
            IdleExit::Never
        ));
        assert!(matches!(
            IdleExit::for_scope(None, false, true),
            IdleExit::Never
        ));
        // A temporary server uses its own policy even if also supervised.
        assert!(matches!(
            IdleExit::for_scope(Some(temp_policy(None)), true, false),
            IdleExit::After(policy) if policy.idle_timeout_secs == 60
        ));
        // Self-dev and auto-spawned servers idle-exit again after 5 minutes.
        assert!(matches!(
            IdleExit::for_scope(None, false, false),
            IdleExit::After(policy)
                if policy.idle_timeout_secs == super::super::lifecycle::UNSUPERVISED_IDLE_SECS
                    && policy.owner_pid.is_none()
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
            WorkPresence {
                recurring_active: true,
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
