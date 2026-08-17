use super::Agent;
use crate::logging;
use crate::message::{Message, ToolDefinition};

impl Agent {
    pub(super) fn log_prompt_prefix_accounting(
        &self,
        split: &crate::prompt::SplitSystemPrompt,
        tools: &[ToolDefinition],
    ) {
        let system_tokens = split.estimated_tokens();
        let tool_tokens = ToolDefinition::aggregate_prompt_token_estimate(tools);
        let prefix_tokens = system_tokens + tool_tokens;
        logging::info(&format!(
            "Prompt prefix estimate: total={} tokens (system={} tools={})",
            prefix_tokens, system_tokens, tool_tokens
        ));
    }

    pub(super) fn build_memory_prompt_nonblocking_shared(
        &self,
        messages: std::sync::Arc<[Message]>,
        _memory_event_tx: Option<crate::memory::MemoryEventSink>,
    ) -> Option<crate::memory::PendingMemory> {
        if !self.memory_enabled {
            return None;
        }

        let session_id = &self.session.id;

        let fresh_user_turn = crate::message::ends_with_fresh_user_turn(&messages);
        let pending = if fresh_user_turn {
            crate::memory::take_pending_memory(session_id)
        } else {
            None
        };

        // Use the persistent memory-agent pipeline as the single source of truth.
        // Running both this and the legacy MemoryManager background retrieval path
        // can prepare overlapping pending prompts for the same turn, which makes
        // memory injection feel overly aggressive.
        // Relevance results are consumed only at the start of a fresh user turn.
        // Enqueuing again after every tool result runs the local embedding model
        // for each provider continuation without creating an additional injection
        // opportunity. One update per user turn keeps memory current while avoiding
        // redundant 512-token inference during tool-heavy agent loops.
        if fresh_user_turn {
            crate::memory_agent::update_context_sync_with_dir(
                session_id,
                messages,
                self.session.working_dir.clone(),
            );
        }

        pending
    }

    fn append_current_turn_system_reminder(&self, split: &mut crate::prompt::SplitSystemPrompt) {
        let Some(reminder) = self
            .current_turn_system_reminder
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
        else {
            return;
        };

        if !split.dynamic_part.is_empty() {
            split.dynamic_part.push_str("\n\n");
        }
        split.dynamic_part.push_str("# System Reminder\n\n");
        split.dynamic_part.push_str(reminder);
    }

    /// Build split system prompt for better caching
    /// Returns static (cacheable) and dynamic (not cached) parts separately
    pub(super) fn build_system_prompt_split(
        &self,
        memory_prompt: Option<&str>,
    ) -> crate::prompt::SplitSystemPrompt {
        if let Some(ref override_prompt) = self.system_prompt_override {
            let (assembly, context_info) = crate::prompt::override_prompt_assembly(override_prompt);
            let frozen = self.prompt_snapshot.get_or_init(|| {
                crate::prompt::FrozenPromptAssembly::capture(&assembly, &context_info)
            });
            return crate::prompt::SplitSystemPrompt {
                static_part: frozen.static_text.clone(),
                dynamic_part: String::new(),
            };
        }

        let skills = self.current_skills_snapshot();
        let skill_prompt = self
            .active_skill
            .as_ref()
            .and_then(|name| skills.get(name).map(|skill| skill.get_prompt().to_string()));

        let available_skills: Vec<crate::prompt::SkillInfo> = self
            .current_skills_snapshot()
            .list()
            .iter()
            .map(|skill| crate::prompt::SkillInfo {
                name: skill.name.clone(),
                description: skill.description.clone(),
            })
            .collect();

        let working_dir = self
            .session
            .working_dir
            .as_ref()
            .map(std::path::PathBuf::from);

        // Roadmap P1: build the assembly, then freeze the static layers into
        // the session on first build. Later turns reuse the frozen static text
        // (new-session semantics for file changes); dynamic parts rebuild.
        let (assembly, _context_info) = crate::prompt::build_prompt_assembly_with_agents_md(
            skill_prompt.as_deref(),
            &available_skills,
            self.session.is_canary,
            memory_prompt,
            working_dir.as_deref(),
            self.agents_md_snapshot.clone(),
        );
        let frozen = self.prompt_snapshot.get_or_init(|| {
            crate::prompt::FrozenPromptAssembly::capture(&assembly, &_context_info)
        });
        let mut split = assembly.split();
        split.static_part = frozen.static_text.clone();

        self.append_current_turn_system_reminder(&mut split);
        crate::prompt::append_swarm_effort_directive(
            &mut split,
            self.provider.reasoning_effort().as_deref(),
        );

        split
    }

    /// Non-blocking memory prompt - takes pending result and spawns check for next turn
    #[cfg(test)]
    pub(super) fn build_memory_prompt_nonblocking(
        &self,
        messages: &[Message],
        _memory_event_tx: Option<crate::memory::MemoryEventSink>,
    ) -> Option<crate::memory::PendingMemory> {
        self.build_memory_prompt_nonblocking_shared(messages.to_vec().into(), _memory_event_tx)
    }
}

#[cfg(test)]
mod freeze_tests {
    use super::*;
    use crate::message::{Message, StreamEvent, ToolDefinition};
    use crate::provider::{EventStream, Provider};
    use crate::tool::Registry;
    use anyhow::Result;
    use async_trait::async_trait;
    use std::sync::Arc;
    use tokio::sync::mpsc as tokio_mpsc;
    use tokio_stream::wrappers::ReceiverStream;

    struct FreezeProvider;

    #[async_trait]
    impl Provider for FreezeProvider {
        async fn complete(
            &self,
            _messages: &[Message],
            _tools: &[ToolDefinition],
            _system: &str,
            _resume_session_id: Option<&str>,
        ) -> Result<EventStream> {
            let (_tx, rx) = tokio_mpsc::channel::<Result<StreamEvent>>(1);
            Ok(Box::pin(ReceiverStream::new(rx)))
        }

        fn name(&self) -> &str {
            "freeze-test"
        }

        fn fork(&self) -> Arc<dyn Provider> {
            Arc::new(Self)
        }
    }

    /// Roadmap P1: static layers freeze into the session at first build;
    /// mid-session file edits only affect new sessions.
    #[tokio::test]
    async fn prompt_assembly_freezes_static_layers_for_the_session() {
        let _guard = crate::storage::lock_test_env();
        let temp_home = tempfile::tempdir().expect("temp home");
        let prev_home = std::env::var_os("JCODE_HOME");
        crate::env::set_var("JCODE_HOME", temp_home.path());

        let project = tempfile::tempdir().expect("temp project");
        std::fs::create_dir_all(project.path().join(".jcode")).expect("mkdir .jcode");
        std::fs::write(
            project.path().join(".jcode/prompt-overlay.md"),
            "overlay-v1",
        )
        .expect("write overlay v1");

        let provider: Arc<dyn Provider> = Arc::new(FreezeProvider);
        let registry = Registry::new(provider.clone()).await;
        let mut agent = Agent::new(provider, registry);
        agent.session.working_dir = Some(project.path().to_string_lossy().to_string());

        let first = agent.build_system_prompt_split(None);
        assert!(
            first.static_part.contains("overlay-v1"),
            "first build should include the overlay"
        );
        let digest_first = agent
            .prompt_snapshot
            .get()
            .expect("snapshot captured")
            .digest
            .clone();

        // Mid-session edit: static prompt and digest must not change.
        std::fs::write(
            project.path().join(".jcode/prompt-overlay.md"),
            "overlay-v2-changed",
        )
        .expect("write overlay v2");
        let second = agent.build_system_prompt_split(Some("mem-turn-2"));
        assert!(
            second.static_part.contains("overlay-v1"),
            "frozen static text must keep the capture-time overlay"
        );
        assert!(
            !second.static_part.contains("overlay-v2-changed"),
            "frozen static text must ignore mid-session edits"
        );
        assert!(
            second.dynamic_part.contains("mem-turn-2"),
            "dynamic layers still rebuild per turn"
        );
        assert_eq!(
            agent.prompt_snapshot.get().expect("snapshot").digest,
            digest_first,
            "digest must stay stable across the session"
        );

        // A new session (fresh Agent) captures the edited content.
        let provider2: Arc<dyn Provider> = Arc::new(FreezeProvider);
        let registry2 = Registry::new(provider2.clone()).await;
        let mut agent2 = Agent::new(provider2, registry2);
        agent2.session.working_dir = Some(project.path().to_string_lossy().to_string());
        let third = agent2.build_system_prompt_split(None);
        assert!(
            third.static_part.contains("overlay-v2-changed"),
            "a new session captures the edited overlay"
        );

        match prev_home {
            Some(value) => crate::env::set_var("JCODE_HOME", value),
            None => crate::env::remove_var("JCODE_HOME"),
        }
    }

    /// Ambient-style override flows through the same frozen assembly path.
    #[tokio::test]
    async fn system_prompt_override_freezes_through_assembly() {
        let _guard = crate::storage::lock_test_env();
        let temp_home = tempfile::tempdir().expect("temp home");
        let prev_home = std::env::var_os("JCODE_HOME");
        crate::env::set_var("JCODE_HOME", temp_home.path());

        let provider: Arc<dyn Provider> = Arc::new(FreezeProvider);
        let registry = Registry::new(provider.clone()).await;
        let mut agent = Agent::new(provider, registry);
        agent.system_prompt_override = Some("OVERRIDE PROMPT".to_string());

        let first = agent.build_system_prompt_split(None);
        assert_eq!(first.static_part, "OVERRIDE PROMPT");
        assert!(first.dynamic_part.is_empty());
        let snapshot = agent.prompt_snapshot.get().expect("snapshot captured");
        assert!(snapshot.digest.starts_with("prompt:1:"));
        assert_eq!(snapshot.attribution.len(), 1);
        assert_eq!(snapshot.attribution[0].id, "override");

        let second = agent.build_system_prompt_split(Some("ignored-memory"));
        assert_eq!(second.static_part, "OVERRIDE PROMPT");

        match prev_home {
            Some(value) => crate::env::set_var("JCODE_HOME", value),
            None => crate::env::remove_var("JCODE_HOME"),
        }
    }
}
