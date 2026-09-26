use super::{
    schedule::Schedule,
    store::{Automation, Run, RunStatus, Store},
};
use anyhow::Result;
use axum::{
    Json, Router,
    body::{Body, to_bytes},
    extract::State,
    http::{HeaderMap, HeaderValue, Method, Request, StatusCode, header},
    middleware::{self, Next},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
};
use chrono::{Duration, Utc};
use rand::{Rng, distr::Alphanumeric};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Instant};
use tokio::{
    net::TcpListener,
    sync::{Mutex, Semaphore},
};
use tokio_util::sync::CancellationToken;
const MAX_BODY: usize = 16 * 1024;
const SESSION_AGE: i64 = 24 * 60 * 60;
const BOOTSTRAP_AGE: i64 = 5 * 60;
#[cfg(test)]
#[path = "auth_tests.rs"]
mod auth_tests;
#[derive(Clone)]
pub struct WebConfig {
    pub local_origin: String,
    pub tailnet_origin: Option<String>,
    pub control_token: String,
    pub default_working_dir: PathBuf,
    pub default_provider: Option<String>,
    pub default_model: Option<String>,
}
#[derive(Clone)]
struct AppState {
    store: Arc<Mutex<Store>>,
    config: WebConfig,
    sessions: Arc<Mutex<HashMap<String, (chrono::DateTime<Utc>, String, bool)>>>,
    bootstrap: Arc<Mutex<HashMap<String, (String, chrono::DateTime<Utc>)>>>,
    requests: Arc<Semaphore>,
    attempts: Arc<Mutex<(Instant, u32)>>,
}
pub async fn serve(
    listener: TcpListener,
    store: Arc<Mutex<Store>>,
    config: WebConfig,
    cancellation: CancellationToken,
) -> Result<()> {
    let state = app_state(store, config);
    let app = router(state);
    axum::serve(listener, app)
        .with_graceful_shutdown(cancellation.cancelled_owned())
        .await?;
    Ok(())
}
fn app_state(store: Arc<Mutex<Store>>, config: WebConfig) -> AppState {
    AppState {
        store,
        config,
        sessions: Default::default(),
        bootstrap: Default::default(),
        requests: Arc::new(Semaphore::new(16)),
        attempts: Arc::new(Mutex::new((Instant::now(), 0))),
    }
}
fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/assets/htmx.min.js", get(htmx))
        .route("/assets/bulletin.js", get(bulletin_js))
        .route("/api/bootstrap", post(bootstrap))
        .route("/api/status", get(control_status))
        .route("/login", get(login_page).post(login))
        .route("/logout", post(logout))
        .route("/api/automations", get(list).post(create))
        .route("/api/preview", post(preview))
        .route("/api/automations/{id}/pause", post(pause))
        .route("/api/automations/{id}/resume", post(resume))
        .route("/api/automations/{id}", post(update))
        .route("/api/runs", get(runs))
        .with_state(state.clone())
        .layer(axum::extract::DefaultBodyLimit::max(MAX_BODY))
        .layer(middleware::from_fn_with_state(state, security))
}
async fn security(State(state): State<AppState>, request: Request<Body>, next: Next) -> Response {
    let Ok(_permit) = state.requests.clone().try_acquire_owned() else {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    };
    let response = authorize(state, request, next).await;
    let mut response = response;
    let h = response.headers_mut();
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    h.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("strict-origin"),
    );
    h.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    h.insert(header::CONTENT_SECURITY_POLICY,HeaderValue::from_static("default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; form-action 'self'; base-uri 'none'; frame-ancestors 'none'"));
    response
}
async fn authorize(state: AppState, request: Request<Body>, next: Next) -> Response {
    let (parts, body) = request.into_parts();
    let host = parts
        .headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let local = origin_matches_host(&state.config.local_origin, host);
    let remote = state
        .config
        .tailnet_origin
        .as_deref()
        .is_some_and(|o| origin_matches_host(o, host));
    if !local && !remote {
        return StatusCode::FORBIDDEN.into_response();
    }
    let origin = if remote {
        state.config.tailnet_origin.as_deref().unwrap()
    } else {
        &state.config.local_origin
    };
    if let Some(value) = parts.headers.get(header::ORIGIN) {
        if value.to_str().ok() != Some(origin) {
            return StatusCode::FORBIDDEN.into_response();
        }
    }
    if parts.method != Method::GET
        && parts
            .headers
            .get(header::ORIGIN)
            .and_then(|v| v.to_str().ok())
            != Some(origin)
    {
        return StatusCode::FORBIDDEN.into_response();
    }
    let bytes =
        match tokio::time::timeout(std::time::Duration::from_secs(10), to_bytes(body, MAX_BODY))
            .await
        {
            Ok(Ok(bytes)) => bytes,
            Ok(Err(_)) => return StatusCode::PAYLOAD_TOO_LARGE.into_response(),
            Err(_) => return StatusCode::REQUEST_TIMEOUT.into_response(),
        };
    let path = parts.uri.path();
    if path == "/api/bootstrap" || path == "/api/status" {
        if !local {
            return StatusCode::FORBIDDEN.into_response();
        }
    } else if path == "/login" {
        if parts.method == Method::POST {
            let mut attempts = state.attempts.lock().await;
            if attempts.0.elapsed().as_secs() >= 60 {
                *attempts = (Instant::now(), 0)
            }
            if attempts.1 >= 30 {
                return StatusCode::TOO_MANY_REQUESTS.into_response();
            };
            attempts.1 += 1;
            drop(attempts);
            let token = url::form_urlencoded::parse(&bytes)
                .find(|(k, _)| k == "token")
                .map(|(_, v)| v.into_owned());
            let tokens = state.bootstrap.lock().await;
            if !token
                .as_ref()
                .and_then(|t| tokens.get(t))
                .is_some_and(|(target, expiry)| target == origin && *expiry > Utc::now())
            {
                return StatusCode::UNAUTHORIZED.into_response();
            }
        }
    } else if !path.starts_with("/assets/") {
        let sid = cookie(&parts.headers, "bulletin_session");
        let mut sessions = state.sessions.lock().await;
        sessions.retain(|_, (expiry, _, _)| *expiry > Utc::now());
        let session = sid
            .and_then(|id| sessions.get(id))
            .filter(|(_, _, is_remote)| *is_remote == remote)
            .cloned();
        drop(sessions);
        if session.is_none() && path != "/" {
            return StatusCode::UNAUTHORIZED.into_response();
        }
        if session.is_none() && sid.is_some() {
            return StatusCode::UNAUTHORIZED.into_response();
        }
        if parts.method != Method::GET {
            let Some((_, csrf, _)) = session else {
                return StatusCode::UNAUTHORIZED.into_response();
            };
            let submitted = parts
                .headers
                .get("x-csrf-token")
                .and_then(|v| v.to_str().ok())
                .map(str::to_owned)
                .or_else(|| {
                    parts
                        .headers
                        .get(header::CONTENT_TYPE)
                        .and_then(|v| v.to_str().ok())
                        .filter(|v| v.starts_with("application/x-www-form-urlencoded"))?;
                    url::form_urlencoded::parse(&bytes)
                        .find(|(k, _)| k == "csrf")
                        .map(|(_, v)| v.into_owned())
                });
            if submitted.as_deref() != Some(csrf.as_str()) {
                return StatusCode::FORBIDDEN.into_response();
            }
        }
    }
    next.run(Request::from_parts(parts, Body::from(bytes)))
        .await
}
fn origin_matches_host(origin: &str, host: &str) -> bool {
    let Ok(url) = url::Url::parse(origin) else {
        return false;
    };
    let Some(name) = url.host_str() else {
        return false;
    };
    let expected = match url.port() {
        Some(port) => format!("{name}:{port}"),
        None => name.to_string(),
    };
    host.eq_ignore_ascii_case(&expected)
}
fn cookie<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .filter_map(|s| s.trim().split_once('='))
        .find_map(|(k, v)| (k == name).then_some(v))
}
fn token() -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(48)
        .map(char::from)
        .collect()
}
#[derive(Deserialize)]
struct BootstrapRequest {
    target: String,
}
#[derive(Serialize)]
struct BootstrapResponse {
    url: String,
}
async fn control_status(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        != Some(format!("Bearer {}", state.config.control_token).as_str())
    {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let store = state.store.lock().await;
    Json(serde_json::json!({
        "service": "jcode-automation-bulletin",
        "persistence_ready": store.persistence_ready(),
        "automations": store.automations().len(),
        "enabled_automations": store.automations().iter().filter(|a| a.enabled).count(),
        "active_runs": store.runs().iter().filter(|r| r.status == RunStatus::Running).count(),
        "tailnet_configured": state.config.tailnet_origin.is_some(),
    }))
    .into_response()
}
async fn bootstrap(
    State(s): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<BootstrapRequest>,
) -> Response {
    if headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        != Some(format!("Bearer {}", s.config.control_token).as_str())
    {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let origin = match input.target.as_str() {
        "local" => &s.config.local_origin,
        "tailnet" => match &s.config.tailnet_origin {
            Some(v) => v,
            None => return StatusCode::NOT_FOUND.into_response(),
        },
        _ => return StatusCode::BAD_REQUEST.into_response(),
    };
    let mut tokens = s.bootstrap.lock().await;
    tokens.retain(|_, (_, expiry)| *expiry > Utc::now());
    if tokens.len() >= 64 {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    }
    let token = token();
    tokens.insert(
        token.clone(),
        (
            origin.clone(),
            Utc::now() + Duration::seconds(BOOTSTRAP_AGE),
        ),
    );
    Json(BootstrapResponse {
        url: format!("{origin}/login?token={token}"),
    })
    .into_response()
}
#[derive(Deserialize)]
struct Login {
    token: String,
}
async fn login_page(axum::extract::Query(input): axum::extract::Query<Login>) -> Html<String> {
    Html(format!(
        "<!doctype html><html lang=\"en\"><meta name=viewport content=\"width=device-width,initial-scale=1\"><title>Pair bulletin</title><h1>Pair this browser</h1><form method=post action=\"/login\"><input type=hidden name=token value=\"{}\"><button>Continue</button></form></html>",
        esc(&input.token)
    ))
}
async fn login(State(s): State<AppState>, axum::Form(input): axum::Form<Login>) -> Response {
    let target = s.bootstrap.lock().await.remove(&input.token);
    let Some((origin, expiry)) = target.filter(|(_, expiry)| *expiry > Utc::now()) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let _ = expiry;
    let remote = origin != s.config.local_origin;
    let sid = token();
    let csrf = token();
    let mut sessions = s.sessions.lock().await;
    sessions.retain(|_, (expiry, _, _)| *expiry > Utc::now());
    if sessions.len() >= 128 {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    }
    sessions.insert(
        sid.clone(),
        (
            Utc::now() + Duration::seconds(SESSION_AGE),
            csrf.clone(),
            remote,
        ),
    );
    let mut response = Redirect::to("/").into_response();
    for (name, value, http_only) in [
        ("bulletin_session", sid, true),
        ("bulletin_csrf", csrf, false),
    ] {
        append_cookie(
            &mut response,
            format!(
                "{name}={value}; Path=/; SameSite=Strict; Max-Age={SESSION_AGE}{}{}",
                if remote { "; Secure" } else { "" },
                if http_only { "; HttpOnly" } else { "" }
            ),
        );
    }
    response
}
fn append_cookie(response: &mut Response, value: String) {
    if let Ok(value) = HeaderValue::from_str(&value) {
        response.headers_mut().append(header::SET_COOKIE, value);
    }
}
async fn logout(State(s): State<AppState>, headers: HeaderMap) -> Response {
    if let Some(id) = cookie(&headers, "bulletin_session") {
        s.sessions.lock().await.remove(id);
    }
    let mut response = Redirect::to("/").into_response();
    for name in ["bulletin_session", "bulletin_csrf"] {
        append_cookie(
            &mut response,
            format!("{name}=; Path=/; Max-Age=0; HttpOnly; SameSite=Strict"),
        );
    }
    response
}

async fn index(
    State(s): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(edit): axum::extract::Query<EditQuery>,
) -> Response {
    let authenticated = if let Some(id) = cookie(&headers, "bulletin_session") {
        let mut sessions = s.sessions.lock().await;
        sessions.retain(|_, (expiry, _, _)| *expiry > Utc::now());
        sessions.contains_key(id)
    } else {
        false
    };
    if authenticated {
        let csrf = cookie(&headers, "bulletin_csrf").unwrap_or("");
        let st = s.store.lock().await;
        let mut html = include_str!("assets/index.html")
            .replace("__CSRF__", &esc(csrf))
            .replace(
                "__EDIT_ID__",
                edit.edit.as_deref().map(esc).as_deref().unwrap_or(""),
            );
        if let Some(id) = edit.edit.as_deref() {
            let Some(a) = st.automations().iter().find(|a| a.id == id) else {
                return StatusCode::NOT_FOUND.into_response();
            };
            html = html
                .replace("<h2>Create schedule</h2>", "<h2>Edit schedule</h2>")
                .replace(
                    "action=\"/api/automations\"",
                    &format!("action=\"/api/automations/{}\"", esc(&a.id)),
                )
                .replace("Create schedule", "Save changes");
            let (kind, seconds, weekdays, time, timezone) = match &a.schedule {
                Schedule::Interval { seconds } => (
                    "interval",
                    seconds.to_string(),
                    String::new(),
                    String::new(),
                    String::new(),
                ),
                Schedule::Calendar {
                    weekdays,
                    time,
                    timezone,
                } => (
                    "calendar",
                    String::new(),
                    weekdays
                        .iter()
                        .map(u32::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                    time.clone(),
                    timezone.clone(),
                ),
            };
            let values = [
                ("skill", a.skill.clone()),
                ("arguments", a.arguments.clone()),
                ("working_dir", a.working_dir.to_string_lossy().into_owned()),
                ("seconds", seconds),
                ("weekdays", weekdays),
                ("time", time),
                ("timezone", timezone),
            ];
            for (name, value) in values {
                let marker = format!("name=\"{name}\"");
                for default in ["3600", ""] {
                    html = html.replace(&format!("{marker} value=\"{default}\""), &marker);
                }
                html = html.replace(&marker, &format!("{marker} value=\"{}\"", esc(&value)));
            }
            html = html.replace(
                &format!("<option value=\"{kind}\">"),
                &format!("<option value=\"{kind}\" selected>"),
            );
        }
        html = html.replace(
            "__AUTOMATIONS__",
            &render_list(st.automations(), 0).replace("__CSRF__", &esc(csrf)),
        );
        html = html.replace("__RUNS__", &render_runs(st.runs(), 0));
        html = html.replace("__CSRF__", &esc(csrf));
        let mut response = Html(html).into_response();
        response
            .headers_mut()
            .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
        response
    } else {
        Html("<!doctype html><html><meta name=viewport content=\"width=device-width,initial-scale=1\"><h1>Automation bulletin</h1><p>Use a fresh bootstrap link to sign in.</p></html>").into_response()
    }
}
async fn htmx() -> Response {
    (
        [
            (header::CONTENT_TYPE, "text/javascript; charset=utf-8"),
            (header::CACHE_CONTROL, "public, max-age=86400"),
        ],
        include_str!("assets/htmx.min.js"),
    )
        .into_response()
}
async fn bulletin_js() -> Response {
    (
        [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
        include_str!("assets/bulletin.js"),
    )
        .into_response()
}
async fn list(
    State(s): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(q): axum::extract::Query<Page>,
) -> Response {
    let st = s.store.lock().await;
    let csrf = cookie(&headers, "bulletin_csrf").unwrap_or("");
    Html(render_list(st.automations(), q.page.unwrap_or(0)).replace("__CSRF__", &esc(csrf)))
        .into_response()
}
async fn runs(
    State(s): State<AppState>,
    axum::extract::Query(q): axum::extract::Query<Page>,
) -> Response {
    let st = s.store.lock().await;
    Html(render_runs(st.runs(), q.page.unwrap_or(0))).into_response()
}
#[derive(Deserialize, Default)]
struct Page {
    page: Option<usize>,
}
#[derive(Deserialize, Default)]
struct EditQuery {
    edit: Option<String>,
}

#[derive(Clone, Deserialize)]
struct CreateForm {
    #[serde(default)]
    csrf: String,
    preview_page: Option<bool>,
    edit_id: Option<String>,
    #[serde(default)]
    skill: String,
    #[serde(default)]
    arguments: String,
    working_dir: Option<String>,
    kind: String,
    seconds: Option<String>,
    weekdays: Option<String>,
    time: Option<String>,
    timezone: Option<String>,
}
fn form_error(f: &CreateForm, action: &str, message: &str) -> Response {
    let mut html = include_str!("assets/index.html").replace(
        "action=\"/api/automations\"",
        &format!("action=\"{}\"", esc(action)),
    );
    html = html.replace("__AUTOMATIONS__", "").replace("__RUNS__", "");
    html = html.replace("__CSRF__", &esc(&f.csrf));
    let fields: [(&str, &str); 8] = [
        ("skill", f.skill.as_str()),
        ("arguments", f.arguments.as_str()),
        ("working_dir", f.working_dir.as_deref().unwrap_or("")),
        ("kind", f.kind.as_str()),
        ("weekdays", f.weekdays.as_deref().unwrap_or("")),
        ("time", f.time.as_deref().unwrap_or("")),
        ("timezone", f.timezone.as_deref().unwrap_or("")),
        ("seconds", f.seconds.as_deref().unwrap_or("")),
    ];
    html = html.replace(
        "<button>Create schedule</button>",
        "<button>Save or create schedule</button><a href=\"/\">Cancel</a>",
    );
    for (name, value) in fields {
        let value = esc(value);
        if name == "kind" {
            html = html.replace(
                "<option value=\"interval\">Interval</option>",
                &format!(
                    "<option value=\"interval\" {}>Interval</option>",
                    if value == "interval" { "selected" } else { "" }
                ),
            );
            html = html.replace(
                "<option value=\"calendar\">Calendar</option>",
                &format!(
                    "<option value=\"calendar\" {}>Calendar</option>",
                    if value == "calendar" { "selected" } else { "" }
                ),
            );
        } else {
            html = html.replace(
                &format!("name=\"{name}\" "),
                &format!("name=\"{name}\" value=\"{value}\" "),
            );
        }
    }
    html = html.replace(
        "Enter schedule details to preview the next three runs.",
        &format!("<p role=\"alert\">{}</p>", esc(message)),
    );
    let mut response = (StatusCode::BAD_REQUEST, Html(html)).into_response();
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}
fn parse_weekdays(value: &str) -> anyhow::Result<Vec<u32>> {
    if value.is_empty() {
        return Ok(Vec::new());
    }
    let days: Result<Vec<u32>, _> = value.split(',').map(str::parse).collect();
    let days = days?;
    if days.iter().any(|d| *d > 6)
        || days.iter().collect::<std::collections::HashSet<_>>().len() != days.len()
    {
        anyhow::bail!("invalid weekday list")
    };
    Ok(days)
}
fn parse_interval(value: &str) -> anyhow::Result<u64> {
    let (number, multiplier) = match value.strip_suffix(['s', 'm', 'h', 'd']) {
        Some(v) => {
            let unit = value.chars().last().unwrap();
            (
                v,
                match unit {
                    's' => 1,
                    'm' => 60,
                    'h' => 3600,
                    'd' => 86400,
                    _ => unreachable!(),
                },
            )
        }
        None => (value, 1),
    };
    let seconds = number
        .parse::<u64>()?
        .checked_mul(multiplier)
        .ok_or_else(|| anyhow::anyhow!("interval overflow"))?;
    anyhow::ensure!(seconds >= 60, "interval must be at least 60 seconds");
    Ok(seconds)
}
async fn create(State(s): State<AppState>, axum::Form(f): axum::Form<CreateForm>) -> Response {
    let copy = f.clone();
    let schedule = if f.kind == "interval" {
        Schedule::Interval {
            seconds: match parse_interval(f.seconds.as_deref().unwrap_or("0")) {
                Ok(v) => v,
                Err(_) => {
                    return form_error(
                        &copy,
                        "/api/automations",
                        "Invalid interval. Use seconds or 15m, 1h, 1d (minimum 60 seconds).",
                    );
                }
            },
        }
    } else if f.kind == "calendar" {
        let weekdays = match parse_weekdays(f.weekdays.as_deref().unwrap_or("")) {
            Ok(v) => v,
            Err(_) => return form_error(&copy, "/api/automations", "Invalid weekday list."),
        };
        Schedule::Calendar {
            weekdays,
            time: f.time.unwrap_or_default(),
            timezone: f.timezone.unwrap_or_default(),
        }
    } else {
        return form_error(&copy, "/api/automations", "Invalid schedule kind.");
    };
    let now = Utc::now();
    let mut st = s.store.lock().await;
    let next_due = match schedule.next_after(now) {
        Ok(v) => v,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };
    let a = Automation {
        id: uuid::Uuid::new_v4().to_string(),
        skill: f.skill,
        arguments: f.arguments,
        working_dir: f
            .working_dir
            .filter(|v| !v.trim().is_empty())
            .map(PathBuf::from)
            .unwrap_or(s.config.default_working_dir.clone()),
        schedule,
        enabled: true,
        created_at: now,
        next_due,
        provider: s.config.default_provider.clone(),
        model: s.config.default_model.clone(),
        error: None,
    };
    match st.create(a, &s.config.default_working_dir.join("skills")) {
        Ok(()) => Redirect::to("/").into_response(),
        Err(e) => form_error(&copy, "/api/automations", &e.to_string()),
    }
}
async fn pause(
    State(s): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response {
    match s.store.lock().await.pause(&id) {
        Ok(()) => Redirect::to("/").into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}
async fn resume(
    State(s): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response {
    match s.store.lock().await.resume(&id, Utc::now()) {
        Ok(()) => Redirect::to("/").into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}
async fn update(
    State(s): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    axum::Form(f): axum::Form<CreateForm>,
) -> Response {
    let copy = f.clone();
    let schedule = if f.kind == "interval" {
        Schedule::Interval {
            seconds: match parse_interval(f.seconds.as_deref().unwrap_or("0")) {
                Ok(v) => v,
                Err(_) => {
                    return form_error(
                        &copy,
                        &format!("/api/automations/{}", esc(&id)),
                        "Invalid interval. Use seconds or 15m, 1h, 1d (minimum 60 seconds).",
                    );
                }
            },
        }
    } else if f.kind == "calendar" {
        Schedule::Calendar {
            weekdays: match parse_weekdays(f.weekdays.as_deref().unwrap_or("")) {
                Ok(v) => v,
                Err(_) => {
                    return form_error(
                        &copy,
                        &format!("/api/automations/{}", esc(&id)),
                        "Invalid weekday list.",
                    );
                }
            },
            time: f.time.unwrap_or_default(),
            timezone: f.timezone.unwrap_or_default(),
        }
    } else {
        return form_error(
            &copy,
            &format!("/api/automations/{}", esc(&id)),
            "Invalid schedule kind.",
        );
    };
    let now = Utc::now();
    let mut st = s.store.lock().await;
    let Some(old) = st.automations().iter().find(|a| a.id == id).cloned() else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let a = Automation {
        id: id.clone(),
        skill: f.skill,
        arguments: f.arguments,
        working_dir: f
            .working_dir
            .filter(|v| !v.trim().is_empty())
            .map(PathBuf::from)
            .unwrap_or(old.working_dir),
        schedule,
        enabled: old.enabled,
        created_at: old.created_at,
        next_due: old.next_due,
        provider: old.provider,
        model: old.model,
        error: None,
    };
    match st.update(a, &s.config.default_working_dir.join("skills"), now) {
        Ok(()) => Redirect::to("/").into_response(),
        Err(e) => form_error(
            &copy,
            &format!("/api/automations/{}", esc(&id)),
            &e.to_string(),
        ),
    }
}

async fn preview(axum::Form(f): axum::Form<CreateForm>) -> Response {
    let schedule = if f.kind == "interval" {
        Schedule::Interval {
            seconds: match parse_interval(f.seconds.as_deref().unwrap_or("0")) {
                Ok(v) => v,
                Err(_) => {
                    return form_error(
                        &f,
                        "/api/automations",
                        "Invalid interval. Use seconds or 15m, 1h, 1d (minimum 60 seconds).",
                    );
                }
            },
        }
    } else if f.kind == "calendar" {
        Schedule::Calendar {
            weekdays: match parse_weekdays(f.weekdays.as_deref().unwrap_or("")) {
                Ok(v) => v,
                Err(_) => return form_error(&f, "/api/automations", "Invalid weekday list."),
            },
            time: f.time.unwrap_or_default(),
            timezone: f.timezone.unwrap_or_default(),
        }
    } else {
        return form_error(&f, "/api/automations", "Invalid schedule kind.");
    };
    let mut after = Utc::now();
    let mut html = String::from("<ol>");
    for _ in 0..3 {
        match schedule.next_after(after) {
            Ok(next) => {
                let label = match &schedule {
                    Schedule::Calendar { timezone, .. } => timezone
                        .parse::<chrono_tz::Tz>()
                        .map(|zone| {
                            next.with_timezone(&zone)
                                .format("%A, %Y-%m-%d %H:%M %Z")
                                .to_string()
                        })
                        .unwrap_or_else(|_| next.to_rfc3339()),
                    Schedule::Interval { .. } => next.format("%A, %Y-%m-%d %H:%M UTC").to_string(),
                };
                html.push_str(&format!("<li>{}</li>", esc(&label)));
                after = next;
            }
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    "Invalid schedule. Check interval, weekdays, time, and IANA zone.",
                )
                    .into_response();
            }
        }
    }
    html.push_str("</ol>");
    if f.preview_page == Some(true) {
        let back = f
            .edit_id
            .as_deref()
            .filter(|id| !id.is_empty())
            .map(|id| format!("/?edit={}", esc(id)))
            .unwrap_or_else(|| "/".into());
        Html(format!("<!doctype html><html lang=\"en\"><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>Schedule preview</title><body><h1>Next three runs</h1>{html}<p><a href=\"{back}\">Return to schedule</a></p></body></html>")).into_response()
    } else {
        Html(html).into_response()
    }
}

fn render_list(items: &[Automation], page: usize) -> String {
    let mut out = String::from("<ul>");
    for a in items.iter().rev().skip(page.saturating_mul(50)).take(50) {
        let action = if a.enabled { "pause" } else { "resume" };
        out.push_str(&format!("<li><strong>{}</strong> · next {} <a href=\"/?edit={}\">Edit</a><form method=post action=\"/api/automations/{}/{}\"><input type=hidden name=csrf value=\"__CSRF__\"><button>{}</button></form></li>",esc(&a.skill),esc(&a.next_due.to_rfc3339()),esc(&a.id),esc(&a.id),action,if a.enabled{"Pause"}else{"Resume"}));
    }
    out.push_str("</ul>");
    if page > 0 {
        out.push_str(&format!("<a href=\"/api/automations?page={}\" hx-get=\"/api/automations?page={}\" hx-target=\"#automations\">Previous schedules</a> ",page-1,page-1));
    }
    if (page + 1) * 50 < items.len() {
        out.push_str(&format!("<a href=\"/api/automations?page={}\" hx-get=\"/api/automations?page={}\" hx-target=\"#automations\">Next schedules</a>",page+1,page+1));
    }
    out
}
fn render_runs(items: &[Run], page: usize) -> String {
    let mut out = String::from("<ul>");
    for r in items.iter().rev().skip(page.saturating_mul(50)).take(50) {
        let duration = r
            .ended_at
            .map(|e| (e - r.started_at).num_seconds().max(0).to_string() + "s")
            .unwrap_or_else(|| "running".into());
        out.push_str(&format!("<li><strong>{}</strong> · {} · {} {}<details><summary>Output</summary><pre>{}</pre></details></li>",esc(&format!("{:?}",r.status)),esc(&r.started_at.to_rfc3339()),esc(&duration),if r.truncated{"(truncated)"}else{""},esc(&r.output)));
    }
    out.push_str("</ul>");
    if page > 0 {
        out.push_str(&format!("<a href=\"/api/runs?page={}\" hx-get=\"/api/runs?page={}\" hx-target=\"#runs\">Previous runs</a> ",page-1,page-1));
    }
    if (page + 1) * 50 < items.len() {
        out.push_str(&format!("<a href=\"/api/runs?page={}\" hx-get=\"/api/runs?page={}\" hx-target=\"#runs\">Next runs</a>",page+1,page+1));
    }
    out
}
fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn html_escapes_untrusted_values() {
        assert_eq!(esc("<&\"'"), "&lt;&amp;&quot;&#39;");
    }
    #[test]
    fn host_requires_exact_origin_port() {
        assert!(origin_matches_host(
            "http://127.0.0.1:8123",
            "127.0.0.1:8123"
        ));
        assert!(!origin_matches_host(
            "http://127.0.0.1:8123",
            "127.0.0.1:8124"
        ));
        assert!(!origin_matches_host("http://127.0.0.1:8123", "evil:8123"));
    }
}
#[cfg(test)]
#[path = "ui_tests.rs"]
mod ui_tests;
