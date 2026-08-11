use jcode_rendered_artifact_types::{RenderedArtifact, RenderedArtifactKind};
use serde_json::{Value, json};

#[test]
fn recognized_kinds_use_stable_lowercase_wire_names() {
    let cases = [
        (RenderedArtifactKind::Markdown, json!("markdown")),
        (RenderedArtifactKind::Message, json!("message")),
        (RenderedArtifactKind::Code, json!("code")),
    ];

    for (kind, expected) in cases {
        assert_eq!(serde_json::to_value(kind).unwrap(), expected);
    }
}

#[test]
fn artifact_round_trips_all_metadata() {
    let artifact = RenderedArtifact {
        kind: RenderedArtifactKind::Code,
        title: Some("Example".to_owned()),
        language: Some("rust".to_owned()),
    };

    let encoded = serde_json::to_value(&artifact).unwrap();
    assert_eq!(
        encoded,
        json!({
            "kind": "code",
            "title": "Example",
            "language": "rust"
        })
    );
    assert_eq!(
        serde_json::from_value::<RenderedArtifact>(encoded).unwrap(),
        artifact
    );
}

#[test]
fn missing_optional_fields_are_backward_compatible() {
    let artifact: RenderedArtifact = serde_json::from_value(json!({
        "kind": "markdown"
    }))
    .unwrap();

    assert_eq!(artifact.kind, RenderedArtifactKind::Markdown);
    assert_eq!(artifact.title, None);
    assert_eq!(artifact.language, None);
    assert_eq!(
        serde_json::to_value(artifact).unwrap(),
        json!({ "kind": "markdown" })
    );
}

#[test]
fn explicit_null_optional_fields_are_accepted() {
    let artifact: RenderedArtifact = serde_json::from_value(json!({
        "kind": "message",
        "title": null,
        "language": null
    }))
    .unwrap();

    assert_eq!(artifact.kind, RenderedArtifactKind::Message);
    assert_eq!(artifact.title, None);
    assert_eq!(artifact.language, None);
}

#[test]
fn artifact_body_remains_outside_the_descriptor() {
    let encoded = serde_json::to_value(RenderedArtifact {
        kind: RenderedArtifactKind::Markdown,
        title: Some("Plan".to_owned()),
        language: None,
    })
    .unwrap();

    assert_eq!(encoded.get("body"), None);
    assert_eq!(encoded.get("content"), None);
    assert_eq!(encoded.get("language"), None);
    assert!(matches!(encoded, Value::Object(_)));
}
