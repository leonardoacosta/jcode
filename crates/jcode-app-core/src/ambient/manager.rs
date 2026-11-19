use anyhow::Result;
use chrono::Utc;

use super::paths::{ambient_dir, queue_path, transcripts_dir};
use super::{
    AmbientCycleResult, AmbientState, AmbientStatus, DeliveryRecord, RecoveryReport,
    ScheduleRequest, ScheduledItem, ScheduledQueue, Settlement,
};
use crate::config::config;

// ---------------------------------------------------------------------------
// AmbientManager
// ---------------------------------------------------------------------------

/// A direct-delivery item overdue by more than this at runner startup is
/// marked `missed` instead of being delivered late and silently.
const MISSED_GRACE_MINUTES: i64 = 5;

pub struct AmbientManager {
    state: AmbientState,
    queue: ScheduledQueue,
}

impl AmbientManager {
    pub fn new() -> Result<Self> {
        // Ensure storage layout exists
        let _ = ambient_dir()?;
        let _ = transcripts_dir()?;

        let state = AmbientState::load()?;
        let queue = ScheduledQueue::load(queue_path()?);

        Ok(Self { state, queue })
    }

    pub fn is_enabled() -> bool {
        config().ambient.enabled
    }

    /// Check whether it's time to run a cycle based on current state and queue.
    pub fn should_run(&self) -> bool {
        if !Self::is_enabled() {
            return false;
        }

        match &self.state.status {
            AmbientStatus::Disabled | AmbientStatus::Paused { .. } => false,
            AmbientStatus::Running { .. } => false, // already running
            AmbientStatus::Idle => true,
            AmbientStatus::Scheduled { next_wake } => Utc::now() >= *next_wake,
        }
    }

    pub fn record_cycle_result(&mut self, result: AmbientCycleResult) -> Result<()> {
        self.state.record_cycle(&result);
        self.state.save()?;

        // If the cycle produced a schedule request, enqueue it
        if let Some(ref req) = result.next_schedule {
            self.schedule(req.clone())?;
        }

        Ok(())
    }

    /// Claim all ready scheduled items (`dispatched`, still persisted). Each
    /// must be settled with [`Self::settle`].
    pub fn take_ready_items(&mut self) -> Vec<ScheduledItem> {
        self.queue.pop_ready()
    }

    /// Claim only ready items targeted at direct delivery into a specific
    /// resumed or spawned session. Each must be settled with [`Self::settle`].
    pub fn take_ready_direct_items(&mut self) -> Vec<ScheduledItem> {
        self.queue.take_ready_direct_items()
    }

    /// Record how a claimed item ended. See [`ScheduledQueue::settle`].
    pub fn settle(&mut self, id: &str, outcome: Settlement) -> Result<bool> {
        self.queue.settle(id, outcome)
    }

    /// Startup recovery. Marks, never delivers. See
    /// [`ScheduledQueue::recover_startup`].
    pub fn recover_scheduled_runs(&mut self) -> Result<RecoveryReport> {
        self.queue
            .recover_startup(chrono::Duration::minutes(MISSED_GRACE_MINUTES))
    }

    /// Explicit manual retry of an uncertain or missed item.
    pub fn retry_schedule(&mut self, id: &str) -> Result<Option<ScheduledItem>> {
        self.queue.retry(id)
    }

    /// Add a schedule request to the queue. Returns the item ID.
    pub fn schedule(&mut self, request: ScheduleRequest) -> Result<String> {
        let item = Self::scheduled_item(request);
        let id = item.id.clone();
        self.queue.try_push(item)?;
        Ok(id)
    }

    pub(super) fn scheduled_item(request: ScheduleRequest) -> ScheduledItem {
        let id = format!("sched_{:032x}", rand::random::<u128>());
        let scheduled_for = request.wake_at.unwrap_or_else(|| {
            Utc::now() + chrono::Duration::minutes(request.wake_in_minutes.unwrap_or(30) as i64)
        });

        let origin_generation =
            crate::server::session_control::current_generation(&request.created_by_session);

        ScheduledItem {
            id: id.clone(),
            scheduled_for,
            context: request.context,
            priority: request.priority,
            target: request.target,
            created_by_session: request.created_by_session,
            created_at: Utc::now(),
            working_dir: request.working_dir,
            task_description: request.task_description,
            relevant_files: request.relevant_files,
            git_branch: request.git_branch,
            additional_context: request.additional_context,
            delivery: DeliveryRecord {
                origin_generation,
                ..Default::default()
            },
        }
    }

    /// Cancel a pending scheduled item by ID. It moves to history as `cancelled`.
    pub fn cancel_schedule(&mut self, id: &str) -> Result<Option<ScheduledItem>> {
        self.queue.cancel(id)
    }

    pub fn state(&self) -> &AmbientState {
        &self.state
    }

    pub fn queue(&self) -> &ScheduledQueue {
        &self.queue
    }
}
