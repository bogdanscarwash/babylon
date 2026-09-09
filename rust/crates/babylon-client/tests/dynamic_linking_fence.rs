// R1.4 fence, compiled half (Director ruling 2026-08-18, BSL refactor
// program plan §14: "bevy/dynamic_linking — RULED: APPROVED WITH FENCE").
//
// `dynamic_linking` is an OPT-IN, LOCAL-ITERATION-ONLY convenience for
// faster incremental Bevy rebuilds (`mise run rust:client-dev-dylib`), never
// meant to ship or gate anything. `cargo test --workspace --locked` — the
// exact command `rust:check` runs (CI's rust-gate job) — never passes
// `--features`, so this only sees the feature active if it became a
// DEFAULT feature by mistake. That is exactly the drift this test exists
// to catch, loudly, inside the same `cargo test` output CI already reads.
//
// The static half of the fence (workflows/mise tasks never referencing the
// feature, and the crate's own `default` list never including it) lives in
// tools/check_dynamic_linking_fence.sh.
#[test]
fn dynamic_linking_is_never_active_in_a_gate_build() {
    assert!(
        !cfg!(feature = "dynamic_linking"),
        "bevy/dynamic_linking is compiled into this build, but this test \
         only runs under `cargo test --workspace --locked` (no --features) \
         — the feature must have become a DEFAULT feature. That violates \
         the R1.4 fence: dynamic_linking is local-iteration-only, via \
         `mise run rust:client-dev-dylib`, and must never ship in a CI, \
         release, or pin-ceremony build."
    );
}
