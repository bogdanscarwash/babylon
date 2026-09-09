# Retained Python tests

Python tests cover reference-data construction, data provenance, repository
commands, CI/reporting, and the native observer launcher. The Python simulation
engine and its tests have been retired. Rust owns material transitions,
conservation, deterministic replay, PostgreSQL persistence, and their executable
contracts in `rust/crates/*/tests` and crate-local test modules.

Run the applicable file with `mise run test:q -- tests/unit/path/to/file.py`.
`mise run check` runs static repository checks and retained Python units.
`mise run test:unit-ci` selects fast units for changed Python inputs. Release
validation adds non-unit contracts and slow units through `test:rest-ci`;
reference-dependent cases run in the separate reference-data job. Their union
avoids repeating the fast unit suite.

`tests/conftest.py` controls BLAS threads, deterministic property-test settings,
random-state isolation, logging capture, and refusal of accidental model API
requests. Current fixtures belong next to their consuming data or tool tests.

Keep a test when it protects actual current behavior or a required boundary.
Do not retain documentary snapshots of closed work, duplicate Rust contract
verification in Python, or recreate a retired engine to preserve test counts.
Language-neutral vectors and observed data remain valuable even when their
former Python verifier or generator is available only in Git history.
