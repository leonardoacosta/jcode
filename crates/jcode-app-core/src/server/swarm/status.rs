use super::*;

pub(in crate::server) fn swarm_broadcast_key(
    swarm_id: &str,
    swarm_members: &Arc<RwLock<HashMap<String, SwarmMember>>>,
    swarms_by_id: &Arc<RwLock<HashMap<String, HashSet<String>>>>,
) -> String {
    format!(
        "{:p}:{:p}:{swarm_id}",
        Arc::as_ptr(swarm_members),
        Arc::as_ptr(swarms_by_id)
    )
}

pub(in crate::server) async fn broadcast_swarm_status_now(
    session_ids: Vec<String>,
    swarm_members: &Arc<RwLock<HashMap<String, SwarmMember>>>,
) {
    if session_ids.is_empty() {
        return;
    }

    let members_guard = swarm_members.read().await;
    let broadcast_terminal_retention = swarm_status_broadcast_terminal_retention();
    let members_list: Vec<crate::protocol::SwarmMemberStatus> = session_ids
        .iter()
        .filter_map(|sid| {
            members_guard
                .get(sid)
                .filter(|m| member_in_status_broadcast(m, broadcast_terminal_retention))
                .map(|m| {
                    let tokens = crate::session_metrics::token_totals(&m.session_id);
                    let context_limit = m
                        .runtime
                        .model
                        .as_deref()
                        .and_then(crate::provider::models::context_limit_for_model)
                        .map(|limit| limit as u64);
                    (m, tokens, context_limit)
                })
                .map(
                    |(m, tokens, context_limit)| crate::protocol::SwarmMemberStatus {
                        session_id: m.session_id.clone(),
                        friendly_name: m.friendly_name.clone(),
                        status: m.status.clone(),
                        detail: m.detail.clone(),
                        task_label: m.task_label.clone(),
                        role: Some(m.role.clone()),
                        is_headless: Some(m.is_headless),
                        live_attachments: Some(m.event_txs.len()),
                        status_age_secs: Some(status_age_secs(m.last_status_change)),
                        output_tail: m.output_tail.clone(),
                        report_back_to_session_id: m.report_back_to_session_id.clone(),
                        todo_progress: m.todo_progress,
                        todo_items: m.todo_items.clone(),
                        runtime: crate::protocol::SwarmMemberRuntime {
                            model: m.runtime.model.clone(),
                            provider: m.runtime.provider.clone(),
                            auth_method: m.runtime.auth_method.clone(),
                            effort: m.runtime.effort.clone(),
                            elapsed_secs: if matches!(
                                m.status.as_str(),
                                "running" | "streaming" | "thinking"
                            ) {
                                Some(m.joined_at.elapsed().as_secs())
                            } else {
                                Some(m.runtime.elapsed_secs.unwrap_or(0))
                            },
                            activity_age_secs: crate::session_metrics::last_activity_age_secs(
                                &m.session_id,
                            ),
                            context_tokens: tokens.map(|(context, _)| context).filter(|t| *t > 0),
                            context_limit,
                            output_tokens: tokens.map(|(_, output)| output).filter(|t| *t > 0),
                        },
                    },
                )
        })
        .collect();

    drop(members_guard);
    let event = ServerEvent::SwarmStatus {
        members: members_list,
    };
    for sid in session_ids {
        let _ = fanout_session_event(swarm_members, &sid, event.clone()).await;
    }
}

pub(in crate::server) async fn broadcast_swarm_status(
    swarm_id: &str,
    swarm_members: &Arc<RwLock<HashMap<String, SwarmMember>>>,
    swarms_by_id: &Arc<RwLock<HashMap<String, HashSet<String>>>>,
) {
    let session_ids: Vec<String> = {
        let swarms = swarms_by_id.read().await;
        swarms
            .get(swarm_id)
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default()
    };
    if session_ids.is_empty() {
        return;
    }

    if session_ids.len() < swarm_status_debounce_member_threshold() {
        broadcast_swarm_status_now(session_ids, swarm_members).await;
        return;
    }

    let key = swarm_broadcast_key(swarm_id, swarm_members, swarms_by_id);
    let should_spawn = {
        let mut pending = pending_swarm_status_broadcasts()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let entry = pending.entry(key.clone()).or_default();
        if entry.scheduled {
            entry.dirty = true;
            false
        } else {
            entry.scheduled = true;
            entry.dirty = false;
            true
        }
    };

    if !should_spawn {
        return;
    }

    let swarm_id = swarm_id.to_string();
    let swarm_members = Arc::clone(swarm_members);
    let swarms_by_id = Arc::clone(swarms_by_id);
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(swarm_status_debounce_ms())).await;
            let session_ids: Vec<String> = {
                let swarms = swarms_by_id.read().await;
                swarms
                    .get(&swarm_id)
                    .map(|s| s.iter().cloned().collect())
                    .unwrap_or_default()
            };
            broadcast_swarm_status_now(session_ids, &swarm_members).await;

            let mut pending = pending_swarm_status_broadcasts()
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let Some(entry) = pending.get_mut(&key) else {
                break;
            };
            if entry.dirty {
                entry.dirty = false;
                continue;
            }
            pending.remove(&key);
            break;
        }
    });
}
