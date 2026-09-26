use super::*;

#[test]
fn form_assets_have_one_edit_id_and_no_stale_inline_state() {
    let html = include_str!("assets/index.html");
    let script = include_str!("assets/bulletin.js");
    assert_eq!(html.matches("name=\"edit_id\"").count(), 1);
    assert!(!html.contains("hx-on:"));
    assert!(!script.contains("dirty.form"));
}

#[test]
fn weekday_form_values_reject_bad_days_without_dropping_them() {
    assert_eq!(parse_weekdays("0,2,6").unwrap(), vec![0, 2, 6]);
    assert!(parse_weekdays("0,7").is_err());
    assert!(parse_weekdays("0,bad").is_err());
    assert!(parse_weekdays("0,0").is_err());
}

#[test]
fn interval_shorthand_is_checked_and_keeps_seconds_api() {
    assert_eq!(parse_interval("900").unwrap(), 900);
    assert_eq!(parse_interval("15m").unwrap(), 900);
    assert_eq!(parse_interval("1h").unwrap(), 3600);
    assert_eq!(parse_interval("1d").unwrap(), 86400);
    assert!(parse_interval("59").is_err());
    assert!(parse_interval("0m").is_err());
    assert!(parse_interval("999999999999999999d").is_err());
}

#[tokio::test]
async fn preview_returns_three_times_and_rejects_invalid_calendar() {
    let dir = tempfile::tempdir().unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin = format!("http://{}", listener.local_addr().unwrap());
    let store = Arc::new(Mutex::new(
        Store::open(dir.path().join("state.json")).unwrap(),
    ));
    let config = WebConfig {
        local_origin: origin.clone(),
        tailnet_origin: None,
        control_token: "ui-test".into(),
        default_working_dir: dir.path().into(),
        default_provider: None,
        default_model: None,
    };
    std::fs::create_dir_all(dir.path().join(".jcode/skills/demo")).unwrap();
    std::fs::write(
        dir.path().join(".jcode/skills/demo/SKILL.md"),
        "---\nname: demo\ndescription: demo\n---\n",
    )
    .unwrap();
    let cancel = CancellationToken::new();
    let task = tokio::spawn(serve(listener, store, config, cancel.clone()));
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let pairing = client
        .post(format!("{origin}/api/bootstrap"))
        .header("Origin", &origin)
        .bearer_auth("ui-test")
        .json(&serde_json::json!({"target":"local"}))
        .send()
        .await
        .unwrap();
    let value: serde_json::Value = pairing.json().await.unwrap();
    let login_url = url::Url::parse(value["url"].as_str().unwrap()).unwrap();
    let token = login_url
        .query_pairs()
        .find(|(k, _)| k == "token")
        .unwrap()
        .1
        .into_owned();
    let login = client
        .post(format!("{origin}/login"))
        .header("Origin", &origin)
        .form(&[("token", token.as_str())])
        .send()
        .await
        .unwrap();
    let cookies: Vec<String> = login
        .headers()
        .get_all("set-cookie")
        .iter()
        .map(|v| v.to_str().unwrap().split(';').next().unwrap().to_owned())
        .collect();
    let cookie = cookies.join("; ");
    let csrf = cookies
        .iter()
        .find_map(|c| c.strip_prefix("bulletin_csrf="))
        .unwrap();
    let page = client
        .get(format!("{origin}/"))
        .header("Cookie", &cookie)
        .send()
        .await
        .unwrap();
    let page = page.text().await.unwrap();
    assert!(page.contains("name=\"skill\""));
    assert!(page.contains("name=\"csrf\""));
    let created = client
        .post(format!("{origin}/api/automations"))
        .header("Origin", &origin)
        .header("Cookie", &cookie)
        .form(&[
            ("csrf", csrf),
            ("skill", "demo"),
            ("arguments", "initial"),
            ("working_dir", ""),
            ("kind", "interval"),
            ("seconds", "900"),
        ])
        .send()
        .await
        .unwrap();
    assert_eq!(created.status(), 303);
    let state = client
        .get(format!("{origin}/api/automations"))
        .header("Cookie", &cookie)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(state.contains("demo"));
    let id = state
        .split("href=\"/?edit=")
        .nth(1)
        .unwrap()
        .split('\"')
        .next()
        .unwrap();
    let edit_page = client
        .get(format!("{origin}/?edit={id}"))
        .header("Cookie", &cookie)
        .send()
        .await
        .unwrap();
    let edit_page = edit_page.text().await.unwrap();
    assert!(edit_page.contains(&format!("action=\"/api/automations/{id}\"")));
    assert!(edit_page.contains("name=\"arguments\" value=\"initial\""));
    assert!(edit_page.contains("Save changes"));
    assert!(!edit_page.contains("name=\"seconds\" value=\"3600\" value="));
    assert_eq!(
        client
            .get(format!("{origin}/?edit=missing"))
            .header("Cookie", &cookie)
            .send()
            .await
            .unwrap()
            .status(),
        404
    );
    assert!(state.contains("href=\"/?edit="));
    assert!(state.contains("form method=post action=\"/api/automations/"));
    let native_preview = client
        .post(format!("{origin}/api/preview"))
        .header("Origin", &origin)
        .header("Cookie", &cookie)
        .form(&[
            ("csrf", csrf),
            ("kind", "interval"),
            ("seconds", "15m"),
            ("preview_page", "true"),
            ("edit_id", id),
        ])
        .send()
        .await
        .unwrap();
    assert_eq!(native_preview.status(), 200);
    let native_preview = native_preview.text().await.unwrap();
    assert!(native_preview.contains("<!doctype html>"));
    assert!(native_preview.contains("Next three runs"));
    assert!(native_preview.contains(&format!("href=\"/?edit={id}\">Return to schedule")));
    assert_eq!(native_preview.matches("<li>").count(), 3);
    let ajax_preview = client
        .post(format!("{origin}/api/preview"))
        .header("Origin", &origin)
        .header("Cookie", &cookie)
        .header("x-csrf-token", csrf)
        .form(&[("kind", "interval"), ("seconds", "900")])
        .send()
        .await
        .unwrap();
    let ajax_preview = ajax_preview.text().await.unwrap();
    assert!(!ajax_preview.contains("<!doctype html>"));
    assert_eq!(ajax_preview.matches("<li>").count(), 3);
    let native_pause = client
        .post(format!("{origin}/api/automations/{id}/pause"))
        .header("Origin", &origin)
        .header("Cookie", &cookie)
        .form(&[("csrf", csrf)])
        .send()
        .await
        .unwrap();
    assert_eq!(native_pause.status(), 303);
    assert_eq!(native_pause.headers().get("location").unwrap(), "/");
    let native_resume = client
        .post(format!("{origin}/api/automations/{id}/resume"))
        .header("Origin", &origin)
        .header("Cookie", &cookie)
        .form(&[("csrf", csrf)])
        .send()
        .await
        .unwrap();
    assert_eq!(native_resume.status(), 303);
    let edited = client
        .post(format!("{origin}/api/automations/{id}"))
        .header("Origin", &origin)
        .header("Cookie", &cookie)
        .form(&[
            ("csrf", csrf),
            ("skill", "demo"),
            ("arguments", "edited"),
            ("working_dir", ""),
            ("kind", "interval"),
            ("seconds", "1800"),
        ])
        .send()
        .await
        .unwrap();
    assert_eq!(edited.status(), 303);
    let failed_edit = client
        .post(format!("{origin}/api/automations/{id}"))
        .header("Origin", &origin)
        .header("Cookie", &cookie)
        .form(&[
            ("csrf", csrf),
            ("skill", "bad<script>"),
            ("arguments", "keep me"),
            ("working_dir", ""),
            ("kind", "interval"),
            ("seconds", "1800"),
        ])
        .send()
        .await
        .unwrap();
    assert_eq!(failed_edit.status(), 400);
    let error_page = failed_edit.text().await.unwrap();
    assert!(error_page.contains(&format!("action=\"/api/automations/{id}\"")));
    assert!(error_page.contains("value=\"keep me\""));
    assert!(error_page.contains("name=\"csrf\" value=\""));
    let corrected = client
        .post(format!("{origin}/api/automations/{id}"))
        .header("Origin", &origin)
        .header("Cookie", &cookie)
        .form(&[
            ("csrf", csrf),
            ("skill", "demo"),
            ("arguments", "corrected"),
            ("working_dir", ""),
            ("kind", "interval"),
            ("seconds", "1800"),
        ])
        .send()
        .await
        .unwrap();
    assert_eq!(corrected.status(), 303);
    let rows = client
        .get(format!("{origin}/api/automations"))
        .header("Cookie", &cookie)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert_eq!(rows.matches("href=\"/?edit=").count(), 1);
    let good = client
        .post(format!("{origin}/api/preview"))
        .header("Origin", &origin)
        .header("Cookie", &cookie)
        .header("x-csrf-token", csrf)
        .form(&[
            ("kind", "interval"),
            ("seconds", "900"),
            ("weekdays", ""),
            ("time", ""),
            ("timezone", ""),
        ])
        .send()
        .await
        .unwrap();
    assert_eq!(good.status(), 200);
    assert_eq!(good.text().await.unwrap().matches("<li>").count(), 3);
    let bad = client
        .post(format!("{origin}/api/preview"))
        .header("Origin", &origin)
        .header("Cookie", &cookie)
        .header("x-csrf-token", csrf)
        .form(&[
            ("kind", "calendar"),
            ("seconds", ""),
            ("weekdays", "1"),
            ("time", "12:00"),
            ("timezone", "not-a-zone"),
        ])
        .send()
        .await
        .unwrap();
    assert_eq!(bad.status(), 400);
    cancel.cancel();
    tokio::time::timeout(std::time::Duration::from_secs(3), task)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
}
