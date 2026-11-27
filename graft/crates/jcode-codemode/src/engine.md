# crates/jcode-codemode/src/engine.rs

- PRELUDE · constant · L13-L55 — const PRELUDE: &str = r#"
- Shared · struct · L57-L66 — struct Shared
- push_log · function · L69-L86 — fn push_log(&self, level: &str, line: String)
- CancelOnDrop · struct · L89-L89 — struct CancelOnDrop(Arc<AtomicBool>);
- drop · function · L92-L94 — fn drop(&mut self)
- run · function · L97-L159 — pub(crate) async fn run(
- run_on_thread · function · L161-L235 — async fn run_on_thread(
- format_js_error · function · L237-L243 — fn format_js_error(raw: &str) -> String
- install_globals · function · L245-L350 — fn install_globals<'js>(ctx: &rquickjs::Ctx<'js>, shared: &Arc<Shared>) -> rquickjs::Result<()>
- call_tool · function · L352-L405 — async fn call_tool(shared: &Arc<Shared>, name: String, raw_args: String) -> String
- AbortOnDrop · struct · L394-L394 — struct AbortOnDrop(tokio::task::JoinHandle<Result<Value, String>>);
- drop · function · L396-L398 — fn drop(&mut self)
- first_line · function · L407-L419 — fn first_line(text: &str) -> String
