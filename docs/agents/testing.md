# Testing

Choose the smallest check that detects the material failure. When a lasting
regression test is useful, establish RED before changing the implementation.
Keep language-neutral contracts and real behavior assertions; retire tests
whose only consumer was the deleted Python engine.

Rust tests belong beside the owning crate. Run Cargo from `rust/` so the pinned
toolchain applies. `mise run rust:test:q -- -p CRATE` runs scoped nextest tests
and writes the standard report. Read `reports/test-results/rust/latest.json`
first. The complete local gate is `mise run rust:check-no-docs`; it includes
doctests and BSL sentinels but does not generate documentation.

Python tests cover reference preparation and repository/operator tools. Run
`mise run test:q -- tests/unit/path/to/test_file.py` for one file and
`mise run check` for the local Python gate. Use synthetic inputs and temporary
databases for loader tests. A reference-data test must declare its external
fixture requirement and cannot silently substitute an unrelated database.

Serialize heavy Cargo and PostgreSQL gates through existing host controls.
Use each worktree's own environment and Cargo target. Never overlap test
runners against a shared mutable database. Preserve exact committed markers,
hashes, read permissions, and rollback assertions when optimizing a test.
