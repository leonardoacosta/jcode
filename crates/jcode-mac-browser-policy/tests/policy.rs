use std::collections::BTreeSet;

use jcode_mac_browser_policy::{
    Action, ActionClass, Approval, Context, Decision, Denial, HardDeny, Lease, MetadataPolicy,
    PolicyEngine, Scope, Target, redact_metadata,
};

fn target() -> Target {
    Target {
        browser: "chrome".into(),
        profile: "Default".into(),
        tab: "tab-7".into(),
        origin: "https://example.com".into(),
        generation: 4,
        context: Context::Ordinary,
    }
}

#[test]
fn classifies_every_policy_action() {
    let cases = [
        (Action::FleetHealth, ActionClass::Health),
        (Action::PolicyStatus, ActionClass::Health),
        (Action::ListBrowsers, ActionClass::Inventory),
        (Action::ListWindows, ActionClass::Inventory),
        (Action::ListTabs, ActionClass::Inventory),
        (Action::InspectContent, ActionClass::Mutation),
        (Action::Navigate, ActionClass::Mutation),
        (Action::Click, ActionClass::Mutation),
        (Action::Type, ActionClass::Mutation),
        (Action::FillForm, ActionClass::Mutation),
        (Action::Upload, ActionClass::Mutation),
        (Action::Download, ActionClass::Mutation),
        (Action::CreateTab, ActionClass::Mutation),
        (Action::CloseTab, ActionClass::Mutation),
        (Action::EditPolicy, ActionClass::Authority),
        (Action::SelfApprove, ActionClass::Authority),
        (Action::IssueLease, ActionClass::Authority),
        (Action::ReleaseEmergencyStop, ActionClass::Authority),
    ];
    for (action, expected) in cases {
        assert_eq!(action.class(), expected, "{action:?}");
    }
}

#[test]
fn inventory_is_read_only_but_content_inspection_requires_approval() {
    let mut engine = PolicyEngine::new();
    assert_eq!(
        engine.authorize("r1", Action::ListTabs, &target(), 10),
        Decision::Allow
    );
    assert_eq!(
        engine.authorize("r2", Action::InspectContent, &target(), 10),
        Decision::RequireLocalApproval
    );
}

#[test]
fn metadata_redaction_removes_secrets_and_obeys_visibility_policy() {
    let visible = redact_metadata(
        "Private title",
        "https://alice:secret@example.com/path?q=token#fragment",
        MetadataPolicy::default(),
    );
    assert_eq!(visible.title.as_deref(), Some("Private title"));
    assert_eq!(visible.url.as_deref(), Some("https://example.com/path"));
    assert_eq!(visible.origin.as_deref(), Some("https://example.com"));

    let hidden = redact_metadata(
        "Private title",
        "https://example.com/path?q=token",
        MetadataPolicy {
            reveal_title: false,
            reveal_origin: false,
            reveal_path: false,
        },
    );
    assert_eq!(hidden.title, None);
    assert_eq!(hidden.url, None);
    assert_eq!(hidden.origin, None);
}

#[test]
fn authority_actions_are_immutable_remote_denies() {
    let mut engine = PolicyEngine::new();
    for action in [
        Action::EditPolicy,
        Action::SelfApprove,
        Action::IssueLease,
        Action::ReleaseEmergencyStop,
    ] {
        assert_eq!(
            engine.authorize("remote", action, &target(), 1),
            Decision::Deny(Denial::RemoteAuthorityOperation)
        );
    }
}

#[test]
fn every_hard_denied_context_wins_over_approval_and_lease() {
    let cases = [
        (Context::Incognito, HardDeny::Incognito),
        (Context::PasswordManager, HardDeny::PasswordManager),
        (Context::BrowserSettings, HardDeny::BrowserSettings),
        (Context::ExtensionManagement, HardDeny::ExtensionManagement),
        (
            Context::PrivilegedBrowserUrl,
            HardDeny::PrivilegedBrowserUrl,
        ),
        (
            Context::PaymentConfirmation,
            HardDeny::PaymentOrBankingConfirmation,
        ),
        (
            Context::BankingConfirmation,
            HardDeny::PaymentOrBankingConfirmation,
        ),
        (Context::AccountSecurity, HardDeny::AccountSecurity),
        (
            Context::AuthenticationSettings,
            HardDeny::AuthenticationOrRecovery,
        ),
        (
            Context::RecoverySettings,
            HardDeny::AuthenticationOrRecovery,
        ),
    ];
    for (context, category) in cases {
        let mut denied = target();
        denied.context = context;
        let mut engine = PolicyEngine::new();
        engine.approve_once(Approval::new("r", Action::Navigate, denied.clone()));
        engine
            .issue_lease(
                Lease::new(Scope::for_target(&denied), [Action::Navigate], 100),
                0,
            )
            .unwrap();
        assert_eq!(
            engine.authorize("r", Action::Navigate, &denied, 1),
            Decision::Deny(Denial::HardDenied(category))
        );
    }
}

#[test]
fn one_action_approval_is_exact_and_consumed_once() {
    let mut engine = PolicyEngine::new();
    engine.approve_once(Approval::new("request-1", Action::Navigate, target()));
    assert_eq!(
        engine.authorize("request-2", Action::Navigate, &target(), 1),
        Decision::RequireLocalApproval
    );
    assert_eq!(
        engine.authorize("request-1", Action::Click, &target(), 1),
        Decision::RequireLocalApproval
    );
    assert_eq!(
        engine.authorize("request-1", Action::Navigate, &target(), 1),
        Decision::Allow
    );
    assert_eq!(
        engine.authorize("request-1", Action::Navigate, &target(), 1),
        Decision::RequireLocalApproval
    );
}

#[test]
fn lease_must_match_every_scope_dimension_action_generation_and_expiry() {
    let mut engine = PolicyEngine::new();
    engine
        .issue_lease(
            Lease::new(
                Scope::for_target(&target()),
                [Action::Navigate, Action::Click],
                100,
            ),
            10,
        )
        .unwrap();
    assert_eq!(
        engine.authorize("r", Action::Navigate, &target(), 99),
        Decision::Allow
    );

    let mutations: Vec<Target> = [
        ("edge", "Default", "tab-7", "https://example.com", 4),
        ("chrome", "Work", "tab-7", "https://example.com", 4),
        ("chrome", "Default", "tab-8", "https://example.com", 4),
        ("chrome", "Default", "tab-7", "https://other.example", 4),
        ("chrome", "Default", "tab-7", "https://example.com", 5),
    ]
    .into_iter()
    .map(|(browser, profile, tab, origin, generation)| Target {
        browser: browser.into(),
        profile: profile.into(),
        tab: tab.into(),
        origin: origin.into(),
        generation,
        context: Context::Ordinary,
    })
    .collect();
    for candidate in mutations {
        assert_eq!(
            engine.authorize("r", Action::Navigate, &candidate, 99),
            Decision::RequireLocalApproval
        );
    }
    assert_eq!(
        engine.authorize("r", Action::Type, &target(), 99),
        Decision::RequireLocalApproval
    );
    assert_eq!(
        engine.authorize("r", Action::Navigate, &target(), 110),
        Decision::RequireLocalApproval
    );
}

#[test]
fn invalidation_events_revoke_leases() {
    fn leased_engine() -> PolicyEngine {
        let mut engine = PolicyEngine::new();
        engine
            .issue_lease(
                Lease::new(Scope::for_target(&target()), [Action::Navigate], 100),
                0,
            )
            .unwrap();
        engine
    }

    let mut restarted = PolicyEngine::new();
    assert_eq!(
        restarted.authorize("r", Action::Navigate, &target(), 1),
        Decision::RequireLocalApproval
    );

    let mut reload = leased_engine();
    reload.reload_policy();
    assert_eq!(
        reload.authorize("r", Action::Navigate, &target(), 1),
        Decision::RequireLocalApproval
    );

    let mut changed = leased_engine();
    changed.target_changed("chrome", "Default", "tab-7");
    assert_eq!(
        changed.authorize("r", Action::Navigate, &target(), 1),
        Decision::RequireLocalApproval
    );
}

#[test]
fn emergency_stop_revokes_leases_and_only_preserves_health_status() {
    let mut engine = PolicyEngine::new();
    engine
        .issue_lease(
            Lease::new(
                Scope::for_target(&target()),
                BTreeSet::from([Action::Navigate]),
                100,
            ),
            0,
        )
        .unwrap();
    engine.activate_emergency_stop();

    assert_eq!(
        engine.authorize("h", Action::FleetHealth, &target(), 1),
        Decision::Allow
    );
    assert_eq!(
        engine.authorize("s", Action::PolicyStatus, &target(), 1),
        Decision::Allow
    );
    assert_eq!(
        engine.authorize("i", Action::ListTabs, &target(), 1),
        Decision::Deny(Denial::EmergencyStop)
    );
    assert_eq!(
        engine.authorize("m", Action::Navigate, &target(), 1),
        Decision::Deny(Denial::EmergencyStop)
    );
    assert_eq!(
        engine.authorize("x", Action::ReleaseEmergencyStop, &target(), 1),
        Decision::Deny(Denial::RemoteAuthorityOperation)
    );

    engine.release_emergency_stop_locally();
    assert_eq!(
        engine.authorize("m", Action::Navigate, &target(), 1),
        Decision::RequireLocalApproval
    );
}

#[test]
fn lease_duration_is_bounded_to_fifteen_minutes() {
    let mut engine = PolicyEngine::new();
    let too_long = Lease::new(Scope::for_target(&target()), [Action::Navigate], 901);
    assert_eq!(
        engine.issue_lease(too_long, 0),
        Err(Denial::LeaseDurationExceeded)
    );
}
