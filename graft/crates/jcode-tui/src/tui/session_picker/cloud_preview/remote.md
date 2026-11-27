# crates/jcode-tui/src/tui/session_picker/cloud_preview/remote.py

- timing · function · L13-L16 — def timing(stage, outcome, started): # Fixed labels only. Never print exceptions, identifiers or projected data.
- timed · function · L19-L31 — def timed(stage)
- decorate · function · L20-L30 — def decorate(function)
- run · function · L21-L29 — def run(*args, **kwargs)
- string_end · function · L47-L51 — def string_end(data, start)
- skip_space · function · L54-L57 — def skip_space(data, pos)
- value_end · function · L60-L86 — def value_end(data, pos, ranges=None)
- fields · function · L89-L116 — def fields(data, pos=0, message_ranges=None)
- journal_fields · function · L119-L127 — def journal_fields(data): # fields already scans every value. Its return value is the object boundary, # so journal entries need no redundant whole-container value_end scan.
- recent_messages · function · L130-L166 — def recent_messages(data, start, end, messages, ranges=None): # Locate the tail without deserializing old message bodies. The range deque # is bounded even for snapshots with millions of messages.
- project · function · L170-L202 — def project(path, preview)
- main · function · L206-L245 — def main()
