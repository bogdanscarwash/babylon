Run CI Locally
==============

This guide shows how to run CI checks locally before pushing, using both
direct commands and ``gh act`` for full workflow simulation.

Prerequisites
-------------

- mise installed with pinned Python and uv tools: ``mise install``
- Locked dependencies installed: ``mise run install``
- rustup installed; Cargo commands run from ``rust/`` to use its toolchain pin
- All configured Git hooks installed: ``mise run hooks``
- For ``gh act``: GitHub CLI with act extension (``gh extension install nektos/gh-act``)
- For ``gh act``: Docker running

Quick Checks (Direct Commands)
------------------------------

Run individual CI checks directly:

**Lint check:**

.. code-block:: bash

   uv run ruff check .

   # Auto-fix issues
   uv run ruff check . --fix

**Type check:**

.. code-block:: bash

   uv run mypy src

**Run tests:**

.. code-block:: bash

   # All non-AI tests
   uv run pytest -m "not ai" --tb=short

   # Fast unit tests only
   mise run test:unit

**Check documentation:**

.. code-block:: bash

   # Build docs
   cd docs && uv run sphinx-build -b html . _build/html

   # Strict mode (warnings as errors)
   mise run docs:strict

**Check formatting:**

.. code-block:: bash

   # Check only (no changes)
   uv run ruff format --check .

   # Auto-format
   uv run ruff format .

Using Mise Tasks
----------------

Mise provides convenient task shortcuts:

.. code-block:: bash

   # List all available tasks
   mise tasks

   # Quick CI (lint + format + typecheck + unit tests)
   mise run ci

   # Full test suite
   mise run test:all

   # Type checking only
   mise run typecheck

   # Documentation build
   mise run docs:build

   # Live documentation server (auto-reload)
   mise run docs:live

Full CI Simulation with gh act
------------------------------

``gh act`` runs GitHub Actions workflows locally in Docker, simulating
exactly what CI will do.

**Dry run (see what would execute):**

.. code-block:: bash

   gh act --dryrun

**Run specific job:**

.. code-block:: bash

   # Run the main CI job
   gh act -j ci

   # Run documentation build
   gh act -j docs

   # Run style check
   gh act -j style

**Simulate specific events:**

.. code-block:: bash

   # Simulate push event
   gh act push

   # Simulate pull request event
   gh act pull_request

**Custom event payloads:**

Create ``.github/test-events/pr-to-dev.json``:

.. code-block:: json

   {
     "pull_request": {
       "base": { "ref": "dev" },
       "head": { "ref": "feature/test" }
     }
   }

Then run:

.. code-block:: bash

   gh act pull_request -e .github/test-events/pr-to-dev.json

gh act Limitations
------------------

Some GitHub-specific features don't work locally:

- **Secrets**: Not available (by design—this is a security feature)
- **GitHub API calls**: May fail without proper authentication
- **Caching**: Works differently than GitHub's infrastructure
- **Artifacts**: Upload/download behaves differently

For most development purposes, dry-run validation is sufficient:

.. code-block:: bash

   # Validates YAML syntax and shows execution plan
   gh act --dryrun

Pre-Commit Hooks
----------------

``mise run hooks`` installs all three governed stages: ``pre-commit``,
``commit-msg``, and ``pre-push``. The ordinary commit stage operates on the
staged paths. Pre-commit supplies the exact remote-to-local refs to the push
stage. The range classifier retains deletion-only changes while skipping an
unrelated push without fetching ``dev`` or guessing a merge base.

Run the hooks manually with:

.. code-block:: bash

   # Commit-stage hooks on staged files
   uv run pre-commit run

   # Commit-stage hooks on every tracked file
   uv run pre-commit run --all-files

   # One commit-stage hook
   uv run pre-commit run ruff

   # Push-stage hooks for the current branch range
   uv run pre-commit run --hook-stage pre-push \
     --from-ref origin/dev --to-ref HEAD

The configuration groups checks by feedback cost:

- Commit time: worktree contract, Ruff, MyPy, lock consistency, text and
  structured-file hygiene, documentation lint, actionlint, ShellCheck,
  Hadolint, Gitleaks, and Rust formatting.
- Commit-message time: Commitizen and the baseline-ceremony declaration.
- Push time: the Python smoke and full-tree sentinel checks, Semgrep,
  import boundaries, maintainability, LFS pointers, the baseline range check,
  and the non-documentation Rust gate when its exact inputs changed.
- Meta checks: hook selectors and exclusions must still apply to tracked
  paths. This prevents a deleted estate from leaving inert hooks behind.

Local Rust validation stops before Rustdoc, as repository policy requires:

.. code-block:: bash

   mise run rust:check-no-docs

.. vale off

Rust test reports
-----------------

Install the exact reporting tools once, then use the scoped runner for the
ordinary edit-test loop:

.. code-block:: bash

   mise run rust:test:install-tools
   mise run rust:test:q -- -p babylon-kernel
   mise run rust:test:summary
   mise run rust:test:failed

Each run writes a SHA- and run-qualified directory below
``reports/test-results/rust/`` and updates the explicit ``latest.json``
pointer. ``summary.json`` and ``summary.md`` are the bounded agent interface:
they identify the exact command, source head, toolchain, exit class, outcome
counts, the first 20 exceptional tests, bounded diagnostics, slowest tests,
and rerun commands. ``failures.jsonl`` retains every exceptional test.
``junit.xml`` and ``run.log`` retain complete drill-down evidence. The reporter
still writes a classified summary when compilation or discovery fails before
JUnit exists.

The blocking Rust gate runs one nextest non-doctest pass and then a separate
``cargo test --workspace --doc --locked`` proof. The second leg is required
because nextest does not execute doctests, including compile-fail contracts.
Ignored PostgreSQL tests remain outside the ordinary runner and retain their
existing serial shell-owned execution.

``mise run rust:coverage`` performs a separately instrumented advisory run and
writes compact and full coverage receipts below
``reports/test-results/rust-coverage/``. It is not part of the pull-request
gate and defines no percentage floor until a measured baseline supports one.

.. vale on

CI Job Reference
----------------

The authoritative workflow and required-check inventory changes more often
than this how-to guide. See :doc:`/reference/ci-workflow` for the current
ordinary CI, CodeQL, release-qualification, and branch-enforcement contracts.

Troubleshooting
---------------

**"command not found: gh"**
   Install GitHub CLI: https://cli.github.com/

**"gh act: command not found"**
   Install the act extension:

   .. code-block:: bash

      gh extension install nektos/gh-act

**"Cannot connect to Docker daemon"**
   Ensure Docker is running. On Linux:

   .. code-block:: bash

      sudo systemctl start docker

**MyPy errors on third-party libraries**
   Install type stubs:

   .. code-block:: bash

      uv add --group dev types-requests  # example

**Pre-commit hooks slow**
   Skip hooks for WIP commits:

   .. code-block:: bash

      git commit --no-verify -m "wip: work in progress"

   .. warning::

      Remember to run hooks before final commit!

See Also
--------

- :doc:`/how-to/contribute` - Full contribution workflow
- :doc:`/reference/ci-workflow` - CI technical reference
- :doc:`/reference/configuration` - Configuration options
