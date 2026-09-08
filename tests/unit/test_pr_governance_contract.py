"""Repository contract for accepting and merging pull requests.

These sentinels catch a regression that makes a required review fact optional,
permits an unverified merge path, or drops the emergency ``main`` boundary.
"""

from __future__ import annotations

import re
from pathlib import Path

import pytest
import yaml

ROOT = Path(__file__).resolve().parents[2]
TEMPLATE = Path(".github/PULL_REQUEST_TEMPLATE.md")
DEPENDABOT_CONFIG = Path(".github/dependabot.yml")
GOVERNANCE_SURFACES = (
    TEMPLATE,
    Path("CONTRIBUTORS.md"),
    Path("docs/agents/governance.md"),
)
MAIN_RELEASE_SURFACES = (
    Path("CONTRIBUTORS.md"),
    Path("docs/agents/governance.md"),
)
MERGE_COMMAND = "mise run pr:merge -- N"
COPILOT_ADVISORY_SURFACES = (
    *GOVERNANCE_SURFACES,
    Path("CLAUDE.md"),
    Path(".mise.toml"),
)
COPILOT_ADVISORY_ADR_NAME = "ADR241_copilot_advisory_merge_policy"
COPILOT_ADVISORY_ADR = Path("ai/decisions") / f"{COPILOT_ADVISORY_ADR_NAME}.yaml"
ADR_INDEX = Path("ai/decisions/index.yaml")


def _text(path: Path) -> str:
    """Return one live governance surface as repository text."""
    return (ROOT / path).read_text(encoding="utf-8")


def _normalized(path: Path) -> str:
    """Return lowercase prose with Markdown line wrapping removed."""
    return " ".join(_text(path).lower().split())


@pytest.mark.parametrize("path", GOVERNANCE_SURFACES)
def test_every_pr_declares_one_linear_delivery_disposition(path: Path) -> None:
    """Removing either partial or closing Linear syntax must fail the gate."""
    text = _text(path)
    assert "Part of PER-N" in text
    assert "Fixes PER-N" in text


def test_pr_template_contributors_link_resolves_to_repository_root() -> None:
    """The contributor link must work from the template's ``.github`` path."""
    match = re.search(r"\[CONTRIBUTORS\.md\]\(([^)]+)\)", _text(TEMPLATE))
    assert match is not None
    target = (ROOT / TEMPLATE.parent / match.group(1)).resolve()
    assert target == (ROOT / "CONTRIBUTORS.md").resolve()
    assert target.is_file()


@pytest.mark.parametrize("path", GOVERNANCE_SURFACES)
def test_merge_evidence_is_pinned_to_reviewed_head_and_base(path: Path) -> None:
    """Review and green checks must identify the exact head and target base."""
    text = _normalized(path)
    assert "exact reviewed head sha" in text
    assert "base branch" in text
    assert "blocking checks" in text


@pytest.mark.parametrize("path", COPILOT_ADVISORY_SURFACES)
def test_copilot_evidence_is_advisory_but_unresolved_threads_block(path: Path) -> None:
    """Copilot availability cannot block, while unresolved threads still do."""
    text = _normalized(path)
    assert "copilot" in text
    assert "advisory" in text
    assert "unresolved review thread" in text
    assert "block" in text or "refuse" in text


def test_copilot_advisory_decision_is_precisely_scoped_and_indexed() -> None:
    """The successor ADR must preserve every non-Copilot merge boundary."""
    record = yaml.safe_load(_text(COPILOT_ADVISORY_ADR))
    assert isinstance(record, dict)
    decision = record[COPILOT_ADVISORY_ADR_NAME]
    text = " ".join(str(decision["decision"]).split())
    supersedes = " ".join(str(item) for item in decision["supersedes"])

    assert decision["status"] == "accepted"
    assert decision["date"] == "2026-08-26"
    assert "Copilot" in text and "advisory" in text
    assert "unresolved review thread" in text and "blocks" in text
    assert "required ci" in text.lower()
    assert "exact head" in text and "exact base" in text
    assert "PER identity" in text
    assert MERGE_COMMAND in text
    assert "ADR181" in supersedes and "Copilot" in supersedes
    assert "ADR230" in supersedes and "Copilot" in supersedes
    assert "only" in supersedes

    index = yaml.safe_load(_text(ADR_INDEX))
    assert isinstance(index, dict)
    entry = index["decisions"][COPILOT_ADVISORY_ADR_NAME]
    assert index["meta"]["version"] == "1.87.0"
    assert entry["file"] == COPILOT_ADVISORY_ADR.name
    assert entry["status"] == "accepted"


@pytest.mark.parametrize("path", GOVERNANCE_SURFACES)
def test_behavior_and_baseline_dispositions_are_explicit(path: Path) -> None:
    """A PR must account for durable behavior evidence and baseline drift."""
    text = _normalized(path)
    assert "behavioral-contract disposition" in text
    assert "no behavior change" in text
    assert "baseline" in text
    assert "ceremony" in text


@pytest.mark.parametrize("path", GOVERNANCE_SURFACES)
def test_only_sanctioned_merge_command_is_documented(path: Path) -> None:
    """Direct GitHub CLI merge paths must remain explicitly forbidden."""
    text = _text(path)
    assert MERGE_COMMAND in text
    assert "Do not run `gh pr merge` directly" in text


@pytest.mark.parametrize("path", GOVERNANCE_SURFACES)
def test_source_branch_is_preserved_by_default(path: Path) -> None:
    """Routine merge guidance must not silently delete the evidence lane."""
    assert "preserve the source branch by default" in _normalized(path)


@pytest.mark.parametrize("path", GOVERNANCE_SURFACES)
def test_main_documents_both_director_only_sources(path: Path) -> None:
    """A release from dev and a critical hotfix are the only main sources."""
    text = _normalized(path)
    assert "director" in text
    assert "release pr" in text
    assert "from `dev`" in text
    assert "critical hotfix" in text
    assert "backport" in text


@pytest.mark.parametrize("path", MAIN_RELEASE_SURFACES)
def test_main_release_procedure_uses_premerge_qualification(path: Path) -> None:
    """Main must prove dev, use Director mode, and return ancestry by PR."""
    text = _text(path)
    normalized = _normalized(path)
    assert "git merge-base --is-ancestor origin/main origin/dev" in text
    assert "gh workflow run main.yml --ref dev" in text
    assert "mise run pr:merge -- N --director-main" in text
    assert "mise run release:prepare-dev-sync -- vX.Y.Z N" in text
    assert "mise run release:tag -- --yes" in text
    assert "complete combined manifest" in normalized
    assert text.index("gh workflow run main.yml --ref dev") < text.index(
        "mise run pr:merge -- N --director-main"
    )
    assert text.index("mise run pr:merge -- N --director-main") < text.index(
        "mise run release:prepare-dev-sync -- vX.Y.Z N"
    )
    assert text.index("mise run release:prepare-dev-sync -- vX.Y.Z N") < text.index(
        "mise run release:tag -- --yes"
    )
    assert "gh pr update-branch" not in text
    assert "dev:main" not in text
    assert "tools/promote.sh" not in text


def test_dependabot_config_names_pr_qualification_not_retired_promotion() -> None:
    """Point-of-use guidance must not send releases through tools/promote.sh."""
    text = _normalized(DEPENDABOT_CONFIG)
    assert "main qualification" in text
    assert "release pr" in text
    assert "tools/promote.sh" not in text
    assert "every every" not in text
