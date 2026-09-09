"""Contracts for the deterministic governance-context budget."""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

TOOLS_DIR = Path(__file__).resolve().parents[3] / "tools"
sys.path.insert(0, str(TOOLS_DIR))

from check_governance_mass import (  # type: ignore[import-not-found]  # noqa: E402
    MAX_ROUTING_LINES,
    MAX_SKILL_BODY_BYTES,
    GovernanceBudgetError,
    measure_governance,
)

pytestmark = pytest.mark.unit


def _write(root: Path, relative: str, content: str) -> None:
    target = root / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(content, encoding="utf-8")


def _fixture_repo(tmp_path: Path) -> Path:
    repo = tmp_path / "repo"
    _write(repo, "ai/decisions/ADR001_one.yaml", "status: accepted\n")
    _write(repo, "ai/decisions/index.yaml", "decisions: {}\n")
    _write(repo, "docs/superpowers/plans/old.md", "historical\n")
    _write(repo, "CLAUDE.md", "Use the bounded repository instructions.\n")
    _write(repo, "AGENTS.md", "Use the bounded repository instructions.\n")
    _write(
        repo,
        ".agents/skills/one/SKILL.md",
        """---
name: one
description: Use for one bounded job. Do not use elsewhere.
---

Run the deterministic command.
""",
    )
    _write(
        repo,
        "docs/agents/governance.md",
        "Run mise run adr search before reading source.\n",
    )
    _write(
        repo,
        "ai/README.md",
        "Query with mise run adr; the tracked index remains a sentinel input.\n",
    )
    return repo


def test_measurement_separates_corpus_history_and_skill_discovery(tmp_path: Path) -> None:
    repo = _fixture_repo(tmp_path)

    report = measure_governance(repo)

    assert report["adr_corpus"]["files"] == 1
    assert report["adr_corpus"]["largest_path"] == "ai/decisions/ADR001_one.yaml"
    assert report["plan_history"]["files"] == 1
    assert report["repo_skills"]["files"] == 1
    assert report["repo_skills"]["discovery_chars"] == len(
        "oneUse for one bounded job. Do not use elsewhere."
    )
    assert report["violations"] == []


def test_oversized_skill_body_fails_the_budget(tmp_path: Path) -> None:
    repo = _fixture_repo(tmp_path)
    _write(
        repo,
        ".agents/skills/one/SKILL.md",
        "---\nname: one\ndescription: bounded\n---\n" + ("x" * MAX_SKILL_BODY_BYTES),
    )

    with pytest.raises(GovernanceBudgetError, match="skill-body-bytes"):
        measure_governance(repo, check=True)


@pytest.mark.parametrize(
    "directive",
    [
        "Read ai/decisions/index.yaml before every architecture question.\n",
        "5. Consult ai/decisions/index.yaml before every architecture question.\n",
        "You must review ai/decisions/index.yaml before architecture work.\n",
        "- Open ai/decisions/index.yaml before architecture work.\n",
        "Read AI/DECISIONS/INDEX.YAML before architecture work.\n",
        "Read the complete ADR index at\n`ai/decisions/index.yaml` before architecture work.\n",
    ],
)
def test_live_instruction_to_read_full_index_fails(tmp_path: Path, directive: str) -> None:
    repo = _fixture_repo(tmp_path)
    _write(repo, "docs/agents/governance.md", directive)

    with pytest.raises(GovernanceBudgetError) as error:
        measure_governance(repo, check=True)

    assert str(error.value) == (
        "full-index-routing: docs/agents/governance.md directs agents to load the full index"
    )


@pytest.mark.parametrize(
    ("relative", "content"),
    [
        ("CLAUDE.md", "Read ai/decisions/index.yaml before architecture work.\n"),
        ("AGENTS.md", "Read ai/decisions/index.yaml before architecture work.\n"),
        (
            ".agents/skills/one/SKILL.md",
            "---\nname: one\ndescription: bounded\n---\n"
            "Read ai/decisions/index.yaml before architecture work.\n",
        ),
    ],
)
def test_every_recurring_instruction_surface_enforces_query_first_routing(
    tmp_path: Path,
    relative: str,
    content: str,
) -> None:
    repo = _fixture_repo(tmp_path)
    _write(repo, relative, content)

    with pytest.raises(GovernanceBudgetError) as error:
        measure_governance(repo, check=True)

    assert str(error.value) == (
        f"full-index-routing: {relative} directs agents to load the full index"
    )


def test_coordinated_affirmative_action_is_not_hidden_by_nearer_negation(
    tmp_path: Path,
) -> None:
    repo = _fixture_repo(tmp_path)
    _write(
        repo,
        "docs/agents/governance.md",
        "Read, but do not review, ai/decisions/index.yaml for lookup.\n",
    )

    with pytest.raises(GovernanceBudgetError, match="full-index-routing"):
        measure_governance(repo, check=True)


def test_routing_file_over_the_scan_bound_fails_closed(tmp_path: Path) -> None:
    repo = _fixture_repo(tmp_path)
    _write(
        repo,
        "docs/agents/governance.md",
        "\n".join(["bounded routing"] * MAX_ROUTING_LINES)
        + "\nRead ai/decisions/index.yaml before architecture work.\n",
    )

    with pytest.raises(GovernanceBudgetError) as error:
        measure_governance(repo, check=True)

    assert str(error.value).startswith(
        f"routing-lines: docs/agents/governance.md exceeds {MAX_ROUTING_LINES} lines"
    )


def test_multiline_negated_full_index_instruction_is_allowed(tmp_path: Path) -> None:
    repo = _fixture_repo(tmp_path)
    _write(
        repo,
        "docs/agents/governance.md",
        "Do not\nread the complete ADR index at\n`ai/decisions/index.yaml`.\n",
    )

    report = measure_governance(repo, check=True)

    assert report["violations"] == []


def test_neighboring_read_sentence_does_not_cancel_index_negation(tmp_path: Path) -> None:
    repo = _fixture_repo(tmp_path)
    _write(
        repo,
        "docs/agents/governance.md",
        (
            "Read only the selected ADR record for its rationale.\n"
            "Do not load `ai/decisions/index.yaml`\n"
            "for lookup.\n"
        ),
    )

    report = measure_governance(repo, check=True)

    assert report["violations"] == []


def test_same_sentence_unrelated_read_does_not_cancel_index_negation(
    tmp_path: Path,
) -> None:
    repo = _fixture_repo(tmp_path)
    _write(
        repo,
        "docs/agents/governance.md",
        "Read ADR221, but do not load ai/decisions/index.yaml for lookup.\n",
    )

    report = measure_governance(repo, check=True)

    assert report["violations"] == []


def test_negated_neighbor_does_not_hide_multiline_read_directive(tmp_path: Path) -> None:
    repo = _fixture_repo(tmp_path)
    _write(
        repo,
        "docs/agents/governance.md",
        (
            "Do not read `ai/decisions/index.yaml` during ordinary work.\n"
            "Read the complete ADR index at\n"
            "`ai/decisions/index.yaml` before architecture work.\n"
        ),
    )

    with pytest.raises(GovernanceBudgetError, match="full-index-routing"):
        measure_governance(repo, check=True)
