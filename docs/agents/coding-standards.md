# Coding Standards

Use the owning Rust crate's typed contracts for game state and mechanics.
Keep one canonical implementation path and explicit errors. Determinism
requires checked arithmetic, finite values, canonical ordering, and equal
output bytes for equal inputs. Readers must authenticate committed state.

Retained Python data tools use explicit types and validated immutable models
where a schema is needed. Validate external input before mutating a database;
pass dependencies explicitly. Use specific type-ignore codes.

Choose regression tests for real failure modes. Use RED/GREEN when a lasting
test earns its maintenance cost, and validate simple scripts directly.
Do not add tests that only repeat implementation text.

Keep production names free of the `test_` prefix. Use conventional commits
and the required co-author trailer through `mise run commit`. Existing
formatters and linters own mechanical style.
