# crates/jcode-productivity-core/src/aggregate.rs

- top_n · function · L9-L17 — fn top_n(map: HashMap<String, u64>, n: usize) -> Vec<Tally>
- build_report · function · L20-L163 — pub fn build_report(scan: ScanResult) -> ProductivityReport
- streaks · function · L168-L211 — fn streaks(dates: &BTreeSet<String>) -> (u64, u64)
- chronotype · function · L213-L222 — fn chronotype(peak_hour: u8) -> &'static str
- power_score · function · L226-L236 — fn power_score(r: &ProductivityReport) -> u64
- archetype · function · L239-L284 — fn archetype(r: &ProductivityReport) -> (String, String)
- badges · function · L286-L313 — fn badges(r: &ProductivityReport) -> Vec<String>
- weekday_label · function · L316-L326 — fn weekday_label(idx: usize) -> &'static str
- report_from_summaries · function · L329-L337 — pub fn report_from_summaries(summaries: Vec<SessionSummary>) -> ProductivityReport
- today_weekday · function · L341-L343 — pub(crate) fn today_weekday() -> usize
