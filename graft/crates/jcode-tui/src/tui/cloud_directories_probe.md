# crates/jcode-tui/src/tui/cloud_directories_probe.py

- value_pattern · function · L29-L37 — def value_pattern(depth)
- metadata · function · L51-L154 — def metadata(data, check_time, journal=False)
- space · function · L54-L60 — def space(pos)
- string · function · L62-L78 — def string(pos)
- skip · function · L80-L109 — def skip(pos, depth=0)
- obj · function · L111-L147 — def obj(pos, wanted)
- session_fields · function · L157-L198 — def session_fields(path, check_time): # Large sessions journal metadata without rewriting the snapshot. The newest # complete record is authoritative, exactly as load_startup_stub handles it.
- collect · function · L201-L367 — def collect(current, home=None, state=None, alive=None, current_session_id=None)
- check_time · function · L207-L209 — def check_time()
- children · function · L211-L225 — def children(path)
- read_json · function · L227-L233 — def read_json(path)
- process_alive · function · L235-L249 — def process_alive(pid)
- marker_pid · function · L253-L259 — def marker_pid(path)
- directory · function · L261-L265 — def directory(raw)
- repo_root · function · L267-L271 — def repo_root(path)
- display · function · L349-L354 — def display(path)
