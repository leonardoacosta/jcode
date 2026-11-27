# crates/jcode-tui-render/examples/swarm_gallery_live.rs

- MockWorker · struct · L33-L45 — struct MockWorker
- new · function · L48-L59 — fn new(name: &str, role: Option<&'static str>, script: Vec<&str>) -> Self
- tick · function · L61-L86 — fn tick(&mut self)
- age_secs · function · L88-L90 — fn age_secs(&self) -> u64
- workers_to_members · function · L93-L120 — fn workers_to_members(workers: &[MockWorker]) -> Vec<GalleryMember>
- make_workers · function · L122-L222 — fn make_workers(n: usize) -> Vec<MockWorker>
- render_gallery_lines · function · L224-L232 — fn render_gallery_lines(
- draw · function · L234-L323 — fn draw(f: &mut Frame, workers: &[MockWorker], max_pct: usize, paused: bool)
- main · function · L325-L341 — fn main() -> io::Result<()>
- run · function · L343-L398 — fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()>
