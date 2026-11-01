use super::*;

/// Walk the `report_back_to_session_id` chain upward from `session_id`,
/// returning the list of ancestor session ids (parent first, root last).
///
/// The spawner/parent edge is encoded by `report_back_to_session_id`: a child
/// spawned by `P` reports back to `P`. Walking that chain reconstructs the spawn
/// tree without persisting a separate parent field. Cycles (which should never
/// happen) are guarded against with a visited set.
pub(in crate::server) fn swarm_ancestors(
    members: &HashMap<String, SwarmMember>,
    session_id: &str,
) -> Vec<String> {
    let mut ancestors = Vec::new();
    let mut visited: HashSet<String> = HashSet::new();
    visited.insert(session_id.to_string());
    let mut current = session_id.to_string();
    while let Some(parent) = members
        .get(&current)
        .and_then(|member| member.report_back_to_session_id.clone())
    {
        if parent == current || !visited.insert(parent.clone()) {
            break;
        }
        ancestors.push(parent.clone());
        current = parent;
    }
    ancestors
}

/// Depth of `session_id` in the spawn tree: number of ancestors reachable via
/// the report-back chain. Root coordinators (no report-back owner) are depth 0.
///
/// Test-only: the spawn tree no longer enforces a depth cap, so production code
/// does not consult depth. Kept (behind `cfg(test)`) because the spawn-tree tests
/// assert ancestor-chain depth directly.
#[cfg(test)]
pub(in crate::server) fn swarm_spawn_depth(
    members: &HashMap<String, SwarmMember>,
    session_id: &str,
) -> u32 {
    swarm_ancestors(members, session_id).len() as u32
}

/// True when `ancestor` is `session_id` itself or any transitive spawner of it.
/// Used to decide whether a requester may manage (stop/control) a target: an
/// agent owns its entire spawned subtree.
pub(in crate::server) fn swarm_is_self_or_ancestor(
    members: &HashMap<String, SwarmMember>,
    ancestor: &str,
    session_id: &str,
) -> bool {
    ancestor == session_id
        || swarm_ancestors(members, session_id)
            .iter()
            .any(|candidate| candidate == ancestor)
}

/// Members spawned by `session_id`, directly or transitively.
pub(in crate::server) fn swarm_descendants(
    members: &HashMap<String, SwarmMember>,
    session_id: &str,
) -> Vec<String> {
    let mut descendants: Vec<String> = members
        .keys()
        .filter(|candidate| {
            candidate.as_str() != session_id
                && swarm_ancestors(members, candidate)
                    .iter()
                    .any(|ancestor| ancestor == session_id)
        })
        .cloned()
        .collect();
    descendants.sort();
    descendants
}

/// Stop the running turns of every worker `session_id` spawned (workers run
/// their own server-side turns, so cancelling only the coordinator left them
/// spending tokens). Idle workers are left alone. Returns the stopped ids.
pub(in crate::server) async fn cancel_spawned_descendant_turns(
    session_id: &str,
    swarm_members: &Arc<RwLock<HashMap<String, SwarmMember>>>,
    swarms_by_id: &Arc<RwLock<HashMap<String, HashSet<String>>>>,
    event_history: Option<&Arc<RwLock<std::collections::VecDeque<SwarmEvent>>>>,
    event_counter: Option<&Arc<std::sync::atomic::AtomicU64>>,
    swarm_event_tx: Option<&broadcast::Sender<SwarmEvent>>,
) -> Vec<String> {
    let descendants = swarm_descendants(&*swarm_members.read().await, session_id);
    let mut stopped = Vec::new();
    for descendant in descendants {
        let signals = crate::turn_cancel_registry::active_turn_signals(&descendant);
        if signals.is_empty() {
            continue;
        }
        for signal in &signals {
            signal.fire();
        }
        // Workers usually have no attached client; the status update below
        // is what the coordinator's swarm view shows.
        fanout_session_event(
            swarm_members,
            &descendant,
            ServerEvent::TurnStopped {
                reason: crate::protocol::TurnStopReason::Interrupted,
                message: format!("Stopped because its coordinator {session_id} was interrupted."),
                provider_stop_reason: None,
            },
        )
        .await;
        update_member_status(
            &descendant,
            "stopped",
            Some("coordinator interrupted".to_string()),
            swarm_members,
            swarms_by_id,
            event_history,
            event_counter,
            swarm_event_tx,
        )
        .await;
        stopped.push(descendant);
    }
    if !stopped.is_empty() {
        let names = stopped.join(",");
        crate::logging::info(&format!(
            "SWARM_CASCADE_CANCEL coordinator={session_id} stopped={names}"
        ));
    }
    stopped
}
