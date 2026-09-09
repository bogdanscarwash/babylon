# About Babylon

Authority: `CONSTITUTION.md` v4.0.0 and `NORTH_STAR.md`.

Babylon is an `entertainment-first emergent political-economy game`. It uses a
deterministic simulation engine. Theory sets limits on cause and
effect. Theory does not set an outcome. Historical cases test behavior. The
game is not a forecast.

Today:

- Rust engine crates and BSL content are in `rust/`.
- `babylon-client` is a `Bevy admin/viewer; no player action`.
- Python remains the frozen behavioral reference, data pipeline, tests,
  and selected periphery.
- `Postgres` and `pgvector` are available, and Gate 3 will add the v4 schema
  set, durable game data, and semantic Archive.
- `rustworkx` backs the frozen Python topology. `babylon-graph` gives Rust its
  native `graph` and `hypergraph` contracts.
- The browser client is legacy and does not gate v1.

Read `CLAUDE.md` for instructions and `docs/concepts/architecture.rst` for the
implemented-versus-planned architecture boundary. Linear alone controls status
and tasks.
