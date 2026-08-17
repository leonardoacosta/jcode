# Modes and Capabilities

Bindings map native primitives to these normalized capabilities without redefining policy:

- `respond`: obtain one current structured or free-form operator response.
- `shell`: run bounded local reads and explicitly authorized commands.
- `delegate`: assign a tracked item to a worker under the repository's identity rules.
- `apply`: invoke the harness's shared apply workflow with a selected proposal set.

## Report mode

Render the normalized inventory and perform no mutation, archive, tracker write, configuration edit,
delegation, apply call, Git operation, or sync. Do not request confirmation because report mode has no
action phase.

## Interactive mode

Render the same report first. Offer one bundled decision containing only non-empty eligible action
classes and only capabilities the binding declares. Each option must describe its concrete item set
and outcome. A free-form response may be accepted only when every referenced ID and action is known
and non-conflicting.

If `respond` is unavailable, cancelled, aborted, or yields no answer, preserve the full report, name
the exact actions left unavailable or unanswered, and perform no mutation. If only some execution
capabilities are unavailable, supported action classes may run only when independently confirmed;
list each skipped unsupported class. Never substitute a bespoke implementation for `delegate` or
`apply`.

Bindings may name their native surfaces, but those names belong only in the binding. The shared skill
owns questions, confirmation boundaries, ordering, concurrency, and failure semantics.
