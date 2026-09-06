# Babylon code review

Read `CLAUDE.md` for current boundaries. `CONSTITUTION.md` governs
game law; `NORTH_STAR.md` describes intended play. Linear owns current scope.

Review changed behavior from its real input through authoritative Rust state,
committed PostgreSQL rows, observer projection, and visible result. Report a
defect only when you can identify its trigger and consequence. Give a precise
code location and a small reproducer or concrete example.

- Equal input bytes must produce equal output bytes and hashes. Check ordering,
  finite values, checked arithmetic, explicit errors, and replay identity.
- Rust owns mechanics and game-managed PostgreSQL writes. An observation,
  narrator, or Bevy display must not create a second authority path.
- Check transaction boundaries, exact commit markers, failed-write rollback,
  restart behavior, reader privileges, visibility, and campaign isolation.
- Trace allocations, repeated serialization, database round trips, and query
  plans when changes affect tick or observer latency.
- Authored Michigan values live in `content/scenarios/michigan/defines.toml`.
  Check units, validation, one 28-day tick, and the saved campaign identity.
  Editing the file must not change the parameters of an existing campaign.
- BSL rules need material causes and governed evidence. Flag a new primitive,
  fixed response curve, or downstream outcome imposed by an external event.
- Delete obsolete implementations and their coupled tests. Keep something only
  for a current consumer or a unique necessary contract. Git preserves history;
  the frozen Python tag supplies old citations, not current game authority.
- For CI, check changed-input selection, deleted paths, failure propagation,
  trusted cache publication, and complete release qualification. A skipped
  prerequisite must not turn the required result green.

Skip formatting, naming preferences, speculative abstractions, compatibility
layers, and requests to restore deleted code without a current consumer.
Do not repeat lint output or treat missing Copilot feedback as a merge blocker.
Do not claim that passing tests proves a human understood the game surface.

One actionable defect per comment. If a concern lacks evidence, state the
uncertainty instead of presenting it as a confirmed fault.
