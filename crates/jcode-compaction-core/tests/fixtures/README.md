# Synthetic compaction checker fixtures

Run `scripts/dev_cargo.sh test --profile selfdev -p jcode-compaction-core --test fact_retention -- --nocapture`.

Each JSON fixture has `kind: synthetic`, user/assistant `messages`, optional `previous_summary`, `must_retain` facts with unique IDs and literal text, and a `scripted_summary`. No real session data is used.

The test calls the production `build_compaction_prompt` interface and verifies that each expected fact is present in its input. The checker uses literal, case-sensitive substring presence in the supplied summary. It does not evaluate semantic equivalence, contradictions, or model behavior. The updated-requirement fixture requires the latest decision literally; it is not a general contradiction detector.

Reports label `scripted_checker_only`, name retained/missing IDs, and report `utf8-bytes-div4-ceil-v1` approximate pre/post counts. Pre counts include the production summarization instruction, not only conversation text. These are not provider token counts or evidence of improved compaction quality. Removing an expected fact must fail the checker. Missing provider output is `inconclusive`, never a pass.

The test does not call a provider or implement live evaluation. Actual production compaction with a model, model/revision/input-digest reporting, and opt-in live mode remain separate unfinished requirements. Do not export real sessions or send private content to a provider based on these fixtures.
