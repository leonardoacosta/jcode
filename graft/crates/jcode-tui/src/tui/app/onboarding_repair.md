# crates/jcode-tui/src/tui/app/onboarding_repair.rs

- RepairAgent · enum · L24-L32 — pub(crate) enum RepairAgent
- label · function · L36-L46 — pub(crate) fn label(self) -> &'static str
- launch_command · function · L50-L60 — pub(crate) fn launch_command(self) -> &'static str
- credential_rel_paths · function · L64-L77 — fn credential_rel_paths(self) -> &'static [&'static str]
- all · function · L81-L91 — fn all() -> [RepairAgent; 7]
- external_home_path · function · L96-L101 — fn external_home_path(rel: &str) -> PathBuf
- agent_last_used · function · L105-L119 — fn agent_last_used(agent: RepairAgent) -> Option<SystemTime>
- detect_preferred_repair_agent · function · L124-L130 — pub(crate) fn detect_preferred_repair_agent() -> Option<RepairAgent>
- build_repair_brief · function · L139-L198 — pub(crate) fn build_repair_brief(
- repair_brief_path · function · L203-L207 — pub(crate) fn repair_brief_path() -> Option<PathBuf>
- persist_repair_brief · function · L211-L218 — pub(crate) fn persist_repair_brief(brief: &str) -> Option<PathBuf>
- repair_brief_status · function · L222-L236 — pub(crate) fn repair_brief_status(agent: Option<RepairAgent>, copied: bool) -> String
- tests · module · L239-L290 — mod tests
- brief_lists_the_non_interactive_commands_and_log_path · function · L243-L266 — fn brief_lists_the_non_interactive_commands_and_log_path()
- brief_without_provider_or_agent_is_still_actionable · function · L269-L279 — fn brief_without_provider_or_agent_is_still_actionable()
- status_names_the_agent_and_launch_command · function · L282-L289 — fn status_names_the_agent_and_launch_command()
