use std::collections::BTreeSet;

use url::Url;

pub const MAX_LEASE_SECONDS: u64 = 15 * 60;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Action {
    FleetHealth,
    PolicyStatus,
    ListBrowsers,
    ListWindows,
    ListTabs,
    InspectContent,
    Navigate,
    Click,
    Type,
    FillForm,
    Upload,
    Download,
    CreateTab,
    CloseTab,
    EditPolicy,
    SelfApprove,
    IssueLease,
    ReleaseEmergencyStop,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActionClass {
    Health,
    Inventory,
    Mutation,
    Authority,
}

impl Action {
    pub const fn class(self) -> ActionClass {
        match self {
            Self::FleetHealth | Self::PolicyStatus => ActionClass::Health,
            Self::ListBrowsers | Self::ListWindows | Self::ListTabs => ActionClass::Inventory,
            Self::InspectContent
            | Self::Navigate
            | Self::Click
            | Self::Type
            | Self::FillForm
            | Self::Upload
            | Self::Download
            | Self::CreateTab
            | Self::CloseTab => ActionClass::Mutation,
            Self::EditPolicy
            | Self::SelfApprove
            | Self::IssueLease
            | Self::ReleaseEmergencyStop => ActionClass::Authority,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Context {
    Ordinary,
    Incognito,
    PasswordManager,
    BrowserSettings,
    ExtensionManagement,
    PrivilegedBrowserUrl,
    PaymentConfirmation,
    BankingConfirmation,
    AccountSecurity,
    AuthenticationSettings,
    RecoverySettings,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HardDeny {
    Incognito,
    PasswordManager,
    BrowserSettings,
    ExtensionManagement,
    PrivilegedBrowserUrl,
    PaymentOrBankingConfirmation,
    AccountSecurity,
    AuthenticationOrRecovery,
}

impl Context {
    const fn hard_deny(self) -> Option<HardDeny> {
        match self {
            Self::Ordinary => None,
            Self::Incognito => Some(HardDeny::Incognito),
            Self::PasswordManager => Some(HardDeny::PasswordManager),
            Self::BrowserSettings => Some(HardDeny::BrowserSettings),
            Self::ExtensionManagement => Some(HardDeny::ExtensionManagement),
            Self::PrivilegedBrowserUrl => Some(HardDeny::PrivilegedBrowserUrl),
            Self::PaymentConfirmation | Self::BankingConfirmation => {
                Some(HardDeny::PaymentOrBankingConfirmation)
            }
            Self::AccountSecurity => Some(HardDeny::AccountSecurity),
            Self::AuthenticationSettings | Self::RecoverySettings => {
                Some(HardDeny::AuthenticationOrRecovery)
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Target {
    pub browser: String,
    pub profile: String,
    pub tab: String,
    pub origin: String,
    pub generation: u64,
    pub context: Context,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Scope {
    browser: String,
    profile: String,
    tab: String,
    origin: String,
    generation: u64,
}

impl Scope {
    pub fn for_target(target: &Target) -> Self {
        Self {
            browser: target.browser.clone(),
            profile: target.profile.clone(),
            tab: target.tab.clone(),
            origin: target.origin.clone(),
            generation: target.generation,
        }
    }

    fn matches(&self, target: &Target) -> bool {
        self.browser == target.browser
            && self.profile == target.profile
            && self.tab == target.tab
            && self.origin == target.origin
            && self.generation == target.generation
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Approval {
    request_id: String,
    action: Action,
    scope: Scope,
}

impl Approval {
    pub fn new(request_id: impl Into<String>, action: Action, target: Target) -> Self {
        Self {
            request_id: request_id.into(),
            action,
            scope: Scope::for_target(&target),
        }
    }

    fn matches(&self, request_id: &str, action: Action, target: &Target) -> bool {
        self.request_id == request_id && self.action == action && self.scope.matches(target)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Lease {
    lease_id: String,
    scope: Scope,
    actions: BTreeSet<Action>,
    duration_seconds: u64,
    expires_at: u64,
}

impl Lease {
    pub fn new(
        scope: Scope,
        actions: impl IntoIterator<Item = Action>,
        duration_seconds: u64,
    ) -> Self {
        Self::with_id("", scope, actions, duration_seconds)
    }

    pub fn with_id(
        lease_id: impl Into<String>,
        scope: Scope,
        actions: impl IntoIterator<Item = Action>,
        duration_seconds: u64,
    ) -> Self {
        Self {
            lease_id: lease_id.into(),
            scope,
            actions: actions.into_iter().collect(),
            duration_seconds,
            expires_at: 0,
        }
    }

    fn matches(&self, action: Action, target: &Target, now: u64) -> bool {
        now < self.expires_at && self.actions.contains(&action) && self.scope.matches(target)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Denial {
    RemoteAuthorityOperation,
    HardDenied(HardDeny),
    EmergencyStop,
    LeaseDurationExceeded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Decision {
    Allow,
    RequireLocalApproval,
    Deny(Denial),
}

#[derive(Debug, Default)]
pub struct PolicyEngine {
    approvals: Vec<Approval>,
    leases: Vec<Lease>,
    emergency_stop: bool,
}

impl PolicyEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn approve_once(&mut self, approval: Approval) {
        self.approvals.push(approval);
    }

    pub fn issue_lease(&mut self, mut lease: Lease, now: u64) -> Result<(), Denial> {
        if lease.duration_seconds > MAX_LEASE_SECONDS {
            return Err(Denial::LeaseDurationExceeded);
        }
        lease.expires_at = now.saturating_add(lease.duration_seconds);
        self.leases.push(lease);
        Ok(())
    }

    pub fn revoke_lease(&mut self, lease_id: &str) -> bool {
        let before = self.leases.len();
        self.leases.retain(|lease| lease.lease_id != lease_id);
        before != self.leases.len()
    }

    pub fn authorize(
        &mut self,
        request_id: &str,
        action: Action,
        target: &Target,
        now: u64,
    ) -> Decision {
        if action.class() == ActionClass::Authority {
            return Decision::Deny(Denial::RemoteAuthorityOperation);
        }
        if let Some(category) = target.context.hard_deny() {
            return Decision::Deny(Denial::HardDenied(category));
        }
        if self.emergency_stop {
            return if action.class() == ActionClass::Health {
                Decision::Allow
            } else {
                Decision::Deny(Denial::EmergencyStop)
            };
        }
        match action.class() {
            ActionClass::Health | ActionClass::Inventory => Decision::Allow,
            ActionClass::Authority => unreachable!("authority actions returned above"),
            ActionClass::Mutation => {
                self.leases.retain(|lease| now < lease.expires_at);
                if self
                    .leases
                    .iter()
                    .any(|lease| lease.matches(action, target, now))
                {
                    return Decision::Allow;
                }
                if let Some(index) = self
                    .approvals
                    .iter()
                    .position(|approval| approval.matches(request_id, action, target))
                {
                    self.approvals.remove(index);
                    return Decision::Allow;
                }
                Decision::RequireLocalApproval
            }
        }
    }

    pub fn reload_policy(&mut self) {
        self.revoke_transient_authority();
    }

    pub fn target_changed(&mut self, browser: &str, profile: &str, tab: &str) {
        self.leases.retain(|lease| {
            lease.scope.browser != browser
                || lease.scope.profile != profile
                || lease.scope.tab != tab
        });
        self.approvals.retain(|approval| {
            approval.scope.browser != browser
                || approval.scope.profile != profile
                || approval.scope.tab != tab
        });
    }

    pub fn activate_emergency_stop(&mut self) {
        self.emergency_stop = true;
        self.revoke_transient_authority();
    }

    pub fn release_emergency_stop_locally(&mut self) {
        self.emergency_stop = false;
    }

    fn revoke_transient_authority(&mut self) {
        self.leases.clear();
        self.approvals.clear();
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MetadataPolicy {
    pub reveal_title: bool,
    pub reveal_origin: bool,
    pub reveal_path: bool,
}

impl Default for MetadataPolicy {
    fn default() -> Self {
        Self {
            reveal_title: true,
            reveal_origin: true,
            reveal_path: true,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RedactedMetadata {
    pub title: Option<String>,
    pub url: Option<String>,
    pub origin: Option<String>,
}

pub fn redact_metadata(title: &str, raw_url: &str, policy: MetadataPolicy) -> RedactedMetadata {
    let parsed = Url::parse(raw_url).ok();
    let origin = parsed
        .as_ref()
        .map(|url| url.origin().ascii_serialization());
    let url = if policy.reveal_path {
        parsed.map(|mut url| {
            let _ = url.set_username("");
            let _ = url.set_password(None);
            url.set_query(None);
            url.set_fragment(None);
            url.to_string()
        })
    } else {
        None
    };

    RedactedMetadata {
        title: policy.reveal_title.then(|| title.to_owned()),
        url,
        origin: policy.reveal_origin.then_some(origin).flatten(),
    }
}
