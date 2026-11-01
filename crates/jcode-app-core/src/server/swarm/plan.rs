use super::*;

/// Broadcast the authoritative swarm plan snapshot.
///
/// Plan snapshots are sent to explicit plan participants. If a plan has no
/// participants yet, fall back to all current swarm members.
pub(in crate::server) async fn broadcast_swarm_plan(
    swarm_id: &str,
    reason: Option<String>,
    swarm_plans: &Arc<RwLock<HashMap<String, VersionedPlan>>>,
    swarm_members: &Arc<RwLock<HashMap<String, SwarmMember>>>,
    swarms_by_id: &Arc<RwLock<HashMap<String, HashSet<String>>>>,
) {
    broadcast_swarm_plan_with_previous(
        swarm_id,
        reason,
        None,
        swarm_plans,
        swarm_members,
        swarms_by_id,
    )
    .await;
}

pub(in crate::server) async fn broadcast_swarm_plan_with_previous(
    swarm_id: &str,
    reason: Option<String>,
    previous_items: Option<&[PlanItem]>,
    swarm_plans: &Arc<RwLock<HashMap<String, VersionedPlan>>>,
    swarm_members: &Arc<RwLock<HashMap<String, SwarmMember>>>,
    swarms_by_id: &Arc<RwLock<HashMap<String, HashSet<String>>>>,
) {
    let (version, items, summary, mut participants): (
        u64,
        Vec<PlanItem>,
        crate::protocol::PlanGraphStatus,
        Vec<String>,
    ) = {
        let plans = swarm_plans.read().await;
        let Some(vp) = plans.get(swarm_id) else {
            return;
        };
        let newly_ready_ids = previous_items
            .map(|before| newly_ready_item_ids(before, &vp.items))
            .unwrap_or_default();
        let mut p: Vec<String> = vp.participants.iter().cloned().collect();
        p.sort();
        (
            vp.version,
            vp.items.clone(),
            crate::protocol::PlanGraphStatus::from_versioned_plan(
                swarm_id,
                vp,
                Some(3),
                newly_ready_ids,
            ),
            p,
        )
    };

    if participants.is_empty() {
        let swarms = swarms_by_id.read().await;
        participants = swarms
            .get(swarm_id)
            .map(|s| {
                let mut ids: Vec<String> = s.iter().cloned().collect();
                ids.sort();
                ids
            })
            .unwrap_or_default();
    }

    if participants.is_empty() {
        return;
    }

    let item_count = items.len();
    let reason_label = reason.clone().unwrap_or_else(|| "unspecified".to_string());
    let event = ServerEvent::SwarmPlan {
        swarm_id: swarm_id.to_string(),
        version,
        items,
        participants: participants.clone(),
        reason,
        summary: Some(summary),
    };

    let members = swarm_members.read().await;
    let participant_count = participants.len();
    let mut delivered_count = 0usize;
    // The coordinator's session todos are the user-facing view of this plan;
    // keep them in sync on every broadcast so all completion paths (turn
    // end, report, manual status) reach the pinned todo band.
    let coordinator_ids: Vec<String> = participants
        .iter()
        .filter(|sid| {
            members
                .get(*sid)
                .is_some_and(|member| member.role == "coordinator")
        })
        .cloned()
        .collect();
    let sync_items = match &event {
        ServerEvent::SwarmPlan { items, .. } if !coordinator_ids.is_empty() => Some(items.clone()),
        _ => None,
    };
    for sid in participants {
        if let Some(member) = members.get(&sid)
            && member.event_tx.send(event.clone()).is_ok()
        {
            delivered_count += 1;
        }
    }
    drop(members);
    if let Some(items) = sync_items {
        for coordinator in &coordinator_ids {
            crate::server::swarm_todo_link::sync_plan(coordinator, &items);
        }
    }
    log_swarm_lifecycle(
        "plan_broadcast",
        vec![
            ("swarm_id", swarm_id.to_string()),
            ("version", version.to_string()),
            ("item_count", item_count.to_string()),
            ("participant_count", participant_count.to_string()),
            ("delivered_count", delivered_count.to_string()),
            ("reason", reason_label),
        ],
    );
}

/// Send the current swarm plan snapshot to ONE session (subscribe/resume
/// refresh). Unlike [`broadcast_swarm_plan`] this does not fan out to all
/// participants: reconnecting clients would otherwise show no plan graph
/// until the next plan mutation happens to broadcast.
pub(in crate::server) async fn send_swarm_plan_to_session(
    session_id: &str,
    swarm_members: &Arc<RwLock<HashMap<String, SwarmMember>>>,
    swarm_plans: &Arc<RwLock<HashMap<String, VersionedPlan>>>,
) {
    let swarm_id = {
        let members = swarm_members.read().await;
        members
            .get(session_id)
            .and_then(|member| member.swarm_id.clone())
    };
    let Some(swarm_id) = swarm_id else {
        return;
    };

    let event = {
        let plans = swarm_plans.read().await;
        let Some(vp) = plans.get(&swarm_id) else {
            return;
        };
        if vp.items.is_empty() {
            return;
        }
        let mut participants: Vec<String> = vp.participants.iter().cloned().collect();
        participants.sort();
        ServerEvent::SwarmPlan {
            swarm_id: swarm_id.clone(),
            version: vp.version,
            items: vp.items.clone(),
            participants,
            reason: Some("reconnect".to_string()),
            summary: Some(crate::protocol::PlanGraphStatus::from_versioned_plan(
                &swarm_id,
                vp,
                Some(3),
                Vec::new(),
            )),
        }
    };

    let members = swarm_members.read().await;
    if let Some(member) = members.get(session_id) {
        let _ = member.event_tx.send(event);
    }
}

pub(in crate::server) async fn rename_plan_participant(
    swarm_id: &str,
    old_session_id: &str,
    new_session_id: &str,
    swarm_plans: &Arc<RwLock<HashMap<String, VersionedPlan>>>,
) {
    let mut plans = swarm_plans.write().await;
    if let Some(vp) = plans.get_mut(swarm_id) {
        vp.rename_session(old_session_id, new_session_id);
    }
}

pub(in crate::server) async fn remove_plan_participant(
    swarm_id: &str,
    session_id: &str,
    swarm_plans: &Arc<RwLock<HashMap<String, VersionedPlan>>>,
) {
    let mut plans = swarm_plans.write().await;
    if let Some(vp) = plans.get_mut(swarm_id) {
        vp.participants.remove(session_id);
    }
}
