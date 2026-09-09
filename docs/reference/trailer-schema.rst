Git Trailer Schema
===================

The machine-checkable grammar for the structured git trailers every agent
commit carries, and the hook estate that enforces the parts of it that are
enforced today. This is git doctrine adoption **item 1** of 5 (v2 roadmap
draft, ``ai/_inbox/archive/tui-roadmap-update.md`` §5.9; approved for Program
24 P1, ``project/roadmap.md`` §4, 2026-07-20): "trailer schema + generated PR
bodies + in-repo hook estate."

.. contents:: On this page
   :local:
   :depth: 2

Why trailers, not a status document
------------------------------------

§5.3 of the draft names the problem this solves: *"History is provenance."*
A commit binds its work to a plan, a train, a pin, and a baseline claim
through structured trailers, so that plan progress, PR bodies, and the
ceremony ledger can all be **derived from the commit record** instead of
hand-maintained beside it — the repository is the shared memory of a system
whose workers (agents, across sessions) have none.

The schema below is what :mod:`tools.trailer_schema` parses and validates,
and what :mod:`tools.generate_pr_body` surfaces into a generated PR body.
Nothing here invents new git machinery: no git notes, no custom ref
namespaces beyond tags (§5.9's own anti-exotica rule) — just conventional
trailer lines, parsed with git's own ``%(trailers:...)`` machinery
(``git log --format``), not a hand-rolled heuristic.

The schema
----------

Every field below is a standard git trailer (a ``Key: value`` line in the
trailing block of a commit message, one per line). The worked example is the
literal §5.3 sample commit:

.. code-block:: text

    Task: 2026-07-18-track1-organizers-map#10
    Train: train/archive-keel
    Lane: pages
    Safety: P
    Pinned: dev@72def41c
    Baselines: untouched
    Session: <run id>
    Co-Authored-By: …

.. list-table::
   :header-rows: 1
   :widths: 15 30 20 35

   * - Trailer
     - Grammar
     - Example
     - Meaning
   * - ``Task``
     - ``<plan-slug>#<task-number>``
     - ``2026-07-18-track1-organizers-map#10``
     - The plan task this commit discharges.
   * - ``Train``
     - ``train/<name>``
     - ``train/archive-keel``
     - The train branch this task's work integrates into.
   * - ``Lane``
     - ``[a-z][a-z0-9-]*``
     - ``pages``
     - Work-order lane label (plan-defined slug; no fixed enum yet).
   * - ``Safety``
     - ``P`` or ``S``
     - ``P``
     - Parallel-safe (``[P]``) or must-run-serial (``[S]``) — the
       ``WORK-ORDERS.md`` work-order legend, carried onto the commit.
   * - ``Pinned``
     - ``<ref>@<sha>``
     - ``dev@72def41c``
     - The base ref + SHA this task branched from — guards the "base-branch
       trap" (branching from a stale ``origin/dev`` instead of the intended
       local HEAD).
   * - ``Baselines``
     - ``untouched`` or ``blessed(<slug>)``
     - ``untouched``
     - Whether this commit touches ``tests/baselines/**``. The
       ``blessed(<slug>)`` form is the existing §6.5 ceremony declaration
       (below) — this schema does not redefine it, only cites it.
   * - ``Session``
     - any non-empty token
     - a run/session id
     - Opaque identifier of the agent (or human) run that produced the
       commit — the "who, at 3 a.m." half of the provenance question.
   * - ``Co-Authored-By``
     - ``Name <email>``
     - ``Claude Fable 5 <noreply@anthropic.com>``
     - Standard GitHub co-author trailer. Used constantly in this repo's
       history already; not itself part of the §5.3 coordination-database
       schema, but recognized by :mod:`tools.trailer_schema` since it
       appears on the same commits.

``Lane``/``Safety`` are shown on one display line in the §5.3 prose sample
for readability; as actual git trailers they are two separate lines (git's
trailer format is one ``Key: value`` per line — a single line cannot carry
two keys).

Required vs. informational
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

``Task``, ``Train``, ``Lane``, ``Safety``, ``Pinned``, ``Baselines``, and
``Session`` are the required §5.3 schema (``tools.trailer_schema.
REQUIRED_TRAILER_KEYS``). ``Co-Authored-By`` is documented for grammar
checking but not required by this schema — it is demanded by convention and
by GitHub's own co-author UI, not by this doctrine.

**What is NOT yet mechanically enforced**: no hook in this repo currently
rejects a commit for *missing* the required trailer set — only the
``Baselines: blessed(<slug>)`` half is enforced today (see `Baselines: the
one enforced trailer`_ below). ``mise run commit`` (``.mise.toml``) already
exists as a **hook-safe commit wrapper** (pre-runs hooks, re-stages fixes,
verifies HEAD moved) but does **not** today inject ``Task``/``Train``/
``Lane``/``Safety``/``Pinned``/``Session`` automatically from worktree-local
environment. The project retired its obsolete worktree helpers. Explicit Git
or Codex worktree creation does not inject these trailers.
This item (1) ships the schema and the generator that *consumes* trailers
where present, not the injection mechanism. Do not read this document as
asserting that every commit in this repository already carries the full
set — it does not, yet.

Baselines: the one enforced trailer
--------------------------------------

The ``Baselines`` trailer's ``blessed(<slug>)`` form is the one part of this
schema with a live enforcement mechanism, predating this item (§6.5,
`PR #226 <https://github.com/percy-raskova/babylon>`__, ``tools/
check_baseline_ceremony.py``): any commit touching ``tests/baselines/**``
must carry ``Baselines: blessed(<ceremony-slug>)`` or be rejected. See
``CLAUDE.md`` "Baseline ceremonies" for the full doctrine; this document's
:mod:`tools.trailer_schema`  mirrors that gate's trailer regex exactly (kept
as a sibling literal, not a shared import, so the two modules have no
runtime dependency on each other) so the two never drift apart. Completing
the *rest* of the blessing mechanization (item 3) is a separate, later
side-task.

Generated PR bodies
--------------------

``tools/generate_pr_body.py`` (wired as ``mise run pr:body``) reads every
non-merge commit in a range (default ``origin/dev..HEAD``), and renders:

1. **Commits grouped by conventional-commit type, then scope** — in
   Conventional-Commits-spec order (``feat``, ``fix``, ``refactor``,
   ``perf``, ``docs``, ``test``, ``build``, ``ci``, ``chore``, ``style``,
   ``revert``, ``other``), oldest commit first within each group so the
   output is a stable function of the range's content.
2. **A Provenance section** — every declared trailer key, with the sorted,
   de-duplicated set of values seen across the range (e.g. every distinct
   ``Task`` id touched, every distinct ``Baselines`` state declared).
3. **A Trailer warnings section** (only present if triggered) — any trailer
   value that fails its schema grammar above. Missing trailers are *not*
   flagged here (a range legitimately mixes commits that predate the full
   trailer convention); only malformed *present* values are.
4. **The standard footer** — every generated body ends with::

       ---
       *Generated by `tools/generate_pr_body.py` (git doctrine §5.9 item 1 —
       trailer schema + generated PR bodies).*

   so a reader can tell a generated body from a hand-written one at a
   glance. This is a new convention established by this item — there was no
   pre-existing "generated-with" footer text in this repository before it
   (only the ``Co-Authored-By`` commit trailer, which is a different
   thing). Change the exact footer text and this paragraph together if it
   ever needs to change.

This tool does **not** implement ``train:pr`` (the full train-integration
generator §5.3 describes, which also consumes rulings and writes directly to
a PR via ``gh``) — that machinery, and the reservations/single-flight-lock
items (4-5), activate only when the first fan-out plan demands them
(``project/roadmap.md`` §4). ``generate_pr_body.py`` is the standalone,
composable piece: point it at a range, get a body, paste it (or pipe it to
``gh pr create --body-file``) yourself.

Hook estate: local hooks vs. their CI leg
--------------------------------------------

All hooks live in-repo, in ``.pre-commit-config.yaml`` — no hook lives only
in a developer's global git config or a CI-only script with no local
equivalent. The table below is the audit this item's DoD asks for: which
CI job (if any) is the **authoritative** re-run of each local hook, per
``.github/workflows/ci.yml`` and ``main.yml``.

.. list-table::
   :header-rows: 1
   :widths: 22 12 45 21

   * - Local hook (``.pre-commit-config.yaml``)
     - Stage
     - Authoritative CI leg
     - Notes
   * - ``worktree-contract`` / pre-commit meta checks
     - pre-commit
     - Unit Tests (selector-policy contract only)
     - The local contract validates workstation state. The meta hooks and unit
       contract reject selectors that no longer match any tracked path.
   * - ``ruff`` / ``ruff-format``
     - pre-commit
     - Fast Gate — "Lint (ruff check)" / "Format (ruff format --check)"
     - Same command family (``mise run lint:check`` / ``format:check``).
   * - ``mypy``
     - pre-commit (files-scoped)
     - Fast Gate — "Type check (mypy strict)"
     - Local hook and CI both run ``mypy src`` (``mise run typecheck``).
   * - ``pytest-fast``
     - pre-push
     - Unit Tests (xdist, coverage gate)
     - Explicitly labeled "NOT CI-predictive" in the hook itself — it is a
       fast local smoke check (2 dirs, ``-m "not red_phase and not slow"``),
       not a subset of the full CI unit shard; green locally does not
       guarantee green in CI.
   * - ``import-boundaries``
     - pre-push
     - Fast Gate — "Import-boundary contracts"
     - Same command (``lint-imports`` / ``mise run lint:imports``).
   * - ``sentinels-static`` / ``surface-baseline``
     - pre-push
     - Fast Gate — static sentinels / public-surface baseline
     - The local hooks run the same full-tree Mise tasks as hosted CI.
   * - ``rust-fmt-check`` / ``rust-full-gate``
     - pre-commit / pre-push
     - Rust Gate
     - Formatting runs on Rust source or formatter/toolchain policy. The push
       hook re-diffs pre-commit's exact remote-to-local refs so deleted paths
       still select ``rust:check-no-docs``; hosted CI adds the Rustdoc proof.
   * - ``bsl-repo-sentinels``
     - pre-push
     - Rust Gate
     - Covers the non-Rust inputs to the Rust-owned repository sentinels:
       ``ai/decisions`` and all eight exact retired Python authority paths.
       The deletion-aware range classifier owns both hook selections.
   * - ``uv-lock-consistency``
     - pre-commit (files-scoped)
     - Fast Gate — "uv lock consistency"
     - Both run ``uv lock --check``.
   * - ``baseline-ceremony`` / ``baseline-ceremony-range``
     - commit-msg / pre-push
     - Baseline Ceremony Gate (§6.5 provenance)
     - Local leg is explicitly best-effort (``--amend``/pathspec commits can
       slip past it, per the gate's own docstring); the local and CI range
       legs close that gap over the full pushed or PR range.
   * - ``gitleaks``
     - pre-commit
     - Secret Scan (gitleaks, full history)
     - Same tool; CI scans full history, the local hook scans the commit.
   * - ``commitizen``
     - commit-msg
     - **none found**
     - No CI job re-validates conventional-commit subject grammar; this is
       local-only today.
   * - ``radon-mi``
     - pre-push
     - **none found**
     - No CI job runs ``radon`` in ``ci.yml``/``main.yml``.
   * - ``semgrep``
     - pre-push
     - **none found**
     - ``mise run qa:patterns`` wraps the same check but is not wired into
       any CI workflow found in ``.github/workflows/``.
   * - ``rstcheck`` / ``doc8``
     - pre-commit
     - **none found** (adjacent: Documentation Build)
     - ``docs.yml`` builds Sphinx on push to ``main`` (not on PRs, and
       without ``-W``/warnings-as-errors) — it would surface a broken
       ``toctree`` but not the RST-style issues these hooks catch.
   * - ``markdownlint``
     - pre-commit
     - **none found**
     - Local-only.
   * - ``actionlint`` / ``shellcheck`` / ``hadolint-docker``
     - pre-commit
     - **none found**
     - Local-only; no workflow lints workflows, shell scripts, or the
       Dockerfile in CI today.
   * - ``lfs-pointer-check``
     - pre-push
     - **none found**
     - Local-only.
   * - trailing-whitespace / end-of-file-fixer / check-yaml / check-json /
       check-toml / check-added-large-files / check-merge-conflict /
       check-case-conflict / forbid-new-submodules / no-commit-to-branch
     - pre-commit
     - Fast Gate — "Repo hygiene gate" (partial overlap only)
     - ``check:hygiene`` (``tools/check_repo_hygiene.py``) independently
       re-checks the root allowlist, tracked-ignored files, and blob size —
       overlapping concerns, not the same tool re-run.

**Reading this table**: "none found" means exactly that — a targeted search
of every ``.github/workflows/*.yml`` job step for the tool's name found no
match, not that the check is unimportant. Per this codebase's documentation
philosophy (verifiability over comprehensiveness), this table records what
was actually found rather than asserting CI parity that doesn't exist. If a
"none found" gap should be closed, that is a separate, explicitly-scoped
follow-up — not silently assumed away here.
