CI and Release Validation
=========================

``.github/workflows/ci.yml`` runs on dev pushes and pull requests to dev or
main. ``tools/ci_scope.py`` selects jobs from the complete changed-path set,
including deletions. Changes to CI policy, unknown executable inputs, contracts,
or reference inputs select all affected validation. Documentation-only changes
retain repository checks and security scans.

Dev feedback
------------

The dev gate runs repository hygiene, static boundary checks, Ruff, mypy,
lock validation, baseline provenance, secret scanning, and infrastructure
security checks. Python changes run the retained data/operator unit tests and
dependency audit. Rust changes run formatting, Clippy, nextest, doctests and
BSL sentinels. Dev CI and ordinary pre-push checks select all library and
binary unit targets plus the integration targets declared by the Rust test
reporter. They use ``mise run rust:check-no-docs -- --dev``. The default command
retains the full workspace test selection. All-target Clippy, doctests, and
BSL sentinels run in both modes. PostgreSQL changes run the pinned fresh-runtime
smoke.

The frozen Python engine and its formula, scenario, regression, coverage-floor
and optimization jobs are retired. Language-neutral vectors, Rust mechanics
contracts, source datasets, and historical evidence remain.

``CI Gate`` waits for every job with ``always()``. It requires success from
every selected job and permits a skipped result only when the scope plan
explicitly deselected that job. A failed prerequisite, absent receipt,
unexpected skip or cancellation fails the gate. Required-check names must
match both ``tools/pr_policy.py`` and ``.github/settings/pr-policy.json``
before this workflow change can be merged. Changing files does not change
GitHub's live rulesets; the policy synchronization is a separate operation.

CodeQL continues on dev and main pull requests, protected-branch pushes,
weekly runs and manual dispatch. Its native code-scanning rule requires zero
open alerts. Gitleaks, Trivy, Python dependency auditing and cargo-deny retain
their existing security responsibilities.

Main qualification
------------------

Pull requests to main select full Rust validation, including documentation,
and all six PostgreSQL focuses: ``runtime_smoke``, ``reference_integrity``,
``runtime``, ``archive``, ``reader``, and ``client``. These retain fresh schema
activation, H3 integrity and rollback, runtime and writer bounds, Archive,
authenticated reader, and live client contracts. Manual CI dispatch also selects
full validation.

``main.yml`` adds the event contract, retained non-unit Python behavior,
PostgreSQL restart/determinism, reference-data contracts, release documentation
and container-image security checks. Reference-data qualification needs the
configured data fixture; an absent fixture cannot count as release evidence.
Documentation warning debt remains explicitly advisory in its build step.

The Director controls main merges. Follow ``docs/agents/governance.md`` for
exact-source qualification, the main-to-dev lineage sync, and release tagging.
``release.yml`` currently creates release notes; it does not yet package the
complete runnable observer session. The session includes the Bevy client,
Rust runtime, launcher and a compatible PostgreSQL service. A client executable
alone is not the complete distribution.

Performance and caches
----------------------

Measure test execution separately from installation, compilation, database
activation and cache transfer. Five-minute targets apply to complete jobs;
a fast test body does not prove the whole job meets its target.

Cargo caches use the pinned toolchain and dependency lock together with
source identity. Pull requests restore compatible protected-branch caches.
Only successful trusted protected-branch runs publish reusable outputs.
PostgreSQL uses the pinned runtime and disposable test databases. Transaction
durability, constraints and authentication remain enabled during performance
measurement.

Pull-request runs cancel when a newer head supersedes them. Protected dev
pushes retain complete evidence for each commit. Rust reports live under
``reports/test-results/rust/``; start with ``latest.json`` and its compact
summary before opening full logs.

Review and merge
----------------

Copilot instructions focus review on causal behavior, authority boundaries,
transaction safety, deterministic identity, and measured performance. Treat
its findings as advisory evidence and check them against the current diff.
Every review thread still requires a disposition and resolution.

Use ``mise run pr:merge -- N`` after authorization and exact-head checks.
The merge helper validates the actual head, required check producers, review
threads and code-scanning alerts. GitHub rulesets preserve the PR requirement
and prevent force pushes and branch deletion.

Local validation
----------------

Run the smallest applicable test first. Use ``mise run check`` for retained
Python tooling, or scoped Cargo tests followed by the applicable native gate.
``mise run rust:check-no-docs`` is the complete local Rust gate. Documentation
generation requires an explicit request. Run heavy Cargo and PostgreSQL jobs
serially using each worktree's own target and environment.
