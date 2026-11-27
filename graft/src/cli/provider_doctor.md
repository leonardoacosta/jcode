# src/cli/provider_doctor.rs

- run_provider_doctor_command · function · L13-L84 — pub async fn run_provider_doctor_command(
- emit_report · function · L86-L95 — fn emit_report(report: &DoctorReport, emit_json: bool)
- status_symbol · function · L97-L105 — fn status_symbol(status: LiveVerificationStageStatus) -> &'static str
- status_color · function · L107-L114 — fn status_color(status: LiveVerificationStageStatus) -> &'static str
- format_report · function · L116-L173 — fn format_report(report: &DoctorReport, colorize: bool) -> String
- next_step_hint · function · L175-L207 — fn next_step_hint(checkpoint: &str) -> String
- report_to_json · function · L209-L233 — fn report_to_json(report: &DoctorReport) -> String
- run_coverage_command · function · L235-L263 — pub fn run_coverage_command(emit_json: bool) -> Result<()>
- format_coverage · function · L265-L306 — fn format_coverage(report: &serde_json::Value, width: usize) -> String
- coverage_tests · module · L309-L386 — mod coverage_tests
- coverage_command_needs_no_provider_and_existing_diagnosis_still_does · function · L314-L335 — fn coverage_command_needs_no_provider_and_existing_diagnosis_still_does()
- coverage_table_is_cell_bounded_and_shows_unknown_evidence · function · L338-L349 — fn coverage_table_is_cell_bounded_and_shows_unknown_evidence()
- coverage_table_strips_escape_sequences_and_controls · function · L352-L372 — fn coverage_table_strips_escape_sequences_and_controls()
- coverage_table_preserves_skipped_reason_and_bounds_wide_text · function · L375-L385 — fn coverage_table_preserves_skipped_reason_and_bounds_wide_text()
