use super::*;

/// Spawned workers whose work is finished (`ready` report-back or a terminal
/// status) and whose status has not changed for at least `idle_after`.
/// Only sessions spawned by another agent (`report_back_to_session_id` set)
/// and not holding the coordinator role are eligible; user-created sessions
/// are never reaped.
pub(in crate::server) fn idle_spawned_worker_reap_candidates(
    members: &HashMap<String, SwarmMember>,
    idle_after: Duration,
) -> Vec<String> {
    members
        .values()
        .filter(|member| member.report_back_to_session_id.is_some())
        .filter(|member| member.role != "coordinator")
        .filter(|member| member.status == "ready" || member_status_is_terminal(&member.status))
        .filter(|member| member.last_status_change.elapsed() >= idle_after)
        .map(|member| member.session_id.clone())
        .collect()
}

/// Outcome of salvaging one dead member's plan assignments.
#[derive(Debug, Default, PartialEq, Eq)]
pub(in crate::server) struct DeadMemberSalvage {
    /// Tasks released back to `queued` for automatic re-dispatch.
    pub requeued_task_ids: Vec<String>,
    /// Tasks marked `failed` because the automatic reclaim cap was reached.
    pub failed_task_ids: Vec<String>,
}

impl DeadMemberSalvage {
    pub(in crate::server) fn is_empty(&self) -> bool {
        self.requeued_task_ids.is_empty() && self.failed_task_ids.is_empty()
    }

    /// Human-readable notification body for the coordinator/owner.
    fn describe(&self, worker_label: &str) -> String {
        let mut parts = vec![format!(
            "⚠ Worker {} died while holding swarm task assignment(s).",
            worker_label
        )];
        if !self.requeued_task_ids.is_empty() {
            parts.push(format!(
                "Requeued for automatic re-dispatch: {}.",
                self.requeued_task_ids.join(", ")
            ));
        }
        if !self.failed_task_ids.is_empty() {
            parts.push(format!(
                "Marked failed (automatic reclaim cap reached): {}. Use retry or assign_task to redispatch explicitly.",
                self.failed_task_ids.join(", ")
            ));
        }
        parts.push(
            "Queued tasks will be picked up by assign_next/run_plan; check plan_status for details."
                .to_string(),
        );
        parts.join(" ")
    }
}

/// Requeue (or, past [`crate::plan::MAX_DEAD_ASSIGNEE_RECLAIMS`], fail) every
/// non-terminal plan item assigned to `session_id`.
///
/// This is the eager counterpart to the assign-time stranded-task reclaim: a
/// worker that crashes, stops, or leaves the swarm mid-task leaves its items
/// `running`/`queued` and assigned to a corpse, where the scheduler cannot see
/// them and a driving `run_plan` stalls into its transient-stall error.
/// Salvaging at the moment the member dies converts that silent strand into
/// normal queued work. Uses the same per-node reclaim counter and cap as the
/// assign-time path so repeatedly lethal nodes fail loudly instead of cycling
/// workers forever.
pub(in crate::server) fn salvage_plan_assignments_of(
    plan: &mut VersionedPlan,
    session_id: &str,
) -> DeadMemberSalvage {
    let now_ms = now_unix_ms();
    let mut outcome = DeadMemberSalvage::default();
    let assigned_ids: Vec<String> = plan
        .items
        .iter()
        .filter(|item| {
            item.assigned_to.as_deref() == Some(session_id)
                && !crate::plan::is_terminal_status(&item.status)
        })
        .map(|item| item.id.clone())
        .collect();
    for task_id in assigned_ids {
        let reclaims = plan
            .task_progress
            .get(&task_id)
            .and_then(|progress| progress.dead_assignee_reclaims)
            .unwrap_or(0);
        if reclaims >= crate::plan::MAX_DEAD_ASSIGNEE_RECLAIMS {
            if let Some(item) = plan.items.iter_mut().find(|item| item.id == task_id) {
                item.status = "failed".to_string();
                item.assigned_to = None;
            }
            let progress = plan.task_progress.entry(task_id.clone()).or_default();
            progress.assigned_session_id = None;
            progress.completed_at_unix_ms = Some(now_ms);
            progress.stale_since_unix_ms = None;
            progress.checkpoint_summary = Some(truncate_detail(
                &format!(
                    "failed: assigned worker {} died and the automatic reclaim cap was reached",
                    session_id
                ),
                120,
            ));
            plan.version += 1;
            outcome.failed_task_ids.push(task_id);
        } else if crate::plan::reclaim_stranded_assignment(plan, &task_id) {
            if let Some(item) = plan.items.iter_mut().find(|item| item.id == task_id) {
                item.status = "queued".to_string();
            }
            let progress = plan.task_progress.entry(task_id.clone()).or_default();
            progress.stale_since_unix_ms = None;
            outcome.requeued_task_ids.push(task_id);
        }
    }
    outcome
}

/// Salvage `session_id`'s plan assignments in `swarm_id`, then persist,
/// broadcast the plan change, and notify the swarm coordinator so the death is
/// visible instead of silent. No-ops (and skips all I/O) when the member held
/// no non-terminal assignments.
pub(in crate::server) async fn salvage_assignments_of_dead_member(
    session_id: &str,
    swarm_id: &str,
    swarm_members: &Arc<RwLock<HashMap<String, SwarmMember>>>,
    swarms_by_id: &Arc<RwLock<HashMap<String, HashSet<String>>>>,
    swarm_plans: &Arc<RwLock<HashMap<String, VersionedPlan>>>,
    swarm_coordinators: &Arc<RwLock<HashMap<String, String>>>,
) -> DeadMemberSalvage {
    let outcome = {
        let mut plans = swarm_plans.write().await;
        match plans.get_mut(swarm_id) {
            Some(plan) => salvage_plan_assignments_of(plan, session_id),
            None => DeadMemberSalvage::default(),
        }
    };
    if outcome.is_empty() {
        return outcome;
    }

    log_swarm_lifecycle(
        "dead_member_tasks_salvaged",
        vec![
            ("session_id", session_id.to_string()),
            ("swarm_id", swarm_id.to_string()),
            ("requeued_task_ids", outcome.requeued_task_ids.join(",")),
            ("failed_task_ids", outcome.failed_task_ids.join(",")),
        ],
    );

    let swarm_state = SwarmState {
        members: Arc::clone(swarm_members),
        swarms_by_id: Arc::clone(swarms_by_id),
        plans: Arc::clone(swarm_plans),
        coordinators: Arc::clone(swarm_coordinators),
    };
    persist_swarm_state_for(swarm_id, &swarm_state).await;
    broadcast_swarm_plan(
        swarm_id,
        Some("task_salvaged_dead_worker".to_string()),
        swarm_plans,
        swarm_members,
        swarms_by_id,
    )
    .await;
    notify_coordinator_of_salvage(
        session_id,
        swarm_id,
        &outcome,
        swarm_members,
        swarm_coordinators,
    )
    .await;
    outcome
}

/// Deliver a salvage notification to the swarm's current coordinator (when it
/// is not the dead session itself).
pub(in crate::server) async fn notify_coordinator_of_salvage(
    session_id: &str,
    swarm_id: &str,
    outcome: &DeadMemberSalvage,
    swarm_members: &Arc<RwLock<HashMap<String, SwarmMember>>>,
    swarm_coordinators: &Arc<RwLock<HashMap<String, String>>>,
) {
    let coordinator_id = {
        let coordinators = swarm_coordinators.read().await;
        coordinators.get(swarm_id).cloned()
    };
    let Some(coordinator_id) = coordinator_id.filter(|id| id != session_id) else {
        return;
    };
    let label = {
        let members = swarm_members.read().await;
        members
            .get(session_id)
            .and_then(|member| member.friendly_name.clone())
    }
    .unwrap_or_else(|| session_id[..8.min(session_id.len())].to_string());
    let _ = fanout_session_event(
        swarm_members,
        &coordinator_id,
        ServerEvent::Notification {
            from_session: session_id.to_string(),
            from_name: Some(label.clone()),
            notification_type: NotificationType::Message {
                scope: Some("swarm".to_string()),
                channel: None,
                tldr: None,
            },
            message: outcome.describe(&label),
        },
    )
    .await;
}

pub(in crate::server) async fn remove_session_from_swarm(
    session_id: &str,
    swarm_id: &str,
    swarm_members: &Arc<RwLock<HashMap<String, SwarmMember>>>,
    swarms_by_id: &Arc<RwLock<HashMap<String, HashSet<String>>>>,
    swarm_coordinators: &Arc<RwLock<HashMap<String, String>>>,
    swarm_plans: &Arc<RwLock<HashMap<String, VersionedPlan>>>,
) {
    let started = Instant::now();
    log_swarm_lifecycle(
        "member_remove_start",
        vec![
            ("session_id", session_id.to_string()),
            ("swarm_id", swarm_id.to_string()),
        ],
    );
    // Capture the departing member's own spawner before any teardown. Some
    // callers remove the member from the map before calling us, so this is
    // best-effort: when unavailable the orphan-reparenting below falls back to
    // the swarm coordinator.
    let departing_parent: Option<String> = {
        let members = swarm_members.read().await;
        members
            .get(session_id)
            .and_then(|member| member.report_back_to_session_id.clone())
    };
    // A leaving member can no longer drive its plan assignments (crash, stop,
    // disconnect, feature-off all funnel through here). Salvage before any
    // membership state is torn down so the coordinator notification can still
    // resolve names and fan out.
    salvage_assignments_of_dead_member(
        session_id,
        swarm_id,
        swarm_members,
        swarms_by_id,
        swarm_plans,
        swarm_coordinators,
    )
    .await;
    remove_plan_participant(swarm_id, session_id, swarm_plans).await;
    // Removal covers stop, crash and orphan detection: free its file claims.
    unregister_claim_worker(session_id);

    {
        let mut swarms = swarms_by_id.write().await;
        if let Some(swarm) = swarms.get_mut(swarm_id) {
            swarm.remove(session_id);
            if swarm.is_empty() {
                swarms.remove(swarm_id);
            }
        }
    }

    let was_coordinator = {
        let coordinators = swarm_coordinators.read().await;
        coordinators
            .get(swarm_id)
            .map(|id| id == session_id)
            .unwrap_or(false)
    };

    let mut elected_coordinator = None;
    if was_coordinator {
        let new_coordinator = {
            let swarms = swarms_by_id.read().await;
            let members = swarm_members.read().await;
            swarms.get(swarm_id).and_then(|swarm| {
                swarm
                    .iter()
                    .filter_map(|id| {
                        members
                            .get(id)
                            .filter(|member| !member.is_headless)
                            .map(|_| id.clone())
                    })
                    .min()
            })
        };

        {
            let mut coordinators = swarm_coordinators.write().await;
            coordinators.remove(swarm_id);
            if let Some(ref new_id) = new_coordinator {
                coordinators.insert(swarm_id.to_string(), new_id.clone());
            }
        }

        if let Some(new_id) = new_coordinator {
            elected_coordinator = Some(new_id.clone());
            {
                let mut members = swarm_members.write().await;
                if let Some(member) = members.get_mut(&new_id) {
                    member.role = "coordinator".to_string();
                }
            }
            let mut plans = swarm_plans.write().await;
            if let Some(vp) = plans.get_mut(swarm_id) {
                vp.participants.insert(new_id.clone());
            }
            let members = swarm_members.read().await;
            if let Some(member) = members.get(&new_id) {
                let _ = member.event_tx.send(ServerEvent::Notification {
                    from_session: new_id.clone(),
                    from_name: member.friendly_name.clone(),
                    notification_type: NotificationType::Message {
                        scope: Some("swarm".to_string()),
                        channel: None,
                        tldr: None,
                    },
                    message: "You are now the coordinator for this swarm.".to_string(),
                });
            }
        }
    }

    {
        let mut members = swarm_members.write().await;
        if let Some(member) = members.get_mut(session_id) {
            member.role = "agent".to_string();
        }
    }

    // Reparent the departing member's direct children so the spawn tree never
    // holds dangling report-back edges. Orphaned subtrees would otherwise
    // silently change ownership semantics: stop permissions, subtree broadcast
    // scope, and completion report-back all walk this chain. Children are
    // attached to their grandparent when it is still a live member of this
    // swarm, otherwise to the current coordinator, otherwise they become
    // roots (report_back_to_session_id = None).
    let fallback_parent: Option<String> = {
        let grandparent_is_live = if let Some(ref parent) = departing_parent {
            parent != session_id && {
                let members = swarm_members.read().await;
                members
                    .get(parent)
                    .is_some_and(|member| member.swarm_id.as_deref() == Some(swarm_id))
            }
        } else {
            false
        };
        if grandparent_is_live {
            departing_parent.clone()
        } else {
            let coordinators = swarm_coordinators.read().await;
            coordinators
                .get(swarm_id)
                .filter(|coordinator| coordinator.as_str() != session_id)
                .cloned()
        }
    };
    let mut reparented: Vec<String> = Vec::new();
    // With no live fallback, keep the children pointing at their departing
    // spawner. The common cause is a TUI owner dropping off during a server
    // reload: promoting its workers to roots let one of them (a headless
    // worker) claim the coordinator slot, and the owner could no longer
    // resume, retry or wake its own workers when it reconnected moments
    // later. A dangling report-back edge is harmless: the ancestor walk stops
    // at a missing member and report-back delivery already tolerates an
    // absent recipient.
    if fallback_parent.is_some() {
        let mut members = swarm_members.write().await;
        for member in members.values_mut() {
            if member.swarm_id.as_deref() == Some(swarm_id)
                && member.report_back_to_session_id.as_deref() == Some(session_id)
            {
                member.report_back_to_session_id = fallback_parent
                    .clone()
                    .filter(|parent| parent != &member.session_id);
                reparented.push(member.session_id.clone());
            }
        }
    }
    if !reparented.is_empty() {
        log_swarm_lifecycle(
            "member_remove_reparent",
            vec![
                ("session_id", session_id.to_string()),
                ("swarm_id", swarm_id.to_string()),
                (
                    "new_parent",
                    fallback_parent
                        .clone()
                        .unwrap_or_else(|| "none (promoted to root)".to_string()),
                ),
                ("reparented_children", reparented.join(",")),
            ],
        );
    }

    if swarm_plans.read().await.contains_key(swarm_id) {
        let swarm_state = SwarmState {
            members: Arc::clone(swarm_members),
            swarms_by_id: Arc::clone(swarms_by_id),
            plans: Arc::clone(swarm_plans),
            coordinators: Arc::clone(swarm_coordinators),
        };
        persist_swarm_state_for(swarm_id, &swarm_state).await;
    } else {
        let swarm_state = SwarmState {
            members: Arc::clone(swarm_members),
            swarms_by_id: Arc::clone(swarms_by_id),
            plans: Arc::clone(swarm_plans),
            coordinators: Arc::clone(swarm_coordinators),
        };
        remove_persisted_swarm_state_for(swarm_id, &swarm_state).await;
    }

    let remaining_member_count = swarms_by_id
        .read()
        .await
        .get(swarm_id)
        .map(|members| members.len())
        .unwrap_or_default();
    log_swarm_lifecycle(
        "member_remove_done",
        vec![
            ("session_id", session_id.to_string()),
            ("swarm_id", swarm_id.to_string()),
            ("was_coordinator", was_coordinator.to_string()),
            (
                "new_coordinator_session_id",
                elected_coordinator.unwrap_or_else(|| "none".to_string()),
            ),
            ("remaining_member_count", remaining_member_count.to_string()),
            ("elapsed_ms", started.elapsed().as_millis().to_string()),
        ],
    );
    broadcast_swarm_status(swarm_id, swarm_members, swarms_by_id).await;
}

/// Whether `member` is an idle headless worker a restart stopped and whose
/// owner has not yet been told (issue #1651).
pub(in crate::server) fn member_awaits_unrestored_notice(member: &SwarmMember) -> bool {
    member.is_headless
        && member.status == "stopped"
        && member.detail.as_deref().is_some_and(|detail| {
            detail.contains(crate::server::swarm_persistence::IDLE_WORKER_NOT_RESTORED_NOTE)
                && !detail.contains(crate::server::swarm_persistence::OWNER_NOTIFIED_NOTE)
        })
}

/// Tell `owner_session_id` about idle headless workers that the last server
/// restart stopped (issue #1651).
///
/// No headless process survives a restart, so an idle worker loads as
/// `stopped`. A TUI member re-marks itself ready on subscribe, but a headless
/// worker has nothing to re-attach, so without this its coordinator only
/// learns it is gone by polling the member list. Called when a session attaches
/// (subscribe or resume): workers that report back to it, or whose swarm it
/// coordinates, are listed in one swarm notification and marked notified so
/// the notice is delivered once. Returns the number of workers reported.
pub(in crate::server) async fn notify_owner_of_unrestored_idle_workers(
    owner_session_id: &str,
    swarm_state: &SwarmState,
) -> usize {
    let coordinated_swarms: HashSet<String> = {
        let coordinators = swarm_state.coordinators.read().await;
        coordinators
            .iter()
            .filter(|(_, coordinator)| coordinator.as_str() == owner_session_id)
            .map(|(swarm_id, _)| swarm_id.clone())
            .collect()
    };
    let (owner_name, reported) = {
        let mut members = swarm_state.members.write().await;
        let owner_name = members
            .get(owner_session_id)
            .and_then(|member| member.friendly_name.clone());
        let mut reported: Vec<(String, String, Option<String>)> = Vec::new();
        for member in members.values_mut() {
            if member.session_id == owner_session_id || !member_awaits_unrestored_notice(member) {
                continue;
            }
            let owned_by_report_back =
                member.report_back_to_session_id.as_deref() == Some(owner_session_id);
            let owned_by_coordination = member.report_back_to_session_id.is_none()
                && member
                    .swarm_id
                    .as_ref()
                    .is_some_and(|swarm_id| coordinated_swarms.contains(swarm_id));
            if !(owned_by_report_back || owned_by_coordination) {
                continue;
            }
            member.detail = Some(format!(
                "{} ({})",
                member.detail.as_deref().unwrap_or_default(),
                crate::server::swarm_persistence::OWNER_NOTIFIED_NOTE
            ));
            reported.push((
                member.session_id.clone(),
                member
                    .friendly_name
                    .clone()
                    .unwrap_or_else(|| member.session_id.clone()),
                member.swarm_id.clone(),
            ));
        }
        reported.sort();
        (owner_name, reported)
    };
    if reported.is_empty() {
        return 0;
    }

    let names = reported
        .iter()
        .map(|(_, name, _)| name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let count = reported.len();
    let message = if count == 1 {
        format!(
            "Idle headless worker {names} was stopped by the server restart and not restored. \
             It had no work in flight. Spawn a replacement if you still need it."
        )
    } else {
        format!(
            "{count} idle headless workers were stopped by the server restart and not restored: {names}. \
             They had no work in flight. Spawn replacements if you still need them."
        )
    };
    crate::logging::info(&format!(
        "Notifying {owner_session_id} of {count} idle headless worker(s) not restored after restart: {names}"
    ));
    let (first_session, _, _) = &reported[0];
    let _ = fanout_session_event(
        &swarm_state.members,
        owner_session_id,
        ServerEvent::Notification {
            from_session: first_session.clone(),
            from_name: owner_name,
            notification_type: NotificationType::Message {
                scope: Some("swarm".to_string()),
                channel: None,
                tldr: Some(format!(
                    "{count} idle worker{} not restored after restart",
                    if count == 1 { "" } else { "s" }
                )),
            },
            message,
        },
    )
    .await;

    let swarms: HashSet<String> = reported
        .iter()
        .filter_map(|(_, _, swarm_id)| swarm_id.clone())
        .collect();
    for swarm_id in swarms {
        persist_swarm_state_for(&swarm_id, swarm_state).await;
        broadcast_swarm_status(&swarm_id, &swarm_state.members, &swarm_state.swarms_by_id).await;
    }
    count
}

/// Set a member's stable task label, derived from its spawn prompt or task
/// assignment. Unlike `detail` (transient status text), the label survives
/// status churn so UIs can always answer "what was this agent for?". A later
/// assignment overwrites the label: the member is now doing that task.
pub(in crate::server) async fn set_member_task_label(
    session_id: &str,
    task_text: &str,
    swarm_members: &Arc<RwLock<HashMap<String, SwarmMember>>>,
) {
    let Some(label) = jcode_swarm_core::derive_swarm_task_label(task_text) else {
        return;
    };
    let mut members = swarm_members.write().await;
    if let Some(member) = members.get_mut(session_id) {
        member.task_label = Some(label);
    }
}

pub(in crate::server) async fn record_swarm_event(
    event_history: &Arc<RwLock<std::collections::VecDeque<SwarmEvent>>>,
    event_counter: &Arc<std::sync::atomic::AtomicU64>,
    swarm_event_tx: &broadcast::Sender<SwarmEvent>,
    session_id: String,
    session_name: Option<String>,
    swarm_id: Option<String>,
    event: SwarmEventType,
) {
    let swarm_event = SwarmEvent {
        id: event_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
        session_id,
        session_name,
        swarm_id,
        event,
        timestamp: Instant::now(),
        absolute_time: std::time::SystemTime::now(),
    };
    let _ = swarm_event_tx.send(swarm_event.clone());
    let mut history = event_history.write().await;
    history.push_back(swarm_event);
    if history.len() > MAX_EVENT_HISTORY {
        history.pop_front();
    }
}

pub(in crate::server) async fn record_swarm_event_for_session(
    session_id: &str,
    event: SwarmEventType,
    swarm_members: &Arc<RwLock<HashMap<String, SwarmMember>>>,
    event_history: &Arc<RwLock<std::collections::VecDeque<SwarmEvent>>>,
    event_counter: &Arc<std::sync::atomic::AtomicU64>,
    swarm_event_tx: &broadcast::Sender<SwarmEvent>,
) {
    let (session_name, swarm_id) = {
        let members = swarm_members.read().await;
        if let Some(member) = members.get(session_id) {
            (member.friendly_name.clone(), member.swarm_id.clone())
        } else {
            (None, None)
        }
    };
    record_swarm_event(
        event_history,
        event_counter,
        swarm_event_tx,
        session_id.to_string(),
        session_name,
        swarm_id,
        event,
    )
    .await;
}

#[expect(
    clippy::too_many_arguments,
    reason = "member status updates need swarm membership, broadcast state, and optional event history sinks"
)]
pub(in crate::server) async fn update_member_status(
    session_id: &str,
    status: &str,
    detail: Option<String>,
    swarm_members: &Arc<RwLock<HashMap<String, SwarmMember>>>,
    swarms_by_id: &Arc<RwLock<HashMap<String, HashSet<String>>>>,
    event_history: Option<&Arc<RwLock<std::collections::VecDeque<SwarmEvent>>>>,
    event_counter: Option<&Arc<std::sync::atomic::AtomicU64>>,
    swarm_event_tx: Option<&broadcast::Sender<SwarmEvent>>,
) {
    update_member_status_with_report(
        session_id,
        status,
        detail,
        None,
        swarm_members,
        swarms_by_id,
        event_history,
        event_counter,
        swarm_event_tx,
    )
    .await;
}

#[expect(
    clippy::too_many_arguments,
    reason = "member status updates need swarm membership, broadcast state, optional report text, and event history sinks"
)]
pub(in crate::server) async fn update_member_status_with_report(
    session_id: &str,
    status: &str,
    detail: Option<String>,
    completion_report: Option<String>,
    swarm_members: &Arc<RwLock<HashMap<String, SwarmMember>>>,
    swarms_by_id: &Arc<RwLock<HashMap<String, HashSet<String>>>>,
    event_history: Option<&Arc<RwLock<std::collections::VecDeque<SwarmEvent>>>>,
    event_counter: Option<&Arc<std::sync::atomic::AtomicU64>>,
    swarm_event_tx: Option<&broadcast::Sender<SwarmEvent>>,
) {
    update_member_status_with_report_tldr(
        session_id,
        status,
        detail,
        completion_report,
        None,
        swarm_members,
        swarms_by_id,
        event_history,
        event_counter,
        swarm_event_tx,
    )
    .await
}

#[expect(
    clippy::too_many_arguments,
    reason = "member status updates need swarm membership, broadcast state, optional report text, and event history sinks"
)]
pub(in crate::server) async fn update_member_status_with_report_tldr(
    session_id: &str,
    status: &str,
    detail: Option<String>,
    completion_report: Option<String>,
    report_tldr: Option<String>,
    swarm_members: &Arc<RwLock<HashMap<String, SwarmMember>>>,
    swarms_by_id: &Arc<RwLock<HashMap<String, HashSet<String>>>>,
    event_history: Option<&Arc<RwLock<std::collections::VecDeque<SwarmEvent>>>>,
    event_counter: Option<&Arc<std::sync::atomic::AtomicU64>>,
    swarm_event_tx: Option<&broadcast::Sender<SwarmEvent>>,
) {
    let completion_report = normalize_completion_report(completion_report);
    if matches!(
        status,
        "completed" | "done" | "stopped" | "failed" | "crashed" | "orphaned"
    ) || (status == "ready" && completion_report.is_some())
    {
        release_file_claims_of(session_id);
    }
    let detail_present = detail.is_some();
    let (
        swarm_id,
        agent_name,
        member_changed,
        status_changed,
        old_status,
        _is_headless,
        report_back_to_session_id,
    ) = {
        let mut members = swarm_members.write().await;
        if let Some(member) = members.get_mut(session_id) {
            let previous_status = member.status.clone();
            let status_changed = member.status != status;
            let detail_changed = member.detail != detail;
            let report_changed =
                completion_report.is_some() && member.latest_completion_report != completion_report;
            let member_changed = status_changed || detail_changed || report_changed;
            if status_changed {
                member.last_status_change = Instant::now();
                if matches!(status, "running" | "streaming" | "thinking") {
                    member.runtime.elapsed_secs = None;
                } else if matches!(
                    previous_status.as_str(),
                    "running" | "streaming" | "thinking"
                ) {
                    member.runtime.elapsed_secs = Some(member.joined_at.elapsed().as_secs());
                }
            }
            let name = member.friendly_name.clone();
            let is_headless = member.is_headless;
            let report_back_to_session_id = member.report_back_to_session_id.clone();
            member.status = status.to_string();
            member.detail = detail;
            // Clear any live output tail when the worker reaches a terminal or
            // idle state so the inline gallery viewport doesn't keep showing
            // stale in-progress text after the turn finishes.
            if matches!(
                status,
                "ready" | "completed" | "done" | "failed" | "crashed" | "stopped"
            ) {
                member.output_tail = None;
            }
            if completion_report.is_some() {
                member.latest_completion_report = completion_report.clone();
            }
            (
                member.swarm_id.clone(),
                name,
                member_changed,
                status_changed,
                previous_status,
                is_headless,
                report_back_to_session_id,
            )
        } else {
            (None, None, false, false, String::new(), false, None)
        }
    };
    if let Some(ref id) = swarm_id {
        if !member_changed {
            return;
        }

        log_swarm_lifecycle(
            "member_status_updated",
            vec![
                ("session_id", session_id.to_string()),
                ("swarm_id", id.clone()),
                ("old_status", old_status.clone()),
                ("new_status", status.to_string()),
                ("status_changed", status_changed.to_string()),
                ("detail_present", detail_present.to_string()),
                (
                    "completion_report_present",
                    completion_report.is_some().to_string(),
                ),
                (
                    "report_back_to_session_id",
                    report_back_to_session_id
                        .clone()
                        .unwrap_or_else(|| "none".to_string()),
                ),
            ],
        );

        if status_changed
            && let (Some(history), Some(counter), Some(tx)) =
                (event_history, event_counter, swarm_event_tx)
        {
            record_swarm_event(
                history,
                counter,
                tx,
                session_id.to_string(),
                agent_name.clone(),
                Some(id.clone()),
                SwarmEventType::StatusChange {
                    old_status: old_status.clone(),
                    new_status: status.to_string(),
                },
            )
            .await;
        }

        broadcast_swarm_status(id, swarm_members, swarms_by_id).await;

        let should_notify_coordinator = status_changed
            && ((status == "completed")
                || (report_back_to_session_id.is_some()
                    && old_status == "running"
                    && matches!(status, "ready" | "failed" | "stopped"))
                // A crash is never routine: notify whoever is responsible
                // (owner, else coordinator) whenever a member dies while it
                // was doing or holding work, so worker deaths cannot pass
                // silently.
                || (status == "crashed"
                    && matches!(
                        old_status.as_str(),
                        "running" | "running_stale" | "queued"
                    )));
        if should_notify_coordinator {
            let fallback_coordinator_id =
                if report_back_to_session_id.as_deref() == Some(session_id) {
                    None
                } else {
                    let members = swarm_members.read().await;
                    members
                        .values()
                        .find(|m| {
                            m.swarm_id.as_deref() == Some(id)
                                && m.role == "coordinator"
                                && m.session_id != session_id
                        })
                        .map(|m| m.session_id.clone())
                };
            let recipient_session_id = report_back_to_session_id
                .clone()
                .filter(|owner_id| owner_id != session_id)
                .or(fallback_coordinator_id);
            if let Some(recipient_session_id) = recipient_session_id {
                let name = agent_name
                    .as_deref()
                    .unwrap_or(&session_id[..8.min(session_id.len())]);
                let msg =
                    completion_notification_message(name, status, completion_report.as_deref());
                let _ = fanout_session_event(
                    swarm_members,
                    &recipient_session_id,
                    ServerEvent::Notification {
                        from_session: session_id.to_string(),
                        from_name: agent_name.clone(),
                        notification_type: NotificationType::Message {
                            scope: Some("swarm".to_string()),
                            channel: None,
                            tldr: report_tldr.clone(),
                        },
                        message: msg,
                    },
                )
                .await;
            }
        }
    }
}
