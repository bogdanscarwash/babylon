<!-- Vale: code-span punctuation is merged across adjacent contract blocks. -->
<!-- vale ste.ParagraphLength = NO -->
<!-- vale ste.UnapprovedWords = NO -->
<!-- vale Vale.Spelling = NO -->
<!-- vale ste.ProcedureLength = NO -->
<!-- Vale: exact Rust, Python, BSL, record, error, command, and fixture names follow. -->
<!-- vale Vale.Terms = NO -->
<!-- vale ste.NounClusters = NO -->
<!-- vale ste.Dictionary = NO -->
<!-- vale write-good.TooWordy = NO -->
<!-- vale ste.SentenceLength = NO -->
<!-- vale ste.Gerunds = NO -->
<!-- vale ste.Semicolon = NO -->
<!-- vale ste.Modals = NO -->
<!-- vale write-good.E-Prime = NO -->
<!-- vale ste.PassiveVoice = NO -->
<!-- vale strunk.ActiveVoice = NO -->
<!-- vale ste.Ambiguity = NO -->

# T3 Synthetic Emergence Evidence Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox <code>- [ ]</code> syntax for tracking.

**Goal:** Build the synthetic-only T3 evidence foundation: exact post-commit record bytes, discrete output classifiers, independent Python vectors, a bounded scoped BSL footprint auditor, and a synthetic profile/cone harness, without creating a live membership observable or a path back into engine judgment.

**Architecture:** A new downstream Rust crate, **babylon-evidence**, owns T3 wire records, classifiers, and synthetic validators. Its only in-repository dependencies are **babylon-kernel**, **babylon-bsl**, and the PER-55-owned **babylon-practice-contract**; it also uses the declared Unicode-normalization utility. Kernel, graph, BSL, practice-contract, tick, persistence, and client crates must not depend on it. BSL semantic footprint analysis remains inside **babylon-bsl**, while **bsl-lint** enforces the cross-crate non-authorability dependency boundary.

**Tech Stack:** Rust 2021 with MSRV 1.87, **babylon-kernel**, **babylon-bsl**, **babylon-practice-contract**, the workspace's existing **unicode-normalization** 0.1.25 with Unicode 17.0.0 data, SHA-256, Python 3.12 with **unicodedata2** 17.0.1 and Unicode 17.0.0 data, Cargo, pytest, Mise, and Vale.

**Spec:** <code>docs/superpowers/specs/2026-08-23-neel-relational-territory-practice-design.md</code>, especially sections 9.1 through 9.8 and section 11.

**Status:** Ready to execute only after PER-52 and PER-55 are Done and the executor has moved PER-54 to In Progress on the current branch. This plan covers only section 11 item 5, “T3 groundwork.” It cannot satisfy “T3 live proof.”

## Global Constraints

- The classifier runs only over completed evidence and has no engine, BSL binding, next-tick, action-preview, AI-objective, or eligibility consumer.
- No engine crate depends on **babylon-evidence**. A reverse dependency is a gating failure.
- No task adds a mathematical or BSL primitive, fitted curve, smoothing, breakpoint optimization, response table, threshold ladder, or fixed response function.
- Every T3 envelope is exactly <code>ASCII(domain) || 0x00 || u16_be(1) || u32_be(payload_length) || payload</code>.
- A record digest is SHA-256 over the complete envelope, never the payload alone.
- Implement only records whose complete byte order is frozen in the committed specification.
- Do not define or encode live <code>RelationalScopeV1</code>, <code>MembershipContributionV1</code>, or <code>MembershipContributionSetV1</code>. PER-44 owns their final field tables and territorial join.
- Digest fields that refer to those three deferred records remain opaque typed 32-byte inputs in synthetic vectors.
- Do not invent a canonical empty exogenous-ledger digest. Synthetic vectors use an explicitly named synthetic digest and never call it the live canonical empty ledger.
- Do not claim that a <code>PracticeAttemptLedgerV1</code> header matches an authoritative accepted-intent ledger. Gate 5 owns that producer and canonical row contract.
- Task 9 seals a synthetic driver-contract manifest that names every synthetic
  predicate and binds the exact <code>driver.rs</code> source bytes. Driver
  predicates are available only through the opaque validated handle. The
  handle is returned only when the manifest digest equals
  <code>SfsPreregistrationV1.driver_contract_digest</code>. This proves the
  synthetic harness executed its preregistered driver contract; it does not
  claim a live action producer or complete run.
- Do not create <code>CommittedTickEnvelope</code>, PostgreSQL tables, Archive rows, player actions, external events, authoritative ledgers, a membership resolver, or an encounter-to-membership producer.
- Do not add a production causal-cone manifest. This plan proves the audit and reachability machinery with synthetic components only.
- The Python encoder and Rust encoder are independent implementations. Neither implementation computes the other implementation's expected result at test time.
- Every checked-in vector pins literal bytes or a literal digest. A round trip through one encoder is supporting evidence, never the cross-language oracle.
- Binary64 arithmetic uses ordinary IEEE-754 addition or subtraction only. Reject non-finite values and intermediates. Normalize negative zero to positive zero before bit comparison or encoding.
- The SFS classifier uses the exact predicate order and class codes in section 9.5. Persistence uses its separate exact predicate order and codes.
- Every loop added by this plan has a statically visible maximum. Use the specification's limits: 64 profile components, 64 entries per profile set, 65,535 schedule, ledger, or intervention-delta rows, 157 trace samples, and window width 2 through 52.
- Use fixed-bound indexed loops after a checked length preflight. Do not add an unbounded <code>while</code> loop.
- Every public data type is explicit and private-fielded. Every failure has a specific enum variant; there is no generic fallback error.
- A failed encoder or validator publishes no partial bytes, digest, classification, or profile.
- No baseline under <code>tests/baselines/**</code> changes in this train.
- The scoped footprint auditor claims only exact profile equality and its bounded forbidden corpus. It never claims universal detection of disguised authored shapes.
- ADR226 records this boundary before the first code-bearing commit. At execution time, verify that ADR226 is still free; an allocation conflict stops the train for explicit renumbering.
- PER-54 is the sole implementation owner for this train. Before implementation, refresh PER-52, PER-55, and PER-54 from Linear, require both prerequisites to be Done, and move PER-54 from Todo to In Progress. PER-55 supplies the one governed `PracticeIdV1` code table; T3 must not copy it. PER-54 remains only groundwork and does not unblock PER-59 without PER-57, PER-58, and PER-22.
- Run Rust format, scoped tests, scoped Clippy, the BSL sentinel, Python checks, and targeted Vale. Do not run <code>mise run rust:check</code> because it generates documentation.

## Scope Cut: What This Plan Can Prove

This plan can prove:

- exact T3 envelope framing and SHA-256 identity;
- exact bytes for every presently frozen record;
- rejection of malformed, oversized, duplicate, out-of-order, truncated, trailing, non-NFC, non-ASCII, non-finite, and invalid-code inputs;
- all committed SFS and persistence classifier goldens;
- exact candidate projection from a synthetic attempt ledger;
- exact synthetic flat-cadence realization from sealed
  <code>PracticeIntentV1</code> values and a source-bound driver contract;
- exact synthetic run-identity mutation sensitivity;
- exact scoped BSL read/effect footprints and a bounded forbidden corpus;
- exact synthetic proof-profile and causal-cone equality;
- a compile-time dependency direction with no engine-to-evidence edge.

This plan cannot prove:

- that membership data exists or changes;
- that a territorial join is governed;
- that an exogenous ledger is canonically empty;
- that a committed initial envelope carried preregistration;
- that every executable input enters a live run identity;
- that a real action schedule projected from authoritative accepted rows;
- that a real root-to-membership-sink causal cone is complete;
- that topology, distribution, restart, or backend twins ran;
- that slow-fast-slow behavior emerged in a game run;
- that persistence separation is live;
- that any T3 artifact is player-facing or Archive-durable.

## Per-Task TDD Invariant

Every code task below has a named RED command and expected failure. Add only
the implementation needed to make that contract green. After the first green
run, refactor only the files owned by that task: remove duplication introduced
in the green pass, keep public bytes and behavior unchanged, rerun the smallest
named test, and run the scoped format/lint command before the commit. If the
green pass already has the smallest coherent shape, record that no refactor was
warranted and still rerun the contract. Never weaken a test, regenerate a
literal from the Rust implementation under test, or edit a baseline during the
refactor phase.

## Dependency Shape

~~~mermaid
flowchart LR
    K["babylon-kernel"] --> B["babylon-bsl"]
    K --> P["babylon-practice-contract"]
    P --> B
    K --> E["babylon-evidence"]
    B --> E
    P --> E
    B --> T["babylon-tick"]
    E --> S["Synthetic vectors and proof harness"]
    L["bsl-lint"] --> D["Dependency-direction sentinel"]
    D -. "forbids" .-> X["kernel/graph/bsl/practice-contract/tick/persistence/client -> evidence"]
    G3["Gate 3 envelope and ledgers"] -. "future activation" .-> LIVE["Live T3 producer"]
    P44["PER-44 membership and join"] -. "future activation" .-> LIVE
    G5["Gate 5 attempts and accepted rows"] -. "future activation" .-> LIVE
    E -. "future consumer only" .-> LIVE
~~~

## File Ownership Map

| File | Responsibility |
|---|---|
| <code>ai/decisions/ADR226_t3_synthetic_emergence_evidence_boundary.yaml</code> | Record the downstream post-commit boundary, synthetic claim limit, and activation dependencies. |
| <code>ai/decisions/index.yaml</code> | Register ADR226 exactly once. |
| <code>rust/Cargo.toml</code>, <code>rust/Cargo.lock</code> | Add the new workspace member and its pinned dependency closure. |
| <code>rust/crates/babylon-evidence/src/digest.rs</code> | Private-field 32-byte digest inputs and complete-envelope record digests. |
| <code>rust/crates/babylon-evidence/src/wire.rs</code> | Uniform envelope writer/reader, bounded cursor, NFC/ASCII and binary64 rules. |
| <code>rust/crates/babylon-evidence/src/classifier.rs</code> | SFS and persistence output classifiers only. |
| <code>rust/crates/babylon-evidence/src/records.rs</code> | Run, sample, trace, preregistration, schedule, and attempt-ledger wire records. |
| <code>rust/crates/babylon-evidence/src/profile.rs</code> | Component profile, proof profile, causal cone, intervention delta, and persistence-comparison wire records. |
| <code>rust/crates/babylon-evidence/src/validation.rs</code> | Synthetic-only projection, identity-delta, reachability, and driver validators. |
| <code>rust/crates/babylon-bsl/src/sfs_profile.rs</code> | Bounded AST footprint extraction and exact opt-in policy validation. |
| <code>rust/crates/bsl-lint/src/sfs_non_authorability.rs</code> | Cross-crate dependency-direction sentinel. |
| <code>tools/sfs_contract_vectors.py</code> | Independent Python encoder and classifier fixture producer/checker. |
| <code>tests/unit/tools/test_sfs_contract_vectors.py</code> | Python byte pins, limits, and real-descriptor mutation teeth. |
| <code>rust/crates/babylon-evidence/tests/fixtures/*</code> | Shared canonical wire, classifier, mutation, and synthetic-manifest vectors. |
| <code>rust/crates/babylon-bsl/tests/fixtures/sfs_profile/**</code> | Allowed and forbidden BSL audit corpus. |

---

### Task 1: Record the Synthetic Post-Commit Architecture Boundary

**Files:**

- Create: <code>ai/decisions/ADR226_t3_synthetic_emergence_evidence_boundary.yaml</code>
- Modify: <code>ai/decisions/index.yaml</code>

**Interfaces:**

- Consumes: the committed design's sections 9 and 11 plus ADR220, ADR221, ADR223, and ADR224.
- Produces: accepted ADR226 and a bidirectionally synchronized decision-index row.

- [ ] **Step 0: Establish live Linear ownership**

Refresh PER-52, PER-55, and PER-54 through the repository's Linear workflow.
Require PER-52 and PER-55 to be Done and PER-54 to be Todo. Move PER-54 to In
Progress before the first repository edit. Stop on any other status,
dependency, or owner conflict; do not weaken or recreate the issue relationship
locally.

- [ ] **Step 1: Verify the reserved decision number remains free**

Run:

~~~bash
rg --files -g 'ADR226*' ai/decisions
rg -n '^  ADR226_' ai/decisions/index.yaml
~~~

Expected: both commands return no matches. Any match stops execution; do not silently reuse or overwrite the number.

- [ ] **Step 2: Create the ADR without its index row**

Create the ADR with this exact decision content:

~~~yaml
ADR226_t3_synthetic_emergence_evidence_boundary:
  status: "accepted"
  date: "2026-08-23"
  title: >-
    T3 emergence evidence is a downstream post-commit evaluator with exact
    language-neutral records, discrete output classification, scoped
    non-authorability audits, and synthetic groundwork that cannot claim a
    live membership observable before Gate 3, Gate 5, and PER-44 activation
  context: |
    Constitution v4 requires aggregate patterns to emerge from local material
    relations rather than an authored response curve. The committed Neel
    relational-territory and situated-practice design therefore permits a
    post-commit classifier but prohibits the aggregate, wave stage, shape
    class, or political subjectivity from authoritative state or BSL-readable
    payloads.

    The live Rust tick currently publishes an in-memory TickReport with graph
    and nominal-world hashes plus identity-free audit receipts. It does not
    publish Gate 3's CommittedTickEnvelope, authoritative exogenous or action
    ledgers, an Archive outbox, or a complete run identity. PER-44 has not yet
    frozen attributed-membership identity or the territorial join, and no
    encounter-to-membership producer exists.

    Exact wire encoders, discrete classifier goldens, a bounded scoped BSL
    footprint auditor, and synthetic proof-profile/cone vectors do not require
    those live producers. Landing those contracts first gives later activation
    a language-neutral behavioral oracle without overstating executable game
    behavior.
  decision: |
    1. DOWNSTREAM EVALUATOR. A new babylon-evidence crate owns T3 record bytes,
       output classifiers, and post-commit validation. Its only engine-crate
       dependencies are babylon-kernel, babylon-bsl, and the PER-55-owned
       babylon-practice-contract; the existing workspace-resolved
       unicode-normalization crate is its declared text utility.
       Kernel, graph, BSL, practice-contract, tick, persistence, and
       client crates do not depend on babylon-evidence. Classification has no
       path into a later tick, action preview, AI objective, eligibility rule,
       or BSL binding.

    2. NO NEW PRIMITIVE. The evaluator uses finite IEEE-754 binary64 addition,
       subtraction, comparison, and bit equality only. It does not fit or
       smooth a curve, optimize a breakpoint, predict a result, or add an
       engine or BSL primitive. Its six SFS classes and four persistence
       classes recognize completed output under the exact committed predicate
       order.

    3. EXACT RECORD IDENTITY. Every T3 record uses schema version 1 and the
       uniform domain-NUL-version-u32-payload envelope. SHA-256 covers the
       complete envelope. Rust and Python independently reproduce checked-in
       literal bytes, digests, rejection vectors, and classifier vectors.

    4. SYNTHETIC SCOPE. Groundwork may encode only the records whose complete
       field order is frozen by the committed design. RelationalScopeV1,
       MembershipContributionV1, and MembershipContributionSetV1 remain
       absent until PER-44 freezes their identities and territorial join.
       Synthetic records may carry opaque digests for those deferred records
       but cannot claim a live observable.

    5. SCOPED BSL AUDIT. babylon-bsl owns bounded semantic AST inspection for
       exact read, query, operator, intrinsic, comparison/clamp-context, and
       effect footprints. The audit is opt-in and exact-allowlist. It rejects
       time, calendar, RNG, forbidden intrinsic, direct-observable, authored
       schedule, dead-permission, and unauthorized-effect fixtures. It claims
       only its bounded corpus, never universal semantic detection.

    6. CROSS-CRATE SENTINEL. bsl-lint owns the repository relationship check
       that rejects an engine-to-evidence dependency or a disallowed evidence
       dependency. Semantic BSL checks stay in babylon-bsl.

    7. SYNTHETIC CONE ONLY. Synthetic components exercise proof-profile bytes,
       source-digest binding, producer-consumer reachability, exact cone
       equality, candidate projection, run-identity mutation, and driver
       controls. No production causal-cone or host-component completeness
       claim lands in this train.

    8. ACTIVATION GATE. A live T3 producer remains blocked on Gate 3's complete
       committed-envelope and ledger identities, Gate 5's canonical attempts
       and accepted rows, PER-44's attributed membership and territorial join,
       and the separately owned encounter-to-membership producer. Topology,
       distribution, restart, backend, persistence, and player-facing Archive
       evidence remain in the later live-proof train.
  consequences: |
    - A Rust or Python rewrite can be checked against exact bytes and
      classifications without consulting an implementation-specific oracle.
    - Engine code cannot read classifications because the dependency points
      only from evidence toward existing pure contracts.
    - Synthetic green tests do not satisfy the live T3 milestone and cannot be
      presented as game emergence.
    - PER-44 retains ownership of membership identity and the territorial join;
      Gate 3 and Gate 5 retain their envelope and action-ledger ownership.
    - The footprint auditor adds no global BSL restriction and does not reopen
      ADR224's completed restricted-role contract.
  related:
    - ADR220_rust_owned_postgresql_persistence_boundary
    - ADR221_game_first_refoundation_v4
    - ADR223_whole_tick_atomicity_world_hash
    - ADR224_bsl_causal_composition_contract
    - ADR227_practice_contract_groundwork
~~~

- [ ] **Step 3: Run the ADR registry sentinel and verify the red phase**

Run from <code>rust/</code>:

~~~bash
cargo run -p bsl-lint --locked -- namespace-unique
~~~

Expected: exit 1 with a finding that <code>ADR226_t3_synthetic_emergence_evidence_boundary</code> has no <code>index.yaml</code> entry.

- [ ] **Step 4: Add the exact index row**

Insert this row in numeric order immediately before the first registered ADR
whose number is greater than 226, or append it after the last registered ADR
when no greater number exists:

~~~yaml
  ADR226_t3_synthetic_emergence_evidence_boundary:
    title: 'T3 emergence evidence is a downstream post-commit evaluator with exact language-neutral records, discrete output classification, scoped non-authorability audits, and synthetic groundwork that cannot claim a live membership observable before Gate 3, Gate 5, and PER-44 activation'
    status: accepted
    date: '2026-08-23'
    file: ADR226_t3_synthetic_emergence_evidence_boundary.yaml
~~~

- [ ] **Step 5: Run the documentation gates**

Run:

~~~bash
cd rust
cargo run -p bsl-lint --locked -- namespace-unique
cd ..
vale ai/decisions/ADR226_t3_synthetic_emergence_evidence_boundary.yaml
~~~

Expected: sentinel exit 0 and Vale reports zero errors and warnings.

- [ ] **Step 6: Commit the architecture boundary**

~~~bash
git add ai/decisions/ADR226_t3_synthetic_emergence_evidence_boundary.yaml ai/decisions/index.yaml
mise run commit -- "docs(architecture): record T3 synthetic evidence boundary"
~~~

---

### Task 2: Add the Downstream Crate and Non-Authorability Sentinel

**Files:**

- Modify: <code>rust/Cargo.toml</code>
- Modify mechanically: <code>rust/Cargo.lock</code>
- Modify: <code>.mise.toml</code>
- Create: <code>rust/crates/babylon-evidence/Cargo.toml</code>
- Create: <code>rust/crates/babylon-evidence/src/lib.rs</code>
- Modify: <code>rust/crates/bsl-lint/Cargo.toml</code>
- Modify: <code>rust/crates/bsl-lint/src/main.rs</code>
- Create: <code>rust/crates/bsl-lint/src/sfs_non_authorability.rs</code>
- Create: <code>rust/crates/bsl-lint/tests/sfs_non_authorability.rs</code>
- Create: <code>rust/crates/bsl-lint/tests/fixtures/sfs_non_authorability/clean/**</code>
- Create: <code>rust/crates/bsl-lint/tests/fixtures/sfs_non_authorability/reversed-direct/**</code>
- Create: <code>rust/crates/bsl-lint/tests/fixtures/sfs_non_authorability/reversed-alias/**</code>
- Create: <code>rust/crates/bsl-lint/tests/fixtures/sfs_non_authorability/reversed-target/**</code>
- Create: <code>rust/crates/bsl-lint/tests/fixtures/sfs_non_authorability/reversed-workspace/**</code>
- Create: <code>rust/crates/bsl-lint/tests/fixtures/sfs_non_authorability/reversed-two-hop/**</code>
- Create: <code>rust/crates/bsl-lint/tests/fixtures/sfs_non_authorability/disallowed-direct/**</code>
- Create: <code>rust/crates/bsl-lint/tests/fixtures/sfs_non_authorability/disallowed-alias/**</code>
- Create: <code>rust/crates/bsl-lint/tests/fixtures/sfs_non_authorability/reserved-rust/**</code>
- Create: <code>rust/crates/bsl-lint/tests/fixtures/sfs_non_authorability/reserved-bsl/**</code>
- Create: <code>rust/crates/bsl-lint/tests/fixtures/sfs_non_authorability/reserved-scenario/**</code>
- Create: <code>rust/crates/bsl-lint/tests/fixtures/sfs_non_authorability/reserved-two-hop-helper/**</code>

**Interfaces:**

- Consumes: the <code>CheckFn</code> registry, Cargo path-dependency syntax, and
  a bounded live-filesystem source walk.
- Produces: workspace crate <code>babylon-evidence</code>, registered check
  <code>sfs-non-authorability</code>, and accurate Mise gate metadata.

- [ ] **Step 1: Add the empty downstream crate**

Add <code>"crates/babylon-evidence"</code> after
<code>"crates/babylon-bsl"</code> in the workspace member list. Create:

~~~toml
[package]
name = "babylon-evidence"
description = "Post-commit synthetic evidence contracts"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
publish = false

[dependencies]
babylon-bsl = { path = "../babylon-bsl" }
babylon-kernel = { path = "../babylon-kernel" }
babylon-practice-contract = { path = "../babylon-practice-contract" }
unicode-normalization = "=0.1.25"

[dev-dependencies]
pretty_assertions = "1"
~~~

Create <code>src/lib.rs</code>:

~~~rust
//! Post-commit evidence contracts that never feed engine judgment.
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
~~~

Before refreshing the lock, add <code>toml = "1"</code>,
<code>proc-macro2 = "1"</code>, and
<code>syn = { version = "2", features = ["full"] }</code> to
<code>bsl-lint</code>'s dependencies. These reuse versions already present in
the workspace lock; do not infer Cargo edges or Rust syntax with a regular
expression. The sentinel uses <code>syn::parse_file</code> only after the
bounded token preflight below; it must not enable or call the recursively
unbounded <code>syn::visit::Visit</code> traversal.
Replace the package description with exactly <code>Repo-relationship sentinels
for BSL identity, namespaces, and T3 non-authorability</code> so the new
registered responsibility does not leave crate metadata stale.

The workspace already resolves <code>unicode-normalization</code> 0.1.25 for
live BSL NFC handling. The exact evidence dependency must reuse that package
and leave its lock entry and every existing BSL edge unchanged. Add a metadata
test that proves both crates resolve version 0.1.25 and that its exported
<code>UNICODE_VERSION</code> is <code>(17, 0, 0)</code>. Do not add a second
normalizer, vendor Unicode tables, or edit live BSL normalization behavior.

After every manifest edit above is present, refresh the workspace lock exactly
once:

~~~bash
mise run rust:lock-refresh
~~~

Immediately inspect <code>git diff -- rust/Cargo.lock</code>. The diff may add
only entries caused by the new workspace member and the already-declared dependency closure for
<code>babylon-evidence</code> and <code>bsl-lint</code>. The existing 0.1.25
package and every existing BSL dependency edge must remain.
Then run <code>cargo check -p babylon-evidence --locked</code> from
<code>rust/</code>. Every later Cargo command in this train also uses
<code>--locked</code>. Stop on unrelated lock churn.

Run the repository's complete supply-chain gate after the lock refresh:

~~~bash
cd rust
cargo deny check advisories bans licenses sources
~~~

Any advisory, duplicate-ban, license, or source refusal is a STOP; do not add a
waiver or relax <code>deny.toml</code>.

- [ ] **Step 2: Write the failing dependency-direction integration tests**

The test invokes <code>sfs-non-authorability</code> against the closed minimal fixture workspaces below:

~~~rust
#[test]
fn the_clean_downstream_shape_passes() {
    let (code, stdout) = run("clean");
    assert_eq!(code, 0, "stdout was:\n{stdout}");
}

#[test]
fn every_manifest_spelling_resolves_the_forbidden_package_edge() {
    let cases = [
        ("reversed-direct", "babylon-tick -> babylon-evidence"),
        ("reversed-alias", "babylon-tick -> babylon-evidence"),
        ("reversed-target", "babylon-tick -> babylon-evidence"),
        ("reversed-workspace", "babylon-tick -> babylon-evidence"),
        ("reversed-two-hop", "babylon-tick -> local-helper -> babylon-evidence"),
        ("disallowed-direct", "babylon-evidence -> babylon-tick"),
        ("disallowed-alias", "babylon-evidence -> babylon-tick"),
    ];
    for (fixture, expected_edge) in cases {
        let (code, stdout) = run(fixture);
        assert_eq!(code, 1, "fixture {fixture}; stdout was:\n{stdout}");
        assert!(stdout.contains(expected_edge), "fixture {fixture}; stdout was:\n{stdout}");
    }
}

#[test]
fn every_reserved_token_and_language_surface_fails() {
    let cases = [
        ("reserved-rust", "aggregate.rs", "sfs/aggregate"),
        ("reserved-rust", "classification.rs", "sfs/classification"),
        ("reserved-rust", "wave_stage.rs", "sfs/wave-stage"),
        ("reserved-rust", "hinterland_class.rs", "sfs/hinterland-class"),
        ("reserved-rust", "political_subjectivity.rs", "sfs/political-subjectivity"),
        ("reserved-rust", "aggregate_type.rs", "SfsAggregate"),
        ("reserved-rust", "classification_type.rs", "SfsClassification"),
        ("reserved-rust", "wave_stage_type.rs", "SfsWaveStage"),
        ("reserved-rust", "hinterland_class_type.rs", "SfsHinterlandClass"),
        ("reserved-rust", "political_subjectivity_type.rs", "SfsPoliticalSubjectivity"),
        ("reserved-bsl", "aggregate.bsl", "sfs/aggregate"),
        ("reserved-bsl", "classification.bsl", "sfs/classification"),
        ("reserved-bsl", "wave_stage.bsl", "sfs/wave-stage"),
        ("reserved-bsl", "hinterland_class.bsl", "sfs/hinterland-class"),
        ("reserved-bsl", "political_subjectivity.bsl", "sfs/political-subjectivity"),
        ("reserved-scenario", "aggregate.bscn", "sfs/aggregate"),
        ("reserved-scenario", "classification.bscn", "sfs/classification"),
        ("reserved-scenario", "wave_stage.bscn", "sfs/wave-stage"),
        ("reserved-scenario", "hinterland_class.bscn", "sfs/hinterland-class"),
        ("reserved-scenario", "political_subjectivity.bscn", "sfs/political-subjectivity"),
        ("reserved-two-hop-helper", "aggregate.rs", "sfs/aggregate"),
    ];
    for (fixture, file, token) in cases {
        let (code, stdout) = run(fixture);
        assert_eq!(code, 1, "fixture {fixture}; stdout was:\n{stdout}");
        assert!(stdout.contains(file), "missing {file}; stdout was:\n{stdout}");
        assert!(stdout.contains(token), "missing {token}; stdout was:\n{stdout}");
    }
}

#[test]
fn the_real_workspace_has_no_evidence_feedback_edge() {
    let output = Command::new(env!("CARGO_BIN_EXE_bsl-lint"))
        .arg("sfs-non-authorability")
        .output()
        .expect("bsl-lint must run");
    assert_eq!(output.status.code(), Some(0));
}
~~~

Each dependency-edge fixture under <code>clean</code>, the five
<code>reversed-*</code> roots, and the two <code>disallowed-*</code> roots
contains only <code>rust/Cargo.toml</code> and the
minimal crate manifests necessary to express its edge. Each reserved-token
fixture adds exactly the minimal production source mutants named in the
table-driven test for its language. The named table covers
the direct dependency key, an alias with
<code>package = "babylon-evidence"</code>, a target-specific dependency table,
and <code>workspace = true</code> inherited from
<code>[workspace.dependencies]</code>. Every reversed form reports the
resolved package edge <code>babylon-tick -&gt; babylon-evidence</code>. Both
disallowed forms report the reverse edge, including its alias spelling.

The two-hop fixture reports the complete resolved local path. The
<code>reserved-two-hop-helper</code> fixture contains no reserved token in an
<code>ENGINE_CRATES</code> member; its only token is in a local helper reachable
from <code>babylon-tick</code>, so a seven-root-only source scan must fail the
test.

- [ ] **Step 3: Run the test and verify the red phase**

Run from <code>rust/</code>:

~~~bash
cargo test -p bsl-lint --test sfs_non_authorability --locked
~~~

Expected: the child binary exits 2 because <code>sfs-non-authorability</code> is not registered.

- [ ] **Step 4: Implement the exact manifest-edge check**

Register:

~~~rust
mod sfs_non_authorability;

const CHECKS: &[(&str, CheckFn)] = &[
    (citation_drift::CHECK, citation_drift::run),
    (namespace_unique::CHECK, namespace_unique::run),
    (sfs_non_authorability::CHECK, sfs_non_authorability::run),
];
~~~

The module exposes:

~~~rust
pub const CHECK: &str = "sfs-non-authorability";

const ENGINE_CRATES: [&str; 7] = [
    "babylon-kernel",
    "babylon-graph",
    "babylon-bsl",
    "babylon-practice-contract",
    "babylon-tick",
    "babylon-persistence",
    "babylon-client",
];

const ALLOWED_EVIDENCE_PATH_DEPS: [&str; 3] = [
    "babylon-kernel",
    "babylon-bsl",
    "babylon-practice-contract",
];

const RESERVED_ENGINE_TOKENS: [&str; 10] = [
    "sfs/aggregate",
    "sfs/classification",
    "sfs/wave-stage",
    "sfs/hinterland-class",
    "sfs/political-subjectivity",
    "SfsAggregate",
    "SfsClassification",
    "SfsWaveStage",
    "SfsHinterlandClass",
    "SfsPoliticalSubjectivity",
];

pub fn run(repo: &Repo, roots: &[String]) -> Result<Vec<Finding>, String>;
~~~

Implementation rules:

1. Accept zero or one root; default to <code>rust</code>.
2. Admit at most 32 manifest files total: the workspace-root manifest plus at
   most 31 local package manifests.
   Before <code>toml::Value</code> parsing, refuse a manifest above 262,144
   bytes or aggregate admitted manifest bytes above 4,194,304. Run a
   byte-state preflight that distinguishes comments, basic strings, literal
   strings, and both multiline string forms; outside those spans it permits at
   most 65,536 structural tokens, 64 dotted-key/table-path components, and 32
   nested inline array/table levels. The scan advances with
   <code>for byte_index in 0..262_144</code> after a checked byte preflight and
   returns a typed error for an unterminated string/comment state, depth 33,
   path component 65, structural token 65,537, per-file byte 262,145, or
   aggregate byte 4,194,305 before invoking the recursive TOML parser. After
   parsing, walk at most 65,536 TOML values with an explicit stack bounded to
   65,536 and depth 32. A malformed or out-of-contract manifest returns an
   explicit check error and never becomes an empty edge set. Exact max and
   max-plus-one fixtures cover every boundary.
3. Resolve at most 256 entries across <code>dependencies</code>,
   <code>dev-dependencies</code>, <code>build-dependencies</code>, every
   target-specific dependency table, and inherited
   <code>workspace.dependencies</code>. Resolve aliases through
   <code>package</code>. For a path dependency, lexically normalize its path
   beneath the fixture/workspace root, read the target manifest, and use its
   <code>package.name</code>; refuse a path that escapes the root or lacks a
   package manifest. A <code>workspace = true</code> entry resolves through
   the same root dependency row before package comparison.
4. Build one closed local-package graph with at most 32 nodes and 256 resolved
   edges. Sort package names and edge pairs by complete UTF-8 bytes, map them
   to fixed indices, and compute reachability with at most 32 expansion passes
   from each of the seven engine roots. Emit a Fail finding when any complete
   local path from a resolved engine package reaches
   <code>babylon-evidence</code>, regardless of dependency key, alias, target
   table, inheritance, or intervening helper crates. Report the byte-least
   complete violating package path. A direct edge is the one-hop case; the
   two-hop fixture pins transitive refusal.
5. Emit a Fail finding when a resolved local path edge from
   <code>babylon-evidence</code> targets any package outside the three allowed
   engine contracts. Registry utilities remain outside this local dependency
   rule.
6. Emit a Fail finding when the evidence crate is absent from the workspace or its manifest is missing.
7. Enumerate the live filesystem beneath every non-test local package root
   reachable from the seven engine packages in the sealed graph,
   excluding <code>babylon-evidence</code>,
   not <code>Repo::working_tree_files</code>, <code>git ls-files</code>, or an
   index-derived list.

   This source leg must include tracked, staged, and untracked
   production files so a newly created pre-commit source cannot escape the
   check. Use an explicit directory work queue with ceilings of 512
   directories, depth 16, 16,384 total directory entries, and 4,096 admitted
   production <code>.rs</code>, <code>.bsl</code>, and <code>.bscn</code> paths.
   Each directory iterator advances inside a fixed indexed loop; the 16,385th
   entry, 513th directory, depth 17, or 4,097th production path returns a typed
   overflow error.

   Never follow a symlink. Refuse a symlink, a canonical path
   outside its declared crate root, a non-UTF-8 path, or an I/O error. Exclude
   every component named <code>tests</code> and the complete
   <code>babylon-evidence</code> root. Sort the admitted canonical relative
   paths before reading.
8. Refuse a source file larger than 262,144 bytes before parsing. For Rust,
   tokenize with <code>proc_macro2</code>, enforce at most 65,536 token trees,
   token-group depth 64, and an explicit-stack size of 65,536 with fixed
   indexed loops. Call <code>syn::parse_file</code> after the bounded walk.
   Inspect the identifiers
   and string literals through the same bounded iterative token-tree walk;
   never use <code>syn::Visit</code> and never inspect comments.

   For BSL and
   BSCN, call the existing iterative reader, require its
   <code>MAX_READER_NESTING_DEPTH</code> refusal. Preflight the resulting
   S-expression with the existing 1,048,576-node, depth-256, stack-65,536
   causal-contract limits before a fixed indexed semantic walk. Inspect BSL
   declared fields, binding reads, and effects, plus scenario attribute
   declarations and writes. Add exact byte, token/node, nesting, stack, and
   filesystem-overflow tests for all three language legs.
9. Exact reserved identifiers in a Rust type/field name or string literal, a
   BSL declaration/read/effect, or a scenario declaration/write fail unless
   the literal belongs to the one digest-pinned deny registry below.
10. The one deny registry named
   <code>FORBIDDEN_AUTHORITATIVE_IDENTIFIERS_V1</code> is exempt only at its
   declaration site in <code>babylon-bsl/src/sfs_profile.rs</code>. Its ten
   sorted LF-terminated rows must hash to
   <code>65e7a808f3b078da9c91e424f8fc6ca0a1309eac9882a707c8033aaf0620fb4b</code>.
   Any different bytes, second declaration, or use as an authoritative field,
   BSL read/effect, or scenario attribute fails. The sentinel and Task 8 test
   pin the same digest without making an engine crate depend on the linter.
11. Add production-shaped Rust, BSL, and <code>.bscn</code> mutants for every
   applicable reserved-token surface and assert the exact finding. Include all
   ten reserved tokens across the table-driven corpus; every token must fail in
   at least one parsed surface. All five slash-qualified names must each fail
   in Rust, BSL, and BSCN forms; all five PascalCase names must fail on the
   applicable Rust identifier surface. This proves that a matcher which skips
   a token or language surface fails the test suite. Every scenario mutant
   declares or writes its reserved field and requires the exact finding.
12. Sort findings by file, line, and evidence before returning.
13. Use <code>for index in 0..32</code> for the manifest and reachability walks,
   <code>for index in 0..256</code> for the dependency-entry walk, and
   <code>for index in 0..4_096</code> for the source-path walk after checked
   count preflights.

Update <code>tasks."check:bsl-sentinels".description</code> in
<code>.mise.toml</code> to name all three checks: <code>citation-drift</code>,
<code>namespace-unique</code>, and <code>sfs-non-authorability</code>. Describe
the third as the bounded dependency-direction and tracked/staged/untracked
production-source exact-token sentinel. Do not claim universal disguised-name
recognition.

This exact-token scan kills direct authoring under the reserved T3 names. It
does not claim to recognize a disguised field; the scoped BSL footprint and
later complete live causal cone carry the stronger evidence.

- [ ] **Step 5: Run the scoped and real sentinels**

~~~bash
cd rust
cargo test -p bsl-lint --test sfs_non_authorability --locked
cargo run -p bsl-lint --locked -- sfs-non-authorability
cargo run -p bsl-lint --locked -- all
cargo clippy -p bsl-lint --all-targets --locked -- -D warnings
cargo clippy -p babylon-evidence --all-targets --locked -- -D warnings -D clippy::pedantic
cargo fmt --all -- --check
~~~

Expected: all tests pass and both real-estate checks exit 0.

- [ ] **Step 6: Commit the crate boundary**

~~~bash
git add .mise.toml rust/Cargo.toml rust/Cargo.lock rust/crates/babylon-evidence rust/crates/bsl-lint/Cargo.toml rust/crates/bsl-lint/src/main.rs rust/crates/bsl-lint/src/sfs_non_authorability.rs rust/crates/bsl-lint/tests/sfs_non_authorability.rs rust/crates/bsl-lint/tests/fixtures/sfs_non_authorability
mise run commit -- "feat(evidence): establish downstream non-authorability boundary"
~~~

---

### Task 3: Implement the Uniform Envelope and Digest Types

**Files:**

- Create: <code>rust/crates/babylon-evidence/src/digest.rs</code>
- Create: <code>rust/crates/babylon-evidence/src/wire.rs</code>
- Modify: <code>rust/crates/babylon-evidence/src/lib.rs</code>
- Create: <code>rust/crates/babylon-evidence/tests/wire_envelope.rs</code>

**Interfaces:**

- Consumes: <code>babylon_kernel::sha256_of</code> and <code>unicode_normalization::UnicodeNormalization</code>.
- Produces: <code>Digest32</code>, <code>RecordDigest</code>, <code>T3Record</code>, <code>canonical_envelope</code>, <code>record_digest</code>, <code>decode_envelope</code>, <code>PayloadEncoder</code>, <code>PayloadCursor</code>, and <code>SfsWireError</code>.

- [ ] **Step 1: Write the failing literal-envelope tests**

Pin a one-byte payload without asking the encoder for the expectation:

~~~rust
#[derive(Debug, PartialEq, Eq)]
struct OneByte(u8);

impl T3Record for OneByte {
    const DOMAIN: &'static [u8] = b"babylon.sfs-sample.v1";
    const MAX_PAYLOAD_BYTES: usize = 1;
    type Error = SfsWireError;

    fn encode_payload(&self, out: &mut PayloadEncoder) -> Result<(), SfsWireError> {
        out.push_u8(self.0)
    }

    fn decode_payload(cursor: &mut PayloadCursor<'_>) -> Result<Self, SfsWireError> {
        Ok(Self(cursor.read_u8()?))
    }
}

#[test]
fn envelope_bytes_are_domain_nul_version_length_payload() {
    let expected = [
        b"babylon.sfs-sample.v1".as_slice(),
        &[0],
        &[0, 1],
        &[0, 0, 0, 1],
        &[0xaa],
    ]
    .concat();
    assert_eq!(canonical_envelope(&OneByte(0xaa)).unwrap(), expected);
    assert_eq!(record_digest(&OneByte(0xaa)).unwrap().as_bytes(), &sha256_of(&expected));
}

#[test]
fn decode_refuses_trailing_and_truncated_envelopes() {
    let bytes = canonical_envelope(&OneByte(0xaa)).unwrap();
    assert_eq!(decode_envelope::<OneByte>(&bytes).unwrap(), OneByte(0xaa));
    assert_eq!(
        decode_envelope::<OneByte>(&bytes[..bytes.len() - 1]),
        Err(SfsWireError::TruncatedEnvelope)
    );
    let mut trailing = bytes;
    trailing.push(0);
    assert_eq!(
        decode_envelope::<OneByte>(&trailing),
        Err(SfsWireError::TrailingBytes { count: 1 })
    );
}
~~~

Also pin an empty, embedded-NUL, non-ASCII, and 65-byte record domain; a wrong
decoded domain; version 2; a payload beyond its record maximum; an
empty/overlong ASCII string; a non-ASCII algorithm ID; valid precomposed
<code>café</code>, U+0100, Arabic, CJK, and supplementary-plane U+10000
witnesses; decomposed <code>cafe\u{301}</code>; signed finite binary64
encoding; negative-zero normalization; a negative-value refusal on the
non-negative helper; infinity; NaN; and cursor underflow. Rust and Python must
accept or reject the complete Unicode witness corpus identically. The Rust
test also requires <code>unicode_normalization::UNICODE_VERSION ==
(17, 0, 0)</code>.

- [ ] **Step 2: Run the test and verify the red phase**

~~~bash
cd rust
cargo test -p babylon-evidence --test wire_envelope --locked
~~~

Expected: unresolved imports for the wire and digest API.

- [ ] **Step 3: Implement the private-field digest types**

~~~rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Digest32([u8; 32]);

impl Digest32 {
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.0 == [0; 32]
    }

    #[must_use]
    pub fn to_hex(&self) -> String {
        let mut output = String::with_capacity(64);
        for index in 0..32 {
            use std::fmt::Write as _;
            write!(output, "{:02x}", self.0[index])
                .expect("writing to String cannot fail");
        }
        output
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RecordDigest(Digest32);

impl RecordDigest {
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        self.0.as_bytes()
    }

    #[must_use]
    pub fn to_hex(&self) -> String {
        self.0.to_hex()
    }
}
~~~

Only <code>record_digest</code> constructs <code>RecordDigest</code>; it has no
public raw-byte constructor. Do not add an implicit conversion from
<code>RecordDigest</code> back to an input digest. The record constructors name
every digest-bearing field.

- [ ] **Step 4: Implement the envelope and bounded cursor**

~~~rust
pub trait T3Record: Sized {
    const DOMAIN: &'static [u8];
    const MAX_PAYLOAD_BYTES: usize;
    type Error: From<SfsWireError>;

    fn encode_payload(&self, out: &mut PayloadEncoder) -> Result<(), SfsWireError>;
    fn decode_payload(cursor: &mut PayloadCursor<'_>) -> Result<Self, Self::Error>;
}

pub fn canonical_envelope<T: T3Record>(record: &T) -> Result<Vec<u8>, SfsWireError>;
pub fn record_digest<T: T3Record>(record: &T) -> Result<RecordDigest, SfsWireError>;
pub fn decode_envelope<T: T3Record>(bytes: &[u8]) -> Result<T, T::Error>;
~~~

<code>canonical_envelope</code> first requires a nonempty ASCII domain of at most 64 bytes with no NUL. It writes into a private staged payload, checks its record-specific maximum and <code>u32</code> representability, then publishes one complete vector. <code>decode_envelope</code> applies the same static-domain validation, locates the first NUL within the encoded 64-byte domain bound, checks the exact domain and schema version 1, checks the declared <code>u32</code> payload length against remaining bytes and the record maximum, delegates to <code>decode_payload</code>, and requires cursor exhaustion.

The one-byte wire witness sets <code>type Error = SfsWireError</code>. Every
validated record family sets its associated error to
<code>SfsRecordError</code> or <code>SfsProfileRecordError</code> and implements
<code>From&lt;SfsWireError&gt;</code>. Envelope failures and record-specific
semantic failures therefore remain distinguishable without a generic catch-all.

Use this explicit error enum:

~~~rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SfsWireError {
    UnsupportedSchemaVersion { found: u16 },
    WrongDomain,
    DomainEmpty,
    DomainContainsNul,
    DomainNonAscii,
    DomainTooLong,
    PayloadTooLong { limit: usize, actual: usize },
    CountTooLarge { field: &'static str, limit: usize, actual: usize },
    StringEmpty { field: &'static str },
    StringTooLong { field: &'static str, limit: usize, actual: usize },
    NonAscii { field: &'static str },
    InvalidUtf8 { field: &'static str },
    NonNfc { field: &'static str },
    DuplicateEntry { field: &'static str },
    OutOfOrder { field: &'static str },
    InvalidCode { field: &'static str, value: u8 },
    NonFinite { field: &'static str },
    Negative { field: &'static str },
    ArithmeticOverflow { field: &'static str },
    TruncatedEnvelope,
    TrailingBytes { count: usize },
}
~~~

<code>PayloadEncoder</code> provides checked <code>push_u8</code>,
<code>push_u16</code>, <code>push_u32</code>, <code>push_u64</code>,
<code>push_digest</code>, <code>push_finite_f64</code>,
<code>push_finite_non_negative_f64</code>, <code>push_ascii</code>,
<code>push_nfc_utf8</code>, and <code>push_complete_envelope</code>.
<code>PayloadCursor</code> provides the exact inverse operations. The signed
helper accepts a finite negative value for persistence separation and
normalizes either zero sign. The non-negative helper delegates to it and then
returns <code>Negative</code> for a value below zero.

V1 accepts the full Unicode scalar repertoire under Unicode 17.0.0 NFC; it
does not narrow the schema to selected scripts. Reuse the workspace's existing
<code>unicode-normalization</code> 0.1.25 package and require its exported
Unicode version tuple to equal <code>(17, 0, 0)</code>. The independent Python
encoder uses exact dev dependency <code>unicodedata2 == 17.0.1</code> and
requires <code>unicodedata2.unidata_version == "17.0.0"</code> before it reads or
writes a vector. A version mismatch is a hard refusal, never a reason to
rewrite an input or accept runtime-dependent bytes.

Preflight each declared
byte maximum and the 256-scalar maximum before normalization; every T3 NFC
field has a maximum of 256 UTF-8 bytes, so a conforming value cannot exceed
that scalar bound. Scan with a fixed <code>for index in 0..256</code> loop.

Decoders apply the same Unicode-version and NFC checks. The cross-runtime
witness corpus includes Latin, Arabic, CJK, combining-mark, and
supplementary-plane cases so a dependency or host-data drift fails visibly.

- [ ] **Step 5: Run the wire tests and static checks**

~~~bash
cd rust
cargo test -p babylon-evidence --test wire_envelope --locked
cargo clippy -p babylon-evidence --all-targets --locked -- -D warnings -D clippy::pedantic
cargo fmt --all -- --check
~~~

Expected: all tests pass and Clippy emits no warning.

- [ ] **Step 6: Commit the canonical envelope**

~~~bash
git add rust/crates/babylon-evidence/src/digest.rs rust/crates/babylon-evidence/src/wire.rs rust/crates/babylon-evidence/src/lib.rs rust/crates/babylon-evidence/tests/wire_envelope.rs
mise run commit -- "feat(evidence): add canonical T3 envelope"
~~~

---

### Task 4: Add the Discrete Output Classifiers

**Files:**

- Create: <code>rust/crates/babylon-evidence/src/classifier.rs</code>
- Modify: <code>rust/crates/babylon-evidence/src/lib.rs</code>
- Create: <code>rust/crates/babylon-evidence/tests/classifier_goldens.rs</code>

**Interfaces:**

- Consumes: finite binary64 mass and separation vectors.
- Produces: <code>SfsClass</code>, <code>PersistenceClass</code>, <code>classify_sfs</code>, and <code>classify_persistence</code>.

- [ ] **Step 1: Write all committed SFS goldens before the classifier**

~~~rust
#[test]
fn the_eight_w2_vectors_pin_predicate_order() {
    let cases = [
        (&[0.0, 1.0, 2.0, 5.0, 8.0, 10.0, 11.0][..], SfsClass::Continuing),
        (&[0.0, 1.0, 2.0, 5.0, 8.0, 8.0, 8.0][..], SfsClass::LatePlateau),
        (&[5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0][..], SfsClass::FlatPlateau),
        (&[0.0, 1.0, 2.0, 5.0, 8.0, 6.0, 4.0][..], SfsClass::Reversal),
        (&[0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0][..], SfsClass::ConstantRate),
        (&[0.0, 3.0, 6.0, 7.0, 8.0, 10.0, 12.0][..], SfsClass::Other),
        (&[0.0, 1.0, 2.0, 5.0, 8.0, 0.0, 8.0][..], SfsClass::Other),
        (&[0.0, 2.0, 2.0, 4.0, 4.0, 6.0, 6.0][..], SfsClass::Other),
    ];
    for (masses, expected) in cases {
        assert_eq!(classify_sfs(2, masses).unwrap(), expected);
        assert_eq!(expected.code(), expected as u8);
    }
}
~~~

Add failures for <code>w=1</code>, <code>w=53</code>, a length other than <code>3w+1</code>, NaN, infinity, and a subtraction that becomes non-finite. Pin negative zero as equivalent to positive zero for bit predicates.

- [ ] **Step 2: Write all four persistence goldens**

~~~rust
#[test]
fn the_four_persistence_vectors_pin_predicate_order() {
    let cases = [
        (&[2.0, 0.0, 0.0][..], PersistenceClass::Reconverged),
        (&[2.0, 1.0, -1.0][..], PersistenceClass::Reversed),
        (&[2.0, 1.0, 0.5][..], PersistenceClass::Persistent),
        (&[2.0, 0.0, 1.0][..], PersistenceClass::Mixed),
    ];
    for (separations, expected) in cases {
        assert_eq!(classify_persistence(2, separations).unwrap(), expected);
    }
}
~~~

Add <code>P=1</code> and <code>P=53</code> refusals, removal-window off-by-one,
wrong <code>P+1</code> length, checked-length arithmetic, invalid code, NaN,
and signed-zero tests.

- [ ] **Step 3: Run the classifier tests and verify the red phase**

~~~bash
cd rust
cargo test -p babylon-evidence --test classifier_goldens --locked
~~~

Expected: unresolved classifier imports.

- [ ] **Step 4: Implement the exact enums and bounded algorithms**

~~~rust
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SfsClass {
    FlatPlateau = 0,
    Reversal = 1,
    Continuing = 2,
    LatePlateau = 3,
    ConstantRate = 4,
    Other = 5,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersistenceClass {
    Reconverged = 0,
    Reversed = 1,
    Persistent = 2,
    Mixed = 3,
}

impl SfsClass {
    pub const fn code(self) -> u8;
    pub const fn from_code(value: u8) -> Option<Self>;
}

impl PersistenceClass {
    pub const fn code(self) -> u8;
    pub const fn from_code(value: u8) -> Option<Self>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SfsClassError {
    InvalidWindowWidth { found: u16 },
    WrongLength { expected: usize, actual: usize },
    NonFiniteMass { index: usize },
    NonFiniteDelta { index: usize },
    NonFiniteWindowGain { window: u8 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersistenceClassError {
    InvalidPostWidth { found: u16 },
    WrongLength { expected: usize, actual: usize },
    NonFiniteSeparation { index: usize },
}

pub fn classify_sfs(window_width: u16, masses: &[f64]) -> Result<SfsClass, SfsClassError>;
pub fn classify_persistence(
    post_width: u16,
    separations: &[f64],
) -> Result<PersistenceClass, PersistenceClassError>;
~~~

Implementation order for <code>classify_sfs</code> is exactly:

1. validate <code>2 <= w <= 52</code> and checked expected length <code>3w+1</code>;
2. normalize every zero and reject non-finite mass;
3. compute at most 156 one-operation deltas with <code>for index in 1..=156</code>, stopping at the checked length; reject a non-finite result and normalize either zero sign immediately;
4. compute G0, G1, and G2 with one subtraction each, reject a non-finite result, and normalize either zero sign immediately;
5. test flat plateau;
6. test reversal;
7. test continuing;
8. test late plateau;
9. test constant positive finite rate;
10. return other.

Do not algebraically rearrange G0, G1, or G2. Do not introduce an epsilon.

<code>classify_persistence</code> first requires <code>2 &lt;= P &lt;= 52</code>,
then computes <code>P+1</code> with checked arithmetic and requires that exact
number of finite values. It normalizes either zero sign before bit or sign
predicates, then applies reconverged, reversed, persistent, and mixed in that
order. Its traversal uses a fixed <code>for index in 0..53</code> ceiling and
stops at the checked length.

- [ ] **Step 5: Run classifier and Clippy gates**

~~~bash
cd rust
cargo test -p babylon-evidence --test classifier_goldens --locked
cargo clippy -p babylon-evidence --all-targets --locked -- -D warnings -D clippy::pedantic
cargo fmt --all -- --check
~~~

- [ ] **Step 6: Commit the classifiers**

~~~bash
git add rust/crates/babylon-evidence/src/classifier.rs rust/crates/babylon-evidence/src/lib.rs rust/crates/babylon-evidence/tests/classifier_goldens.rs
mise run commit -- "feat(evidence): add discrete emergence classifiers"
~~~

---

### Task 5: Encode Run, Trace, Preregistration, and Attempt Records

**Files:**

- Create: <code>rust/crates/babylon-evidence/src/records.rs</code>
- Modify: <code>rust/crates/babylon-evidence/src/lib.rs</code>
- Create: <code>rust/crates/babylon-evidence/tests/record_contracts.rs</code>

**Interfaces:**

- Consumes: Task 3 wire API, Task 4 classifiers,
  <code>babylon_kernel::SessionId</code>, PER-55's
  <code>babylon_practice_contract::PracticeIdV1</code>, and opaque
  <code>Digest32</code> inputs.
- Produces: six fully specified T3 records, their two fixed row types, and exact candidate projection.

- [ ] **Step 1: Write the failing constructor and wire-contract tests**

The test file must cover:

- <code>SessionId</code> byte lengths 1, 256, and 257 plus a non-NFC value;
- all ordered <code>RunIdentityV1</code> fields;
- one-field mutation of each run-identity field changes complete bytes and digest;
- <code>SfsSampleV1</code> rejects negative and non-finite aggregate mass and normalizes negative zero;
- <code>SfsTraceV1</code> requires interval 1, <code>3w+1</code> samples, consecutive ticks, exact start tick, and computed classification;
- the trace decoder rejects SFS class code 6 before it compares a valid stored class with the recomputed class;
- candidate rows sort by attempt tick then stable row digest and reject duplicates;
- stable row digest changes when any preimage field changes;
- preregistration accepts only empty-exogenous policy 0, flat cadence 0,
  positive stride, values admitted by the shared <code>PracticeIdV1</code>, and
  checked <code>start_tick = preregistered_at_tick + 1</code>;
- attempt rows sort by the candidate key, keep rejected rows, reject unknown disposition codes, and require a nonzero disposition digest;
- projecting every attempt row's first four fields reproduces exact candidate-schedule bytes;
- row count 65,535 succeeds in the dedicated bounded test and 65,536 fails before allocation growth.

Use a helper that creates asymmetric digests where byte 0 identifies the field. Do not use all-zero digests except where the wire explicitly permits a zero sentinel.

- [ ] **Step 2: Run the record test and verify the red phase**

~~~bash
cd rust
cargo test -p babylon-evidence --test record_contracts --locked
~~~

Expected: unresolved record imports.

- [ ] **Step 3: Define the exact record types**

Use these public shapes with private fields and validating constructors:

~~~rust
pub struct RunIdentityV1 {
    session: SessionId,
    scenario_digest: Digest32,
    prelude_declarations_digest: Digest32,
    vocabulary_digest: Digest32,
    rule_ast_digest: Digest32,
    host_component_manifest_digest: Digest32,
    defines_digest: Digest32,
    intrinsic_cost_cap_digest: Digest32,
    reference_manifest_digest: Digest32,
    governed_footprint_manifest_digest: Digest32,
    sfs_proof_profile_digest: Digest32,
    sfs_preregistration_digest: Digest32,
    initial_committed_envelope_digest: Digest32,
    initial_nominal_world_hash: Digest32,
    exogenous_input_ledger_digest: Digest32,
    practice_attempt_ledger_digest: Digest32,
    rng_algorithm_id: String,
    graph_contract_id: String,
}

pub struct SfsSampleV1 {
    tick: u64,
    nominal_world_hash: Digest32,
    committed_envelope_digest: Digest32,
    sorted_contribution_digest: Digest32,
    aggregate: f64,
}

pub struct SfsTraceV1 {
    run_identity_digest: Digest32,
    relational_scope_digest: Digest32,
    organization_node_id: u64,
    start_tick: u64,
    sample_interval: u16,
    window_width: u16,
    samples: Vec<SfsSampleV1>,
    classification: SfsClass,
}

pub struct PracticeCandidateRowV1 {
    stable_row_id_digest: Digest32,
    attempt_tick: u64,
    practice_input_authority_digest: Digest32,
    practice_intent_digest: Digest32,
}

pub struct PracticeCandidateScheduleV1 {
    rows: Vec<PracticeCandidateRowV1>,
}

pub struct SfsPreregistrationV1 {
    preregistered_at_tick: u64,
    start_tick: u64,
    relational_scope_digest: Digest32,
    practice_candidate_schedule_digest: Digest32,
    sfs_proof_profile_digest: Digest32,
    driver_contract_digest: Digest32,
    mutation_manifest_digest: Digest32,
    expected_exogenous_ledger_digest: Digest32,
    exogenous_policy: u8,
    cadence_kind: u8,
    first_attempt_tick: u64,
    attempt_stride: u16,
    attempt_count: u16,
    practice_code: PracticeIdV1,
    target_selection_policy_digest: Digest32,
    governed_cost: u32,
    parameter_bytes_digest: Digest32,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PracticeDispositionV1 {
    Accepted = 0,
    Rejected = 1,
}

pub struct PracticeAttemptRowV1 {
    candidate: PracticeCandidateRowV1,
    disposition: PracticeDispositionV1,
    disposition_digest: Digest32,
}

pub struct PracticeAttemptLedgerV1 {
    accepted_intent_ledger_digest: Digest32,
    rows: Vec<PracticeAttemptRowV1>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunIdentityField {
    Session,
    Scenario,
    PreludeDeclarations,
    Vocabulary,
    RuleAst,
    HostComponentManifest,
    Defines,
    IntrinsicCostCap,
    ReferenceManifest,
    GovernedFootprintManifest,
    SfsProofProfile,
    SfsPreregistration,
    InitialCommittedEnvelope,
    InitialNominalWorld,
    ExogenousInputLedger,
    PracticeAttemptLedger,
    RngAlgorithmId,
    GraphContractId,
}
~~~

Add read-only accessors needed by later validators. In particular:

~~~rust
impl RunIdentityV1 {
    pub fn differing_fields(&self, other: &Self) -> Vec<RunIdentityField>;
    pub const fn host_component_manifest_digest(&self) -> Digest32;
    pub const fn governed_footprint_manifest_digest(&self) -> Digest32;
    pub const fn sfs_proof_profile_digest(&self) -> Digest32;
    pub const fn sfs_preregistration_digest(&self) -> Digest32;
    pub const fn exogenous_input_ledger_digest(&self) -> Digest32;
    pub const fn practice_attempt_ledger_digest(&self) -> Digest32;
}

impl SfsPreregistrationV1 {
    pub const fn practice_candidate_schedule_digest(&self) -> Digest32;
    pub const fn sfs_proof_profile_digest(&self) -> Digest32;
    pub const fn driver_contract_digest(&self) -> Digest32;
    pub const fn mutation_manifest_digest(&self) -> Digest32;
    pub const fn expected_exogenous_ledger_digest(&self) -> Digest32;
    pub const fn first_attempt_tick(&self) -> u64;
    pub const fn attempt_stride(&self) -> u16;
    pub const fn attempt_count(&self) -> u16;
    pub const fn practice_code(&self) -> PracticeIdV1;
    pub const fn target_selection_policy_digest(&self) -> Digest32;
    pub const fn governed_cost(&self) -> u32;
    pub const fn parameter_bytes_digest(&self) -> Digest32;
}

impl SfsTraceV1 {
    pub const fn run_identity_digest(&self) -> Digest32;
}

impl PracticeCandidateScheduleV1 {
    pub fn rows(&self) -> &[PracticeCandidateRowV1];
}

impl PracticeCandidateRowV1 {
    pub const fn attempt_tick(&self) -> u64;
    pub const fn practice_intent_digest(&self) -> Digest32;
}
~~~

<code>differing_fields</code> checks the eighteen fields in enum order with a
fixed <code>for index in 0..18</code> dispatch and returns fields in that same
order.

Use these constructor signatures; decoders apply the same validation:

~~~rust
impl RunIdentityV1 {
    // The constructor mirrors one sealed wire record. A second parameter
    // object would duplicate the same eighteen-field type without reducing
    // ambiguity.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        session: SessionId,
        scenario_digest: Digest32,
        prelude_declarations_digest: Digest32,
        vocabulary_digest: Digest32,
        rule_ast_digest: Digest32,
        host_component_manifest_digest: Digest32,
        defines_digest: Digest32,
        intrinsic_cost_cap_digest: Digest32,
        reference_manifest_digest: Digest32,
        governed_footprint_manifest_digest: Digest32,
        sfs_proof_profile_digest: Digest32,
        sfs_preregistration_digest: Digest32,
        initial_committed_envelope_digest: Digest32,
        initial_nominal_world_hash: Digest32,
        exogenous_input_ledger_digest: Digest32,
        practice_attempt_ledger_digest: Digest32,
        rng_algorithm_id: &str,
        graph_contract_id: &str,
    ) -> Result<Self, SfsRecordError>;
}

impl SfsSampleV1 {
    pub fn new(
        tick: u64,
        nominal_world_hash: Digest32,
        committed_envelope_digest: Digest32,
        sorted_contribution_digest: Digest32,
        aggregate: f64,
    ) -> Result<Self, SfsRecordError>;
}

impl SfsTraceV1 {
    pub fn new(
        run_identity_digest: Digest32,
        relational_scope_digest: Digest32,
        organization_node_id: u64,
        start_tick: u64,
        window_width: u16,
        samples: Vec<SfsSampleV1>,
    ) -> Result<Self, SfsRecordError>;
}

impl PracticeCandidateRowV1 {
    pub fn new(
        attempt_tick: u64,
        practice_input_authority_digest: Digest32,
        practice_intent_digest: Digest32,
    ) -> Self;
}

impl PracticeCandidateScheduleV1 {
    pub fn new(rows: Vec<PracticeCandidateRowV1>) -> Result<Self, SfsRecordError>;
}

impl SfsPreregistrationV1 {
    // The constructor mirrors one sealed wire record. A second parameter
    // object would duplicate the same fourteen-field type.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        preregistered_at_tick: u64,
        relational_scope_digest: Digest32,
        practice_candidate_schedule_digest: Digest32,
        sfs_proof_profile_digest: Digest32,
        driver_contract_digest: Digest32,
        mutation_manifest_digest: Digest32,
        expected_exogenous_ledger_digest: Digest32,
        first_attempt_tick: u64,
        attempt_stride: u16,
        attempt_count: u16,
        practice_code: PracticeIdV1,
        target_selection_policy_digest: Digest32,
        governed_cost: u32,
        parameter_bytes_digest: Digest32,
    ) -> Result<Self, SfsRecordError>;
}

impl PracticeAttemptRowV1 {
    pub fn new(
        candidate: PracticeCandidateRowV1,
        disposition: PracticeDispositionV1,
        disposition_digest: Digest32,
    ) -> Result<Self, SfsRecordError>;
}

impl PracticeAttemptLedgerV1 {
    pub fn new(
        accepted_intent_ledger_digest: Digest32,
        rows: Vec<PracticeAttemptRowV1>,
    ) -> Result<Self, SfsRecordError>;
}
~~~

<code>SfsTraceV1::new</code> hard-codes sample interval 1.
<code>SfsPreregistrationV1::new</code> hard-codes exogenous policy 0 and cadence
kind 0 and derives <code>start_tick</code> by checked addition. Raw codes occur
only at decode boundaries through the shared
<code>PracticeIdV1::try_from(u8)</code>. T3 defines no second practice enum or
numeric mapping.

The wire domains and maximums are:

| Type | Domain | Maximum payload basis |
|---|---|---|
| <code>RunIdentityV1</code> | <code>babylon.run-identity.v1</code> | two 64-byte IDs plus fixed fields |
| <code>SfsSampleV1</code> | <code>babylon.sfs-sample.v1</code> | fixed 112-byte payload |
| <code>SfsTraceV1</code> | <code>babylon.sfs-trace.v1</code> | 157 complete sample envelopes |
| <code>SfsPreregistrationV1</code> | <code>babylon.sfs-preregistration.v1</code> | fixed payload |
| <code>PracticeCandidateScheduleV1</code> | <code>babylon.practice-candidate-schedule.v1</code> | 65,535 fixed rows |
| <code>PracticeAttemptLedgerV1</code> | <code>babylon.practice-attempt-ledger.v1</code> | header plus 65,535 fixed rows |

Do not add the three PER-44-owned domains to this module.

- [ ] **Step 4: Implement stable IDs, sorting, and projection**

~~~rust
pub fn practice_attempt_row_id(
    attempt_tick: u64,
    authority_digest: Digest32,
    intent_digest: Digest32,
) -> Digest32;

impl PracticeAttemptLedgerV1 {
    pub fn project_candidates(
        &self,
    ) -> Result<PracticeCandidateScheduleV1, SfsRecordError>;
}
~~~

The stable-row preimage is exactly:

~~~text
ASCII("babylon.practice-attempt-row.v1")
|| 0x00
|| attempt_tick_u64_be
|| practice_input_authority_digest_32
|| practice_intent_digest_32
~~~

Constructors receive vectors, reject a count beyond 65,535 before mutation, sort by the specified key, and reject adjacent duplicate canonical keys. They never silently discard duplicates.

<code>PracticeAttemptLedgerV1</code> validates only its frozen wire and projection rules. Its documentation states that this crate cannot verify the header against an authoritative accepted-intent ledger until Gate 5 supplies that contract.

- [ ] **Step 5: Implement trace construction without an authored class**

<code>SfsTraceV1::new</code> takes no class argument. It extracts normalized sample aggregates and calls <code>classify_sfs</code>; only that returned class enters the record. The decoder maps the stored byte through <code>SfsClass::from_code</code> and returns <code>SfsWireError::InvalidCode { field: "classification", value }</code> for an unknown byte. It then recomputes the class and returns <code>SfsRecordError::ClassificationMismatch</code> when two valid classes differ.

Use specific record errors:

~~~rust
pub enum SfsRecordError {
    Wire(SfsWireError),
    Classifier(SfsClassError),
    InvalidSessionLength { actual: usize },
    InvalidSampleInterval { found: u16 },
    InvalidWindowWidth { found: u16 },
    WrongSampleCount { expected: usize, actual: usize },
    TickDiscontinuity { expected: u64, actual: u64 },
    StableRowDigestMismatch,
    CandidateProjectionMismatch,
    ClassificationMismatch { stored: u8, computed: u8 },
    InvalidCadence,
    InvalidExogenousPolicy,
    InvalidPracticeCode { value: u8 },
    InvalidDisposition { value: u8 },
    ZeroDispositionDigest,
    ArithmeticOverflow { field: &'static str },
}
~~~

- [ ] **Step 6: Run the record tests and scoped crate gate**

~~~bash
cd rust
cargo test -p babylon-evidence --test record_contracts --locked
cargo test -p babylon-evidence --locked
cargo clippy -p babylon-evidence --all-targets --locked -- -D warnings -D clippy::pedantic
cargo fmt --all -- --check
~~~

- [ ] **Step 7: Commit the frozen run and trace records**

~~~bash
git add rust/crates/babylon-evidence/src/records.rs rust/crates/babylon-evidence/src/lib.rs rust/crates/babylon-evidence/tests/record_contracts.rs
mise run commit -- "feat(evidence): encode frozen T3 run records"
~~~

---

### Task 6: Encode Proof Profiles, Cones, Intervention Deltas, and Persistence Comparisons

**Files:**

- Create: <code>rust/crates/babylon-evidence/src/profile.rs</code>
- Modify: <code>rust/crates/babylon-evidence/src/lib.rs</code>
- Create: <code>rust/crates/babylon-evidence/tests/profile_wire.rs</code>

**Interfaces:**

- Consumes: Tasks 3 and 4.
- Produces: the remaining fully frozen T3 record encoders, with structural validation but no live ledger or host-manifest claim.

- [ ] **Step 1: Write the failing profile and persistence wire tests**

Pin:

- component-kind codes 0 through 3 and rejection of 4;
- component ID lengths 0, 1, 256, and 257;
- set entry lengths 0, 1, 96, and 97;
- 64 entries and components accepted, 65 rejected;
- duplicate and byte-order errors;
- nested component envelopes carry their own domain/version/u32 length and no second length prefix;
- proof-profile payload consumption with no trailing bytes;
- causal-cone roots, sinks, and components each sort and reject duplicates;
- intervention ADD, REMOVE, and REPLACE zero/nonzero rules;
- intervention rows sort by stable row digest and reject duplicates;
- intervention row counts 0 and 65,535 succeed, while 65,536 refuses before
  sorting or payload allocation;
- persistence ledger-kind codes 0 and 1, class codes 0 through 3, rejection of class code 4, and exact <code>P+1</code> separation count;
- a comparison that changes both selected ledgers is structurally invalid in the synthetic twin validator added in Task 9.

- [ ] **Step 2: Run the profile test and verify the red phase**

~~~bash
cd rust
cargo test -p babylon-evidence --test profile_wire --locked
~~~

Expected: unresolved profile imports.

- [ ] **Step 3: Define exact profile records**

~~~rust
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentKindV1 {
    BslRule = 0,
    RustBoundary = 1,
    Reducer = 2,
    PostCommitProducer = 3,
}

pub struct CanonicalProfileSet {
    entries: Vec<String>,
}

pub struct SfsComponentProofProfileV1 {
    component_id: String,
    component_kind: ComponentKindV1,
    component_source_digest: Digest32,
    field_reads: CanonicalProfileSet,
    edge_reads: CanonicalProfileSet,
    constant_reads: CanonicalProfileSet,
    queries: CanonicalProfileSet,
    operators: CanonicalProfileSet,
    intrinsics: CanonicalProfileSet,
    comparison_clamp_contexts: CanonicalProfileSet,
    effects: CanonicalProfileSet,
}

pub struct SfsProofProfileV1 {
    governed_manifest_digest: Digest32,
    forbidden_corpus_digest: Digest32,
    audit_semantics_id: String,
    audit_source_digest: Digest32,
    causal_cone_digest: Digest32,
    components: Vec<SfsComponentProofProfileV1>,
}

pub struct CausalConeV1 {
    roots: Vec<String>,
    sinks: Vec<String>,
    components: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SfsProfileRecordError {
    Wire(SfsWireError),
    InvalidComponentKind { value: u8 },
    InvalidAuditSemanticsId,
    InvalidLedgerKind { value: u8 },
    InvalidInterventionOperation { value: u8 },
    InvalidInterventionRow,
    DuplicateComponentId,
    DuplicateConeId { set: &'static str },
    Classification(PersistenceClassError),
    ClassificationMismatch { stored: u8, computed: u8 },
}
~~~

Use exact domains:

- <code>babylon.sfs-component-proof-profile.v1</code>
- <code>babylon.sfs-proof-profile.v1</code>
- <code>babylon.sfs-causal-cone.v1</code>

The proof profile accepts audit-semantics ID <code>babylon.sfs.audit.v1</code> only in V1 synthetic vectors. It checks ASCII length 1 through 64.

Add the read-only accessors used by Task 9:

~~~rust
impl SfsComponentProofProfileV1 {
    pub fn component_id(&self) -> &str;
    pub const fn component_kind(&self) -> ComponentKindV1;
    pub const fn component_source_digest(&self) -> Digest32;
}

impl SfsProofProfileV1 {
    pub fn components(&self) -> &[SfsComponentProofProfileV1];
    pub const fn governed_manifest_digest(&self) -> Digest32;
    pub const fn causal_cone_digest(&self) -> Digest32;
}

impl CausalConeV1 {
    pub fn roots(&self) -> &[String];
    pub fn sinks(&self) -> &[String];
    pub fn components(&self) -> &[String];
}
~~~

Use these validating constructor signatures:

~~~rust
impl CanonicalProfileSet {
    pub fn new(
        field: &'static str,
        entries: Vec<String>,
    ) -> Result<Self, SfsProfileRecordError>;
    pub fn entries(&self) -> &[String];
}

impl SfsComponentProofProfileV1 {
    // This signature mirrors the nine-set sealed payload. The local exemption
    // avoids a duplicate parameter object with the same fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        component_id: &str,
        component_kind: ComponentKindV1,
        component_source_digest: Digest32,
        field_reads: CanonicalProfileSet,
        edge_reads: CanonicalProfileSet,
        constant_reads: CanonicalProfileSet,
        queries: CanonicalProfileSet,
        operators: CanonicalProfileSet,
        intrinsics: CanonicalProfileSet,
        comparison_clamp_contexts: CanonicalProfileSet,
        effects: CanonicalProfileSet,
    ) -> Result<Self, SfsProfileRecordError>;
}

impl SfsProofProfileV1 {
    pub fn new(
        governed_manifest_digest: Digest32,
        forbidden_corpus_digest: Digest32,
        audit_semantics_id: &str,
        audit_source_digest: Digest32,
        causal_cone_digest: Digest32,
        components: Vec<SfsComponentProofProfileV1>,
    ) -> Result<Self, SfsProfileRecordError>;
}

impl CausalConeV1 {
    pub fn new(
        roots: Vec<String>,
        sinks: Vec<String>,
        components: Vec<String>,
    ) -> Result<Self, SfsProfileRecordError>;
}
~~~

- [ ] **Step 4: Define intervention and comparison records**

~~~rust
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DifferingLedgerKindV1 {
    ExogenousInput = 0,
    PracticeAttempt = 1,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterventionOperationV1 {
    Add = 0,
    Remove = 1,
    Replace = 2,
}

pub struct InterventionDeltaRowV1 {
    operation: InterventionOperationV1,
    stable_row_id_digest: Digest32,
    control_row_digest: Digest32,
    intervention_row_digest: Digest32,
}

pub struct InterventionDeltaV1 {
    ledger_kind: DifferingLedgerKindV1,
    rows: Vec<InterventionDeltaRowV1>,
}

pub struct PersistenceComparisonV1 {
    control_trace_digest: Digest32,
    intervention_trace_digest: Digest32,
    differing_ledger_kind: DifferingLedgerKindV1,
    control_differing_ledger_digest: Digest32,
    intervention_differing_ledger_digest: Digest32,
    intervention_delta_digest: Digest32,
    last_intervention_tick: u64,
    post_width: u16,
    separations: Vec<f64>,
    persistence_class: PersistenceClass,
}
~~~

Add the read-only identity accessors consumed by Task 9:

~~~rust
impl InterventionDeltaV1 {
    pub const fn ledger_kind(&self) -> DifferingLedgerKindV1;
}

impl PersistenceComparisonV1 {
    pub const fn control_trace_digest(&self) -> Digest32;
    pub const fn intervention_trace_digest(&self) -> Digest32;
    pub const fn differing_ledger_kind(&self) -> DifferingLedgerKindV1;
    pub const fn control_differing_ledger_digest(&self) -> Digest32;
    pub const fn intervention_differing_ledger_digest(&self) -> Digest32;
    pub const fn intervention_delta_digest(&self) -> Digest32;
}
~~~

Use these constructors:

~~~rust
impl InterventionDeltaRowV1 {
    pub fn new(
        operation: InterventionOperationV1,
        stable_row_id_digest: Digest32,
        control_row_digest: Digest32,
        intervention_row_digest: Digest32,
    ) -> Result<Self, SfsProfileRecordError>;
}

impl InterventionDeltaV1 {
    pub fn new(
        ledger_kind: DifferingLedgerKindV1,
        rows: Vec<InterventionDeltaRowV1>,
    ) -> Result<Self, SfsProfileRecordError>;
}

impl PersistenceComparisonV1 {
    // The constructor mirrors one sealed wire record. The local exemption
    // avoids a duplicate parameter object with the same fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        control_trace_digest: Digest32,
        intervention_trace_digest: Digest32,
        differing_ledger_kind: DifferingLedgerKindV1,
        control_differing_ledger_digest: Digest32,
        intervention_differing_ledger_digest: Digest32,
        intervention_delta_digest: Digest32,
        last_intervention_tick: u64,
        post_width: u16,
        separations: Vec<f64>,
    ) -> Result<Self, SfsProfileRecordError>;
}
~~~

Use exact domains:

- <code>babylon.intervention-delta.v1</code>
- <code>babylon.persistence-comparison.v1</code>

<code>InterventionDeltaV1</code> encodes exactly
<code>ledger_kind_u8 || row_count_u32_be ||</code> repeated rows, where each row
is <code>operation_u8 || stable_row_id_digest_32 || control_row_digest_32 ||
intervention_row_digest_32</code>. Its row ceiling is exactly 65,535 and its
maximum payload is 6,356,900 bytes. The constructor preflights
<code>rows.len()</code>, walks the admitted input with
<code>.take(65_536)</code>, sorts by stable-row digest, and rejects an adjacent
duplicate. The decoder preflights the declared count and exact remaining byte
product with checked arithmetic before it allocates or enters a fixed
<code>for index in 0..65_535</code> loop. Tests pin 0, 65,535, 65,536,
truncation, trailing bytes, count multiplication overflow, sort order, and
duplicate refusal.

<code>PersistenceComparisonV1::new</code> takes separations but no class argument. It computes the class with <code>classify_persistence</code>. The decoder maps the stored byte through <code>PersistenceClass::from_code</code> and returns <code>SfsWireError::InvalidCode { field: "persistence_class", value }</code> for an unknown byte. It recomputes a valid stored class and returns <code>SfsProfileRecordError::ClassificationMismatch { stored, computed }</code> on a mismatch.

This task does not parse authoritative ledger rows, resolve effective ticks, recompute accepted subsets, or validate a canonical empty exogenous ledger.

- [ ] **Step 5: Run profile, crate, and Clippy gates**

~~~bash
cd rust
cargo test -p babylon-evidence --test profile_wire --locked
cargo test -p babylon-evidence --locked
cargo clippy -p babylon-evidence --all-targets --locked -- -D warnings -D clippy::pedantic
cargo fmt --all -- --check
~~~

- [ ] **Step 6: Commit the frozen proof records**

~~~bash
git add rust/crates/babylon-evidence/src/profile.rs rust/crates/babylon-evidence/src/lib.rs rust/crates/babylon-evidence/tests/profile_wire.rs
mise run commit -- "feat(evidence): encode synthetic proof records"
~~~

---

### Task 7: Add Independent Python Contract and Classifier Vectors

**Files:**

- Modify: <code>pyproject.toml</code>
- Modify mechanically: <code>uv.lock</code>
- Create: <code>tools/sfs_contract_vectors.py</code>
- Create: <code>tests/unit/tools/test_sfs_contract_vectors.py</code>
- Create generated fixture: <code>rust/crates/babylon-evidence/tests/fixtures/sfs_wire_vectors_v1.txt</code>
- Create generated fixture: <code>rust/crates/babylon-evidence/tests/fixtures/sfs_classifier_vectors_v1.txt</code>
- Create generated fixture: <code>rust/crates/babylon-evidence/tests/fixtures/sfs_identity_mutations_v1.txt</code>
- Create: <code>rust/crates/babylon-evidence/tests/cross_language_vectors.rs</code>

**Interfaces:**

- Consumes: the committed field-order tables and class predicates, not Rust output.
- Produces: bounded <code>--write</code>/<code>--check</code> fixtures consumed by both Python tests and Rust tests.

Before Step 1, add exact dev dependency
<code>unicodedata2==17.0.1</code> to <code>pyproject.toml</code>, refresh
<code>uv.lock</code> once, and inspect that the lock changes only for that
package and its platform artifacts. Run
<code>uv run python -c 'import unicodedata2; assert
unicodedata2.unidata_version == "17.0.0"'</code> and
<code>uv lock --check</code>. A missing compatible artifact or unrelated lock
churn is a STOP, not a reason to fall back to the host standard library.

- [ ] **Step 1: Write the failing Python test around literal expectations**

The Python test imports <code>Sequence</code> from
<code>collections.abc</code> and <code>SimpleNamespace</code> from
<code>types</code>, imports the new tool by file path, and pins:

~~~python
def test_uniform_envelope_is_literal_bytes() -> None:
    payload = b"\xaa"
    expected = (
        b"babylon.sfs-sample.v1"
        + bytes([0])
        + struct.pack(">H", 1)
        + struct.pack(">I", 1)
        + payload
    )
    assert exporter._envelope(b"babylon.sfs-sample.v1", payload) == expected
    assert exporter._digest(expected) == hashlib.sha256(expected).digest()


@pytest.mark.parametrize(
    ("values", "expected"),
    [
        ((0, 1, 2, 5, 8, 10, 11), 2),
        ((0, 1, 2, 5, 8, 8, 8), 3),
        ((5, 5, 5, 5, 5, 5, 5), 0),
        ((0, 1, 2, 5, 8, 6, 4), 1),
        ((0, 1, 2, 3, 4, 5, 6), 4),
        ((0, 3, 6, 7, 8, 10, 12), 5),
        ((0, 1, 2, 5, 8, 0, 8), 5),
        ((0, 2, 2, 4, 4, 6, 6), 5),
    ],
)
def test_sfs_goldens(values: Sequence[int], expected: int) -> None:
    assert exporter._classify_sfs(2, tuple(map(float, values))) == expected
~~~

Also pin one complete asymmetric <code>RunIdentityV1</code> hex string and SHA-256 literal in the test. That literal is transcribed from the specification with <code>struct.pack</code>, not copied from a Rust test run.

- [ ] **Step 2: Run the Python test and verify the red phase**

~~~bash
mise run test:q -- tests/unit/tools/test_sfs_contract_vectors.py
~~~

Expected: import fails because the tool does not exist.

- [ ] **Step 3: Implement an independent bounded Python encoder**

The tool uses only:

~~~python
import argparse
import hashlib
import os
import stat
import struct
import sys
import tempfile
import unicodedata2
from collections.abc import Sequence
from pathlib import Path
from typing import Final
~~~

It must not import <code>babylon-evidence</code>, invoke Cargo, parse Rust output, or read expected bytes from a Rust-generated file.

Define these constants:

~~~python
SCHEMA_VERSION: Final = 1
MAX_ROWS: Final = 65_535
MAX_SAMPLES: Final = 157
MAX_COMPONENTS: Final = 64
MAX_VECTOR_BYTES: Final = 16_777_216
MAX_NFC_SCALARS: Final = 256
UNICODE_DATA_VERSION: Final = "17.0.0"
~~~

Define <code>UnicodeDataVersionError(RuntimeError)</code> whose message names
the expected and actual Unicode data versions. The command entry point and
<code>_nfc_utf8</code> both refuse unless
<code>unicodedata2.unidata_version == UNICODE_DATA_VERSION</code>, so direct
unit calls cannot bypass the version gate.

Define <code>VectorIoError(RuntimeError)</code> with read-only
<code>path: Path</code> and <code>operation: str</code> attributes. Its message
is exactly <code>"{operation} failed for {path}"</code>; preserve the caught
<code>OSError</code> as <code>__cause__</code> with
<code>raise VectorIoError(path, operation) from error</code>.

Define these functions with the exact contracts below:

| Function | Exact contract |
|---|---|
| <code>_envelope(domain: bytes, payload: bytes) -> bytes</code> | Require nonempty ASCII domain with no NUL and at most 64 bytes; require payload length at most <code>u32::MAX</code>; concatenate domain, one zero byte, <code>struct.pack("&gt;H", 1)</code>, <code>struct.pack("&gt;I", len(payload))</code>, and payload. |
| <code>_digest(envelope: bytes) -> bytes</code> | Return <code>hashlib.sha256(envelope).digest()</code>. |
| <code>_nfc_utf8(value: str, field: str, minimum: int, maximum: int) -> bytes</code> | Require Unicode data version 17.0.0; reject more than 256 scalars; scan with <code>for index in range(MAX_NFC_SCALARS)</code>; reject when <code>unicodedata2.normalize("NFC", value) != value</code>; encode strict UTF-8; enforce the inclusive byte bounds without rewriting. |
| <code>_ascii(value: str, field: str, minimum: int, maximum: int) -> bytes</code> | Encode strict ASCII and enforce the inclusive byte bounds. |
| <code>_finite_bits(value: float, non_negative: bool) -> bytes</code> | Reject non-finite values and negative values when requested; convert either signed zero to positive zero; return <code>struct.pack("&gt;d", value)</code>. |
| <code>_classify_sfs(window_width: int, masses: Sequence[float]) -> int</code> | Execute Task 4's ten ordered steps with fixed maximum 156 deltas and return only codes 0 through 5. |
| <code>_classify_persistence(post_width: int, separations: Sequence[float]) -> int</code> | Execute Task 4's four ordered persistence predicates and return only codes 0 through 3. |
| <code>_wire_vectors() -> list[str]</code> | Build one asymmetric canonical row for every record in Tasks 5 and 6 plus minimum/maximum scalar and string fields and nested-envelope vectors. Do not place a 65,535-row schedule or ledger in a hex fixture. |
| <code>_classifier_vectors() -> list[str]</code> | Build the eight SFS and four persistence rows from the committed literals. |
| <code>_identity_mutations() -> list[str]</code> | Change each RunIdentity field once from the common asymmetric base and record literal mutated bytes and SHA-256. |
| <code>_check(path: Path, expected: bytes) -> bool</code> | Open the file once, call <code>os.fstat</code> on that descriptor, reject non-regular or wrong-sized data before reading, read at most <code>len(expected)+1</code>, and compare exact bytes. |
| <code>_write_atomic(path: Path, expected: bytes) -> None</code> | Reject data beyond <code>MAX_VECTOR_BYTES</code>; stage one complete sibling file with <code>tempfile.mkstemp</code>, make one buffered <code>write</code> call and require its return count to equal <code>len(expected)</code>, flush, <code>os.fsync</code>, close, and publish with <code>os.replace</code>. On an <code>OSError</code>, close and unlink only the exact staged path, then raise <code>VectorIoError(path, operation)</code>. |

No unlisted record or classifier enters the tool.

The Python wire test uses the same Unicode witnesses as Task 3: precomposed
<code>café</code>, U+0100, Arabic <code>مساعدة</code>, CJK
<code>互助</code>, and U+10000 pass unchanged under the pinned data;
decomposed <code>cafe\N{COMBINING ACUTE ACCENT}</code> is non-NFC. A monkeypatch
of <code>unicodedata2.unidata_version</code> to any other value raises the exact
version error before vector generation. Rust and Python must emit the same
bytes and digest for every accepted witness.

The eleven wire-vector labels are exactly: <code>run-identity</code>,
<code>sfs-sample</code>, <code>sfs-trace</code>,
<code>sfs-preregistration</code>, <code>practice-candidate-schedule</code>,
<code>practice-attempt-ledger</code>, <code>component-proof-profile</code>,
<code>proof-profile</code>, <code>causal-cone</code>,
<code>intervention-delta</code>, and
<code>persistence-comparison</code>. Row structs appear only inside their
owning record bytes and never receive an invented envelope.

Fixture rows use ASCII, one row per vector, terminal LF, and these exact schemas:

~~~text
wire|label|domain|envelope_hex|sha256_hex
classifier|label|window_width|comma_separated_f64_bits_hex|class_u8
persistence|label|post_width|comma_separated_f64_bits_hex|class_u8
mutation|base_label|field_name|mutated_envelope_hex|mutated_sha256_hex
~~~

Sort rows by record kind then label; reject duplicate labels; and bound each
collection and the total fixture bytes.

The Python unit test and the Rust record tests exercise the 65,535-row
acceptance and 65,536-row preflight refusal outside the checked-in hex files,
while the fixture writer rejects any single rendered row or combined output beyond
<code>MAX_VECTOR_BYTES</code> before it opens a destination, which keeps the
cross-language oracle reviewable without weakening the record-count boundary.

The unit test also mutates the <code>replace</code> operation from Python's
<code>os</code> module to fail and proves that the
old complete destination remains byte-identical and the staged sibling is
removed. Each fixture publishes as one complete replacement; the command
reports the exact destination if replacement of a later fixture fails.

- [ ] **Step 4: Add the real-descriptor check mutation tooth**

Copy the established one-descriptor pattern, not a <code>Path.stat</code>/<code>read_bytes</code> surrogate:

~~~python
def test_check_rejects_bad_descriptor_metadata_before_read(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    class Fixture:
        def __enter__(self) -> "Fixture":
            return self

        def __exit__(self, *_: object) -> None:
            return None

        def fileno(self) -> int:
            return 41

        def read(self, _: int = -1) -> bytes:
            raise AssertionError("metadata-rejected fixture must not be read")

    class FixturePath:
        def open(self, *_: object, **__: object) -> Fixture:
            return Fixture()

        def stat(self, *_: object, **__: object) -> object:
            raise AssertionError("check must use the opened descriptor")

        def __str__(self) -> str:
            return "sfs-vectors.txt"

    def fake_fstat(descriptor: int) -> SimpleNamespace:
        assert descriptor == 41
        return SimpleNamespace(
            st_mode=stat.S_IFREG,
            st_size=exporter.MAX_VECTOR_BYTES + 1,
        )

    monkeypatch.setattr(os, "fstat", fake_fstat)
    assert not exporter._check(FixturePath(), b"expected")
~~~

Temporarily move <code>fixture.read</code> before <code>os.fstat</code> and
require this test to fail. Restore the correct order before the green run.

- [ ] **Step 5: Verify the Rust-consumer RED phase and generate fixtures**

Write the Rust test first. It reads the three fixtures with
<code>include_str!</code>, decodes each hex field independently, reconstructs
the corresponding Rust record, and asserts exact envelope bytes, complete
SHA-256 digest, and class code. It also asserts that every listed single-field
mutation changes both bytes and digest.

Run:

~~~bash
cd rust
cargo test -p babylon-evidence --test cross_language_vectors --locked
~~~

Expected RED: compilation fails because the three included fixture paths do
not exist. Return to the repository root and generate them only with the
independent Python implementation:

~~~bash
cd ..
uv run python tools/sfs_contract_vectors.py --write
~~~

- [ ] **Step 6: Run the cross-language gates**

~~~bash
uv lock --check
mise run test:q -- tests/unit/tools/test_sfs_contract_vectors.py
uv run python tools/sfs_contract_vectors.py --check
uv run ruff check tools/sfs_contract_vectors.py tests/unit/tools/test_sfs_contract_vectors.py
uv run ruff format --check tools/sfs_contract_vectors.py tests/unit/tools/test_sfs_contract_vectors.py
cd rust
cargo test -p babylon-evidence --test cross_language_vectors --locked
cargo clippy -p babylon-evidence --all-targets --locked -- -D warnings -D clippy::pedantic
cargo fmt --all -- --check
~~~

Expected: all pass. Editing one fixture nibble must make both Python <code>--check</code> and the Rust consumer fail; restore the fixture with <code>--write</code>.

- [ ] **Step 7: Commit independent vectors**

~~~bash
git add pyproject.toml uv.lock tools/sfs_contract_vectors.py tests/unit/tools/test_sfs_contract_vectors.py rust/crates/babylon-evidence/tests/fixtures/sfs_wire_vectors_v1.txt rust/crates/babylon-evidence/tests/fixtures/sfs_classifier_vectors_v1.txt rust/crates/babylon-evidence/tests/fixtures/sfs_identity_mutations_v1.txt rust/crates/babylon-evidence/tests/cross_language_vectors.rs
mise run commit -- "test(evidence): add independent T3 contract vectors"
~~~

---

### Task 8: Add the Scoped BSL Footprint Auditor and Forbidden Corpus

**Files:**

- Create: <code>rust/crates/babylon-bsl/src/sfs_profile.rs</code>
- Modify: <code>rust/crates/babylon-bsl/src/fuel.rs</code>
- Modify: <code>rust/crates/babylon-bsl/src/lib.rs</code>
- Create: <code>rust/crates/babylon-bsl/tests/sfs_profile_contract.rs</code>
- Create: <code>rust/crates/babylon-bsl/tests/fixtures/sfs_profile/allowed/scoped_mechanic.bsl</code>
- Create: <code>rust/crates/babylon-bsl/tests/fixtures/sfs_profile/forbidden/*.bsl</code>
- Create: <code>rust/crates/babylon-bsl/tests/fixtures/sfs_profile/sfs_forbidden_manifest_v1.txt</code>
- Create: <code>rust/crates/babylon-bsl/tests/fixtures/sfs_profile/sfs_audit_source_manifest_v1.txt</code>

**Interfaces:**

- Consumes: <code>canonical_bytes</code>, <code>parse_bindings</code>, <code>effect_footprint</code>, <code>check_rule</code>, <code>CardinalityCeilings</code>, <code>IntrinsicCosts</code>, <code>ClosedVocabulary::owner_of_field</code>, <code>BindSource</code>, <code>SExpr</code>, and existing AST limits.
- Produces: <code>SfsRuleFootprint</code>, <code>SfsRuleAuditResult</code>, <code>SfsAuditPolicy</code>, <code>GovernedComparisonSite</code>, <code>SfsComparisonContext</code>, <code>audit_rule_footprint</code>, and <code>validate_sfs_rule_profile</code>.

- [ ] **Step 1: Write the failing allowed-footprint test**

The allowed fixture uses only declared fields/constants, one permitted query, basic operators, and governed local effects. The test constructs an exact policy:

~~~rust
#[test]
fn allowed_rule_equals_its_complete_opt_in_profile() {
    let rule = fixture_rule("allowed/scoped_mechanic.bsl");
    let vocabulary = fixture_vocabulary();
    let ceilings = fixture_cardinality_ceilings();
    let intrinsic_costs = IntrinsicCosts::default();
    let policy = SfsAuditPolicy::new(
        "synthetic-source/scoped-mechanic",
        sha256_of(&canonical_bytes(&rule).unwrap()),
        31,
        ["synthetic-source/quanta"],
        ["synthetic-link/strength"],
        ["synthetic/minimum-link-strength", "synthetic/transfer-quantum"],
        ["edges"],
        [">"],
        [],
        vec![
            GovernedComparisonSite::from_rule_path(
                &rule,
                &[0, 11, 1, 1],
                SfsComparisonContext::ConservationRefusal,
            )
            .unwrap(),
            GovernedComparisonSite::from_rule_path(
                &rule,
                &[0, 11, 1, 2],
                SfsComparisonContext::EligibilityNoEffect,
            )
            .unwrap(),
        ],
        ["node:synthetic-source/quanta"],
    )
    .unwrap();
    let audit = validate_sfs_rule_profile(
        &rule,
        &vocabulary,
        &ceilings,
        &intrinsic_costs,
        &policy,
    )
    .unwrap();
    let footprint = audit.footprint();
    assert_eq!(footprint.source_digest(), sha256_of(&canonical_bytes(&rule).unwrap()));
    assert_eq!(footprint.computed_bound(), 31);
    assert_eq!(footprint, policy.expected_footprint());
    assert_eq!(audit.declared_fuel(), 128);
    assert_eq!(audit.cardinality_input_digest(), expected_cardinality_digest());
    assert_eq!(audit.intrinsic_cost_input_digest(), expected_empty_intrinsic_digest());
}
~~~

Use this exact allowed rule:

~~~lisp
(rule synthetic-source/scoped-mechanic
  :role mechanic
  :evidence designed
  :material-basis "a test-local source transfers one declared quantum only when its stock and an existing synthetic link permit the transfer"
  :fuel 128
  (bindings
    (binding available :field synthetic-source/quanta)
    (binding quantum :const synthetic/transfer-quantum)
    (binding minimum-link-strength :const synthetic/minimum-link-strength))
  (when
    (and
      (> available quantum)
      (> (fold sum (edges EdgeType/SYNTHETIC_LINK)
           (field-of it synthetic-link/strength)) minimum-link-strength)))
  (effects
    (update-node self synthetic-source/quanta (sub quantum))))
~~~

Build <code>fixture_vocabulary()</code> from a test-local declaration set with
<code>NodeType/SYNTHETIC_SOURCE</code>,
<code>EdgeType/SYNTHETIC_LINK</code>,
<code>synthetic-source/quanta</code> as an extensive integer field, the
implicit <code>synthetic-link/strength</code> edge field, and
<code>synthetic/transfer-quantum</code> and
<code>synthetic/minimum-link-strength</code> as integer constants. These names
exist only in the test fixture. They are not a practice cost, ActionBudget,
Capacity, membership efficacy, or a proposed production vocabulary.

The cardinality table contains exactly
<code>EdgeType/SYNTHETIC_LINK -&gt; 8</code> and no max-member rows; the intrinsic
cost table is empty because this rule calls no intrinsic. The existing bound
checker must return the literal computed bound 31 under those inputs, which is
below the source's declared fuel 128.

The
<code>available &gt; quantum</code> site has context
<code>conservation-refusal</code>. The synthetic-link aggregate site has context
<code>eligibility-no-effect</code>. The first path is exactly
<code>[0, 11, 1, 1]</code> and its site digest is
<code>d2529413ba20351ab91c63ff45a06930d7b9de1327cddfa67e6f7a74d85d2896</code>.
The second path is exactly <code>[0, 11, 1, 2]</code> and its site digest is
<code>1989aab489fa9dddc87801dec2389a4f360c20909607209d05c34d70ec8f82e9</code>.
The complete canonical-AST SHA-256 is
<code>50a9d50cf862a846004e68c314d33e9ead66dcce9f29bc2cf49fe7aeb3d7cd45</code>.

Construct the governed sites only from these literal paths and assert all
three literal hashes without discovering permission sites by scanning the
rule, because the fixture and policy keep every expected set visible in this
one test.

Change the declared fuel from 128 to 129 while keeping the computed bound 31
and require the sealed audit result to move. Replace the cardinality row with
a different table that happens to produce the same bound and require the
cardinality input digest to move. Add one unused intrinsic-cost row and require
the intrinsic input digest to move even though the rule still computes bound
31. Task 9 must reject each changed audit result against the original bound
row rather than accepting equal computed bounds.

- [ ] **Step 2: Write the failing forbidden-corpus table test**

Create exactly these fixture categories and expected error variants:

| Fixture | Required rejection |
|---|---|
| <code>tick_read.bsl</code> | <code>ForbiddenBindingSource::Tick</code> |
| <code>year_read.bsl</code> | <code>ForbiddenBindingSource::Year</code> |
| <code>tick_of_year_read.bsl</code> | <code>ForbiddenBindingSource::TickOfYear</code> |
| <code>tick_cycle_read.bsl</code> | <code>ForbiddenBindingSource::TickInCycle</code> |
| <code>rng_read.bsl</code> | <code>ForbiddenIntrinsic("rng-draw")</code> |
| <code>named_shape.bsl</code> | <code>ForbiddenIntrinsic("sigmoid")</code> |
| <code>exp_response.bsl</code> | <code>ForbiddenIntrinsic("exp")</code> |
| <code>log_response.bsl</code> | <code>ForbiddenIntrinsic("log")</code> |
| <code>absolute_schedule.bsl</code> | <code>ForbiddenAbsoluteSchedule</code> |
| <code>response_table.bsl</code> | <code>ForbiddenResponseTable</code> |
| <code>threshold_ladder.bsl</code> | <code>ForbiddenThresholdLadder</code> |
| <code>direct_observable_read.bsl</code> | <code>ForbiddenObservable</code> |
| <code>direct_observable_write.bsl</code> | <code>ForbiddenObservable</code> |
| <code>comparison_without_context.bsl</code> | <code>MissingComparisonContext</code> |
| <code>comparison_selects_magnitude.bsl</code> | <code>ForbiddenComparisonUse</code> |
| <code>dead_comparison_permission.bsl</code> | <code>DeadComparisonContext</code> |
| <code>dead_effect_permission.bsl</code> | <code>FootprintMismatch</code> |
| <code>unauthorized_effect.bsl</code> | <code>UnexpectedEffect("node:organization/cohesion")</code> |

The four forbidden-intrinsic fixtures must reach the semantic refusal after
the mandatory static-bound check. Construct their test-local
<code>IntrinsicCosts</code> from exactly these complete sorted rows:

~~~text
intrinsic|exp|7
intrinsic|log|7
intrinsic|rng-draw|12
intrinsic|sigmoid|40
~~~

The table identity is
<code>1efb2cc310a127ffa35220cada6987bcf9aeff21c9c251699b16ee6344d2761c</code>.
The numeric costs are designed test inputs, not production declarations. All
other forbidden fixtures use the empty intrinsic table. Pin which table each
manifest row uses, run <code>check_rule</code> successfully, and only then
require the named <code>ForbiddenIntrinsic</code> error. A missing cost row must
fail earlier with the existing typed undeclared-intrinsic bound error.

The manifest rows are sorted:

~~~text
label|relative_path|canonical_ast_sha256_hex|intrinsic_cost_table|expected_error_variant
~~~

<code>intrinsic_cost_table</code> is exactly <code>empty</code> or
<code>forbidden-v1</code>; only the four intrinsic rows use
<code>forbidden-v1</code>. The manifest is UTF-8 with LF separators, no CR or blank line, exactly one
terminal LF, and rows sorted by the complete row bytes. Its content identity is
exactly:

~~~text
SHA256(
  ASCII("babylon.sfs-forbidden-corpus-manifest.v1") || 0x00
  || exact_manifest_bytes
)
~~~

The test reads at most 32 fixture rows, recomputes every canonical AST digest,
recomputes and pins the literal manifest digest, rejects a missing or extra
fixture, and requires the exact error variant.

- [ ] **Step 3: Run the BSL contract test and verify the red phase**

~~~bash
cd rust
cargo test -p babylon-bsl --test sfs_profile_contract --locked
~~~

Expected: unresolved <code>sfs_profile</code> imports.

- [ ] **Step 4: Implement bounded footprint extraction**

Expose:

~~~rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SfsRuleFootprint {
    rule_id: String,
    source_digest: [u8; 32],
    computed_bound: u64,
    field_reads: BTreeSet<String>,
    edge_reads: BTreeSet<String>,
    constant_reads: BTreeSet<String>,
    queries: BTreeSet<String>,
    operators: BTreeSet<String>,
    intrinsics: BTreeSet<String>,
    comparison_clamp_contexts: BTreeSet<String>,
    effects: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SfsRuleAuditResult {
    footprint: SfsRuleFootprint,
    declared_fuel: u64,
    cardinality_input_digest: [u8; 32],
    intrinsic_cost_input_digest: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SfsFuelIdentityError {
    RowLimit { table: &'static str, actual: usize },
    DuplicateRow { table: &'static str, row: String },
    KeyEmpty { table: &'static str },
    KeyTooLong { table: &'static str, actual: usize },
    KeyNonAscii { table: &'static str },
    KeyContainsDelimiter { table: &'static str },
}

impl CardinalityCeilings {
    pub fn sfs_identity_digest(&self) -> Result<[u8; 32], SfsFuelIdentityError>;
}

impl IntrinsicCosts {
    pub fn sfs_identity_digest(&self) -> Result<[u8; 32], SfsFuelIdentityError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SfsComparisonContext {
    InputValidity,
    EligibilityNoEffect,
    ConservationRefusal,
    MaterialRouting,
    DomainCeiling,
}

pub struct GovernedComparisonSite {
    site_digest: [u8; 32],
    context: SfsComparisonContext,
}

pub struct SfsAuditPolicy {
    rule_id: String,
    expected: SfsRuleFootprint,
    comparison_sites: Vec<GovernedComparisonSite>,
}

impl SfsComparisonContext {
    pub const fn code(self) -> &'static str;
}

impl GovernedComparisonSite {
    pub fn from_rule_path(
        rule: &SExpr,
        path: &[u32],
        context: SfsComparisonContext,
    ) -> Result<Self, SfsProfileError>;

    pub const fn site_digest(&self) -> &[u8; 32];
    pub fn profile_entry(&self) -> String;
}

impl SfsAuditPolicy {
    pub fn new(
        rule_id: &'static str,
        expected_source_digest: [u8; 32],
        expected_computed_bound: u64,
        field_reads: impl IntoIterator<Item = &'static str>,
        edge_reads: impl IntoIterator<Item = &'static str>,
        constant_reads: impl IntoIterator<Item = &'static str>,
        queries: impl IntoIterator<Item = &'static str>,
        operators: impl IntoIterator<Item = &'static str>,
        intrinsics: impl IntoIterator<Item = &'static str>,
        comparison_sites: Vec<GovernedComparisonSite>,
        effects: impl IntoIterator<Item = &'static str>,
    ) -> Result<Self, SfsProfileError>;

    pub const fn expected_footprint(&self) -> &SfsRuleFootprint;
}

pub fn audit_rule_footprint(
    rule: &SExpr,
    vocabulary: &ClosedVocabulary,
    ceilings: &CardinalityCeilings,
    intrinsic_costs: &IntrinsicCosts,
    comparison_sites: &[GovernedComparisonSite],
) -> Result<SfsRuleAuditResult, SfsProfileError>;

pub fn validate_sfs_rule_profile(
    rule: &SExpr,
    vocabulary: &ClosedVocabulary,
    ceilings: &CardinalityCeilings,
    intrinsic_costs: &IntrinsicCosts,
    policy: &SfsAuditPolicy,
) -> Result<SfsRuleAuditResult, SfsProfileError>;
~~~

Use these explicit failure types:

~~~rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForbiddenBindingSource {
    Tick,
    Year,
    TickOfYear,
    TickInCycle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SfsProfileError {
    AstWalkLimit,
    Bound(BoundError),
    FuelIdentity(SfsFuelIdentityError),
    ComputedBoundMismatch { expected: u64, actual: u64 },
    CanonicalAst,
    UnknownFieldOwner { field: String },
    ForbiddenBindingSource(ForbiddenBindingSource),
    ForbiddenIntrinsic { name: String },
    ForbiddenAbsoluteSchedule,
    ForbiddenResponseTable,
    ForbiddenThresholdLadder,
    ForbiddenObservable { entry: String },
    UnexpectedRead { entry: String },
    UnexpectedEffect { entry: String },
    MissingComparisonContext { site_digest: [u8; 32] },
    DeadComparisonContext { site_digest: [u8; 32] },
    ForbiddenComparisonUse { site_digest: [u8; 32] },
    FootprintMismatch { set: &'static str },
    SourceDigestMismatch,
    DuplicatePolicyEntry { set: &'static str },
    PolicyEntryLimit { set: &'static str, actual: usize },
    EmptyFormPath,
    FormPathLimit { actual: usize },
    UnknownFormPath,
    NotComparisonOrClamp,
}
~~~

Expose read-only getters for every <code>SfsRuleFootprint</code> set plus
<code>rule_id()</code>, <code>source_digest()</code>, and
<code>computed_bound()</code>. <code>SfsRuleAuditResult</code> exposes only
<code>footprint()</code>, <code>declared_fuel()</code>,
<code>cardinality_input_digest()</code>, and
<code>intrinsic_cost_input_digest()</code>. Task 9 uses those getters to
construct the synthetic wire profile without making
<code>babylon-bsl</code> depend on <code>babylon-evidence</code>.

Semantic rules:

1. Add the two bounded canonical digest methods to <code>fuel.rs</code>, where
   the complete private tables are available. A cardinality identity contains
   the combined ceiling and max-member rows; an intrinsic identity contains
   every cost row, including one unused by the audited rule. Each combined
   table admits at most 64 rows and rejects row 65. Keys are 1 through 96
   strict ASCII bytes and reject <code>|</code>, LF, or CR. Values use shortest
   unsigned decimal with no leading zero.

   Cardinality rows are exactly
   <code>ceiling|{enum-ref}|{value}\n</code> or
   <code>max-members|{enum-ref}|{value}\n</code>; intrinsic rows are exactly
   <code>intrinsic|{name}|{cost}\n</code>. Sort complete row bytes and reject a
   duplicate before joining.

   Hash the cardinality bytes after
   <code>ASCII("babylon.sfs-cardinality-ceilings.v1") || 0x00</code> and the
   intrinsic bytes after
   <code>ASCII("babylon.sfs-intrinsic-costs.v1") || 0x00</code>. Pin these
   independent literals:
   <code>58ef2f65a4137c5dfadd41855f6b40282fdcbbd4339cfd1e32047776b56c6474</code>
   for <code>ceiling|EdgeType/SYNTHETIC_LINK|8\n</code>,
   <code>9689f3b7c6cfee41597117f3c97c505f3f6c9406c48d98c6f377badff40140cc</code>
   when <code>max-members|HyperedgeType/SYNTHETIC_GROUP|4\n</code> is added,
   <code>2d35fcf8f676dfa9869eb8e18920d97d0d227cbce50a862eddc29e8f7dc3c6a1</code>
   for the empty intrinsic table, and
   <code>aac6683a351c4f06dcc284d7d2330f6b233b5bc090b9596e5be93a177725e550</code>
   for <code>intrinsic|synthetic-unused|7\n</code>. Tests cover empty, 64,
   65, private encoder duplicate-row input with the exact
   <code>DuplicateRow</code> error, delimiter, non-ASCII, overlong key, and a
   table change that leaves the rule's computed bound unchanged.

2. Extract the rule's declared fuel from the checked AST. Call the existing AST-bound validator and <code>check_rule</code> with the
   supplied <code>CardinalityCeilings</code> and <code>IntrinsicCosts</code>
   before every semantic walk. Seal both input digests, the declared fuel, and
   the returned computed bound into one <code>SfsRuleAuditResult</code>; no
   constructor accepts those values separately. Require
   exact equality with the policy's expected bound; never substitute runtime
   graph cardinality.
3. Build source identity from <code>sha256_of(canonical_bytes(rule))</code>.
4. Classify <code>:field</code> reads with <code>owner_of_field</code>; EdgeType fields enter <code>edge_reads</code>, other declared field owners enter <code>field_reads</code>.
5. Hard-refuse <code>sfs/aggregate</code>, <code>sfs/classification</code>,
   <code>sfs/wave-stage</code>, <code>sfs/hinterland-class</code>, and
   <code>sfs/political-subjectivity</code> in an actual or expected read/effect
   entry. An allowlist row cannot authorize one of these names.
6. Record the exact <code>:const</code> qnames.
7. Record these exact six existing query heads by head string:
   <code>nodes</code>, <code>edges</code>, <code>neighbors</code>,
   <code>hyperedges</code>, <code>members-of</code>, and
   <code>hyperedges-of</code>.
8. Record the arithmetic and comparison operator symbols by exact string.
9. Record the intrinsic call heads by exact declared name.
10. Reject Tick, Year, TickOfYear, and TickInCycle binding sources. When one of
   those values participates in a comparison that guards an effect, report
   <code>ForbiddenAbsoluteSchedule</code>; otherwise report the exact
   <code>ForbiddenBindingSource</code>.
11. Reject <code>rng-draw</code>, <code>sigmoid</code>, <code>exp</code>, and
    <code>log</code> in an opted-in component. The named-shape fixture uses
    <code>sigmoid</code> as a forbidden call head only; this audit does not add
    it to the global BSL intrinsic registry.
12. Treat the rule root as <code>FormPath [0]</code> and append zero-based child
    indices. A path contains 1 through
    <code>MAX_AST_WALK_DEPTH + 1</code> <code>u32</code> components. Compute each
    comparison/clamp site digest exactly as:

    ~~~text
    SHA256(
      ASCII("babylon.sfs-comparison-site.v1") || 0x00
      || SHA256(canonical_bytes(rule))
      || path_component_count_u16_be
      || repeated path_component_u32_be
    )
    ~~~

    The five context codes are exactly <code>input-validity</code>,
    <code>eligibility-no-effect</code>, <code>conservation-refusal</code>,
    <code>material-routing</code>, and <code>domain-ceiling</code>. A
    <code>comparison_clamp_context_set</code> entry is
    <code>context-code:lowercase-site-digest-hex</code>. Require one exact
    governed context row per site and no unused row.
13. Reject a comparison that selects among multiple fixed effect magnitudes by inspecting both branches beneath that site. A response table is two or more sibling branches that select distinct numeric literals for one effect target. A threshold ladder is two or more nested comparisons of the same binding against distinct numeric literals. These definitions bound the named corpus and do not claim semantic completeness.
14. Convert <code>effect_footprint</code> rows to exactly
    <code>node:&lt;qualified-field&gt;</code>,
    <code>edge:&lt;qualified-field&gt;</code>,
    <code>hyperedge:&lt;qualified-field&gt;</code>,
    <code>event:&lt;EventType/MEMBER&gt;</code>, or
    <code>shape:&lt;BSL-shape-head&gt;</code>. The six shape heads are
    <code>add-node</code>, <code>remove-node</code>, <code>add-edge</code>,
    <code>remove-edge</code>, <code>add-hyperedge</code>, and
    <code>remove-hyperedge</code>.
15. Compare every actual set with the expected set for exact equality. Missing and dead permissions fail symmetrically.

Return the first failure in this exact order: AST/canonical/path bounds;
fuel-table identity syntax/count; static fuel/cardinality bound;
computed-bound mismatch;
forbidden binding source or absolute schedule; forbidden observable; forbidden
intrinsic; response table; threshold ladder; forbidden comparison use; source
digest mismatch; missing comparison context; dead comparison context;
unexpected read; unexpected effect; then exact-set mismatch in the Task 6 set
order. Within one category, choose the byte-least entry or site digest. The
forbidden-corpus manifest pins this precedence so an executor cannot make a
mutant pass by reporting a weaker later mismatch.

The Task 9 proof profile binds two additional exact content identities:

~~~text
audit_source_digest = SHA256(
  ASCII("babylon.sfs-audit-source-manifest.v1") || 0x00
  || exact UTF-8 LF bytes of sfs_audit_source_manifest_v1.txt
)

forbidden_corpus_digest = SHA256(
  ASCII("babylon.sfs-forbidden-corpus-manifest.v1") || 0x00
  || exact UTF-8 LF bytes of sfs_forbidden_manifest_v1.txt
)
~~~

The audit-source manifest has exactly these two sorted rows and one terminal
LF:

~~~text
fuel.rs|sha256_of_exact_file_bytes
sfs_profile.rs|sha256_of_exact_file_bytes
~~~

It admits exactly two rows, resolves both paths only under
<code>rust/crates/babylon-bsl/src</code>, refuses a symlink or path escape,
preflights each source at 262,144 bytes, reads one descriptor per file, and
requires the declared digest to equal the exact bytes while the audit test pins
the complete lowercase manifest digest. Mutating either semantic source file while
leaving the manifest unchanged fails file verification; recomputing that row
moves <code>audit_source_digest</code> and invalidates the original Task 9 run
identity.

Both manifests and both source files must contain no CR or UTF-8 BOM and exactly
one terminal LF, and the tests pin both content-identity literals. These digests identify this
scoped synthetic audit; they do not attest to a complete production host
manifest.

The module does not modify global <code>DECLARABLE_INTRINSICS</code>, the ordinary loader, or <code>GOVERNED_EFFECT_ALLOWANCES</code>.

- [ ] **Step 5: Add semantic-identity mutation teeth**

Add three paired tests with these exact names:

1. <code>formatting_only_preserves_identity</code>: a formatting-only rewrite
   produces the same canonical AST digest and footprint.
2. <code>semantic_change_moves_identity</code>: a one-operator semantic change
   produces a different canonical AST digest and either a different exact
   footprint or a profile mismatch.
3. <code>role_relabel_moves_identity_and_fails_profile</code>: change only the
   allowed rule's <code>:role mechanic</code> to <code>:role intent</code>, keep
   the original policy source digest, require a different canonical AST
   digest, and require <code>SourceDigestMismatch</code>. Restore
   <code>:role mechanic</code> and require the original digest and valid
   profile. The test gives intent-to-mechanic relabeling its own mutation tooth.

Add these remaining mutation teeth:

- Alter one manifest digest in memory and require
  <code>SourceDigestMismatch</code>.
- Lower the rule's declared fuel below 31 and require the existing typed
  <code>BoundError</code>.
- Remove the <code>EdgeType/SYNTHETIC_LINK</code> ceiling and require the existing
  missing-ceiling error.
- Change the ceiling from 8 to 9 without changing the policy and require
  <code>ComputedBoundMismatch</code>.

Together, these tests kill identity and underdeclared-fuel mutants and prove
that the scoped audit uses fixed declared fuel and cardinality rather than the
runtime graph.

- [ ] **Step 6: Run BSL tests, sentinel, and Clippy**

~~~bash
cd rust
cargo test -p babylon-bsl --test sfs_profile_contract --locked
cargo test -p babylon-bsl --locked sfs_profile
cargo run -p bsl-lint --locked -- all
cargo clippy -p babylon-bsl --all-targets --locked -- -D warnings -D clippy::pedantic
cargo fmt --all -- --check
~~~

- [ ] **Step 7: Commit the scoped auditor**

~~~bash
git add rust/crates/babylon-bsl/src/fuel.rs rust/crates/babylon-bsl/src/sfs_profile.rs rust/crates/babylon-bsl/src/lib.rs rust/crates/babylon-bsl/tests/sfs_profile_contract.rs rust/crates/babylon-bsl/tests/fixtures/sfs_profile
mise run commit -- "feat(bsl): audit scoped emergence footprints"
~~~

---

### Task 9: Add the Synthetic Proof-Profile, Cone, Identity, and Driver Harness

**Files:**

- Modify: <code>tools/sfs_contract_vectors.py</code>
- Modify: <code>tests/unit/tools/test_sfs_contract_vectors.py</code>
- Create: <code>rust/crates/babylon-evidence/src/validation.rs</code>
- Create: <code>rust/crates/babylon-evidence/src/driver.rs</code>
- Create: <code>rust/crates/babylon-evidence/src/driver_contract.rs</code>
- Modify: <code>rust/crates/babylon-evidence/src/lib.rs</code>
- Create: <code>rust/crates/babylon-evidence/tests/synthetic_driver_contract.rs</code>
- Create: <code>rust/crates/babylon-evidence/tests/synthetic_proof_harness.rs</code>
- Create: <code>rust/crates/babylon-evidence/tests/fixtures/sfs_synthetic_governed_manifest_v1.txt</code>
- Create: <code>rust/crates/babylon-evidence/tests/fixtures/sfs_synthetic_profile_v1.txt</code>
- Create: <code>rust/crates/babylon-evidence/tests/fixtures/sfs_synthetic_driver_contract_v1.txt</code>
- Create: <code>rust/crates/babylon-evidence/tests/fixtures/sfs_synthetic_driver_v1.txt</code>
- Create: <code>rust/crates/babylon-evidence/tests/fixtures/sfs_mutation_manifest_v1.txt</code>

**Interfaces:**

- Consumes: all earlier evidence records, sealed
  <code>PracticeIntentV1</code> values, the independent Python encoder,
  <code>babylon_bsl::SfsRuleAuditResult</code>, and checked-in synthetic
  component/channel rows.
- Produces: bounded exact cone validation, full synthetic flat-cadence
  equality, a source-bound synthetic driver handle, twin-identity comparison,
  driver-shape refusal, time-shift equality, and mutation-manifest
  completeness.

- [ ] **Step 1: Write the failing exact-cone tests**

Define a three-component synthetic chain:

~~~text
scoped-bsl-rule -> membership-reducer -> post-commit-producer
~~~

The root is <code>scoped-bsl-rule</code>; the sink is
<code>post-commit-producer</code>. This synthetic cone starts at the governed
mechanic because no live practice dispatcher exists in this train; it makes no
claim that a candidate ledger invokes the fixture. Test:

- the exact three-component cone passes;
- a missing middle component fails;
- an extra component fails;
- an unreachable component fails;
- an unprofiled reachable component fails;
- a profile component absent from the cone fails;
- change one non-BSL host component's profile entry. Keep its component ID,
  kind, and source digest; validation fails with
  <code>ConeProfileMismatch</code>, even when the changed proof-profile and run
  identity digests are recomputed consistently;
- a missing root-to-sink path returns <code>NoRootToSinkPath</code>;
- changing one typed channel without changing component reachability leaves
  the <code>CausalConeV1</code> bytes unchanged, changes the governed manifest
  and proof-profile digests, and fails validation against both the original
  manifest and a fully recomputed manifest/profile/run bundle because the
  changed edge no longer matches the producer effect and consumer read. This
  pins the distinct ID-set and typed-edge identity boundaries.
- after parsing the original manifest, mutate one edge channel in a separate
  raw byte buffer. Retain an externally cached original digest. The
  changed bytes must parse to a different private manifest identity and fail
  against the original proof profile. No validator accepts the cached digest
  or a caller-supplied edge slice, so a post-hash edge substitution cannot
  reach cone validation.

Drive the test from
<code>sfs_synthetic_governed_manifest_v1.txt</code>, not from a duplicate Rust
array, and create the complete governed-manifest and synthetic-profile fixtures in
this RED step; do not create an empty or partial fixture. The manifest has
exactly these row schemas:

~~~text
component|component_id_nfc_utf8_hex|component_kind_u8|source_mode|source_payload_hex|source_digest_hex
profile|component_id_nfc_utf8_hex|set_name|entry_nfc_utf8_hex
bound|component_id_nfc_utf8_hex|declared_fuel_u64|computed_bound_u64|cardinality_digest_hex|intrinsic_cost_digest_hex
edge|producer_id_nfc_utf8_hex|consumer_id_nfc_utf8_hex|channel_kind_u8|channel_id_nfc_utf8_hex
~~~

The three component rows sort by component ID and are exactly
<code>scoped-bsl-rule</code> kind 0,
<code>membership-reducer</code> kind 2, and
<code>post-commit-producer</code> kind 3. Every variable NFC text field uses
lowercase, even-length UTF-8 hex, which the parser decodes and validates
without normalization, while the closed <code>source_mode</code> and
<code>set_name</code> fields remain strict ASCII tokens. Sort the complete
file by each full row's ASCII bytes and reject any noncanonical hex,
decoded non-NFC value, delimiter ambiguity, duplicate decoded value, or row
order mismatch. Pin one adversarial mixed-kind fixture whose rows are each
valid but whose <code>profile</code> row precedes a byte-lexicographically
earlier <code>edge</code> row; canonical parsing must refuse until complete-row
sorting restores the edge row first, with the two exact edge rows below:

~~~text
edge|73636f7065642d62736c2d72756c65|6d656d626572736869702d72656475636572|0|73796e7468657469632d736f757263652f7175616e7461
edge|6d656d626572736869702d72656475636572|706f73742d636f6d6d69742d70726f6475636572|5|73796e7468657469632f6d656d626572736869702d726564756365722d6f7574707574
~~~

#### Governed component source binding

- For <code>scoped-bsl-rule</code>, <code>source_mode</code> is
  <code>canonical-bsl</code>, <code>source_payload_hex</code> is the exact output
  of <code>canonical_bytes</code> for Task 8's allowed fixture, and
  <code>source_digest_hex</code> is SHA-256 over those decoded bytes. The parser
  requires exactly one <code>canonical-bsl</code> component. Its decoded payload
  must equal the canonical bytes represented by the sealed
  <code>SfsRuleAuditResult</code>, and its source digest must equal the payload
  SHA-256 and the sealed audit footprint's source digest.

- Substitute another valid canonical rule and recompute the component,
  manifest, profile, preregistration, and run identities. The validator must
  still return <code>ComponentSourceDigestMismatch</code> against the original
  sealed audit.

- The other two rows use <code>source_mode=synthetic-descriptor</code> and the
  exact values below. The first line is the hash-domain hex, and the remaining
  lines are source payloads before hex encoding.

~~~text
SFS_SYNTHETIC_COMPONENT_SOURCE_DOMAIN_HEX = 626162796c6f6e2e7366732d73796e7468657469632d636f6d706f6e656e742d736f757263652e7631
membership-reducer maps one synthetic field value to one reducer output
post-commit-producer emits one synthetic sample after a sealed envelope
~~~

- Their source digest is exactly
  <code>SHA256(hex_decode(SFS_SYNTHETIC_COMPONENT_SOURCE_DOMAIN_HEX) || 0x00 ||
  decoded_source_payload)</code>. Profile rows contain each component's complete
  non-empty sets and only Task 6 set names. A component and set pair with no
  entries has no row. The parser rejects an unknown set name, duplicate entry,
  undeclared component, missing or extra component row, source-digest mismatch,
  or edge endpoint outside the component set.

Use these exact profile rows, which include the two Task 8
<code>comparison_clamp_contexts</code> entries from the frozen
<code>FormPath</code> contract and already appear in complete-row byte order:

~~~text
profile|6d656d626572736869702d72656475636572|effects|726564756365722d6f75747075743a73796e7468657469632f6d656d626572736869702d726564756365722d6f7574707574
profile|6d656d626572736869702d72656475636572|field_reads|73796e7468657469632d736f757263652f7175616e7461
profile|706f73742d636f6d6d69742d70726f6475636572|effects|726563656970743a73796e7468657469632f7366732d73616d706c65
profile|706f73742d636f6d6d69742d70726f6475636572|field_reads|726564756365722d6f75747075743a73796e7468657469632f6d656d626572736869702d726564756365722d6f7574707574
profile|73636f7065642d62736c2d72756c65|comparison_clamp_contexts|636f6e736572766174696f6e2d7265667573616c3a64323532393431336261323033353161623931633633666634356130363933306437623964653133323763646466613637653666376137346438356432383936
profile|73636f7065642d62736c2d72756c65|comparison_clamp_contexts|656c69676962696c6974792d6e6f2d6566666563743a31393839616162343839666139646464633837383031646563323338396134663336306332303930393630373230396430356333346437306563386638326539
profile|73636f7065642d62736c2d72756c65|constant_reads|73796e7468657469632f6d696e696d756d2d6c696e6b2d737472656e677468
profile|73636f7065642d62736c2d72756c65|constant_reads|73796e7468657469632f7472616e736665722d7175616e74756d
profile|73636f7065642d62736c2d72756c65|edge_reads|73796e7468657469632d6c696e6b2f737472656e677468
profile|73636f7065642d62736c2d72756c65|effects|6e6f64653a73796e7468657469632d736f757263652f7175616e7461
profile|73636f7065642d62736c2d72756c65|field_reads|73796e7468657469632d736f757263652f7175616e7461
profile|73636f7065642d62736c2d72756c65|operators|3e
profile|73636f7065642d62736c2d72756c65|queries|6564676573
~~~

For <code>scoped-bsl-rule</code>, require byte equality between these rows and
the corresponding Task 8 footprint getters, and use only one bound row that
names <code>scoped-bsl-rule</code>, declared fuel 128, and
computed bound 31, with this cardinality digest:

~~~text
SHA256(
  SFS_CARDINALITY_CEILINGS_DOMAIN || 0x00
  || ASCII("ceiling|EdgeType/SYNTHETIC_LINK|8\n")
)
~~~

Its intrinsic-cost digest is the exact empty table below for a fixture with no
intrinsic calls:

~~~text
SHA256(SFS_INTRINSIC_COSTS_DOMAIN || 0x00)
~~~

A bound row on a host
component, a missing BSL bound row, a second row, or a mismatch with
the sealed <code>SfsRuleAuditResult</code>'s declared fuel, computed bound,
cardinality input digest, or intrinsic-cost input digest fails, and these
synthetic table digests bind the fixed audit inputs without serving as live
<code>RunIdentityV1</code> completeness evidence.

#### Synthetic manifest identities

The manifest uses UTF-8, LF only, no blank line, and exactly one terminal LF,
with this exact governed-manifest identity:

~~~text
SHA256(
  hex_decode("626162796c6f6e2e7366732d73796e7468657469632d676f7665726e65642d6d616e69666573742e7631") || 0x00
  || exact_manifest_bytes
)
~~~

Its synthetic host-component identity uses only the three complete
<code>component|</code> rows, sorted by full row bytes and joined with one LF
after every row:

~~~text
SHA256(
  hex_decode("626162796c6f6e2e7366732d73796e7468657469632d686f73742d636f6d706f6e656e742d6d616e69666573742e7631") || 0x00
  || exact_component_rows_with_terminal_lf
)
~~~

#### Synthetic identity placement

Pin the lowercase literal digest. Put that digest in
<code>SfsProofProfileV1.governed_manifest_digest</code> and in the synthetic
<code>RunIdentityV1.governed_footprint_manifest_digest</code>. Put the separate
host-component digest in the synthetic
<code>RunIdentityV1.host_component_manifest_digest</code>. This synthetic
descriptor is not a production host-component implementation identity and
cannot be carried into live proof.

Extend Task 7's independent Python tool with:

~~~python
def _synthetic_profile_vectors(
    governed_manifest_path: Path,
    forbidden_manifest_path: Path,
    audit_source_manifest_path: Path,
) -> list[str]:
    """Return the three component, one cone, and one proof-profile rows."""
~~~

The function reads each source with the same one-descriptor size/type checks,
enforces the exact LF and row schemas above, validates every declared source
digest, resolves and verifies both audit-source manifest rows, computes the
four domain-separated content digests, and encodes the six
records from the frozen Task 6 field order. It uses no Rust output. Add mutually
exclusive CLI flags <code>--write-synthetic-profile</code> and
<code>--check-synthetic-profile</code>; both target only
<code>sfs_synthetic_profile_v1.txt</code> through Task 7's atomic writer or
descriptor checker. The Python test pins one complete component envelope, the
cone envelope, the proof-profile SHA-256 literal, CR rejection, a changed bound
row, and a changed edge row.

Write those Python assertions before the function and run:

~~~bash
mise run test:q -- tests/unit/tools/test_sfs_contract_vectors.py
~~~

Expected RED: the new synthetic-profile function is absent. Implement only the
function and two CLI routes, rerun the Python test to green, then continue with
the Rust cone tests in this step.

After the governed manifest exists, generate the RED fixture independently:

~~~bash
uv run python tools/sfs_contract_vectors.py --write-synthetic-profile
~~~

The synthetic driver contract is a second sealed source artifact. Its UTF-8
manifest is at most 4,096 bytes, uses LF only, has exactly seven rows and one
terminal LF, and has this fixed row order:

~~~text
schema|1
predicate|candidate-projection|1
predicate|cumulative-driver-shape|1
predicate|persistence-comparison-identity|1
predicate|aligned-material-sequence|1
predicate|twin-identity-difference|1
source|driver.rs|driver_source_digest_hex
~~~

The source digest is
<code>SHA256(ASCII("babylon.sfs-driver-source.v1") || 0x00 || exact
driver.rs bytes)</code>. The complete contract digest is
<code>SHA256(ASCII("babylon.sfs-synthetic-driver-contract.v1") || 0x00 ||
exact manifest bytes)</code>. <code>driver_contract.rs</code> uses
<code>include_bytes!("driver.rs")</code>, recomputes the source digest, and
accepts no caller-supplied source bytes or digest. Its parser preflights the
byte count before splitting, walks exactly <code>for index in 0..7</code>, and
refuses a missing, extra, reordered, renamed, or version-changed row.

Extend the independent Python tool with
<code>_synthetic_driver_vectors(driver_source_path: Path) -&gt; tuple[bytes,
list[str]]</code> and mutually exclusive
<code>--write-synthetic-driver</code>/<code>--check-synthetic-driver</code>
routes. The writer stages and validates both complete outputs before it
publishes each with an atomic sibling replace; <code>--check</code> refuses a
cross-output mismatch after any interrupted publication.

<code>sfs_synthetic_driver_v1.txt</code> is at most 262,144 bytes and has
exactly two rows under the schema
<code>label|domain_ascii|preimage_hex|sha256_hex</code>, sorted by complete row
bytes, LF only, no blank line, and one terminal LF. The labels are exactly
<code>driver-contract</code> and <code>driver-source</code>. The source preimage
is the complete <code>driver.rs</code> bytes and uses domain
<code>babylon.sfs-driver-source.v1</code>; the contract preimage is the complete
seven-row manifest and uses domain
<code>babylon.sfs-synthetic-driver-contract.v1</code>. Each digest hashes
<code>ASCII(domain) || 0x00 || decoded_preimage_hex</code>.

The two rows
pin the manifest bytes, source digest, and complete contract digest. Rust
independently recomputes all three. A source-byte mutation, a
predicate-version mutation, and a preregistration digest mutation each have a
specific failing test.

- [ ] **Step 2: Write the failing identity and candidate tests**

For one asymmetric fixture bundle:

1. project the final attempt ledger and require byte equality with the preregistered candidate schedule;
2. mutate each <code>RunIdentityV1</code> field in turn and require a different envelope and digest;
3. require synthetic control/intervention identities to differ in exactly one selected ledger digest;
4. reject a twin that changes both ledgers;
5. reject a twin that changes a non-ledger field;
6. require the distinct host and governed manifest digests, proof-profile
   digest, and preregistration digest to match every synthetic run/profile
   identity field named above; mutate each one and require its specific error;
7. with the run-identity digest fixed, mutate the actual attempt-ledger bytes
   and require <code>AttemptLedgerDigestMismatch</code>;
8. mutate the actual exogenous digest independently against the run identity
   and preregistration and require <code>ExogenousLedgerDigestMismatch</code> in
   both directions;
9. create an irregular schedule, recompute and preregister its exact schedule
   digest, and require <code>CandidateCadenceCountMismatch</code> for a row-count
   mismatch and <code>CandidateCadenceTickMismatch</code> for a tick other than
   <code>first_attempt_tick + index * attempt_stride</code>;
10. preregister <code>first_attempt_tick = u64::MAX - 1</code>, stride 2, and
    count 2 with a matching two-row schedule digest. Require
    <code>CandidateCadenceOverflow { index: 1 }</code> before tick comparison;
11. build one valid persistence comparison from the two run identities, two
    traces, and one intervention delta. Mutate each trace's run-identity
    binding, each stored trace digest, the selected ledger kind, each stored
    differing-ledger digest, the delta kind, and the stored delta digest one at
    a time; require the exact comparison-identity error for every mutation;
12. supply one exact <code>PracticeIntentV1</code> per candidate row and mutate
    its practice id, target node, or quoted governed cost. For the tick witness,
    move <code>submit_after_tick</code> and <code>resolve_tick</code> together so
    <code>resolve_tick == submit_after_tick + 1</code> remains valid while the
    resolve tick differs from the candidate attempt tick. Recompute the intent
    and schedule digests, and require the specific
    flat-cadence realization error against the unchanged preregistration
    field. Because V1 practice allowlists admit only empty parameter bytes, test
    parameter identity by mutating only the preregistration's
    <code>parameter_bytes_digest</code> against the unchanged valid intent and
    require <code>CandidateParameterBytesMismatch</code> without using an invalid
    nonempty intent as a candidate witness. A separate adapter-only refusal test
    supplies one structurally invalid parameter sequence and unsorted evidence,
    then requires the exact parameter and complete-intent error mappings.
    Neither malformed value enters a
    synthetic run. A missing, extra, reordered, or digest-mismatched intent also
    refuses;
13. parse the exact synthetic driver manifest, bind its complete digest to the
    preregistration, and obtain the opaque validated driver handle. A changed
    source byte, predicate row/version, manifest digest, or preregistered
    driver digest refuses before any driver predicate can run;
14. use the fixture-only constant named
    <code>SYNTHETIC_EMPTY_EXOGENOUS_DIGEST</code> and never expose it as a
    canonical live value.

- [ ] **Step 3: RED and GREEN the complete driver source before hashing it**

Create <code>driver.rs</code> first with the complete
<code>SyntheticMaterialSample</code> value type and
<code>SyntheticDriverError</code> enum, followed by a <code>#[cfg(test)]</code>
module that references the not-yet-defined private predicate functions, and
register the private module in <code>lib.rs</code>. No Step 3 test or function
may depend on a type deferred to <code>validation.rs</code>. Pin:

- cumulative attempted quanta and governed-cost traces that classify as Continuing or LatePlateau invalidate the synthetic witness;
- constant-rate and Other driver traces do not fail that particular predicate;
- either cumulative driver containing a non-finite or negative value refuses,
  and a value below its predecessor refuses before classification;
- paired synthetic samples with the declared tick offset and equal aligned
  contribution and aggregate bits pass;
- one changed aligned aggregate bit fails with
  <code>AlignedMaterialMismatch</code>;
- aligned sequences with 0, <code>3*w</code>, or <code>3*w+2</code> rows
  refuse because only exactly <code>3*w+1</code> rows realize the declared
  window;
- NaN, either infinity, and a negative aggregate refuse; negative zero stores
  positive-zero bits; and an offset other than the window width refuses;
- input-record permutations yield identical sorted schedule and attempt-ledger bytes;
- a semantic row change changes bytes and digest.

Define these Step 3-owned types before the private predicates and their tests:

~~~rust
pub struct SyntheticMaterialSample {
    tick: u64,
    contribution_digest: Digest32,
    aggregate_bits: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyntheticDriverError {
    CandidateProjectionMismatch,
    CandidateScheduleDigestMismatch,
    AttemptLedgerDigestMismatch,
    ExogenousLedgerDigestMismatch,
    CandidateCadenceCountMismatch { declared: u16, actual: usize },
    CandidateCadenceTickMismatch { index: usize, expected: u64, actual: u64 },
    CandidateCadenceOverflow { index: usize },
    CandidateIntentCountMismatch { expected: usize, actual: usize },
    CandidateIntentDigestMismatch { index: usize },
    CandidateIntentTickMismatch { index: usize },
    CandidatePracticeMismatch { index: usize },
    CandidateTargetPolicyMismatch { index: usize },
    CandidateGovernedCostMismatch { index: usize },
    CandidateParameterBytesMismatch { index: usize },
    TwinChangedBothLedgers,
    TwinChangedWrongLedger,
    TwinChangedNonLedgerField { field: RunIdentityField },
    ControlTraceRunIdentityMismatch,
    InterventionTraceRunIdentityMismatch,
    ComparisonControlTraceDigestMismatch,
    ComparisonInterventionTraceDigestMismatch,
    ComparisonLedgerKindMismatch,
    ComparisonControlLedgerDigestMismatch,
    ComparisonInterventionLedgerDigestMismatch,
    ComparisonInterventionDeltaDigestMismatch,
    DriverAuthoredShape { driver: &'static str, class: SfsClass },
    InvalidCumulativeDriverValue { driver: &'static str, index: usize, bits: u64 },
    CumulativeDriverDecreased { driver: &'static str, index: usize, previous_bits: u64, actual_bits: u64 },
    SampleLimit { actual: usize },
    SampleCountMismatch { control: usize, aligned: usize },
    MaterialSampleCountMismatch { expected: usize, control: usize, aligned: usize },
    ArithmeticOverflow { field: &'static str },
    TickOffsetMismatch { index: usize },
    WrongAlignmentOffset { expected: u16, actual: u16 },
    AlignedMaterialMismatch { index: usize },
    InvalidSyntheticAggregate { bits: u64 },
}

impl SyntheticMaterialSample {
    pub fn new(
        tick: u64,
        contribution_digest: Digest32,
        aggregate: f64,
    ) -> Result<Self, SyntheticDriverError>;
}
~~~

Step 4 re-exports only the value type, error type, and opaque validated-handle
methods. The unvalidated predicate functions remain crate-private. Because the
driver contract binds the complete <code>driver.rs</code> bytes, later edits to
these shared types also restart artifact generation.

Create the complete 41-row
<code>sfs_mutation_manifest_v1.txt</code> specified in Step 6 while writing
these tests.

Run:

~~~bash
cd rust
cargo test -p babylon-evidence --lib driver::tests --locked
~~~

Expected RED: unresolved private driver functions. Implement all five exact
predicate families in <code>driver.rs</code>: candidate/intent realization,
twin identity, persistence comparison identity, cumulative driver shape, and
aligned material sequence. Each function stays below 100 lines and uses
the literal fixed loops in this task. Keep them <code>pub(crate)</code>; the
validated contract handle becomes their only public route in Step 4. Rerun the
same command to green, then run format and Clippy. Do not edit
<code>driver.rs</code> after its source manifest is generated; a later required
change restarts Step 4 and regenerates both exact driver artifacts.

- [ ] **Step 4: RED, generate, and GREEN the source-bound driver contract**

Step 4 owns the complete contract-layer types; it does not depend on the
Step 5 validation error:

~~~rust
pub struct SyntheticDriverContractV1 {
    canonical_bytes: Vec<u8>,
    manifest_digest: Digest32,
    source_digest: Digest32,
}

pub struct ValidatedSyntheticDriver<'a> {
    contract: &'a SyntheticDriverContractV1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyntheticDriverContractError {
    ManifestByteLimit { actual: usize },
    ManifestMalformed { row: usize },
    SourceDigestMismatch,
    ContractDigestMismatch,
    PreregistrationDigestMismatch,
}

pub fn parse_synthetic_driver_contract(
    manifest_bytes: &[u8],
) -> Result<SyntheticDriverContractV1, SyntheticDriverContractError>;

pub fn bind_synthetic_driver<'a>(
    preregistration: &SfsPreregistrationV1,
    contract: &'a SyntheticDriverContractV1,
) -> Result<ValidatedSyntheticDriver<'a>, SyntheticDriverContractError>;
~~~

The opaque handle methods return <code>SyntheticDriverError</code> and delegate
only to Step 3 functions. Step 5 can wrap either complete error enum for its
orchestration API, but it cannot redefine or weaken them.

Add Python tests for the absent driver-vector function/CLI and
<code>synthetic_driver_contract.rs</code> tests for the absent parser, digest
binding, opaque handle, and source/predicate/preregistration mutations. First
run:

~~~bash
mise run test:q -- tests/unit/tools/test_sfs_contract_vectors.py
cd rust
cargo test -p babylon-evidence --test synthetic_driver_contract --locked
~~~

Expected RED: the Python route and Rust contract parser are absent. Implement
the independent Python route and <code>driver_contract.rs</code>, including the
seven-row parser, compile-time <code>include_bytes!("driver.rs")</code> source
comparison, preregistration binding, and handle methods that delegate only to
the crate-private Step 3 functions. Then generate and verify the two artifacts:

~~~bash
uv run python tools/sfs_contract_vectors.py --write-synthetic-driver
uv run python tools/sfs_contract_vectors.py --check-synthetic-driver
mise run test:q -- tests/unit/tools/test_sfs_contract_vectors.py
uv run ruff check tools/sfs_contract_vectors.py tests/unit/tools/test_sfs_contract_vectors.py
uv run ruff format --check tools/sfs_contract_vectors.py tests/unit/tools/test_sfs_contract_vectors.py
cd rust
cargo test -p babylon-evidence --test synthetic_driver_contract --locked
~~~

Expected GREEN: both independent representations and every source/digest
mutation pass. Now run <code>synthetic_proof_harness</code> once and record RED
from the still-unimplemented governed-manifest/cone validators, not from a
missing driver source or fixture.

- [ ] **Step 5: Implement bounded synthetic validators**

Implement the remaining governed-manifest, profile, cone, and orchestration
code in <code>validation.rs</code>. Do not edit the already source-bound
<code>driver.rs</code>; <code>driver_contract.rs</code> may only expose the
sealed-handle delegation already made green in Step 4.

~~~rust
pub struct ProducerConsumerEdgeV1 {
    producer_id: String,
    consumer_id: String,
    channel_kind: SyntheticChannelKindV1,
    channel_id: String,
}

pub struct SyntheticGovernedComponentV1 {
    component_id: String,
    component_kind: ComponentKindV1,
    component_source_digest: Digest32,
}

struct SyntheticProfileRowV1 {
    component_id: String,
    set_name: String,
    entry: String,
}

struct SyntheticBoundRowV1 {
    component_id: String,
    declared_fuel: u64,
    computed_bound: u64,
    cardinality_digest: Digest32,
    intrinsic_cost_digest: Digest32,
}

pub struct SyntheticGovernedManifestV1 {
    canonical_bytes: Vec<u8>,
    manifest_digest: Digest32,
    host_component_manifest_digest: Digest32,
    components: Vec<SyntheticGovernedComponentV1>,
    edges: Vec<ProducerConsumerEdgeV1>,
    profile_rows: Vec<SyntheticProfileRowV1>,
    bound_rows: Vec<SyntheticBoundRowV1>,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntheticChannelKindV1 {
    Field = 0,
    Relation = 1,
    Contribution = 2,
    LedgerRow = 3,
    Receipt = 4,
    ReducerOutput = 5,
}

impl ProducerConsumerEdgeV1 {
    pub fn new(
        producer_id: &str,
        consumer_id: &str,
        channel_kind: SyntheticChannelKindV1,
        channel_id: &str,
    ) -> Result<Self, SfsValidationError>;
}

impl SyntheticGovernedComponentV1 {
    pub fn new(
        component_id: &str,
        component_kind: ComponentKindV1,
        component_source_digest: Digest32,
    ) -> Result<Self, SfsValidationError>;
}

pub fn parse_synthetic_governed_manifest(
    manifest_bytes: &[u8],
    scoped_bsl_rule: &SExpr,
    scoped_bsl_audit: &SfsRuleAuditResult,
) -> Result<SyntheticGovernedManifestV1, SfsValidationError>;

impl SyntheticGovernedManifestV1 {
    pub const fn manifest_digest(&self) -> Digest32;
    pub const fn host_component_manifest_digest(&self) -> Digest32;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SfsValidationError {
    Wire(SfsWireError),
    Classifier(SfsClassError),
    Record(SfsRecordError),
    ProfileRecord(SfsProfileRecordError),
    GovernedManifestByteLimit { actual: usize },
    GovernedManifestTotalRowLimit { actual: usize },
    GovernedManifestLineLimit { row: usize, actual: usize },
    ComponentLimit { actual: usize },
    ProfileRowLimit { actual: usize },
    BoundRowLimit { actual: usize },
    EdgeLimit { actual: usize },
    SourcePayloadLimit { component_id: String, actual: usize },
    DuplicateComponentId { component_id: String },
    DuplicateTypedEdge { producer_id: String, consumer_id: String, channel_id: String },
    UnknownEdgeEndpoint { component_id: String },
    UnknownConeEndpoint { set: &'static str, component_id: String },
    NoRootToSinkPath,
    CausalConeDigestMismatch,
    GovernedComponentSetMismatch,
    ConeProfileMismatch,
    BoundDeclaredFuelMismatch,
    BoundComputedLimitMismatch,
    BoundCardinalityDigestMismatch,
    BoundIntrinsicCostDigestMismatch,
    EdgeProducerEffectMismatch { component_id: String, channel_id: String },
    EdgeConsumerReadMismatch { component_id: String, channel_id: String },
    MissingIncomingCausalEdge { component_id: String, entry: String },
    MissingOutgoingCausalEdge { component_id: String, entry: String },
    Driver(SyntheticDriverError),
    DriverContract(SyntheticDriverContractError),
    GovernedManifestMalformed { row: usize },
    GovernedManifestDigestMismatch,
    HostManifestDigestMismatch,
    GovernedFootprintDigestMismatch,
    ProofProfileDigestMismatch,
    PreregistrationDigestMismatch,
    ComponentKindMismatch { component_id: String },
    ComponentSourceDigestMismatch { component_id: String },
    MutationManifestMalformed { row: usize },
    MutationManifestDigestMismatch,
    MutationCoverageMismatch { mutation_id: String },
}

pub fn component_profile_from_bsl(
    component_id: &str,
    audit: &SfsRuleAuditResult,
) -> Result<SfsComponentProofProfileV1, SfsValidationError>;

pub fn validate_synthetic_cone(
    cone: &CausalConeV1,
    profile: &SfsProofProfileV1,
    governed_manifest: &SyntheticGovernedManifestV1,
) -> Result<(), SfsValidationError>;

pub fn validate_synthetic_profile_identity(
    run_identity: &RunIdentityV1,
    proof_profile: &SfsProofProfileV1,
    preregistration: &SfsPreregistrationV1,
    governed_manifest: &SyntheticGovernedManifestV1,
    mutation_manifest_digest: Digest32,
) -> Result<(), SfsValidationError>;

impl ValidatedSyntheticDriver<'_> {
    pub fn validate_candidate_projection(
        &self,
        run_identity: &RunIdentityV1,
        preregistration: &SfsPreregistrationV1,
        schedule: &PracticeCandidateScheduleV1,
        attempts: &PracticeAttemptLedgerV1,
        intents: &[PracticeIntentV1],
        actual_exogenous_ledger_digest: Digest32,
    ) -> Result<(), SyntheticDriverError>;

    pub fn validate_twin_identity_difference(
        &self,
        control: &RunIdentityV1,
        intervention: &RunIdentityV1,
        selected: DifferingLedgerKindV1,
    ) -> Result<(), SyntheticDriverError>;

    pub fn validate_persistence_comparison_identity(
        &self,
        control: &RunIdentityV1,
        intervention: &RunIdentityV1,
        control_trace: &SfsTraceV1,
        intervention_trace: &SfsTraceV1,
        comparison: &PersistenceComparisonV1,
        intervention_delta: &InterventionDeltaV1,
    ) -> Result<(), SyntheticDriverError>;

    pub fn validate_driver_shapes(
        &self,
        window_width: u16,
        attempted_quanta: &[f64],
        governed_costs: &[f64],
    ) -> Result<(), SyntheticDriverError>;

    pub fn validate_aligned_material_sequence(
        &self,
        control: &[SyntheticMaterialSample],
        aligned: &[SyntheticMaterialSample],
        window_width: u16,
        tick_offset: u16,
    ) -> Result<(), SyntheticDriverError>;
}
~~~

`component_profile_from_bsl` sets kind
`ComponentKindV1::BslRule`, copies the canonical AST source digest,
and converts each read/effect getter to one checked
`CanonicalProfileSet`. It performs no source scan and mints no host
component profile. The governed-manifest parser requires the sealed audit
result's declared fuel, computed bound, cardinality input digest, and intrinsic
cost input digest to equal all four fields of its sole BSL bound row before
this helper's result may enter the proof profile. Equal computed bounds cannot
hide a changed audit input table.

`ValidatedSyntheticDriver::validate_candidate_projection` first computes the complete attempt
ledger record digest and requires it to equal
`run_identity.practice_attempt_ledger_digest()`. It requires the
supplied actual exogenous digest to equal both the preregistered expected digest
and `run_identity.exogenous_input_ledger_digest()`. It then requires
`record_digest(schedule)` to equal the preregistered schedule digest,
preflights the schedule and intent counts against the declared count, and validates
each declared flat-cadence tick with a fixed 65,535-index traversal starting at
zero. For every admitted index, compute
`first_attempt_tick + index * attempt_stride` with checked multiply
and checked add; return `CandidateCadenceOverflow` before comparing a
tick that cannot be represented, and return the specific count or tick error
above for the other mismatches. For each supplied intent, validate and compute
the parameter-bytes digest before the complete intent digest. This ordering gives an
invalid parameter sequence its specific error before the complete encoder sees
the same defect. Wrap each successful practice-contract digest array with
`Digest32::from_bytes` before comparison, and use these exact error
mappings:

~~~rust
let parameter_digest = parameter_bytes_digest(intent)
    .map(Digest32::from_bytes)
    .map_err(|_| SyntheticDriverError::CandidateParameterBytesMismatch { index })?;
let intent_digest = intent_digest(intent)
    .map(Digest32::from_bytes)
    .map_err(|_| SyntheticDriverError::CandidateIntentDigestMismatch { index })?;
let target_digest = Digest32::from_bytes(target_selection_policy_digest(
    intent.target_domain(),
    intent.target_node_id(),
));
~~~

Require the complete intent digest to equal the corresponding candidate row,
map an invalid parameter to `CandidateParameterBytesMismatch`, and map
another intent-codec failure such as unsorted evidence to
`CandidateIntentDigestMismatch`. The typed fixed-target function is
infallible.

The validator then requires intent resolve tick to equal candidate attempt tick,
practice id to equal the preregistration, and quoted governed cost to equal the
preregistration. It requires `target_digest` to equal the
preregistered fixed-target digest and `parameter_digest` to equal the
preregistered parameter-bytes digest. It finally
projects the attempt ledger and compares the two complete schedule envelopes
byte for byte. It does not infer an authoritative accepted subset from the
attempt-ledger header or claim that these supplied sealed intents came from a
live Gate 5 producer.

`ValidatedSyntheticDriver::validate_persistence_comparison_identity` first binds each trace's
stored run-identity digest to the complete digest of its supplied
`RunIdentityV1`, then binds the comparison's two trace digests to the
complete trace-record digests. It calls
`ValidatedSyntheticDriver::validate_twin_identity_difference` with the comparison's own ledger
kind, selects that same ledger digest from each run, and compares both with the
comparison fields. Finally, the supplied `InterventionDeltaV1` kind
must equal the comparison kind and its complete record digest must equal
`intervention_delta_digest`. No separately supplied kind or digest
can override a sealed comparison field.

`SyntheticMaterialSample::new` rejects NaN, either infinity, and any
negative aggregate, normalizes negative zero to positive zero, and stores only
the normalized bits.
`ValidatedSyntheticDriver::validate_aligned_material_sequence` requires
the supplied window width to be 2 through 52 and requires
`tick_offset == window_width` before comparing ticks, contribution
digests, and aggregate bits. Add exact tests for NaN, both infinities, a
negative finite value, negative-zero normalization, and a wrong offset. This
synthetic comparator proves only equality of the fields it receives. It does
not observe or make a claim about world hashes, committed-envelope identity,
input-ledger identity, or the cause of a mismatch.

`validate_synthetic_profile_identity` requires the sealed manifest's
internally derived host and governed digests to equal their distinct
run-identity fields. It requires the governed digest to equal the proof-profile
header, the complete
proof-profile record digest to equal both the run identity and preregistration,
the supplied mutation-manifest digest to equal the preregistration, and the
complete preregistration record digest to equal the run identity. It does not
claim that either synthetic manifest enumerates a live host; the separate
`bind_synthetic_driver` call requires the parsed driver's complete
manifest digest to equal the preregistration before it returns the only handle
that exposes Task 9 predicates.

#### Governed manifest parsing

`parse_synthetic_governed_manifest` owns the admitted raw bytes and
derives the domain-separated manifest and host-component digests from them,
uses the exact row grammar and row ceilings above, validates component,
profile, bound, and edge closure, and returns one private-field
`SyntheticGovernedManifestV1`, whose constructor does not accept a
digest, component vector, edge vector, profile rows, or bound rows
independently.

Before row closure, it recomputes `canonical_bytes(scoped_bsl_rule)`,
requires its SHA-256 to equal the sealed audit source digest, and requires the
decoded sole `canonical-bsl` payload to equal those canonical bytes,
so neither a caller-recomputed manifest nor a digest-only collision surrogate
can substitute another BSL source.

The governed-manifest parser uses these exact limits:
`MAX_GOVERNED_MANIFEST_BYTES = 1_048_576`,
`MAX_GOVERNED_MANIFEST_LINE_BYTES = 131_682` including LF, 64 component rows,
32,768 profile rows, 64 bound rows, 4,096 edge rows, 65,535 decoded source
bytes per component, and 36,992 total rows; it rejects the byte ceiling before
line discovery, scans at most 1,048,576 byte positions for LF delimiters,
stores at most 36,992 bounded line spans, then dispatches with a fixed loop over
exactly 36,992 indices starting at zero. Each row family refuses its
maximum-plus-one witness with the corresponding typed limit error before
growing that family. Tests cover every exact maximum and maximum plus one,
including the combined 256-byte component ID plus 65,535-byte source-payload
component row after lowercase hex framing, a 131,683-byte line, and a
65,536-byte decoded source payload.

#### Synthetic cone validation

The `validate_synthetic_cone` function accepts at most 64 governed components and
4,096 typed edges from that sealed manifest and, before traversal, requires the
manifest's internally derived digest to equal the proof-profile header and
`record_digest(cone)` to equal the proof-profile causal-cone digest; for each
governed component, require its ID, kind, and source digest to equal
the corresponding proof-profile row, reconstruct all eight
`CanonicalProfileSet` values from the sealed manifest's profile rows,
and compare the complete `SfsComponentProofProfileV1`, not only its
identity header. A changed read, effect, query, operator, intrinsic, constant,
edge, or comparison-context set returns `ConeProfileMismatch` even
when the caller recomputes the proof-profile and run identities.

The validator maps sorted component IDs to indices,
builds a fixed 64 by 64 adjacency matrix, and performs at most 64 forward and
64 reverse expansion passes. It intersects nodes reachable from any root with
nodes that can reach any sink. The result must equal both the cone component
IDs and proof-profile component IDs. Duplicate typed edges, a root or sink
outside the component set, and a component source digest that differs from its
parsed governed-manifest row fail before traversal.

Before reachability, derive the exact producer-effect and consumer-read token
for every typed edge: `Field` requires producer effect
`node:{channel_id}` and the consumer's exact field-read entry;
`Relation` requires `edge:{channel_id}` and the exact
edge-read entry. Contribution, LedgerRow, Receipt, and ReducerOutput require
the same lower-case kind prefix on both the producer effect and consumer
field-read token, while every non-root field/edge read must have exactly one incoming
typed edge, and every non-sink effect must have exactly one outgoing typed
edge; root reads and sink effects are the only unmatched endpoints, while missing,
extra, duplicated, or kind-incompatible edges fail even when the caller
recomputes the governed manifest, proof profile, preregistration, and run
identity around the mutation.

The `ValidatedSyntheticDriver::validate_driver_shapes` method first relies on the classifier's exact
window and `3*w+1` length contract and, before classification, checks
both cumulative drivers with a fixed 157-index traversal starting at zero,
requires every value to be finite, non-negative, and greater than or equal to
its predecessor, and checks only the supplied synthetic driver predicate;
authoritative ledger derivation remains deferred to Gate 5.

The `ValidatedSyntheticDriver::validate_aligned_material_sequence` method first requires a window
width of 2 through 52, computes `3*w+1` with checked arithmetic, and rejects
either slice unless both lengths equal that exact count, including the 0,
expected-minus-one, and expected-plus-one witnesses. It then requires
`tick_offset == window_width` and uses one checked tick addition plus a fixed
157-index traversal starting at zero, stopping at the admitted length.
For each aligned row, it requires the exact declared tick offset, contribution
digest equality, and aggregate-bit equality in that order.

The <code>Synthetic</code> names are deliberate. Do not export a function named <code>validate_live_proof</code>, <code>produce_trace</code>, or <code>commit_evidence</code>.

- [ ] **Step 6: Validate and pin the mutation specification**

The manifest schema is:

~~~text
mutation_id|phase|producer_consumer_seam|expected_predicate|activation|test_name_or_dash
~~~

Encode it as UTF-8 with LF separators, no CR or blank line, exactly one
terminal LF, and rows sorted by <code>mutation_id</code> UTF-8 bytes. Its exact
content identity is:

~~~text
SHA256(
  ASCII("babylon.sfs-mutation-manifest.v1") || 0x00
  || exact_manifest_bytes
)
~~~

Pin the lowercase literal digest and place that digest in the synthetic
<code>SfsPreregistrationV1.mutation_manifest_digest</code>. Changing a row,
line ending, terminal newline, or activation must change the digest and make
the preregistration comparison fail.

Include exact rows for every section 9.7 category. Mark runnable rows <code>SYNTHETIC</code>. Mark unavailable rows with their exact dependency:

- <code>GATE3</code> for committed-envelope, exogenous-ledger, restart, and Archive cases;
- <code>GATE5</code> for authoritative action/accepted-row cases;
- <code>G6</code> for inventory, routing, and labor cases;
- <code>PER44</code> for membership payload, join, contribution omission/double-count, and encounter-producer removal;
- <code>LIVE_T3</code> for topology, distribution, graph-backend, and persistence scenario twins.

Use these exact rows:

| ID | Phase | Seam | Expected predicate | Activation | Test |
|---|---|---|---|---|---|
| <code>S01_NAMED_SHAPE</code> | STATIC | BSL AST to intrinsic profile | forbidden intrinsic | SYNTHETIC | <code>forbidden_corpus_exact</code> |
| <code>S02_EXP_RESPONSE</code> | STATIC | BSL AST to intrinsic profile | forbidden exp response | SYNTHETIC | <code>forbidden_corpus_exact</code> |
| <code>S03_LOG_RESPONSE</code> | STATIC | BSL AST to intrinsic profile | forbidden log response | SYNTHETIC | <code>forbidden_corpus_exact</code> |
| <code>S04_STORED_AGGREGATE</code> | STATIC | engine source to authoritative field | reserved engine token | SYNTHETIC | <code>every_reserved_token_and_language_surface_fails</code> |
| <code>S05_STORED_STAGE</code> | STATIC | engine source to authoritative field | reserved engine token | SYNTHETIC | <code>every_reserved_token_and_language_surface_fails</code> |
| <code>S06_TIME_READ</code> | STATIC | BSL binding to rule | forbidden binding source | SYNTHETIC | <code>forbidden_corpus_exact</code> |
| <code>S07_CALENDAR_READ</code> | STATIC | BSL binding to rule | forbidden binding source | SYNTHETIC | <code>forbidden_corpus_exact</code> |
| <code>S08_RNG_READ</code> | STATIC | BSL intrinsic to rule | forbidden intrinsic | SYNTHETIC | <code>forbidden_corpus_exact</code> |
| <code>S09_ABSOLUTE_SCHEDULE</code> | STATIC | time binding to effect guard | forbidden absolute schedule | SYNTHETIC | <code>forbidden_corpus_exact</code> |
| <code>S10_SCENARIO_LATER_STATE</code> | STATIC | scenario field to authoritative state | reserved engine token | SYNTHETIC | <code>every_reserved_token_and_language_surface_fails</code> |
| <code>S11_DIRECT_MEMBERSHIP</code> | STATIC | intent effect to membership payload | exact membership effect refusal | PER44 | <code>-</code> |
| <code>S12_UNAUTHORIZED_WRITE</code> | STATIC | BSL effect to profile | unexpected effect | SYNTHETIC | <code>forbidden_corpus_exact</code> |
| <code>S13_ROLE_RELABEL</code> | STATIC | BSL attribution to source digest | profile source mismatch | SYNTHETIC | <code>role_relabel_moves_identity_and_fails_profile</code> |
| <code>S14_DEAD_PERMISSION</code> | STATIC | governed row to absent AST use | exact footprint mismatch | SYNTHETIC | <code>forbidden_corpus_exact</code> |
| <code>S15_AST_IDENTITY_STASIS</code> | STATIC | semantic AST to content identity | source digest mismatch | SYNTHETIC | <code>semantic_change_moves_identity</code> |
| <code>D01_DRIVER_SHAPE</code> | DRIVER | attempt ledger to cumulative driver trace | authoritative trace derivation plus class refusal | GATE5 | <code>-</code> |
| <code>D02_TIME_SHIFT</code> | DRIVER | paired material samples to aligned comparator | exact aligned material equality | SYNTHETIC | <code>aligned_material_bits_match</code> |
| <code>D03_INPUT_PERMUTATION</code> | DRIVER | input order to canonical ledger | exact bytes equal | SYNTHETIC | <code>input_permutations_preserve_bytes</code> |
| <code>D04_NO_INTENT</code> | DRIVER | accepted actions to mechanics | declared difference | GATE5 | <code>-</code> |
| <code>D05_ALTERNATE_PRACTICE</code> | DRIVER | practice ID to mechanics | declared difference | GATE5 | <code>-</code> |
| <code>D06_ZERO_INVENTORY</code> | DYNAMIC | inventory to aid route | no material transfer | G6 | <code>-</code> |
| <code>D07_ZERO_LABOR</code> | DYNAMIC | labor to aid route | no material transfer | G6 | <code>-</code> |
| <code>D08_SEVER_PRESENCE</code> | DYNAMIC | presence relation to scoped mechanic | declared causal difference | LIVE_T3 | <code>-</code> |
| <code>D09_SEVER_COMMUNICATION</code> | DYNAMIC | communication relation to scoped mechanic | declared causal difference | LIVE_T3 | <code>-</code> |
| <code>D10_SEVER_TRANSPORT</code> | DYNAMIC | transport relation to scoped mechanic | declared causal difference | LIVE_T3 | <code>-</code> |
| <code>D11_SEVER_SOLIDARITY</code> | DYNAMIC | solidarity relation to scoped mechanic | declared causal difference | LIVE_T3 | <code>-</code> |
| <code>D12_SEVER_MEMBERSHIP</code> | DYNAMIC | membership relation to reducer | declared causal difference | PER44 | <code>-</code> |
| <code>D13_REMOVE_RESOLVER</code> | DYNAMIC | encounter producer to membership | observable unavailable | PER44 | <code>-</code> |
| <code>D14_DEGREE_REWIRE</code> | DYNAMIC | topology to local propagation | declared relational difference | LIVE_T3 | <code>-</code> |
| <code>D15_ROUTE_CUT</code> | DYNAMIC | redundant route to cut bridge | declared routing difference | LIVE_T3 | <code>-</code> |
| <code>D16_CARRIER_DISTRIBUTION</code> | DYNAMIC | equal total to carrier distribution | aggregate traces differ lawfully | LIVE_T3 | <code>-</code> |
| <code>E01_MISSING_AS_ZERO</code> | EVALUATOR | missing payload to reducer | unavailable, not zero | PER44 | <code>-</code> |
| <code>E02_CONTRIBUTION_OMISSION</code> | EVALUATOR | contribution set to aggregate | digest or aggregate mismatch | PER44 | <code>-</code> |
| <code>E03_CONTRIBUTION_DOUBLE</code> | EVALUATOR | contribution set to aggregate | duplicate identity refusal | PER44 | <code>-</code> |
| <code>E04_SUMMATION_ORDER</code> | EVALUATOR | sorted contributions to aggregate | canonical order mismatch | PER44 | <code>-</code> |
| <code>E05_WINDOW_OFF_BY_ONE</code> | EVALUATOR | samples to classifier | wrong length refusal | SYNTHETIC | <code>classifier_rejects_off_by_one</code> |
| <code>E06_COMPARATOR</code> | EVALUATOR | deltas to classifier | adversarial class mismatch | SYNTHETIC | <code>the_eight_w2_vectors_pin_predicate_order</code> |
| <code>E07_UNCOMMITTED_SAMPLE</code> | EVALUATOR | committed envelope to sample | envelope identity mismatch | GATE3 | <code>-</code> |
| <code>E08_OMITTED_RUN_FIELD</code> | EVALUATOR | executable input to run identity | mutation changes digest | SYNTHETIC | <code>every_run_field_moves_identity</code> |
| <code>E09_PROCESS_RESTART</code> | EVALUATOR | persisted envelope to replay | exact trace bytes equal | GATE3 | <code>-</code> |
| <code>E10_GRAPH_BACKEND</code> | EVALUATOR | graph backend to committed trace | exact trace bytes equal | LIVE_T3 | <code>-</code> |

The test rejects duplicate mutation IDs, unknown phase or activation values,
an absent row from this table, an extra row, and any row marked
<code>SYNTHETIC</code> without the exact executable test name above.

Write <code>sfs_synthetic_profile_v1.txt</code> with this exact schema, sorted
by <code>label</code>, LF only, and one terminal LF:

~~~text
label|domain|envelope_hex|sha256_hex
~~~

It contains one row for each of the three component profiles, one causal-cone
row, and one proof-profile row. The proof-profile row uses the exact governed
manifest, forbidden-corpus, audit-source, and causal-cone digests defined in
Tasks 8 and 9. The harness recomputes every envelope and digest from the source
manifests before comparing these literals; it must not decode its expected
bytes to construct the actual records.

This manifest specifies blocked dynamic work; it does not claim that the blocked mutations ran.

- [ ] **Step 7: Run harness, crate, and mutation-manifest gates**

~~~bash
mise run test:q -- tests/unit/tools/test_sfs_contract_vectors.py
uv run python tools/sfs_contract_vectors.py --check-synthetic-profile
uv run python tools/sfs_contract_vectors.py --check-synthetic-driver
cd rust
cargo test -p babylon-evidence --test synthetic_proof_harness --locked
cargo test -p babylon-evidence --locked
cargo clippy -p babylon-evidence --all-targets --locked -- -D warnings -D clippy::pedantic
cargo fmt --all -- --check
~~~

- [ ] **Step 8: Commit the synthetic proof harness**

~~~bash
git add tools/sfs_contract_vectors.py tests/unit/tools/test_sfs_contract_vectors.py rust/crates/babylon-evidence/src/validation.rs rust/crates/babylon-evidence/src/driver.rs rust/crates/babylon-evidence/src/driver_contract.rs rust/crates/babylon-evidence/src/lib.rs rust/crates/babylon-evidence/tests/synthetic_driver_contract.rs rust/crates/babylon-evidence/tests/synthetic_proof_harness.rs rust/crates/babylon-evidence/tests/fixtures/sfs_synthetic_governed_manifest_v1.txt rust/crates/babylon-evidence/tests/fixtures/sfs_synthetic_profile_v1.txt rust/crates/babylon-evidence/tests/fixtures/sfs_synthetic_driver_contract_v1.txt rust/crates/babylon-evidence/tests/fixtures/sfs_synthetic_driver_v1.txt rust/crates/babylon-evidence/tests/fixtures/sfs_mutation_manifest_v1.txt
mise run commit -- "test(evidence): add synthetic profile and cone harness"
~~~

---

## Final Verification

Run the light gates first. Do not overlap Cargo commands:

~~~bash
uv lock --check
mise run test:q -- tests/unit/tools/test_sfs_contract_vectors.py
uv run python tools/sfs_contract_vectors.py --check
uv run python tools/sfs_contract_vectors.py --check-synthetic-profile
uv run python tools/sfs_contract_vectors.py --check-synthetic-driver
~~~

Then run Rust legs from <code>rust/</code>, one at a time:

~~~bash
cargo fmt --all -- --check
cargo deny check advisories bans licenses sources
cargo test -p babylon-evidence --locked
cargo clippy -p babylon-evidence --all-targets --locked -- -D warnings -D clippy::pedantic
cargo test -p babylon-bsl --test sfs_profile_contract --locked
cargo clippy -p babylon-bsl --all-targets --locked -- -D warnings -D clippy::pedantic
cargo test -p bsl-lint --locked
cargo clippy -p bsl-lint --all-targets --locked -- -D warnings
cargo run -p bsl-lint --locked -- all
~~~

Return to the repository root:

~~~bash
mise run check
mise run check:bsl-sentinels
mise run qa:regression
mise run qa:vault-regression-ci
mise run check:gate-coverage
vale docs/superpowers/plans/2026-08-23-neel-t3-synthetic-emergence-evidence.md
vale ai/decisions/ADR226_t3_synthetic_emergence_evidence_boundary.yaml
git status --short
~~~

Expected:

- every command exits 0;
- no documentation build runs;
- no baseline file changes;
- no file outside this plan's declared implementation surface changes;
- <code>git status --short</code> shows no uncommitted file owned by this
  plan; any pre-existing unrelated entry remains untouched;
- no output claims a live observable, live causal cone, game emergence, persistence result, Archive evidence, or player agency.

## Linear Completion Handoff

After every final gate passes, resolve the final T3 branch SHA with
<code>git rev-parse HEAD</code> and require the declared T3 surface to be clean.
Post one PER-54 completion comment with every T3 implementation commit SHA, the
final branch SHA, each final-gate result, and the explicit synthetic-only and
live-proof exclusions. Only after that evidence exists, move PER-54 from In
Progress to Done, refresh it, and require the returned state to be Done. Keep
PER-59 blocked by PER-57, PER-58, and PER-22; completing this groundwork alone
does not authorize the live emergence proof.

## Activation Handoff, Not Part of This Plan

After Gate 3, Gate 5, and PER-44 land, write a separate live-proof plan. That plan must consume rather than redefine this train's bytes and classifiers. It must add:

1. PER-44-owned <code>RelationalScopeV1</code>, <code>MembershipContributionV1</code>, and <code>MembershipContributionSetV1</code> field tables plus independent vectors.
2. The encounter-to-membership producer.
3. A post-commit producer that reads complete committed envelopes and has no reverse engine dependency.
4. Canonical empty-exogenous and accepted-attempt ledgers.
5. A production host-component manifest and exact root-to-membership-sink causal cone.
6. Flat-cadence, time-shifted, permuted-driver, topology, distribution, restart, supported-backend, counterfactual, and persistence runs.
7. Fog-safe player-facing Archive evaluation evidence.

The live plan must rerun every synthetic contract here unchanged. A live need that would alter these bytes or classifier predicates requires an explicit contract revision, new schema version, and architecture review.

## Plan Self-Review

### Specification Coverage

- Sections 9.1 and 9.2: Tasks 1 and 2 make the post-commit and no-feedback boundary architectural and executable.
- Sections 9.3 and 9.4: Tasks 3, 5, 6, and 7 implement only frozen bytes and preserve the PER-44 record gap.
- Section 9.5: Tasks 4 and 7 pin every SFS and persistence class plus adversarial vectors.
- Section 9.6: Tasks 6, 8, and 9 implement exact profile bytes, bounded footprint auditing, context rows, source identity, and synthetic cone equality.
- Section 9.7: Tasks 7 through 9 provide executable static mutants, evaluator mutants, identity mutations, and a complete dependency-labeled dynamic mutation specification.
- Section 9.8: This plan satisfies only independent vectors and synthetic/static groundwork. The Scope Cut and Activation Handoff prevent any live-acceptance claim.
- Section 11: The plan lands only item 5 before the Gate 3, Gate 5, PER-44, and live-proof items.

### Deliberate Gaps

- The three PER-44-owned membership/scope encoders do not exist.
- No canonical live empty-ledger or accepted-ledger value exists.
- No live driver-contract artifact exists. Task 9 creates only the
  source-bound synthetic driver contract and proves its preregistration
  equality before synthetic predicates execute.
- No authoritative producer, committed-envelope adapter, database writer, Archive sink, or Bevy surface exists.
- No production host manifest or causal cone exists.
- No topology, distribution, persistence, restart, or backend scenario runs.

Each gap has a named dependency and no synthetic substitute.

### Type and Name Consistency

- <code>canonical_envelope</code>, <code>record_digest</code>, and <code>decode_envelope</code> are defined in Task 3 and reused unchanged.
- <code>classify_sfs</code> and <code>classify_persistence</code> are defined in Task 4 and reused by Tasks 5 through 7 and 9.
- <code>SfsTraceV1</code> and <code>PersistenceComparisonV1</code> compute their class; no caller supplies one.
- <code>validate_sfs_rule_profile</code> is the sole exact BSL policy gate.
- Every synthetic-only validator carries <code>synthetic</code> in its function or input type where confusion with a live producer is possible.
- The record domains, schema version, field order, class codes, and hard limits match the committed specification.

### Placeholder and Scope Scan

- Every task names exact files, public interfaces, RED expectation, GREEN behavior, commands, and commit.
- No step delegates an unspecified implementation choice to its executor.
- No task modifies game mechanics, BSL vocabulary, production content, persistence, Archive, Bevy, membership, or action pipelines.
- No generated documentation command appears in the verification sequence.

Plan complete. Continue autonomously with superpowers:subagent-driven-development: use one fresh implementation worker and two-stage review per task, preserve the dependency order above, and stop only for a constitutional or Linear ownership conflict.
