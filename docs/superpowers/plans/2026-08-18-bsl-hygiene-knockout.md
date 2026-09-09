# BSL Hygiene Knock-out — pre-resume train (Director ruling 2026-08-18)

**Provenance.** Director popup 2026-08-18 (~13:20 EDT): knock-out scope = MAXIMAL — the three
present-pain items + theme 8's fuel-report mode + the authoring-idioms cookbook — executed
BEFORE any BSL-dynamics work resumes. Second ruling same popup: `lifecycle.bsl:308-319`'s
unguarded division = GUARD IT. Source analysis: the BSL refactoring-pressure survey
(workflow wf_cef05333-e91; published artifact "BSL Refactoring Pressure"; raw synthesis at
the session scratchpad `bsl-refactor-synthesis.md`). Branch `feature/bsl-hygiene-knockout`
off dev @ 7b8dbf98. The cookbook (W4) runs in its own docs-only worktree/branch
(`docs/bsl-authoring-cookbook`, wt-cookbook) in parallel.

**Sequencing riders (recorded, not tasks here):** (i) #491 PR A gets merge priority when
posted (the kind arm — found a live dev defect on first contact); (ii) the
`fresh_declared_name` shared-helper extraction rides Community's hyperedge merge; (iii) the
Python-mirror `_conformance_support.py` extraction deadline is the engine-freeze deletion,
not WS3.

**Global constraints.** Read-only toward all in-flight train branches. The pin law binds W5
(all pre-existing pins byte-identical or STOP). Every checker reuses babylon-bsl's own
S-expression reader — never a second parser. New gates are ADDITIVE legs; nothing existing
is relaxed. TDD: red → green per task. Conventional commits; commit per unit.

### Task W1 — the `bsl-lint` host + the two cheapest checks

**Files:** new workspace crate `rust/crates/bsl-lint/` (bin; depends on babylon-bsl as lib);
`.mise.toml` (new task `check:bsl-sentinels`, wired as an additive leg of `rust:check`);
one `#[test]` in the crate owning `state_hash.rs`'s `TAG_*` constants.

- W1.1 RED: integration tests for the two checks against tiny fixture content (a stale
  citation fixture; a duplicated tag/ADR fixture) — the checks don't exist yet.
- W1.2 The host: `bsl-lint <check> [paths]`, one subcommand per check, exit non-zero with
  one `E-SENTINEL <check>: file:line — <what> — <nearest evidence>` line per finding.
  Mirror `tools/sentinel_check.py`'s shape (per-check registry, `--list`).
- W1.3 Check `citation-drift`: parse every `:material-basis` string in
  `rust/crates/babylon-tick/content/rules/*.bsl` (via babylon-bsl's reader); extract
  `<path>:N(-M)` cites. Targets under the frozen Python estate resolve against the
  **`p27-python-freeze` tag content** (`git show tag:path` — the working tree may diverge or
  be deleted later); other targets resolve against the working tree. Assert existence +
  length; tier 2: a keyword token from the citing sentence appears within ±5 lines of the
  cited span (this tier caught both landed incidents — right file, wrong line). Plus: warn
  (not fail) on any `:material-basis` string over 900 bytes (E-LEX-026 caps at 1024;
  measured near-cap blocks: 987/981/959).
- W1.4 Check `namespace-unique`: (a) `ai/decisions/` — every `ADR<N>_*.yaml` number unique
  AND present in `index.yaml`, every index key has a file; (b) E-codes — scan all
  `spec_code()` impls across the workspace, group by code string, fail on any code emitted
  by more than one error class unless a cited allowlist entry names the sharing (seed the
  allowlist with the documented E-LOAD-030 pair); (c) the `TAG_*` section-tag constants —
  a plain `#[test]` asserting pairwise distinctness, placed next to the constants.
- W1.5 Wire `check:bsl-sentinels` into `.mise.toml` and into `rust:check` as an additive
  leg; run the full gate. **Expected: the citation check may RED on real landed drift**
  (the survey found `solidarity.bsl:5,:171` citing a 202-line file as `:97-203`). Fix real
  findings in the same task if they are pure citation corrections (docs-in-content); STOP
  and report if any finding implies a semantic transcription error.
- Commit(s): `feat(lint): bsl-lint host + citation-drift and namespace-unique sentinels`.

### Task W2 — same-tick ordering as LOAD REFUSALS (in-language) + the D116 correction + the audit

**REVISED (Director, 2026-08-18 ~13:40 EDT):** semantic-coherence checks belong IN the
language, not in an external lint — "if you need a sentinel that's a code smell for you
need to expand BSL." Boundary ruled: semantic coherence → load/type refusal; repo
relationship (citations, numbering) → toolchain; checks needing NEW author declarations →
amendment-class. This check is computable from existing declarations → loader hardening,
no amendment, ADR-recorded (the E-LOAD-001/-040/-045 precedent). In-language also means
modders' packs (#531) get the check from the loader itself, not from our CI.

**Files:** `rust/crates/babylon-bsl/src/` (rule_pipeline/scenario load path — two new
E-LOAD codes, next-free per the reference doc's tables); `docs/reference/bsl-language.rst`
(the new codes' spec rows + D116 row correction); babylon-bsl tests.

- W2.1 RED: loader tests — a reader-before-writer `:optional :default` pair must refuse at
  load; a multi-writer field with no earlier unconditional reset and no D127-marked
  unconditional-recompute shape must refuse at load. Both with typed errors + spec codes.
- W2.2 GREEN — refusal 1 (`E-LOAD-0<next>` stale-default-read): for every
  `(binding … :field f :optional :default …)` in rule R, if any rule in the same content
  set writes `f`, the writer's rule-id must sort strictly before R's — else refuse, naming
  both rules and the field. GREEN — refusal 2 (`E-LOAD-0<next+1>` unreset-fan-in): a field
  written by more than one rule (or via fan-in `add`) requires an earlier unconditional
  `set` or the D127 unconditional-recompute shape — else refuse. Both run content-set-wide
  at load, like E-LOAD-001. Loud, named, S-11-conformant messages.
- W2.3 Spec rows for both codes in `bsl-language.rst`'s E-LOAD tables + correct the D116
  register row (its "latent today" premise is false — multi-rule packs are the norm; cite
  the survey). Do not renumber anything.
- W2.4 THE AUDIT — the kind-arm discipline: loading all landed content under the
  strengthened loader IS the audit of the 37 unaudited `:optional :default` bindings. Any
  landed scenario that now refuses = triage: mechanical + pin-safe content repair in-task;
  anything semantic → STOP with the full inventory for controller/Director triage (the T1
  precedent: repair the content, never weaken the arm). Record the full audit table in the
  task report. Pin law: pre-existing pins byte-identical after any repairs, or STOP.
- Commit: `feat(bsl): same-tick ordering load refusals + D116 correction`.

### Task W3 — the fuel-bound report mode

**Files:** `rust/crates/babylon-bsl/` (a report path for the bound checker's computed
value) or `bsl-lint` subcommand `fuel-report` — implementer's call, cheapest honest shape;
`.mise.toml` task `bsl:fuel-check`.

- The bound checker already computes the bound on success (`bound_checker.rs:757-769`);
  expose it: for every rule in every content set, print `rule-id declared=<n> computed=<m>
  headroom=<n-m>`; non-zero exit ONLY on declared < computed (which E-LOAD-040 would
  refuse anyway) — this is a REPORT, not a new gate. No behavior change to loading; zero
  pin impact (prove: tick_goldens byte-identical).
- Commit: `feat(bsl): fuel-bound report mode — measure without the red-run ritual`.

### Task W5 — the lifecycle division guard (Director-ruled)

**Files:** `rust/crates/babylon-tick/content/rules/lifecycle.bsl` (:308-319);
`tests/` conformance for lifecycle.

- Guard the `new-pop-p` division with the nested-if totality shape the same rule already
  uses for `surviving-fraction` (:~248) — a zero denominator yields the honest inert value,
  never E-EVAL-012. Add a conformance vector with `new-pop-p = 0` proving the tick
  survives (the T5-I2 pattern: scratch scenario through the real evaluator). Record the
  Director ruling in the rule's comment (popup 2026-08-18: guard, not loud invariant).
- **Pin law:** all pre-existing pins byte-identical (the guard is unreachable on every
  landed vector — that's WHY it was latent); any pin move = STOP.
- Commit: `fix(content): guard lifecycle's new-pop-p division — Director ruling 2026-08-18`.

### Gate (whole train)

`mise run rust:check` (now including the new sentinel leg) green; `qa:regression` +
`qa:vault-regression-ci` byte-identical; the W2 audit table in the PR body. One PR
(`feature/bsl-hygiene-knockout` → dev) unless W2's audit forces a content-repair split.
