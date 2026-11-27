# crates/jcode-applet-types/src/patch.rs

- PatchOp · enum · L12-L16 — pub enum PatchOp
- path · function · L19-L23 — pub fn path(&self) -> &str
- PatchError · enum · L27-L32 — pub enum PatchError
- fmt · function · L35-L44 — fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
- apply_patch · function · L52-L79 — pub fn apply_patch(
- unescape · function · L81-L83 — fn unescape(token: &str) -> String
- apply_op · function · L85-L132 — fn apply_op(root: &mut Value, op: &PatchOp) -> Result<(), PatchError>
