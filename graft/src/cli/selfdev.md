# src/cli/selfdev.rs

- JCODE_REPO_URL · constant · L15-L15 — const JCODE_REPO_URL: &str = "https://github.com/1jehuang/jcode.git";
- selfdev_clone_dir · function · L17-L19 — fn selfdev_clone_dir() -> Result<PathBuf>
- resolve_or_clone_repo_dir · function · L21-L77 — fn resolve_or_clone_repo_dir() -> Result<PathBuf>
- wait_for_reloading_server · function · L79-L99 — async fn wait_for_reloading_server() -> bool
- run_self_dev · function · L101-L258 — pub async fn run_self_dev(should_build: bool, resume_session: Option<String>) -> Result<()>
- selfdev_tests · module · L261-L261 — mod selfdev_tests;
