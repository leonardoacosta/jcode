use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;
use serde_json::Value;

const PINNED_ORCA_VERSION: &str = "1.4.176";
const PINNED_COMMAND_SCHEMA_VERSION: u64 = 1;
const PINNED_COMMAND_COUNT: u64 = 223;
const ORCHESTRATION_CONTRACT_CAPABILITY: &str = "orchestration.contract.v1";

const COMMAND_REGISTRY_FIXTURE: &str =
    include_str!("command_center_orca/fixtures/1.4.176/command-registry.json");
const JSON_RESPONSE_FIXTURES: &str =
    include_str!("command_center_orca/fixtures/1.4.176/json-responses.json");

const REQUIRED_COMMANDS: &[&str] = &[
    "agent-context",
    "status",
    "repo list",
    "project list",
    "project setups",
    "worktree current",
    "orchestration run-create",
    "orchestration run-use",
    "orchestration run-show",
    "orchestration task-create",
    "orchestration task-list",
    "orchestration dispatch-show",
    "orchestration worker-start",
    "orchestration worker-show",
    "orchestration worker-stop",
    "orchestration worker-abandon",
    "orchestration worker-release",
    "orchestration worker-list",
];

const REQUIRED_RESPONSE_FIXTURES: &[&str] = &[
    "status.ready",
    "repo-list.success",
    "project-list.success",
    "project-setups.success",
    "worktree-current.success",
    "run-create.accepted",
    "run-use.accepted",
    "run-show.success",
    "task-create.accepted",
    "task-list.success",
    "dispatch-show.success",
    "worker-start.ready",
    "worker-start.failed",
    "worker-start.outcome-unknown",
    "worker-show.success",
    "worker-stop.stopped",
    "worker-stop.unknown",
    "worker-abandon.abandoned",
    "worker-release.released",
    "worker-release.pending",
    "worker-release.unknown",
    "worker-list.success",
    "error.typed-rejection",
];

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ResponseFixtureDocument {
    orca_version: String,
    fixtures: Vec<ResponseFixture>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ResponseFixture {
    name: String,
    command: String,
    response: Value,
}

#[derive(Debug)]
pub(super) struct OrcaCompatibilityProfile {
    command_registry: Value,
    responses: BTreeMap<String, ResponseFixture>,
}

impl OrcaCompatibilityProfile {
    pub(super) fn pinned() -> Result<Self, String> {
        let command_registry = serde_json::from_str(COMMAND_REGISTRY_FIXTURE)
            .map_err(|error| format!("invalid pinned command registry fixture: {error}"))?;
        let response_document: ResponseFixtureDocument =
            serde_json::from_str(JSON_RESPONSE_FIXTURES)
                .map_err(|error| format!("invalid pinned response fixtures: {error}"))?;
        if response_document.orca_version != PINNED_ORCA_VERSION {
            return Err("response fixture version does not match the pinned profile".to_string());
        }

        let mut responses = BTreeMap::new();
        for fixture in response_document.fixtures {
            let name = fixture.name.clone();
            if responses.insert(name.clone(), fixture).is_some() {
                return Err(format!("duplicate pinned response fixture {name}"));
            }
        }
        let profile = Self {
            command_registry,
            responses,
        };
        profile.validate_pinned_fixtures()?;
        Ok(profile)
    }

    #[cfg(test)]
    pub(super) fn orca_version(&self) -> &'static str {
        PINNED_ORCA_VERSION
    }

    #[cfg(test)]
    pub(super) fn command_registry_fixture(&self) -> &Value {
        &self.command_registry
    }

    pub(super) fn response_fixture(&self, name: &str) -> Option<&Value> {
        self.responses.get(name).map(|fixture| &fixture.response)
    }

    #[cfg(test)]
    pub(super) fn has_required_command(&self, name: &str) -> bool {
        self.registry_commands()
            .is_ok_and(|commands| commands.iter().any(|command| command["command"] == name))
    }

    pub(super) fn validate_pinned_fixtures(&self) -> Result<(), String> {
        if self.command_registry["orcaVersion"] != PINNED_ORCA_VERSION {
            return Err("command registry fixture version mismatch".to_string());
        }
        if self.command_registry["schemaVersion"].as_u64() != Some(PINNED_COMMAND_SCHEMA_VERSION) {
            return Err("command registry fixture schema version mismatch".to_string());
        }
        if self.command_registry["commandCount"].as_u64() != Some(PINNED_COMMAND_COUNT) {
            return Err("command registry fixture command count mismatch".to_string());
        }

        let commands = self.registry_commands()?;
        let mut command_names = BTreeSet::new();
        for command in commands {
            let name = command["command"]
                .as_str()
                .ok_or_else(|| "pinned command lacks a command name".to_string())?;
            if !command_names.insert(name) {
                return Err(format!("duplicate pinned command {name}"));
            }
            validate_registry_entry(command, name)?;
        }
        let expected_commands = REQUIRED_COMMANDS.iter().copied().collect::<BTreeSet<_>>();
        if command_names != expected_commands {
            return Err("pinned command registry does not match the required profile".to_string());
        }

        for fixture_name in REQUIRED_RESPONSE_FIXTURES {
            let fixture = self
                .responses
                .get(*fixture_name)
                .ok_or_else(|| format!("missing pinned response fixture {fixture_name}"))?;
            validate_envelope(&fixture.response)?;
            validate_fixture_discriminator(fixture_name, &fixture.response)?;
        }
        for fixture in self.responses.values() {
            if !expected_commands.contains(fixture.command.as_str()) {
                return Err(format!(
                    "response fixture {} targets unpinned command {}",
                    fixture.name, fixture.command
                ));
            }
        }
        Ok(())
    }

    pub(super) fn validate_discovery_values(
        &self,
        status: &Value,
        registry: &Value,
    ) -> Result<(), String> {
        self.validate_response_value("status.ready", status)?;
        if status
            .pointer("/result/runtime/appVersion")
            .and_then(Value::as_str)
            != Some(PINNED_ORCA_VERSION)
        {
            return Err("Orca application version does not match the pinned profile".to_string());
        }
        if status
            .pointer("/result/runtime/reachable")
            .and_then(Value::as_bool)
            != Some(true)
            || status
                .pointer("/result/runtime/state")
                .and_then(Value::as_str)
                != Some("ready")
            || status
                .pointer("/result/graph/state")
                .and_then(Value::as_str)
                != Some("ready")
        {
            return Err("Orca runtime is not ready for profile validation".to_string());
        }
        let capabilities = status
            .pointer("/result/runtime/capabilities")
            .and_then(Value::as_array)
            .ok_or_else(|| "Orca status lacks runtime capabilities".to_string())?;
        if !capabilities
            .iter()
            .any(|capability| capability == ORCHESTRATION_CONTRACT_CAPABILITY)
        {
            return Err("Orca orchestration contract capability is unavailable".to_string());
        }

        let registry_object = registry
            .as_object()
            .ok_or_else(|| "Orca command registry is not an object".to_string())?;
        if registry_object.keys().any(|key| {
            !matches!(
                key.as_str(),
                "orcaVersion" | "schemaVersion" | "commandCount" | "commands"
            )
        }) {
            return Err("Orca command registry contains an unknown top-level field".to_string());
        }
        if registry["schemaVersion"].as_u64() != Some(PINNED_COMMAND_SCHEMA_VERSION) {
            return Err("Orca command registry schema version mismatch".to_string());
        }
        if registry["commandCount"].as_u64() != Some(PINNED_COMMAND_COUNT) {
            return Err("Orca command registry command count mismatch".to_string());
        }
        let live_commands = registry["commands"]
            .as_array()
            .ok_or_else(|| "Orca command registry lacks commands".to_string())?;
        let mut live_by_name = BTreeMap::new();
        for command in live_commands {
            let name = command["command"]
                .as_str()
                .ok_or_else(|| "Orca command registry entry lacks a name".to_string())?;
            if live_by_name.insert(name, command).is_some() {
                return Err(format!("Orca command registry duplicates {name}"));
            }
        }
        for expected in self.registry_commands()? {
            let name = expected["command"]
                .as_str()
                .ok_or_else(|| "pinned command registry entry lacks a name".to_string())?;
            let actual = live_by_name
                .get(name)
                .ok_or_else(|| format!("Orca command registry lacks {name}"))?;
            if *actual != expected {
                return Err(format!("Orca command registry entry drifted for {name}"));
            }
        }
        Ok(())
    }

    pub(super) fn validate_response_value(
        &self,
        fixture_name: &str,
        actual: &Value,
    ) -> Result<(), String> {
        let expected = self
            .response_fixture(fixture_name)
            .ok_or_else(|| format!("unknown pinned response fixture {fixture_name}"))?;
        validate_json_shape(expected, actual, "$")?;
        validate_fixture_discriminator(fixture_name, actual)
    }

    fn registry_commands(&self) -> Result<&Vec<Value>, String> {
        self.command_registry["commands"]
            .as_array()
            .ok_or_else(|| "pinned command registry lacks commands".to_string())
    }
}

fn validate_registry_entry(entry: &Value, name: &str) -> Result<(), String> {
    let object = entry
        .as_object()
        .ok_or_else(|| format!("pinned command {name} is not an object"))?;
    let expected_fields = [
        "command",
        "path",
        "aliases",
        "argumentMode",
        "summary",
        "usage",
        "flags",
        "positionalArgs",
        "examples",
        "notes",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    let actual_fields = object.keys().map(String::as_str).collect::<BTreeSet<_>>();
    if actual_fields != expected_fields {
        return Err(format!(
            "pinned command {name} has an unknown registry shape"
        ));
    }
    for field in [
        "path",
        "aliases",
        "flags",
        "positionalArgs",
        "examples",
        "notes",
    ] {
        if !entry[field].is_array() {
            return Err(format!(
                "pinned command {name} field {field} is not an array"
            ));
        }
    }
    for field in ["command", "argumentMode", "summary", "usage"] {
        if !entry[field].is_string() {
            return Err(format!(
                "pinned command {name} field {field} is not a string"
            ));
        }
    }
    Ok(())
}

fn validate_envelope(value: &Value) -> Result<(), String> {
    let object = value
        .as_object()
        .ok_or_else(|| "pinned response is not an object".to_string())?;
    let ok = object
        .get("ok")
        .and_then(Value::as_bool)
        .ok_or_else(|| "pinned response lacks a boolean ok field".to_string())?;
    if !object.get("id").is_some_and(Value::is_string) {
        return Err("pinned response lacks a string id".to_string());
    }
    let meta = object
        .get("_meta")
        .and_then(Value::as_object)
        .ok_or_else(|| "pinned response lacks _meta".to_string())?;
    if !meta
        .get("runtimeId")
        .is_some_and(|runtime_id| runtime_id.is_string() || runtime_id.is_null())
    {
        return Err("pinned response lacks a compatible runtimeId".to_string());
    }
    let expected_fields = if ok {
        ["_meta", "id", "ok", "result"]
            .into_iter()
            .collect::<BTreeSet<_>>()
    } else {
        ["_meta", "error", "id", "ok"]
            .into_iter()
            .collect::<BTreeSet<_>>()
    };
    let actual_fields = object.keys().map(String::as_str).collect::<BTreeSet<_>>();
    if actual_fields != expected_fields {
        return Err("pinned response envelope contains unknown or missing fields".to_string());
    }
    Ok(())
}

fn validate_fixture_discriminator(name: &str, value: &Value) -> Result<(), String> {
    let expected = match name {
        "status.ready" => Some(("/result/runtime/appVersion", PINNED_ORCA_VERSION)),
        "worker-start.ready" => Some(("/result/state", "ready")),
        "worker-start.failed" => Some(("/result/state", "failed")),
        "worker-start.outcome-unknown" => Some(("/result/state", "outcome_unknown")),
        "worker-stop.stopped" => Some(("/result/state", "stopped")),
        "worker-stop.unknown" => Some(("/result/state", "stop_unknown")),
        "worker-abandon.abandoned" => Some(("/result/state", "abandoned")),
        "worker-release.released" => Some(("/result/state", "released")),
        "worker-release.pending" => Some(("/result/state", "release_pending")),
        "worker-release.unknown" => Some(("/result/state", "release_unknown")),
        "error.typed-rejection" => Some(("/error/code", "dispatch_not_found")),
        _ => None,
    };
    if let Some((pointer, expected_value)) = expected
        && value.pointer(pointer).and_then(Value::as_str) != Some(expected_value)
    {
        return Err(format!(
            "response fixture {name} has an unknown discriminator"
        ));
    }
    Ok(())
}

fn validate_json_shape(expected: &Value, actual: &Value, path: &str) -> Result<(), String> {
    match (expected, actual) {
        (Value::Null, Value::Null)
        | (Value::Bool(_), Value::Bool(_))
        | (Value::Number(_), Value::Number(_))
        | (Value::String(_), Value::String(_)) => Ok(()),
        (Value::Array(expected), Value::Array(actual)) => {
            if expected.is_empty() {
                if actual.is_empty() {
                    return Ok(());
                }
                return Err(format!("unexpected non-empty array at {path}"));
            }
            for (index, actual_value) in actual.iter().enumerate() {
                validate_json_shape(&expected[0], actual_value, &format!("{path}/{index}"))?;
            }
            Ok(())
        }
        (Value::Object(expected), Value::Object(actual)) => {
            let expected_keys = expected.keys().collect::<BTreeSet<_>>();
            let actual_keys = actual.keys().collect::<BTreeSet<_>>();
            if expected_keys != actual_keys {
                return Err(format!("object keys differ at {path}"));
            }
            for (key, expected_value) in expected {
                validate_json_shape(
                    expected_value,
                    actual.get(key).expect("keys were compared"),
                    &format!("{path}/{key}"),
                )?;
            }
            Ok(())
        }
        _ => Err(format!("JSON type differs at {path}")),
    }
}
