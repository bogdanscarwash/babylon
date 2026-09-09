<!-- Vale: This design uses governed project names and architecture terms. -->
<!-- vale Vale.Spelling = NO -->
<!-- vale ste.UnapprovedWords = NO -->
<!-- vale ste.NounClusters = NO -->
<!-- vale ste.Gerunds = NO -->

# Rust Contract Authority Design

**Status:** Director-approved on 2026-08-25

## Problem

The Neel territory, practice, and synthetic-evidence groundwork created Rust
contract crates. It then duplicated their behavior in 10,669 lines of Python
contract code, generators, fixture tooling, and tests. The pre-push
maintainability gate exposed that duplication when the two handwritten Python
contract modules received a C rank. A Radon exclusion or a Python refactor would
preserve the wrong ownership boundary.

The older Python sentinel estate presents the same architectural question at a
larger scale. A semantic rule that BSL can express must fail in the BSL reader,
loader, typechecker, or evaluator. An engine invariant must live in Rust types
and executable contracts. Only checks about repository relationships belong in
an external lint host.

## Decision

The Neel gameplay contracts become Rust-only executable authority:

- `babylon-practice-contract` owns practice identities, admission, codecs,
  checked budget arithmetic, and topology validation.
- `babylon-rtd` owns the relational territory dossier schema implementation,
  validation, canonical bytes, and projection identity.
- `babylon-evidence` owns the downstream synthetic-emergence evidence records,
  classifiers, profiles, and proof harness.
- `babylon-bsl` owns in-language semantic refusals and practice declarations.
- `bsl-lint` owns only cross-file, cross-crate, and authority-direction checks.
  These checks span more than one BSL content set.
- `babylon-kernel` remains limited to generic identity, hashing, arithmetic, and
  session primitives. Theory-specific records do not move into the kernel.

The versioned YAML and JSONL files under `contracts/` remain language-neutral
behavioral contracts. They are not a second runtime implementation. Rust tests
bind their exact source bytes, consume their shared vectors, and independently
pin field order, discriminants, canonical bytes, refusal identities, limits,
and mutation teeth.

The current Rust `generated.rs` modules become ordinary reviewed schema
modules. The cutover removes the Python generators. We freeze these V1
contracts. A future V2 generator requires an actual repeated maintenance need.

The six designed practice-budget defaults move from the Python `GameDefines`
mirror to a typed Rust constant paired with the same declarations in
`contracts/practice_contract_v1.yaml`. This authority change is narrow
for the sealed practice contract, not a new mathematical primitive or a live
Gate 5 action system.

## Immediate cutover

The current Neel branch will:

1. Add Rust source-digest and default-budget contracts.
2. Bind the Detroit-Windsor administrative fixture directly to the RTD vector
   corpus and Rust canonicalizer.
3. Retire the four Python contract modules and three Python tools.
4. Retire the T3 tool, related tests, duplicate defines, and Unicode dependency.
5. Add a Rust repository-boundary check that refuses reintroduction of the
   retired gameplay-authority paths.
6. Record the authority correction in a new ADR. Keep historical ADRs and plans unchanged.

The fixed extraction ledger and vector fixtures remain evidence. Rust validates
and mutation-tests them. Normal builds do not regenerate them.

## Sentinel retirement train

After the Neel branch lands on `dev`, a separate branch will audit all 93 Python
sentinel source files, 50 sentinel tests, and 28 registered CLI sensors. Every
row receives exactly one disposition:

1. **BSL semantic invariant:** replace it with a loader or type-system refusal.
2. **Rust engine invariant:** replace it with an unrepresentable state, checked
   transition, property test, conformance scenario, or golden replay.
3. **Repository relationship:** put it in Rust `bsl-lint`.
4. **Python data or periphery invariant:** keep it with that live subsystem
   until the subsystem itself retires.
5. **Obsolete invariant:** delete it only after proving that its protected path
   no longer exists.

No Python gameplay sentinel disappears merely because a Rust port seems
likely. Its Rust or BSL replacement must first show a failing witness.

## Acceptance criteria

- The Radon gate passes without a new exclusion.
- No production Python module implements the practice, RTD, or T3 gameplay
  contracts.
- The shared YAML, JSONL, and fixed fixtures remain consumed by Rust tests.
- Each affected Rust crate passes tests and Clippy with warnings denied.
- BSL lint, conformance, regression, vault, and gate-coverage checks remain
  green.
- The branch keeps the same bytes on current deterministic baselines unless
  an independently approved baseline ceremony occurs.

## Non-goals

- This cutover does not activate player intents, executable shocks, Gate 5, or
  database writes.
- It does not move theory-specific concepts into `babylon-kernel`.
- It does not delete Python data, AI, documentation, external-API, or CLI
  periphery.
- It does not rewrite historical ADRs or completed implementation plans.

<!-- vale ste.Gerunds = YES -->
<!-- vale ste.NounClusters = YES -->
<!-- vale ste.UnapprovedWords = YES -->
<!-- vale Vale.Spelling = YES -->
