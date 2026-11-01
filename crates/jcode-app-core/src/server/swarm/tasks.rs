use super::*;

#[expect(
    clippy::too_many_arguments,
    reason = "task progress touch updates durable progress plus swarm persistence and coordinator-facing state in one helper"
)]
pub(in crate::server) async fn touch_swarm_task_progress(
    swarm_id: &str,
    task_id: &str,
    assigned_session_id: Option<&str>,
    detail: Option<String>,
    checkpoint_summary: Option<String>,
    swarm_members: &Arc<RwLock<HashMap<String, SwarmMember>>>,
    swarms_by_id: &Arc<RwLock<HashMap<String, HashSet<String>>>>,
    swarm_plans: &Arc<RwLock<HashMap<String, VersionedPlan>>>,
    swarm_coordinators: &Arc<RwLock<HashMap<String, String>>>,
) -> bool {
    let now_ms = now_unix_ms();
    let revived = {
        let mut plans = swarm_plans.write().await;
        let Some(plan) = plans.get_mut(swarm_id) else {
            return false;
        };
        let Some(item) = plan.items.iter_mut().find(|item| item.id == task_id) else {
            return false;
        };
        let progress = plan.task_progress.entry(task_id.to_string()).or_default();
        if let Some(session_id) = assigned_session_id {
            progress.assigned_session_id = Some(session_id.to_string());
        }
        // Heartbeats/checkpoints are proof of life for the assigned session:
        // fold them into the member activity clock so swarm status reflects
        // busy workers whose lifecycle status has not changed in a while.
        if let Some(session_id) = progress.assigned_session_id.as_deref() {
            crate::session_metrics::record_activity(session_id);
        }
        progress.last_heartbeat_unix_ms = Some(now_ms);
        progress.heartbeat_count = Some(progress.heartbeat_count.unwrap_or(0) + 1);
        if let Some(detail) = detail {
            progress.last_detail = Some(truncate_detail(&detail, 120));
        }
        if let Some(summary) = checkpoint_summary {
            progress.last_checkpoint_unix_ms = Some(now_ms);
            progress.checkpoint_summary = Some(truncate_detail(&summary, 120));
            progress.checkpoint_count = Some(progress.checkpoint_count.unwrap_or(0) + 1);
        }
        if item.status == "running_stale" {
            item.status = "running".to_string();
            progress.stale_since_unix_ms = None;
            plan.version += 1;
            true
        } else {
            false
        }
    };
    let swarm_state = SwarmState {
        members: Arc::clone(swarm_members),
        swarms_by_id: Arc::clone(swarms_by_id),
        plans: Arc::clone(swarm_plans),
        coordinators: Arc::clone(swarm_coordinators),
    };
    persist_swarm_state_for(swarm_id, &swarm_state).await;
    revived
}

pub(in crate::server) async fn refresh_swarm_task_staleness(
    swarm_members: &Arc<RwLock<HashMap<String, SwarmMember>>>,
    swarms_by_id: &Arc<RwLock<HashMap<String, HashSet<String>>>>,
    swarm_plans: &Arc<RwLock<HashMap<String, VersionedPlan>>>,
    swarm_coordinators: &Arc<RwLock<HashMap<String, String>>>,
) {
    let now_ms = now_unix_ms();
    let stale_after_ms = swarm_task_stale_after().as_millis() as u64;
    let changed_swarm_ids = {
        let mut plans = swarm_plans.write().await;
        let mut changed = Vec::new();
        for (swarm_id, plan) in plans.iter_mut() {
            let mut swarm_changed = false;
            for item in &mut plan.items {
                if !matches!(item.status.as_str(), "running" | "running_stale") {
                    continue;
                }
                let progress = plan.task_progress.entry(item.id.clone()).or_default();
                let last_heartbeat = progress
                    .last_heartbeat_unix_ms
                    .or(progress.started_at_unix_ms)
                    .or(progress.assigned_at_unix_ms);
                let is_stale = last_heartbeat
                    .map(|ts| now_ms.saturating_sub(ts) >= stale_after_ms)
                    .unwrap_or(true);
                match (item.status.as_str(), is_stale) {
                    ("running", true) => {
                        item.status = "running_stale".to_string();
                        progress.stale_since_unix_ms.get_or_insert(now_ms);
                        plan.version += 1;
                        swarm_changed = true;
                    }
                    ("running_stale", false) => {
                        item.status = "running".to_string();
                        progress.stale_since_unix_ms = None;
                        plan.version += 1;
                        swarm_changed = true;
                    }
                    _ => {}
                }
            }
            if swarm_changed {
                changed.push(swarm_id.clone());
            }
        }
        changed
    };

    for swarm_id in changed_swarm_ids {
        let swarm_state = SwarmState {
            members: Arc::clone(swarm_members),
            swarms_by_id: Arc::clone(swarms_by_id),
            plans: Arc::clone(swarm_plans),
            coordinators: Arc::clone(swarm_coordinators),
        };
        persist_swarm_state_for(&swarm_id, &swarm_state).await;
        broadcast_swarm_plan(
            &swarm_id,
            Some("task_staleness_changed".to_string()),
            swarm_plans,
            swarm_members,
            swarms_by_id,
        )
        .await;
    }

    // Second phase: salvage in-flight items whose assignee is dead. Staleness
    // marking above only reflects missing heartbeats; when the assigned member
    // is gone from the swarm or sits in a terminal lifecycle status, no
    // heartbeat or turn-end will ever arrive, so the item must be requeued
    // (or failed at the reclaim cap) instead of pulsing running_stale forever.
    // A terminal-status member gets a grace period before salvage: reload
    // recovery briefly marks resumable members `crashed` before restoring
    // them, and salvaging inside that window would double-assign their work.
    let salvage_grace = swarm_task_stale_after();
    let salvage_candidates: Vec<(String, String)> = {
        let plans = swarm_plans.read().await;
        let members = swarm_members.read().await;
        let mut pairs = std::collections::BTreeSet::new();
        for (swarm_id, plan) in plans.iter() {
            for item in &plan.items {
                if !matches!(item.status.as_str(), "running" | "running_stale" | "queued") {
                    continue;
                }
                let assignee = item.assigned_to.as_deref().or_else(|| {
                    plan.task_progress
                        .get(&item.id)
                        .and_then(|progress| progress.assigned_session_id.as_deref())
                });
                let Some(assignee) = assignee else {
                    continue;
                };
                let assignee_is_dead = match members.get(assignee) {
                    None => true,
                    Some(member) => {
                        member_status_is_dead(&member.status)
                            && member.last_status_change.elapsed() >= salvage_grace
                    }
                };
                if assignee_is_dead {
                    pairs.insert((swarm_id.clone(), assignee.to_string()));
                }
            }
        }
        pairs.into_iter().collect()
    };
    for (swarm_id, session_id) in salvage_candidates {
        salvage_assignments_of_dead_member(
            &session_id,
            &swarm_id,
            swarm_members,
            swarms_by_id,
            swarm_plans,
            swarm_coordinators,
        )
        .await;
    }
}
