# Repository Rules for Agents

`AGENTS.md` points here. Keep this live configuration below 200 lines. Use `NORTH_STAR.md` for game direction, `docs/concepts/architecture.rst` for the live boundary, and `docs/agents/governance.md` for contributor authority.

## Babylon

Babylon is an entertainment-first emergent political-economy game, not a forecast
or scientific reproduction. Theory constrains causes but does not predetermine
results. Determinism proves computational identity, not scientific truth.
Historical cases test causal signatures and counterfactual behavior. Bevy remains
an administrative viewer with no player action.

This checkout implements Gate 2. These four executable gates follow:

<!-- Vale: each protected item is a governed gate name. -->
<!-- vale ste.UnapprovedWords = NO -->
<!-- vale Vale.Terms = NO -->
<!-- vale ste.NounClusters = NO -->
1. **PostgreSQL/H3/Archive decision-loop slice**
1. **Productive & distributive circuit**
1. **Player agency**
1. **COVID emergence benchmark**
<!-- vale Vale.Terms = YES -->
<!-- vale ste.UnapprovedWords = YES -->
<!-- vale ste.NounClusters = YES -->

Read `CONSTITUTION.md` v4.1.0 before a change to game law. ADR221 maps its predecessors and preserves history. `ai/mantras.yaml` is the canonical machine-readable orientation.

## Constitutional compact

- Equal input bytes must produce equal output bytes and hashes.
- The dialectic is primitive. BSL cannot add a mathematical primitive.
- Each formal element needs a material relation. Geography stays fixed.
- Political claims use overlays. Public hyperedges stay first-class.
- AI can parse, retrieve, and narrate. Only the engine judges mechanics.
- Classify substantive values as `Observed`, `Derived`, `Calibrated`, or `Designed`.
- An external-event rule can add only an allowed pressure, burden, or capacity effect.
  It cannot write downstream results.
- A rule cannot impose a sigmoid or a second fixed response curve.
- Each game display must answer a decision question. An administrative display
  cannot pass a game milestone.

Stop and ask the Director before a new primitive, weaker prohibition, or change to the reserved theory line. Do not infer live authority from an old ADR.

## Live architecture

<!-- Vale: these paragraphs preserve literal crate, schema, and Linear identifiers. -->
<!-- vale ste.UnapprovedWords = NO -->
<!-- vale ste.NounClusters = NO -->
The live Rust path is `babylon-kernel`, `babylon-graph`, `babylon-bsl`,
`babylon-tick`, and `babylon-client`. Every BSL rule declares one causal role
and evidence class. Built-in declarations must match
`GOVERNED_RULE_ATTRIBUTIONS`. Unknown mod and fixture IDs remain self-declared.

Mechanics have typed effects. Recognizers, external events, and intents are exact-allowlist and default-deny. Restricted production footprints must equal their unique allowance rows, so CI rejects dead permissions. Executable shocks and intents do not exist.

Same-rank rules compose sequentially. `TickSession` publishes graph, events, identity-free event-then-write `AuditReceipt` rows,
completed time, and `NominalWorldHash` only after the detached tick succeeds. `GraphStateHash` stays graph-only.
The Bevy viewer shows the world hash.

Program 27 froze the Python engine at `p27-python-freeze`. Its 34-system
`SimulationEngine._DEFAULT_SYSTEMS` is the transcription oracle for Rust's
executable `phase_order.rs` causal spine. Five `EndgameDetector` labels remain
reference facts, not promised outcomes. Python also owns data tools and periphery.

Reference Parquet and deterministic SQLite are build artifacts. The Python
`RuntimeDatabase` is separate mutable SQLite.

Gate 3 now has the Rust three-schema boundary, committed tick envelope,
marker-last transaction, checkpoint restart, and Archive dirty receipts. The
fog-safe decision loop and semantic Archive worker remain. ADR250 reordered the
later gates: Gate 4 adds the productive and distributive circuit, and Gate 5
adds next-week intents and Bevy player actions. Gate 6 adds governed
external-event rows and the merged COVID benchmark.

<!-- Vale: the accepted Linear status uses a passive state label. -->
<!-- vale strunk.ActiveVoice = NO -->
<!-- vale ste.PassiveVoice = NO -->
PER-48 is decided.
<!-- vale ste.PassiveVoice = YES -->
<!-- vale strunk.ActiveVoice = YES -->
The one-way cutover is complete. Rust owns authoritative game-managed Postgres.
Python continues its declared data, AI, document, external-API, optimization,
and CLI periphery. It has no game-state writer or transition reader. The legacy
Django browser client lives only in `web/` and does not gate v1.
<!-- vale ste.NounClusters = YES -->
<!-- vale ste.UnapprovedWords = YES -->

## Source rules

Frozen Python uses `kernel < models/formulas < topology < domain < persistence <
engine`. `intelligence` observes. Check it with `mise run lint:imports`.

Put coefficients in `GameDefines` and `src/babylon/data/defines.yaml`. Regenerate with
`tools/generate_defines_config.py`. Use native public hyperedges, strict
types, explicit errors, specific type-ignore codes, and production vocabulary.
Do not use a `test_` prefix in production source.

Use TDD: show RED, make it pass, then refactor. Keep game data typed and
immutable. Python models use frozen Pydantic types.
`model_copy(update=...)` skips validation. Pass dependencies explicitly.

Do not run Sphinx, `cargo doc`, or a CI task that generates documentation unless the Director requests it. This includes local `mise run ci:rust` and its `rust:check` compatibility alias.
The pre-push hook runs `mise run rust:check-no-docs` when Rust or its gate definitions change. Use only targeted Vale and format checks on changed prose.

## Tests and behavior contracts

Run the smallest applicable test first and do not overlap heavy gates. `pytest` covers
data tools for Python, periphery, the frozen reference, and durable
language-neutral contracts. Retire an engine-specific Python test only after a
replacement contract exists.

```bash
mise run test:q -- tests/unit/path/to/test_file.py
mise run check
cd rust && cargo fmt --all -- --check
cd rust && cargo test -p <changed-crate> --locked
mise run rust:check-no-docs
mise run qa:regression
mise run qa:vault-regression
mise run check:gate-coverage
```

Run `check` for the Python gate. For Rust changes, run the applicable format, scoped test, clippy, and BSL-sentinel legs separately without documentation.
Engine, economics, and `GameDefines` changes must also run regression, vault, and coverage gates. Reject `NaN`, infinity, unchecked overflow, and an iteration order that can change.

Do not edit a baseline to hide a fault. An intentional baseline change requires
its ceremony, trailer, and `tools/generate_ceremony_message.py` record.

## Git and merge rules

<!-- Vale: these paragraphs preserve exact Linear fields and Git terms. -->
<!-- vale Vale.Spelling = NO -->
<!-- vale ste.UnapprovedWords = NO -->
<!-- vale ste.NounClusters = NO -->
Linear alone owns current issue identity, scope, status, priority, dependencies,
horizon, milestones, schedule, and work. GitHub owns source, PRs, reviews, and
historical evidence. The team closed GitHub Project #7 and Project #8. They are
historical inputs. The migration is complete, and PER-15 is complete. Neither board is live.

`ai/state.yaml` is historical implementation evidence, and `project/` is non-live context.

Create regular lanes from `dev` and target `dev`. Use `feature/`, `fix/`, `docs/`,
`refactor/`, `test/`, or a `codex/PER-123-short-name` lane. Link the PER identity
as `docs/agents/governance.md` directs: `Part of PER-N` for partial delivery and
`Fixes PER-N` only for final acceptance. Never commit directly to `dev` or `main`.

Delete obsolete code. Do not add shims, aliases, adapters, deprecated paths, or backward-compatibility layers unless the Director explicitly requires them.
A critical hotfix alone can branch from and target `main`. Its Director-only merge requires a `dev` backport.
For a release, prove `origin/main` is an ancestor of `origin/dev`, then qualify exact `dev` with `main.yml`.
After the Director merge, return main through `release:prepare-dev-sync` before `release:tag` can publish.
<!-- vale ste.NounClusters = YES -->
<!-- vale Vale.Spelling = YES -->
Use `type(scope): description`, one logical unit, and the required co-author
trailer. Commit with `mise run commit -- "type(scope): description"`.

Before merge, pin green CI to the PR head SHA. Copilot evidence is advisory. An unresolved review thread blocks.
<!-- vale ste.UnapprovedWords = YES -->
Use only `mise run pr:merge -- N`. Do not use `gh pr merge --auto`. The Director
controls all merges to `main`.

Keep unrelated user changes unchanged. Report an unrelated fault unless the owner expands scope.

## Toolchain and host safety

Use native libraries and `.mise.toml`, `mise.lock`, and `.python-version` pins (ADR252). Install with `mise install --locked` and `mise run install`.
Run Cargo from `rust/` to use `rust-toolchain.toml`. Reference tools use `mise exec -- uv run --frozen python` in the project `.venv`. The builder checks SQLite 3.53.1.

Do not set `PYTHONPATH`. Run heavy gates uncapped and serially. Keep `BLAS=1`.

`earlyoom` is the host backstop. Use `mise run mcp:reap`. Do not use broad
`pkill`.

## Important traps

<!-- Vale: this block preserves exact identifiers from recurring failures. -->
<!-- vale Vale.Spelling = NO -->
<!-- vale ste.UnapprovedWords = NO -->
<!-- vale ste.NounClusters = NO -->
Read `docs/agents/gotchas.md` for per-tick `WorldState.events`, graph round-trip
loss, shared-graph order, Pydantic runtime checks, immutability, and dependency
injection.

- `dynamic_hex_state` is sparse. Read `v_hex_state_asof`. `tick_commit`, not
  `MAX(tick)`, marks durability.
- Run `mise run check:vocabulary`. A direct worktree run needs
  `PYTHONPATH="$PWD/src"`.
- `end-of-file-fixer` can add a trailing newline. Use a tolerant comparison or
  a precise exclusion.
- Workflow `args` can arrive absent or stringified. Parse first. Use a hard
  fallback only for known values.
<!-- vale ste.NounClusters = YES -->
<!-- vale ste.UnapprovedWords = YES -->
<!-- vale Vale.Spelling = YES -->

## Records

Keep old ADRs and reports unchanged. Add an ADR for a new architecture decision.
Linear owns status and tasks. Verify source and executable evidence before claiming behavior.
