use super::super::{Tool, ToolContext, ToolOutput};
use super::transport::{self, Command, Request};
use crate::config;
use anyhow::{Result, bail};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{Value, json};
use std::time::Duration;

pub struct RemoteDesktopTool;
impl RemoteDesktopTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    #[serde(default, rename = "intent")]
    _intent: Option<String>,
    target: String,
    operation: String,
    #[serde(default)]
    reference: Option<String>,
    #[serde(default)]
    query: Option<String>,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    property: Option<String>,
    #[serde(default)]
    app: Option<String>,
    #[serde(default)]
    role: Option<String>,
    #[serde(default)]
    name: Option<String>,
}

fn required(value: Option<String>, field: &str) -> Result<String> {
    let value = value.ok_or_else(|| anyhow::anyhow!("{field} is required"))?;
    if value.trim().is_empty() || value.starts_with('-') || value.chars().any(char::is_control) {
        bail!("{field} must be non-empty plain text and must not start with '-'");
    }
    Ok(value)
}

fn only(input: &Input, fields: &[&str]) -> Result<()> {
    for (field, present) in [
        ("reference", input.reference.is_some()),
        ("query", input.query.is_some()),
        ("text", input.text.is_some()),
        ("property", input.property.is_some()),
        ("app", input.app.is_some()),
        ("role", input.role.is_some()),
        ("name", input.name.is_some()),
    ] {
        if present && !fields.contains(&field) {
            bail!("{field} is not valid for this operation");
        }
    }
    Ok(())
}

fn command(input: &Input) -> Result<Command> {
    Ok(match input.operation.as_str() {
        "status" => {
            only(input, &[])?;
            Command::Status
        }
        "list-apps" => {
            only(input, &[])?;
            Command::ListApps
        }
        "list-windows" => {
            only(input, &[])?;
            Command::ListWindows
        }
        "snapshot" => {
            only(input, &["app"])?;
            Command::Snapshot {
                app: required(input.app.clone(), "app")?,
            }
        }
        "find" => {
            only(input, &["app", "role", "name", "query"])?;
            let app = input
                .app
                .clone()
                .map(|v| required(Some(v), "app"))
                .transpose()?;
            let role = input
                .role
                .clone()
                .map(|v| required(Some(v), "role"))
                .transpose()?;
            let name = input
                .name
                .clone()
                .map(|v| required(Some(v), "name"))
                .transpose()?;
            let text = input
                .query
                .clone()
                .map(|v| required(Some(v), "query"))
                .transpose()?;
            if app.is_none() && role.is_none() && name.is_none() && text.is_none() {
                bail!("find requires at least one selector");
            }
            Command::Find {
                app,
                role,
                name,
                text,
            }
        }
        "get" | "is" => {
            only(input, &["reference", "property"])?;
            let reference = required(input.reference.clone(), "reference")?;
            let property = required(input.property.clone(), "property")?;
            let allowed: &[&str] = if input.operation == "get" {
                &["text", "value", "title", "bounds", "role", "states"]
            } else {
                &[
                    "visible", "enabled", "checked", "focused", "expanded", "selected",
                ]
            };
            if !allowed.contains(&property.as_str()) {
                bail!("unsupported property");
            }
            if input.operation == "get" {
                Command::Get {
                    reference,
                    property,
                }
            } else {
                Command::Is {
                    reference,
                    property,
                }
            }
        }
        "click" => {
            only(input, &["reference"])?;
            Command::Click {
                reference: required(input.reference.clone(), "reference")?,
            }
        }
        "type" => {
            only(input, &["reference", "text"])?;
            Command::Type {
                reference: required(input.reference.clone(), "reference")?,
                text: required(input.text.clone(), "text")?,
            }
        }
        _ => bail!("unsupported remote desktop operation"),
    })
}

#[async_trait]
impl Tool for RemoteDesktopTool {
    fn name(&self) -> &str {
        "remote_desktop"
    }
    fn description(&self) -> &str {
        "Inspect or carefully operate an explicitly selected configured remote desktop. Supports status, list-apps, list-windows, snapshot(app), find(app/role/name/query), get/is(reference, property), click(reference), and type(reference,text). Actions use accessibility references only. Never pass shell commands or CLI flags."
    }
    fn parameters_schema(&self) -> Value {
        json!({"type":"object","additionalProperties":false,"required":["target","operation"],"properties":{
            "intent": super::super::intent_schema_property(), "target":{"type":"string","description":"Exact configured remote desktop target ID."},
            "operation":{"type":"string","enum":["status","list-apps","list-windows","snapshot","find","get","is","click","type"]},
            "reference":{"type":"string","description":"Accessibility reference from a fresh observation."}, "query":{"type":"string","description":"Text selector for find."},
            "text":{"type":"string","description":"Text to type into the referenced control."}, "property":{"type":"string"}, "app":{"type":"string"}, "role":{"type":"string"}, "name":{"type":"string"}
        }})
    }
    async fn execute(&self, value: Value, _ctx: ToolContext) -> Result<ToolOutput> {
        let input: Input = serde_json::from_value(value)?;
        let target = config::config().remote_desktop.target(&input.target)?;
        let cmd = command(&input)?;
        let envelope = transport::execute(&Request::new(
            target.ssh_destination.clone(),
            target.executable_path.clone(),
            cmd,
            Duration::from_secs(target.timeout_secs),
            Some(target.max_output_bytes.min(1_048_576)),
        ))
        .await?;
        let disposition =
            |d: super::transport::Disposition| serde_json::to_value(d).unwrap_or(Value::Null);
        let error = envelope.error.map(|e| json!({"code":e.code,"message":e.message,"suggestion":e.suggestion,"disposition":e.disposition.map(disposition)}));
        let result = json!({"target":input.target,"operation":input.operation,"ok":envelope.ok,"data":envelope.data,"error":error,"disposition":envelope.disposition.map(disposition)});
        let encoded = serde_json::to_string(&result)?;
        let limit = target.max_output_bytes.min(1_048_576);
        let output = if encoded.len() <= limit {
            encoded
        } else {
            serde_json::to_string(
                &json!({"target":input.target,"operation":input.operation,"ok":false,"error":{"code":"OUTPUT_LIMIT","message":"structured result exceeded configured output limit"}}),
            )?
        };
        Ok(ToolOutput::new(output))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(operation: &str) -> Input {
        serde_json::from_value(json!({"target":"mac","operation":operation})).unwrap()
    }

    #[test]
    fn accepts_declared_intent_metadata() {
        let input: Input = serde_json::from_value(
            json!({"target":"mac","operation":"status","intent":"Check desktop status"}),
        )
        .unwrap();
        assert!(matches!(command(&input).unwrap(), Command::Status));
    }

    #[test]
    fn schema_is_finite_and_has_structured_selectors() {
        let schema = RemoteDesktopTool.parameters_schema();
        let operations = schema["properties"]["operation"]["enum"]
            .as_array()
            .unwrap();
        assert!(!operations.iter().any(|v| v == "press"));
        for key in ["app", "role", "name", "property", "query", "reference"] {
            assert!(schema["properties"].get(key).is_some(), "missing {key}");
        }
        assert!(schema["properties"].get("flags").is_none());
        assert!(schema["properties"].get("snapshot_id").is_none());
        assert!(
            serde_json::from_value::<Input>(json!({
                "target": "mac",
                "operation": "click",
                "reference": "ref-1",
                "snapshot_id": "snapshot-1"
            }))
            .is_err()
        );
    }

    #[test]
    fn validates_fields_per_operation_and_allowlists_properties() {
        assert!(command(&input("status")).is_ok());
        assert!(command(&input("press")).is_err());
        assert!(command(&input("snapshot")).is_err());
        let mut get = input("get");
        get.reference = Some("r1".into());
        get.property = Some("text".into());
        assert!(command(&get).is_ok());
        get.property = Some("--screenshot".into());
        assert!(command(&get).is_err());
        let mut find = input("find");
        find.query = Some("Save".into());
        assert!(matches!(
            command(&find).unwrap(),
            Command::Find { text: Some(_), .. }
        ));
        let mut click = input("click");
        click.reference = Some("r1".into());
        click.text = Some("extra".into());
        assert!(command(&click).is_err());

        let mut snapshot = input("snapshot");
        snapshot.app = Some("-x".into());
        assert!(command(&snapshot).is_err());
        let mut find = input("find");
        find.query = Some("--screenshot".into());
        assert!(command(&find).is_err());
    }

    #[test]
    fn delivery_disposition_uses_protocol_snake_case() {
        let disposition = super::super::transport::Disposition {
            delivery: super::super::transport::Delivery::DeliveryUncertain,
            retry: super::super::transport::Retry::Unsafe,
        };
        let value = serde_json::to_value(disposition).unwrap();
        assert_eq!(value["delivery"], "delivery_uncertain");
        assert_eq!(value["retry"], "unsafe");
    }

    #[test]
    fn overflow_response_is_valid_json_and_does_not_include_remote_content() {
        let target = "mac";
        let operation = "snapshot";
        let result = json!({"target":target,"operation":operation,"ok":true,"data":{"text":"sensitive desktop content"}});
        let encoded = serde_json::to_string(&result).unwrap();
        let limit = 64;
        let output = if encoded.len() <= limit {
            encoded
        } else {
            serde_json::to_string(&json!({"target":target,"operation":operation,"ok":false,"error":{"code":"OUTPUT_LIMIT"}})).unwrap()
        };
        let parsed: Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["error"]["code"], "OUTPUT_LIMIT");
        assert!(!output.contains("sensitive desktop content"));
    }
}
