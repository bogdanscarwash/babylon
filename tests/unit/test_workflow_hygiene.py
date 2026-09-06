"""Workflow-file hygiene: catch GitHub-invalid YAML that plain YAML accepts,
plus the scheduled-workflow process classes (ADR181 R9b).

Sentinel for the error class discovered 2026-07-27: commit ``e240a30f``
inserted a ``run:`` step *between* ``- uses: actions/checkout@v7`` and its
``with:`` block in every ``nightly.yml`` job. The result still parses as
YAML (so no local tool objected), but GitHub's workflow validator rejects a
step carrying both ``run:`` and ``with:`` — every push to any branch then
spawned a zero-job stub failure run, and the nightly schedule was dead from
2026-07-22 until the fix. The checkout also silently lost its ``ref: dev``.

Four invariants, one per failure mode:

1. Every step in every workflow declares exactly one of ``run:`` / ``uses:``
   (``with:`` only ever accompanies ``uses:``).
2. Every ``actions/checkout`` step in the scheduled deep-leg workflows
   (``nightly-*.yml`` / ``weekly-*.yml``, the ADR181 R3 split) pins
   ``ref: dev`` — scheduled workflows execute the file from the default
   branch, so an unpinned checkout tests the wrong ref without erroring on a
   dispatch from a non-default ref.
3. Every workflow carrying a ``schedule:`` trigger also declares
   ``workflow_dispatch`` — a cron-only workflow cannot be proof-run, which
   is how the monolithic nightly stayed red 76/76 without a diagnosis loop
   (this is the statically-decidable half of the audit's
   "scheduled workflow must exist on the default branch" rule; the other
   half is not decidable from a PR checkout without network access and is
   enforced by the merge flow itself).
4. Every ``.github/workflows/*.yml`` path referenced in the LIVE doc
   surfaces exists in ``git ls-files`` — the ``openwiki-update.yml`` class:
   docs asserting a workflow that was never committed (a Verifiability
   violation). Historical records (ADRs, reports, plans) are exempt —
   immutability of history.
"""

from __future__ import annotations

import os
import re
import subprocess
import tomllib
from pathlib import Path
from typing import Any

import pytest
import yaml

WORKFLOWS_DIR = Path(".github/workflows")
ACTIONS_DIR = Path(".github/actions")
FROZEN_ENGINE_PATH = WORKFLOWS_DIR / "frozen-engine.yml"
WEEKLY_PY313_PATH = WORKFLOWS_DIR / "weekly-py313.yml"
DEPENDABOT_AUTOMERGE_PATH = WORKFLOWS_DIR / "dependabot-automerge.yml"
DEPENDABOT_CONFIG_PATH = Path(".github/dependabot.yml")
DEPENDABOT_SCHEDULE_TIMEZONE = "America/New_York"
PR_POLICY_PATH = Path(".github/settings/pr-policy.json")
FROZEN_REF = "p27-python-freeze"
HYPERGRAPH_REF = "dc1c06abbbc7a3f8633d1561451e61e101ad2090"

#: Hand-maintained doc surfaces whose workflow references must stay live.
#: Historical quadrants (ai/decisions, reports/, project/, docs/superpowers/
#: plans) are deliberately absent; openwiki/ is generated, never hand-edited.
LIVE_DOC_SURFACES: tuple[str, ...] = (
    "CLAUDE.md",
    "CONTRIBUTORS.md",
    "README.md",
    "NORTH_STAR.md",
    "tests/README.md",
)

_WORKFLOW_REF_RE = re.compile(r"\.github/workflows/([A-Za-z0-9._-]+\.ya?ml)")
_V3_1_COMMIT = "3acd1089b6b4e68177c99b4f4cec245e7b74317c"
_V3_1_BLOB = "a265b85120ed2a90be40c72e63ee5bf27fc6e703"
_V3_2_COMMIT = "cbfc67921283ccb6e00c4b0278288a232281440a"
_V3_2_BLOB = "e905e90d66bddc6e4eca36a3896428f5ce63de5b"
_CONSTITUTION_FETCH_STEP = "Fetch pinned Constitution predecessors (bounded)"
_MISE_ACTION = "jdx/mise-action@c2a87611a18de5b3828c5652fe268e992400cb5c"
_PY313_SYNC = "uv sync --frozen --extra ops --python 3.13"
_ACTION_USES_LINE = re.compile(
    r"^\s*(?:-\s+)?uses:\s+(?P<reference>[^\s#]+)(?:\s+#\s*(?P<tag>\S+))?\s*$"
)
_ACTION_SHA = re.compile(r"[0-9a-f]{40}")
_RELEASE_TAG = re.compile(r"v\d+(?:\.\d+(?:\.\d+)?)?")


def _triggers(workflow: dict[Any, Any]) -> dict[str, Any]:
    """Return the ``on:`` mapping (YAML 1.1 parses the bare key as ``True``)."""
    raw = workflow.get("on", workflow.get(True))
    return raw if isinstance(raw, dict) else {}


def _workflow_path_refs(text: str) -> set[str]:
    """Extract referenced workflow basenames from a doc's text."""
    return set(_WORKFLOW_REF_RE.findall(text))


def _step_shape_errors(workflow: dict[str, Any], filename: str) -> list[str]:
    """Return one message per step whose run/uses/with combination GitHub rejects.

    :param workflow: Parsed workflow mapping (``yaml.safe_load`` output).
    :param filename: Display name used in the error messages.
    :returns: Human-readable violation messages; empty when the file is clean.
    """
    errors: list[str] = []
    jobs = workflow.get("jobs") or {}
    for job_name, job in jobs.items():
        for index, step in enumerate(job.get("steps") or []):
            has_run = "run" in step
            has_uses = "uses" in step
            where = f"{filename} job={job_name} step#{index}"
            if has_run and has_uses:
                errors.append(f"{where}: step has both 'run' and 'uses'")
            elif has_run and "with" in step:
                errors.append(f"{where}: 'with' on a 'run' step (GitHub rejects this)")
            elif not has_run and not has_uses:
                errors.append(f"{where}: step has neither 'run' nor 'uses'")
    return errors


def _workflow_paths() -> list[Path]:
    """Return every live GitHub workflow manifest."""
    return sorted(WORKFLOWS_DIR.glob("*.yml")) + sorted(WORKFLOWS_DIR.glob("*.yaml"))


def _automation_paths() -> list[Path]:
    """Return live workflow and composite-action files."""
    return _workflow_paths() + sorted(ACTIONS_DIR.rglob("action.y*ml"))


def _automation_step_locations(automation: dict[str, Any]) -> list[tuple[str, dict[str, Any]]]:
    """Return stable source identifiers and executable steps."""
    jobs = automation.get("jobs")
    if isinstance(jobs, dict):
        return [
            (f"job={job_name} step#{index}", step)
            for job_name, job in jobs.items()
            for index, step in enumerate(job.get("steps") or [])
        ]
    return [
        (f"composite step#{index}", step)
        for index, step in enumerate((automation.get("runs") or {}).get("steps") or [])
    ]


def _sibling_fabrication_errors(automation: dict[str, Any], filename: str) -> list[str]:
    """Reject executable construction of a local hypergraph sibling."""
    errors: list[str] = []
    for location, step in _automation_step_locations(automation):
        run = str(step.get("run", ""))
        fabricates_sibling = "hypergraph-rs" in run and any(
            command in run for command in ("mkdir", "ln -s", "cp ", "cat >", "tee ")
        )
        if "ci_hypergraph_stub" in run or fabricates_sibling:
            errors.append(f"{filename} {location}: fabricates hypergraph-rs sibling")
    return errors


def test_sibling_fabrication_errors_name_workflow_job_and_step() -> None:
    """A workflow violation must identify the exact executable source step."""
    broken = yaml.safe_load(
        """
        jobs:
          materialize:
            steps:
              - run: mkdir -p ../hypergraph-rs
        """
    )

    assert _sibling_fabrication_errors(broken, ".github/workflows/future.yaml") == [
        ".github/workflows/future.yaml job=materialize step#0: fabricates hypergraph-rs sibling"
    ]


def test_sibling_fabrication_errors_name_composite_step() -> None:
    """A composite violation must identify its action file and executable step."""
    broken = yaml.safe_load(
        """
        runs:
          using: composite
          steps:
            - run: ln -s ../hypergraph-rs hypergraph-rs
        """
    )

    assert _sibling_fabrication_errors(broken, ".github/actions/future/action.yml") == [
        ".github/actions/future/action.yml composite step#0: fabricates hypergraph-rs sibling"
    ]


def test_automation_paths_include_yaml_workflows(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """A valid .yaml workflow must receive the same automation scan as .yml."""
    workflow_directory = tmp_path / "workflows"
    workflow_directory.mkdir()
    yaml_workflow = workflow_directory / "sibling-fabrication.yaml"
    yaml_workflow.write_text("jobs: {}\n")
    monkeypatch.setitem(globals(), "WORKFLOWS_DIR", workflow_directory)

    assert yaml_workflow in _automation_paths()


def _frozen_engine_errors(workflow: dict[str, Any]) -> list[str]:
    """Return violations in the immutable frozen-engine checkout contract."""
    errors: list[str] = []
    steps = ((workflow.get("jobs") or {}).get("frozen-canon") or {}).get("steps") or []
    checkouts = {
        (
            str(step.get("with", {}).get("repository", "")),
            str(step.get("with", {}).get("ref", "")),
        ): str(step.get("with", {}).get("path", ""))
        for step in steps
        if str(step.get("uses", "")).startswith("actions/checkout")
    }
    if checkouts.get(("", FROZEN_REF)) != "babylon":
        errors.append("frozen source must check out at babylon")
    if checkouts.get(("percy-raskova/hypergraph-rs", HYPERGRAPH_REF)) != "hypergraph-rs":
        errors.append("historical hypergraph source must use its full pinned SHA")
    mise_setup_index = next(
        (
            index
            for index, step in enumerate(steps)
            if str(step.get("uses", "")).startswith("jdx/mise-action@")
        ),
        None,
    )
    for index, step in enumerate(steps):
        run = str(step.get("run", ""))
        if "mise run" in run and (mise_setup_index is None or mise_setup_index >= index):
            errors.append(f"frozen mise step#{index} must follow jdx/mise-action")
        if "mise run" not in run and "uv sync" not in run:
            continue
        if step.get("working-directory") != "babylon":
            errors.append(f"frozen command step#{index} must run in babylon")
        if str((step.get("env") or {}).get("UV_FROZEN", "")).lower() not in {"1", "true"}:
            errors.append(f"frozen command step#{index} must set UV_FROZEN")
    return errors


def _frozen_engine_external_action_errors(workflow_text: str) -> list[str]:
    """Return mutable or unannotated external action references in the frozen gate."""
    return _external_action_reference_errors(workflow_text, "frozen-engine.yml")


def _external_action_reference_errors(workflow_text: str, filename: str) -> list[str]:
    """Return mutable or unannotated external action references."""
    errors: list[str] = []
    for line_number, line in enumerate(workflow_text.splitlines(), start=1):
        match = _ACTION_USES_LINE.match(line)
        if match is None:
            continue
        action, separator, reference = match.group("reference").partition("@")
        if action.startswith("./"):
            continue
        if not separator or not _ACTION_SHA.fullmatch(reference):
            errors.append(f"{filename}:{line_number}: external action must use a 40-hex SHA")
        tag = match.group("tag")
        if tag is None or not _RELEASE_TAG.fullmatch(tag):
            errors.append(f"{filename}:{line_number}: external action must have a # vN tag")
    return errors


@pytest.mark.skipif(not WORKFLOWS_DIR.is_dir(), reason=".github/workflows not present")
class TestWorkflowStepShape:
    """Every workflow step is GitHub-valid, not merely YAML-valid."""

    def test_mise_installers_match_repository_minimum(self) -> None:
        """Hosted task runners must satisfy the same exact pin as local tasks."""
        required = tomllib.loads(Path(".mise.toml").read_text())["min_version"]
        violations: list[str] = []
        installers = 0
        for path in _automation_paths():
            automation = yaml.safe_load(path.read_text())
            for location, step in _automation_step_locations(automation):
                if not str(step.get("uses", "")).startswith("jdx/mise-action@"):
                    continue
                installers += 1
                version = (step.get("with") or {}).get("version")
                if version != required:
                    violations.append(
                        f"{path} {location}: mise version={version!r}; required={required!r}"
                    )
        assert installers > 0, "no hosted mise installers were checked"
        assert not violations, "\n".join(violations)

    def test_no_workflow_materializes_a_hypergraph_sibling(self) -> None:
        """Python CI must not depend on a fabricated local checkout."""
        violations: list[str] = []
        for path in _automation_paths():
            violations.extend(
                _sibling_fabrication_errors(yaml.safe_load(path.read_text()), str(path))
            )
        assert not violations, "\n".join(violations)

    def test_frozen_engine_supplies_its_immutable_historical_sibling(self) -> None:
        """The frozen tag gets its real historical path source, never a fabricated one."""
        assert _frozen_engine_errors(yaml.safe_load(FROZEN_ENGINE_PATH.read_text())) == []

    def test_frozen_engine_external_actions_are_sha_pinned_and_release_annotated(self) -> None:
        """Frozen gate action references must be immutable and auditably versioned."""
        errors = _frozen_engine_external_action_errors(FROZEN_ENGINE_PATH.read_text())
        assert not errors, "\n".join(errors)

    def test_no_step_mixes_run_and_with(self) -> None:
        violations: list[str] = []
        for path in _workflow_paths():
            workflow = yaml.safe_load(path.read_text())
            violations.extend(_step_shape_errors(workflow, path.name))
        assert not violations, "\n".join(violations)

    def test_no_step_mixes_run_and_with_scans_yaml_workflows(
        self, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
    ) -> None:
        """The workflow-wide shape guard must reject the same bad shape in .yaml."""
        workflow_directory = tmp_path / "workflows"
        workflow_directory.mkdir()
        (workflow_directory / "future.yaml").write_text(
            """
            jobs:
              test:
                steps:
                  - run: mise run test:q
                    with:
                      ref: dev
            """
        )
        monkeypatch.setitem(globals(), "WORKFLOWS_DIR", workflow_directory)

        with pytest.raises(AssertionError, match=r"future\.yaml job=test step#0"):
            self.test_no_step_mixes_run_and_with()

    def test_checker_catches_the_e240a30f_breakage(self) -> None:
        # Mutation validation: the exact historical bad shape must be flagged.
        broken = yaml.safe_load(
            """
            jobs:
              test-rest:
                steps:
                  - uses: actions/checkout@v7
                  - name: Run setup
                    run: mise run setup
                    with:
                      ref: dev
            """
        )
        errors = _step_shape_errors(broken, "nightly.yml")
        assert errors == [
            "nightly.yml job=test-rest step#1: 'with' on a 'run' step (GitHub rejects this)"
        ]


def _unpinned_checkouts(workflow: dict[str, Any], filename: str) -> list[str]:
    """Return one message per ``actions/checkout`` step not pinning ``ref: dev``."""
    violations: list[str] = []
    for job_name, job in (workflow.get("jobs") or {}).items():
        for index, step in enumerate(job.get("steps") or []):
            uses = str(step.get("uses", ""))
            if not uses.startswith("actions/checkout"):
                continue
            ref = (step.get("with") or {}).get("ref")
            if ref != "dev":
                violations.append(f"{filename} job={job_name} step#{index}: checkout ref={ref!r}")
    return violations


def _constitution_provenance_errors(workflow: dict[str, Any]) -> list[str]:
    """Return violations in the unit job's bounded predecessor supply contract."""
    errors: list[str] = []
    jobs = workflow.get("jobs") or {}
    job = jobs.get("test-unit") or {}
    steps = job.get("steps") or []
    checkout_index = next(
        (
            index
            for index, step in enumerate(steps)
            if str(step.get("uses", "")).startswith("actions/checkout")
        ),
        None,
    )
    unit_index = next(
        (index for index, step in enumerate(steps) if step.get("run") == "mise run test:unit-ci"),
        None,
    )
    fetch_index = next(
        (index for index, step in enumerate(steps) if step.get("name") == _CONSTITUTION_FETCH_STEP),
        None,
    )
    if checkout_index is None:
        return ["test-unit has no actions/checkout step"]
    checkout_with = steps[checkout_index].get("with") or {}
    if checkout_with.get("persist-credentials") is not True:
        errors.append("test-unit checkout must persist credentials for the bounded fetch")
    if checkout_with.get("fetch-depth") == 0:
        errors.append("test-unit checkout must stay shallow, never fetch-depth 0")
    if fetch_index is None:
        errors.append("test-unit has no bounded Constitution predecessor fetch")
        return errors
    if unit_index is None or not checkout_index < fetch_index < unit_index:
        errors.append("bounded predecessor fetch must run after checkout and before unit tests")

    fetch_step = steps[fetch_index]
    if fetch_step.get("shell") != "bash":
        errors.append("bounded predecessor fetch must declare shell: bash")

    run = str(fetch_step.get("run", ""))
    run_lines = [line.strip() for line in run.splitlines() if line.strip()]
    if not run_lines or run_lines[0] != "set -euo pipefail":
        errors.append("bounded predecessor fetch must start with set -euo pipefail")
    normalized = " ".join(run.replace("\\\n", " ").split())
    required_fragments = (
        "git -c protocol.version=2 fetch",
        "--depth=1 --no-tags --prune --no-recurse-submodules origin",
        f'git rev-parse {_V3_1_COMMIT}:CONSTITUTION.md)" = "{_V3_1_BLOB}"',
        f'git rev-parse {_V3_2_COMMIT}:CONSTITUTION.md)" = "{_V3_2_BLOB}"',
    )
    for fragment in required_fragments:
        if fragment not in normalized:
            errors.append(f"bounded predecessor fetch missing {fragment!r}")
    required_refspecs = (
        f"+{_V3_1_COMMIT}:refs/remotes/origin/constitution-v3.1",
        f"+{_V3_2_COMMIT}:refs/remotes/origin/constitution-v3.2",
    )
    run_tokens = normalized.split()
    for refspec in required_refspecs:
        if refspec not in run_tokens:
            errors.append(f"bounded predecessor fetch missing forced refspec {refspec!r}")
    return errors


@pytest.mark.skipif(not WORKFLOWS_DIR.is_dir(), reason=".github/workflows not present")
class TestScheduledWorkflows:
    """The scheduled estate's shape rules (invariants 2 and 3)."""

    def test_deep_leg_checkouts_pin_dev(self) -> None:
        deep_legs = sorted(WORKFLOWS_DIR.glob("nightly-*.yml")) + sorted(
            WORKFLOWS_DIR.glob("weekly-*.yml")
        )
        assert deep_legs, "the ADR181 R3 per-leg split produced no deep-leg workflows"
        violations: list[str] = []
        for path in deep_legs:
            workflow = yaml.safe_load(path.read_text())
            violations.extend(_unpinned_checkouts(workflow, path.name))
        assert not violations, "\n".join(violations)

    def test_every_scheduled_workflow_is_dispatchable(self) -> None:
        violations: list[str] = []
        for path in sorted(WORKFLOWS_DIR.glob("*.yml")):
            triggers = _triggers(yaml.safe_load(path.read_text()))
            if "schedule" in triggers and "workflow_dispatch" not in triggers:
                violations.append(f"{path.name}: schedule without workflow_dispatch")
        assert not violations, "\n".join(violations)

    def test_python313_leg_has_bounded_provenance_and_uv_only_mise_bootstrap(self) -> None:
        """The full forward-compat suite gets history and tools without broad fetches."""
        weekly = yaml.safe_load(WEEKLY_PY313_PATH.read_text())
        ci = yaml.safe_load((WORKFLOWS_DIR / "ci.yml").read_text())
        steps = weekly["jobs"]["py313-forward-compat"]["steps"]

        checkout = next(
            step for step in steps if str(step.get("uses", "")).startswith("actions/checkout")
        )
        assert checkout["with"] == {
            "ref": "dev",
            "fetch-depth": 1,
            "persist-credentials": True,
        }

        ci_fetch = next(
            step
            for step in ci["jobs"]["test-unit"]["steps"]
            if step.get("name") == _CONSTITUTION_FETCH_STEP
        )
        weekly_fetch = next(step for step in steps if step.get("name") == _CONSTITUTION_FETCH_STEP)
        assert weekly_fetch == ci_fetch

        mise = next(step for step in steps if step.get("uses") == _MISE_ACTION)
        required = tomllib.loads(Path(".mise.toml").read_text())["min_version"]
        assert mise.get("with") == {"version": required, "install_args": "uv"}
        assert not any("astral-sh/setup-uv" in str(step.get("uses", "")) for step in steps)

        fetch_index = steps.index(weekly_fetch)
        mise_index = steps.index(mise)
        sync_index = next(
            index for index, step in enumerate(steps) if step.get("run") == _PY313_SYNC
        )
        assert fetch_index < mise_index < sync_index

    def test_checker_catches_an_unpinned_deep_leg_checkout(self) -> None:
        # Mutation validation: the e240a30f ref-loss shape must be flagged.
        broken = yaml.safe_load(
            """
            jobs:
              test-rest:
                steps:
                  - uses: actions/checkout@v7
            """
        )
        assert _unpinned_checkouts(broken, "weekly-test-rest.yml") == [
            "weekly-test-rest.yml job=test-rest step#0: checkout ref=None"
        ]

    def test_checker_catches_a_cron_only_workflow(self) -> None:
        # Mutation validation: yaml parses bare `on:` as the boolean True key.
        broken = yaml.safe_load(
            """
            on:
              schedule:
                - cron: "0 6 * * 3"
            jobs: {}
            """
        )
        triggers = _triggers(broken)
        assert "schedule" in triggers and "workflow_dispatch" not in triggers


@pytest.mark.skipif(not WORKFLOWS_DIR.is_dir(), reason=".github/workflows not present")
class TestConstitutionProvenanceSupply:
    """The unit job gets exact predecessor blobs without a full-history checkout."""

    def test_unit_job_fetches_exact_constitution_predecessors_before_tests(self) -> None:
        workflow = yaml.safe_load((WORKFLOWS_DIR / "ci.yml").read_text())
        assert _constitution_provenance_errors(workflow) == []

    def test_checker_catches_an_unbounded_or_incomplete_fetch(self) -> None:
        broken = yaml.safe_load(
            f"""
            jobs:
              test-unit:
                steps:
                  - uses: actions/checkout@v7
                    with:
                      fetch-depth: 0
                  - name: {_CONSTITUTION_FETCH_STEP}
                    run: git fetch origin {_V3_2_COMMIT}
                  - run: mise run test:unit-ci
            """
        )
        errors = _constitution_provenance_errors(broken)
        assert "test-unit checkout must persist credentials for the bounded fetch" in errors
        assert "test-unit checkout must stay shallow, never fetch-depth 0" in errors
        assert "bounded predecessor fetch must declare shell: bash" in errors
        assert "bounded predecessor fetch must start with set -euo pipefail" in errors
        assert any("protocol.version=2" in error for error in errors)
        assert any(
            "--depth=1 --no-tags --prune --no-recurse-submodules" in error for error in errors
        )
        assert any("forced refspec" in error and "constitution-v3.1" in error for error in errors)
        assert any("CONSTITUTION.md" in error and _V3_1_BLOB in error for error in errors)


@pytest.mark.skipif(not WORKFLOWS_DIR.is_dir(), reason=".github/workflows not present")
class TestDocReferencedWorkflowsTracked:
    """Invariant 4: live docs never assert a workflow git does not track."""

    def test_referenced_workflows_are_tracked(self) -> None:
        tracked = set(
            subprocess.run(  # noqa: S603
                ["git", "ls-files", "--", ".github/workflows"],  # noqa: S607
                capture_output=True,
                text=True,
                check=True,
            ).stdout.split()
        )
        tracked_names = {Path(p).name for p in tracked}
        violations: list[str] = []
        for doc in LIVE_DOC_SURFACES:
            doc_path = Path(doc)
            if not doc_path.is_file():
                continue
            for name in sorted(_workflow_path_refs(doc_path.read_text())):
                if name not in tracked_names:
                    violations.append(f"{doc}: references untracked workflow {name}")
        assert not violations, "\n".join(violations)

    def test_extractor_catches_the_openwiki_class(self) -> None:
        # Mutation validation: a doc referencing a never-committed workflow.
        refs = _workflow_path_refs(
            "The scheduled workflow (.github/workflows/openwiki-update.yml) refreshes the wiki."
        )
        assert refs == {"openwiki-update.yml"}


def _dependabot_update(config: dict[str, Any], ecosystem: str) -> dict[str, Any]:
    """Return one Dependabot ecosystem entry, requiring an unambiguous match."""
    updates = config.get("updates") or []
    assert len(updates) == 4, f"expected four ecosystem entries, got {len(updates)}"
    matches = [
        updates[index] for index in range(4) if updates[index].get("package-ecosystem") == ecosystem
    ]
    assert len(matches) == 1, f"expected one {ecosystem!r} update entry, got {len(matches)}"
    return matches[0]


@pytest.mark.skipif(not WORKFLOWS_DIR.is_dir(), reason=".github/workflows not present")
class TestDependabotPolicy:
    """Dependabot metadata and merge authority stay separate and exact-head pinned."""

    def test_workflow_uses_only_the_trusted_exact_head_phase(self) -> None:
        """Dependabot actor events must not own a write-capable automation phase."""
        workflow = yaml.safe_load(DEPENDABOT_AUTOMERGE_PATH.read_text())
        triggers = _triggers(workflow)
        assert triggers == {
            "workflow_run": {
                "workflows": ["CI"],
                "types": ["completed"],
            }
        }
        assert workflow.get("permissions") == {}
        assert set(workflow["jobs"]) == {"merge"}

        merge = workflow["jobs"]["merge"]
        assert merge["name"] == "Dependabot Eligibility"
        assert merge["permissions"] == {
            "actions": "read",
            "checks": "read",
            "contents": "write",
            "pull-requests": "write",
            "security-events": "read",
        }
        assert "github.event.workflow_run.conclusion == 'success'" in merge["if"]
        assert "github.event.workflow_run.event == 'pull_request'" in merge["if"]
        assert "github.event.workflow_run.name == 'CI'" in merge["if"]
        run_name = str(workflow["run-name"])
        assert "github.event.workflow_run.id" in run_name
        assert "github.event.workflow_run.head_sha" in run_name

    def test_workflow_has_per_pr_concurrency(self) -> None:
        """Duplicate completion events must serialize on the same Dependabot PR."""
        workflow = yaml.safe_load(DEPENDABOT_AUTOMERGE_PATH.read_text())
        concurrency = workflow["concurrency"]
        group = str(concurrency["group"])
        assert "github.event.workflow_run.pull_requests[0].number" in group
        assert "github.event.workflow_run.head_branch" in group
        assert concurrency["cancel-in-progress"] is False

    def test_update_classification_is_owned_only_by_the_exact_head_verifier(self) -> None:
        """Presentation labels cannot become an actor-triggered write dependency."""
        workflow_text = DEPENDABOT_AUTOMERGE_PATH.read_text()
        assert "pull_request_target" not in workflow_text
        assert "dependabot/fetch-metadata" not in workflow_text
        assert "dependencies:automerge" not in workflow_text
        assert "issues/" not in workflow_text
        assert "--dependabot-source-run" in workflow_text
        assert "--dependabot-classifier-run" in workflow_text

    def test_merge_uses_trusted_dev_tools_and_exact_candidate_head(self) -> None:
        """A moved, non-Dependabot, or ambiguous PR must never merge."""
        workflow = yaml.safe_load(DEPENDABOT_AUTOMERGE_PATH.read_text())
        merge = workflow["jobs"]["merge"]
        checkout = next(
            step
            for step in merge["steps"]
            if str(step.get("uses", "")).startswith("actions/checkout")
        )
        assert checkout["with"] == {"ref": "dev", "persist-credentials": False}

        candidate = next(step for step in merge["steps"] if step.get("id") == "candidate")
        candidate_script = str(candidate["run"])
        assert "base=dev" in candidate_script
        assert '.user.login == "dependabot[bot]"' in candidate_script
        assert ".user.id == 49699333" in candidate_script
        assert '.user.type == "Bot"' in candidate_script
        assert ".head.sha == $head" in candidate_script
        assert ".name == $label" not in candidate_script
        assert "pull_count=\"$(jq 'length'" in candidate_script
        assert 'if [ "$pull_count" -ge 100 ]' in candidate_script
        assert "candidate_count" in candidate_script and "-ne 1" in candidate_script
        assert candidate["env"]["EXPECTED_HEAD"] == "${{ github.event.workflow_run.head_sha }}"

        merge_step = next(step for step in merge["steps"] if step.get("name") == "Merge")
        assert merge_step["if"] == "steps.candidate.outputs.eligible == 'true'"
        assert merge_step["env"]["EXPECTED_HEAD"] == "${{ github.event.workflow_run.head_sha }}"
        merge_script = str(merge_step["run"])
        assert 'python3 tools/pr_merge.py "$PR_NUMBER" --expected-head "$EXPECTED_HEAD"' in (
            merge_script
        )
        assert '--dependabot-source-run "$SOURCE_RUN_ID"' in merge_script
        assert '--dependabot-classifier-run "$CLASSIFIER_RUN_ID"' in merge_script

    def test_dependabot_workflow_external_actions_are_immutable_and_annotated(self) -> None:
        """A mutable tag must never select code inside the privileged workflow."""
        errors = _external_action_reference_errors(
            DEPENDABOT_AUTOMERGE_PATH.read_text(),
            DEPENDABOT_AUTOMERGE_PATH.name,
        )
        assert not errors, "\n".join(errors)

    def test_external_action_checker_rejects_a_mutable_release_tag(self) -> None:
        """Mutation witness: a release-looking tag is still a mutable reference."""
        errors = _external_action_reference_errors(
            "steps:\n  - uses: actions/checkout@v7 # v7.0.1\n",
            "future.yml",
        )
        assert errors == ["future.yml:2: external action must use a 40-hex SHA"]

    def test_workflow_uses_only_the_bounded_typed_retry_contract(self) -> None:
        """Only pending evidence retries; every other verifier outcome terminates."""
        workflow_text = DEPENDABOT_AUTOMERGE_PATH.read_text()
        assert re.search(r"\bseq\b", workflow_text) is None
        assert "gh pr checks" not in workflow_text
        assert "gh pr merge" not in workflow_text
        assert "gh pr edit" not in workflow_text

        workflow = yaml.safe_load(workflow_text)
        merge = workflow["jobs"]["merge"]
        merge_step = next(step for step in merge["steps"] if step.get("name") == "Merge")
        script = str(merge_step["run"])
        assert "max_attempts=8" in script
        assert "retry_delay_seconds=30" in script
        assert "1|2)" in script
        assert "3)" in script
        assert "4)" in script
        assert 'sleep "$retry_delay_seconds"' in script
        assert script.count("python3 tools/pr_merge.py") == 1

    @pytest.mark.parametrize(
        ("outcomes", "expected_exit", "expected_calls", "expected_sleeps"),
        [
            ([0], 0, 1, 0),
            ([3], 0, 1, 0),
            ([1], 1, 1, 0),
            ([2], 2, 1, 0),
            ([9], 1, 1, 0),
            ([4, 4, 0], 0, 3, 2),
            ([4] * 8, 4, 8, 7),
        ],
        ids=[
            "merged",
            "major-review",
            "hard-refusal",
            "indeterminate",
            "unexpected",
            "pending-then-green",
            "pending-exhausted",
        ],
    )
    def test_merge_step_enforces_bounded_retry_outcomes(
        self,
        tmp_path: Path,
        outcomes: list[int],
        expected_exit: int,
        expected_calls: int,
        expected_sleeps: int,
    ) -> None:
        """The shell loop retries only outcome 4 and never exceeds eight calls."""
        workflow = yaml.safe_load(DEPENDABOT_AUTOMERGE_PATH.read_text())
        merge_step = next(
            step for step in workflow["jobs"]["merge"]["steps"] if step.get("name") == "Merge"
        )
        fake_bin = tmp_path / "bin"
        fake_bin.mkdir()
        verifier_calls = tmp_path / "verifier-calls"
        verifier_outcomes = tmp_path / "verifier-outcomes"
        verifier_outcomes.write_text("\n".join(str(outcome) for outcome in outcomes) + "\n")
        sleep_calls = tmp_path / "sleep-calls"
        fake_python = fake_bin / "python3"
        fake_python.write_text(
            """#!/bin/sh
calls=0
if [ -f "$VERIFIER_CALLS" ]; then
  calls="$(wc -l < "$VERIFIER_CALLS")"
fi
printf '%s\\n' "$*" >> "$VERIFIER_CALLS"
line=$((calls + 1))
outcome="$(sed -n "${line}p" "$VERIFIER_OUTCOMES")"
if [ -z "$outcome" ]; then
  exit 99
fi
exit "$outcome"
"""
        )
        fake_python.chmod(0o755)
        fake_sleep = fake_bin / "sleep"
        fake_sleep.write_text(
            """#!/bin/sh
printf '%s\\n' "$1" >> "$SLEEP_CALLS"
"""
        )
        fake_sleep.chmod(0o755)
        env = os.environ.copy()
        env.update(
            {
                "PATH": f"{fake_bin}:{env['PATH']}",
                "VERIFIER_CALLS": str(verifier_calls),
                "VERIFIER_OUTCOMES": str(verifier_outcomes),
                "SLEEP_CALLS": str(sleep_calls),
                "EXPECTED_HEAD": "a" * 40,
                "SOURCE_RUN_ID": "123",
                "CLASSIFIER_RUN_ID": "456",
                "PR_NUMBER": "742",
            }
        )

        result = subprocess.run(  # noqa: S603,S607 - executes the trusted workflow step
            ["bash", "-c", str(merge_step["run"])],
            capture_output=True,
            env=env,
            text=True,
            check=False,
            timeout=30,
        )

        assert result.returncode == expected_exit, result.stderr
        assert len(verifier_calls.read_text().splitlines()) == expected_calls
        sleeps = sleep_calls.read_text().splitlines() if sleep_calls.exists() else []
        assert sleeps == ["30"] * expected_sleeps

    def test_config_targets_default_branch_and_never_groups_majors(self) -> None:
        """Security settings must apply and grouped PRs must stay low-risk."""
        config = yaml.safe_load(DEPENDABOT_CONFIG_PATH.read_text())
        updates = config["updates"]
        assert len(updates) == 4
        assert all("target-branch" not in updates[index] for index in range(4))

        uv_groups = _dependabot_update(config, "uv")["groups"]
        assert set(uv_groups) == {"uv-minor-patch", "uv-security"}
        assert set(uv_groups["uv-minor-patch"]["update-types"]) == {"minor", "patch"}
        assert set(uv_groups["uv-security"]["update-types"]) == {"minor", "patch"}

        action_groups = _dependabot_update(config, "github-actions")["groups"]
        assert set(action_groups) == {"github-actions-minor-patch"}
        assert set(action_groups["github-actions-minor-patch"]["update-types"]) == {
            "minor",
            "patch",
        }

        cargo_groups = _dependabot_update(config, "cargo")["groups"]
        assert set(cargo_groups) == {"rust-minor-patch", "rust-security"}
        assert set(cargo_groups["rust-minor-patch"]["update-types"]) == {"minor", "patch"}
        assert set(cargo_groups["rust-security"]["update-types"]) == {"minor", "patch"}

    def test_weekly_ecosystems_are_staggered_across_distinct_days(self) -> None:
        """Weekly update batches must not enqueue three full CI runs together."""
        config = yaml.safe_load(DEPENDABOT_CONFIG_PATH.read_text())
        expected_schedules = {
            "uv": {
                "interval": "weekly",
                "day": "monday",
                "time": "09:00",
                "timezone": DEPENDABOT_SCHEDULE_TIMEZONE,
            },
            "github-actions": {
                "interval": "weekly",
                "day": "tuesday",
                "time": "09:00",
                "timezone": DEPENDABOT_SCHEDULE_TIMEZONE,
            },
            "cargo": {
                "interval": "weekly",
                "day": "thursday",
                "time": "09:00",
                "timezone": DEPENDABOT_SCHEDULE_TIMEZONE,
            },
        }

        actual_schedules = {
            ecosystem: _dependabot_update(config, ecosystem)["schedule"]
            for ecosystem in expected_schedules
        }

        assert actual_schedules == expected_schedules
        assert len({schedule["day"] for schedule in actual_schedules.values()}) == 3
        assert {
            _dependabot_update(config, ecosystem)["schedule"]["timezone"]
            for ecosystem in ("uv", "github-actions", "docker", "cargo")
        } == {DEPENDABOT_SCHEDULE_TIMEZONE}

    def test_weekly_ecosystems_apply_the_cooldown_within_schema_limits(self) -> None:
        """Fresh releases batch for 3 days; majors bake for 7 where Dependabot
        supports semver cooldown keys — github-actions gets default-days only
        (PER-264)."""
        config = yaml.safe_load(DEPENDABOT_CONFIG_PATH.read_text())
        expected_cooldown = {"default-days": 3, "semver-major-days": 7}
        for ecosystem in ("uv", "cargo"):
            assert _dependabot_update(config, ecosystem)["cooldown"] == expected_cooldown
        # Dependabot rejects semver-major-days for the github-actions ecosystem.
        assert _dependabot_update(config, "github-actions")["cooldown"] == {"default-days": 3}
        assert "cooldown" not in _dependabot_update(config, "docker")

    def test_config_uses_uv_and_retains_only_justified_ignores(self) -> None:
        """The deferred-major rails and the pyarrow byte-pin must stay represented."""
        config = yaml.safe_load(DEPENDABOT_CONFIG_PATH.read_text())
        uv = _dependabot_update(config, "uv")
        assert {entry["dependency-name"]: entry["update-types"] for entry in uv["ignore"]} == {
            "mypy": ["version-update:semver-major"],
            "pyarrow": [
                "version-update:semver-major",
                "version-update:semver-minor",
                "version-update:semver-patch",
            ],
        }
        docker = _dependabot_update(config, "docker")
        assert docker["ignore"] == [
            {
                "dependency-name": "postgis/postgis",
                "update-types": ["version-update:semver-major"],
            }
        ]

    def test_config_does_not_request_nonexistent_rust_label(self) -> None:
        """Cargo PR creation must not fail because the removed label is absent."""
        config = yaml.safe_load(DEPENDABOT_CONFIG_PATH.read_text())
        cargo = _dependabot_update(config, "cargo")
        assert cargo["labels"] == ["dependencies"]
        actions = _dependabot_update(config, "github-actions")
        assert list(actions["groups"].values()) == [
            {
                "patterns": ["*"],
                "update-types": ["minor", "patch"],
            }
        ]
