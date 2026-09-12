# Governance and Git

Babylon is an entertainment-first emergent political-economy game. Babylon is
not a forecast and not a scientific reproduction. Theory constrains the causal model but
does not predetermine results.

Determinism proves computational identity, not scientific truth. Historical
cases test causal signatures and counterfactual behavior. The Bevy client is an
administrative viewer with no player action.

The four executable gates are:

<!-- Vale: the next items are governed gate names. -->
<!-- vale Vale.Terms = NO -->
<!-- vale ste.UnapprovedWords = NO -->
<!-- vale ste.NounClusters = NO -->
1. **PostgreSQL/H3/Archive decision-loop slice**
1. **Productive & distributive circuit**
1. **Player agency**
1. **COVID emergence benchmark**
<!-- vale Vale.Terms = YES -->
<!-- vale ste.UnapprovedWords = YES -->
<!-- vale ste.NounClusters = YES -->

Read [`CONSTITUTION.md`](../../CONSTITUTION.md) v4.2.0 for the law. Read
[`NORTH_STAR.md`](../../NORTH_STAR.md) for the game direction. Read
[`CONTRIBUTORS.md`](../../CONTRIBUTORS.md) for the full merge rules.

## Control surfaces

<!-- Vale: the next paragraphs preserve exact Linear fields, work-item IDs,
     GitHub Project names, and pull-request field names from the contract. -->
<!-- vale ste.UnapprovedWords = NO -->
<!-- vale ste.NounClusters = NO -->
<!-- vale ste.Dictionary = NO -->
In Linear, use
**[Babylon v1 — Playable Political Economy](https://linear.app/percy-raskova/project/babylon-v1-playable-political-economy-299b037e7feb)**
and its charter as the current control surface. Start with
[PER-5](https://linear.app/percy-raskova/issue/PER-5/constitution-v4-and-linear-portfolio-cutover).
Use
[PER-15](https://linear.app/percy-raskova/issue/PER-15/migrate-canonical-issue-governance-from-github-to-linear)
for the migration status.

Linear alone is canonical for issue identity, scope, status, priority,
dependencies, horizon, milestones, schedule, and current work. GitHub supplies
source control, pull requests, reviews, and historical evidence.

The team closed GitHub Project #7 and Project #8. They are historical inputs.
The migration is complete, and PER-15 is complete. Both projects remain
recoverable evidence. Neither supplies current fields, views, estimates, or
status.

PER-2 records the accepted delivery automation. Each pull-request description
must link its canonical Linear issue and select exactly one delivery
disposition. Use `Part of PER-N` as a non-closing reference for a partial
delivery. Use `Fixes PER-N` as a closing reference only on the final delivery
that satisfies the issue.

A multi-pull-request issue must remain open after every partial merge. Draft
and review activity keep the linked issue in progress. Only the merged closing
reference completes it. The automation does not require GitHub Project fields.
Imported historical issue sync cannot override canonical Linear project
scope, hierarchy, horizon, milestone, or priority.
<!-- vale ste.Dictionary = YES -->
<!-- vale ste.NounClusters = YES -->
<!-- vale ste.UnapprovedWords = YES -->

## Authority

Babylon uses Agentic Engineering. The human Director controls the reserved
theory line and merges releases to `main`. Gates grant agent autonomy.

A green gate grants a merge license. A red gate means stop. Report the fault
before you continue.

Stop and ask the Director when a task needs one of these changes:

- a new mathematical primitive
- a weaker prohibition
- a change to the reserved theory line

Use the content vocabulary ceremony for new governed terms.

## Git workflow

For a regular change, create each lane from `dev`. Open the PR against `dev`. Do not
commit directly to `dev` or `main`.

Use one of these prefixes for a lane:

- `feature/`
- `fix/`
- `docs/`
- `refactor/`
- `test/`

<!-- Vale: this paragraph preserves literal branch and issue identifiers. -->
<!-- vale ste.UnapprovedWords = NO -->
Codex-managed work can use `codex/`, with a PER identity such as
`codex/PER-123-short-name`. Other lanes can also include `PER-123` when that
makes the manual Linear link clear.
<!-- vale ste.UnapprovedWords = YES -->

Use `type(scope): description` for each commit line. Keep one logical unit in
each commit. Use `mise run commit` so the local hooks can check staged content.

<!-- Vale: this paragraph preserves literal Git emergency-workflow terms. -->
<!-- vale Vale.Spelling = NO -->
<!-- vale ste.UnapprovedWords = NO -->
Only the Director merges to `main`. The two allowed sources are a release PR
from `dev`, or a critical hotfix PR from `fix/*` that branches from `main` and
goes directly to `main`. Every merged hotfix needs a mandatory backport PR to
`dev`.
<!-- vale ste.UnapprovedWords = YES -->
<!-- vale Vale.Spelling = YES -->

<!-- Vale: this procedure preserves literal GitHub workflow and branch terms. -->
<!-- vale Vale.Spelling = NO -->
<!-- vale ste.UnapprovedWords = NO -->
Retrieve the protected branches. Before qualification, prove that `dev` already
contains the previous protected `main` merge:

```bash
git merge-base --is-ancestor origin/main origin/dev
```

Open the `dev` to `main` release PR. It must produce the complete combined
manifest on its exact source SHA. The PR runs full CI and adds uniquely named
release-only qualification checks, including the native download.

For optional diagnostics, run:

```bash
gh workflow run main.yml --ref dev
```

The main PR supplies the authoritative evidence. A preliminary manual run is
not mandatory. After exact-head acceptance, only the Director runs:

```bash
mise run pr:merge -- N --director-main
```

After the merge, the owner publishes the version from exact `origin/main` with
this enforced return path:

```bash
mise run release:prepare-dev-sync -- vX.Y.Z N
mise run release:tag -- --yes
```

Run the first command on a sanctioned lane created at exact `origin/main`.
Commit its real-diff lineage record, open its PR to `dev`, and merge it with the
ordinary sanctioned command. The tag task refuses until that PR makes exact
`origin/main` reachable from `origin/dev`.

A critical hotfix uses the same combined manifest and command, followed by its
mandatory backport PR to `dev`.
<!-- vale ste.UnapprovedWords = YES -->
<!-- vale Vale.Spelling = YES -->

Use the merge rules in `CONTRIBUTORS.md` for all other pull requests.

<!-- Vale: this section preserves literal GitHub review and branch terms. -->
<!-- vale ste.UnapprovedWords = NO -->
## PR acceptance

The merge record identifies the base branch and exact reviewed head SHA.
Blocking checks must complete successfully against that head and base branch.
CodeQL gates pull requests to `main`: the exact-head aggregate must succeed and
its pull-request analysis must have no open findings. Scans after pushes to
`dev` and `main`, weekly scans, and manual scans refresh security findings
asynchronously. Verified asynchronous CodeQL checks do not delay merges to
`dev`; known open findings on the default branch still block all merges.
Dependency and workflow security checks remain part of the development gate.

Copilot review evidence is advisory. An absent, stale, incomplete,
identity-mismatched, or API-failed review does not block a merge. A top-level
comment without a reply is also advisory. Any unresolved review thread blocks
the merge, regardless of its author.

Rust has one current implementation and persistence schema. Refactoring can
retire superseded code and implementation-coupled tests while preserving current
behavioral evidence. Refuse existing incompatible databases before mutation.
Never migrate, reset, or delete them to qualify a change.

Each PR records a behavioral-contract disposition. Changed behavior links a
durable, implementation-independent contract. No behavior change includes an
explanation of why the current contracts are enough.

Each PR also records its baseline disposition. An intentional governed
baseline change includes the declared ceremony, the record from
`tools/generate_ceremony_message.py`, and the required
`Baselines: blessed(<slug>)` trailer. Unintended drift is a fault, not a
ceremony.

The sole merge command is:

```bash
mise run pr:merge -- N
```

Do not run `gh pr merge` directly in any form. Preserve the source branch by
default. Branch deletion requires an explicit owner decision and verification
that no open PR or other work depends on the branch.
<!-- vale ste.UnapprovedWords = YES -->

## Scope and records

Query recorded decision metadata before you read ADR source:

```bash
mise run adr -- search "<topic>"
mise run adr -- show ADR221
```

<!-- vale ste.UnapprovedWords = NO -->
The first result page puts new ADR matches first. If `results_truncated` is
true, use `--offset <next_offset>` when you need more matches.
<!-- vale ste.UnapprovedWords = YES -->

The ignored SQLite catalog is a disposable read model of Git-tracked ADR YAML.
Its recorded status does not prove live behavior. The `show` command returns a
source path and record-root selector. Read only that record for the rationale.
Do not load `ai/decisions/index.yaml`
for lookup. It remains a legacy registry and deterministic sentinel input.
<!-- Vale: this paragraph preserves exact authority and repository path terms. -->
<!-- vale ste.UnapprovedWords = NO -->
`ai/state.yaml` is historical implementation evidence, not current status.
The `project/` tree is non-live context. Linear alone owns current status and
work.
<!-- vale ste.UnapprovedWords = YES -->

Do not change an unrelated fault. Report it unless the task owner expands the
scope.

Add an ADR when the team makes an architecture decision. Keep old ADRs
unchanged because they record the rationale for an earlier choice.

Do not claim that planned behavior exists. Check the source and its executable
test before you change a live status claim.
