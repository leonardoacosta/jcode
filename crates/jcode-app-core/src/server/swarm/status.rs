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

    // Resolve recipients and publish under one guard. Besides avoiding a lock
    // and full payload clone per session, this keeps an older snapshot from
    // being queued after a concurrent newer snapshot on the same channel.
    let mut members_guard = swarm_members.write().await;
    let mut targets = Vec::new();
    for sid in &session_ids {
        let Some(member) = members_guard.get_mut(sid) else {
            continue;
        };
        member.event_txs.retain(|_, tx| !tx.is_closed());
        if let Some((_, tx)) = member.event_txs.iter().next() {
            member.event_tx = tx.clone();
            targets.extend(member.event_txs.values().cloned());
        } else if !member.event_tx.is_closed() {
            targets.push(member.event_tx.clone());
        }
    }
    if targets.is_empty() {
        return;
    }
    let broadcast_terminal_retention = swarm_status_broadcast_terminal_retention();
    let members_list: Vec<crate::protocol::SwarmMemberStatus> = session_ids
        .iter()
        .filter_map(|sid| {
            members_guard
                .get(sid)
                .filter(|m| member_in_status_broadcast(m, broadcast_terminal_retention))
                .map(|m| {
                    let tokens = crate::session_metrics::token_totals(&m.session_id);
                    let wait = crate::wait_registry::current(&m.session_id);
                    let context_limit = m
                        .runtime
                        .model
                        .as_deref()
                        .and_then(crate::provider::models::context_limit_for_model)
                        .map(|limit| limit as u64);
                    (m, tokens, context_limit, wait)
                })
                .map(
                    |(m, tokens, context_limit, wait)| crate::protocol::SwarmMemberStatus {
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
                            live_status: crate::server::live_status::snapshot(&m.session_id),
                            wait_reason: wait.as_ref().map(|(reason, _)| reason.clone()).or_else(
                                || {
                                    (!matches!(
                                        m.status.as_str(),
                                        "running" | "streaming" | "thinking"
                                    ))
                                    .then(|| m.runtime.todo_blocked_reason.clone())
                                    .flatten()
                                },
                            ),
                            wait_age_secs: wait.map(|(_, age)| age),
                            todo_blocked_reason: m.runtime.todo_blocked_reason.clone(),
                            todo_statuses: m.runtime.todo_statuses.clone(),
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

    let event = ServerEvent::SwarmStatus {
        members: members_list,
    };
    // Unbounded queue sends never await. Do not route through the generic
    // event projector: SwarmStatus itself cannot change a live projection.
    crate::server::state::send_event_to_targets(&targets, event);
}

pub(in crate::server) async fn monitor_wait_status(
    swarm_members: Arc<RwLock<HashMap<String, SwarmMember>>>,
    swarms_by_id: Arc<RwLock<HashMap<String, HashSet<String>>>>,
) {
    let mut changes = crate::wait_registry::subscribe_changes();
    let mut live_changes = crate::server::live_status::subscribe_changes();
    // Transitions are pushed by the watch channels above. The tick only ages
    // displayed wait/elapsed counters, and each tick fans a full swarm status
    // out to every attached client, so keep it coarse to let clients sleep.
    let mut interval = tokio::time::interval(Duration::from_secs(5));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut previous = HashSet::<String>::new();
    loop {
        tokio::select! {
            result = changes.changed() => if result.is_err() { return; },
            result = live_changes.changed() => if result.is_err() { return; },
            _ = interval.tick() => {},
        }
        let current: HashSet<_> = crate::wait_registry::session_ids().into_iter().collect();
        let affected: HashSet<_> = {
            let members = swarm_members.read().await;
            previous
                .union(&current)
                .filter_map(|sid| members.get(sid).and_then(|member| member.swarm_id.clone()))
                .chain(
                    members
                        .values()
                        .filter(|member| {
                            matches!(member.status.as_str(), "running" | "streaming" | "thinking")
                        })
                        .filter_map(|member| member.swarm_id.clone()),
                )
                .collect()
        };
        previous = current;
        for swarm_id in affected {
            broadcast_swarm_status(&swarm_id, &swarm_members, &swarms_by_id).await;
        }
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
