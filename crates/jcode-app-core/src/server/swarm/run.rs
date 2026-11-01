use super::*;

pub(in crate::server) async fn run_swarm_task(
    agent: Arc<Mutex<Agent>>,
    description: &str,
    subagent_type: &str,
    prompt: &str,
) -> Result<String> {
    let started = Instant::now();
    let (provider, registry, session_id, working_dir, coordinator_model, provider_key, route) = {
        let agent = agent.lock().await;
        (
            agent.provider_fork(),
            agent.registry(),
            agent.session_id().to_string(),
            agent.working_dir().map(PathBuf::from),
            agent.provider_model(),
            agent.session_provider_key(),
            agent.session_route_api_method(),
        )
    };
    let parent_session_id = session_id.clone();
    let mut session = Session::create(
        Some(session_id),
        Some(format!("{} (@{} swarm)", description, subagent_type)),
    );
    let child_session_id = session.id.clone();
    session.model = Some(coordinator_model);
    // Inherit the coordinator's exact auth identity so the forked worker keeps
    // the same provider/auth route (OAuth vs API, openai-compatible profile)
    // instead of silently falling back to the config default on persistence.
    session.provider_key = provider_key;
    session.route_api_method = route;
    if let Some(dir) = working_dir {
        session.working_dir = Some(dir.display().to_string());
    }
    session.save()?;

    log_swarm_lifecycle(
        "task_start",
        vec![
            ("parent_session_id", parent_session_id.clone()),
            ("child_session_id", child_session_id.clone()),
            ("subagent_type", subagent_type.to_string()),
            ("description_chars", description.chars().count().to_string()),
            ("prompt_chars", prompt.chars().count().to_string()),
        ],
    );

    let mut allowed: HashSet<String> = registry.tool_names().await.into_iter().collect();
    for blocked in ["subagent", "task", "todo", "todowrite", "todoread"] {
        allowed.remove(blocked);
    }
    crate::config::config()
        .tools
        .apply_to_allowed_set(&mut allowed);

    let mut worker = Agent::new_with_session(provider, registry, session, Some(allowed));
    match worker.run_once_capture(prompt).await {
        Ok(output) => {
            log_swarm_lifecycle(
                "task_done",
                vec![
                    ("parent_session_id", parent_session_id),
                    ("child_session_id", child_session_id),
                    ("subagent_type", subagent_type.to_string()),
                    ("output_chars", output.chars().count().to_string()),
                    ("elapsed_ms", started.elapsed().as_millis().to_string()),
                ],
            );
            Ok(output)
        }
        Err(error) => {
            crate::logging::event_warn(
                "SWARM_LIFECYCLE",
                vec![
                    ("phase", "task_error".to_string()),
                    ("parent_session_id", parent_session_id),
                    ("child_session_id", child_session_id),
                    ("subagent_type", subagent_type.to_string()),
                    ("error", error.to_string()),
                    ("elapsed_ms", started.elapsed().as_millis().to_string()),
                ],
            );
            Err(error)
        }
    }
}

pub(in crate::server) async fn run_swarm_message(
    agent: Arc<Mutex<Agent>>,
    message: &str,
) -> Result<String> {
    let started = Instant::now();
    log_swarm_lifecycle(
        "message_start",
        vec![("message_chars", message.chars().count().to_string())],
    );
    let working_dir = {
        let agent = agent.lock().await;
        agent.working_dir().map(|dir| dir.to_string())
    };
    let working_dir_hint = working_dir
        .as_deref()
        .map(|dir| format!("Working directory: {}\n", dir))
        .unwrap_or_default();

    let planner_prompt = format!(
        "{working_dir_hint}You are a task planner. Break the request into 2-4 subtasks. \
Return ONLY a JSON array of objects with keys: description, prompt, subagent_type. \
No extra text.\n\nRequest:\n{message}"
    );

    let plan_text = {
        let mut agent = agent.lock().await;
        agent.run_once_capture(&planner_prompt).await?
    };

    let mut tasks = parse_swarm_tasks(&plan_text);
    if tasks.is_empty() {
        tasks.push(SwarmTaskSpec {
            description: "Main task".to_string(),
            prompt: message.to_string(),
            subagent_type: Some("general".to_string()),
        });
    }
    log_swarm_lifecycle(
        "message_plan_done",
        vec![
            ("task_count", tasks.len().to_string()),
            ("plan_chars", plan_text.chars().count().to_string()),
        ],
    );

    let task_futures = tasks.iter().map(|task| {
        let agent = agent.clone();
        let working_dir_hint = working_dir_hint.clone();
        let description = task.description.clone();
        let prompt = format!("{working_dir_hint}{}", task.prompt);
        let subagent_type = task
            .subagent_type
            .clone()
            .unwrap_or_else(|| "general".to_string());
        async move {
            let output = run_swarm_task(agent, &description, &subagent_type, &prompt).await?;
            Ok::<(String, String), anyhow::Error>((description, output))
        }
    });
    let task_outputs = try_join_all(task_futures).await?;

    let mut integration_prompt = String::new();
    integration_prompt.push_str(
        "You are the coordinator. Complete the original request using the subagent outputs below. ",
    );
    integration_prompt.push_str("Do not stop early; run any requested tests and fix failures.\n\n");
    integration_prompt.push_str("Original request:\n");
    integration_prompt.push_str(message);
    integration_prompt.push_str("\n\nSubagent outputs:\n");
    for (desc, output) in &task_outputs {
        integration_prompt.push_str(&format!("\n--- {} ---\n{}\n", desc, output));
    }
    integration_prompt.push_str("\nNow complete the task.\n");

    let final_output = {
        let mut agent = agent.lock().await;
        agent.run_once_capture(&integration_prompt).await?
    };

    log_swarm_lifecycle(
        "message_done",
        vec![
            ("task_count", task_outputs.len().to_string()),
            ("output_chars", final_output.chars().count().to_string()),
            ("elapsed_ms", started.elapsed().as_millis().to_string()),
        ],
    );

    Ok(final_output)
}

#[derive(Debug, Deserialize)]
pub(in crate::server) struct SwarmTaskSpec {
    pub(in crate::server) description: String,
    pub(in crate::server) prompt: String,
    #[serde(default)]
    pub(in crate::server) subagent_type: Option<String>,
}

pub(in crate::server) fn parse_swarm_tasks(text: &str) -> Vec<SwarmTaskSpec> {
    if let Ok(tasks) = serde_json::from_str::<Vec<SwarmTaskSpec>>(text) {
        return tasks;
    }

    if let (Some(start), Some(end)) = (text.find('['), text.rfind(']'))
        && start < end
        && let Ok(tasks) = serde_json::from_str::<Vec<SwarmTaskSpec>>(&text[start..=end])
    {
        return tasks;
    }

    Vec::new()
}
