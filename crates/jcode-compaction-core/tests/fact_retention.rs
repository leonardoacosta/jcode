use jcode_compaction_core::{Summary, build_compaction_prompt};
use jcode_message_types::Message;
use serde_json::{Value, json};

fn fixture(name: &str) -> Value {
    let source = match name {
        "coding" => include_str!("fixtures/coding.json"),
        "resumed" => include_str!("fixtures/resumed.json"),
        "updated" => include_str!("fixtures/updated.json"),
        _ => panic!("unknown fixture"),
    };
    serde_json::from_str(source).unwrap()
}

fn evaluate(fixture: &Value, summary: Option<&str>) -> Value {
    assert_eq!(fixture["kind"], "synthetic");
    let messages: Vec<_> = fixture["messages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|message| {
            let text = message["text"].as_str().unwrap();
            match message["role"].as_str().unwrap() {
                "user" => Message::user(text),
                "assistant" => Message::assistant_text(text),
                other => panic!("unsupported fixture role: {other}"),
            }
        })
        .collect();
    let previous = fixture["previous_summary"].as_str().map(|text| Summary {
        text: text.into(),
        openai_encrypted_content: None,
        covers_up_to_turn: 0,
        original_turn_count: 1,
    });
    let prompt = build_compaction_prompt(&messages, previous.as_ref(), 100_000);
    let mut retained = Vec::new();
    let mut missing = Vec::new();
    let mut ids = std::collections::HashSet::new();
    for fact in fixture["must_retain"].as_array().unwrap() {
        let text = fact["text"].as_str().unwrap();
        assert!(!text.is_empty());
        assert!(ids.insert(fact["id"].as_str().unwrap()));
        assert!(
            prompt.to_lowercase().contains(&text.to_lowercase()),
            "fact is absent from compaction input: {text}"
        );
        if summary.is_some_and(|summary| summary.contains(text)) {
            retained.push(fact["id"].clone());
        } else {
            missing.push(fact["id"].clone());
        }
    }
    json!({
        "fixture": fixture["id"],
        "evidence_kind": if summary.is_some() { "scripted_checker_only" } else { "no_summary" },
        "verdict": if summary.is_none() { "inconclusive" } else if missing.is_empty() { "passed" } else { "failed" },
        "reason": if summary.is_none() { Some("No provider summary produced; model quality is unknown") } else { None },
        "counter": "utf8-bytes-div4-ceil-v1",
        "pre_tokens_approx": prompt.len().div_ceil(4),
        "post_tokens_approx": summary.map(|text| text.len().div_ceil(4)),
        "retained": retained, "missing": missing,
        "production_prompt_constructed": true,
        "model_quality_measured": false,
    })
}

#[test]
fn scripted_fact_checker_reports_all_fixtures_and_missing_fact() {
    for name in ["coding", "resumed", "updated"] {
        let fixture = fixture(name);
        let report = evaluate(
            &fixture,
            Some(fixture["scripted_summary"].as_str().unwrap()),
        );
        assert_eq!(report["verdict"], "passed");
        assert_eq!(report["evidence_kind"], "scripted_checker_only");
        assert_eq!(report["counter"], "utf8-bytes-div4-ceil-v1");
        assert!(report["pre_tokens_approx"].as_u64().unwrap() > 0);
        assert!(report["post_tokens_approx"].as_u64().unwrap() > 0);
        assert_eq!(report["missing"], json!([]));
        let absent = fixture["must_retain"][0]["text"].as_str().unwrap();
        let missing_summary = fixture["scripted_summary"]
            .as_str()
            .unwrap()
            .replace(absent, "");
        let failed = evaluate(&fixture, Some(&missing_summary));
        assert_eq!(failed["verdict"], "failed");
        assert!(
            failed["missing"]
                .as_array()
                .unwrap()
                .contains(&fixture["must_retain"][0]["id"])
        );
        println!("{}", serde_json::to_string(&report).unwrap());
    }
}

#[test]
fn unavailable_summary_is_inconclusive_not_checker_success() {
    let report = evaluate(&fixture("coding"), None);
    assert_eq!(report["verdict"], "inconclusive");
    assert_eq!(report["evidence_kind"], "no_summary");
    assert!(report["post_tokens_approx"].is_null());
    assert!(
        report["reason"]
            .as_str()
            .unwrap()
            .contains("No provider summary")
    );
}
