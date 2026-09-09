"""Decision contract for release qualification before main advances."""

from __future__ import annotations

from pathlib import Path
from typing import Any

import yaml

ROOT = Path(__file__).resolve().parents[3]
ADR_KEY = "ADR231_main_release_qualification"
ADR_PATH = ROOT / "ai" / "decisions" / f"{ADR_KEY}.yaml"
INDEX_PATH = ROOT / "ai" / "decisions" / "index.yaml"


def _mapping(path: Path) -> dict[str, Any]:
    payload = yaml.safe_load(path.read_text(encoding="utf-8"))
    assert isinstance(payload, dict)
    return payload


def test_release_qualification_decision_records_the_live_boundary() -> None:
    decision = _mapping(ADR_PATH)[ADR_KEY]
    text = " ".join(str(decision["decision"]).split())

    assert decision["status"] == "accepted"
    assert decision["date"] == "2026-08-26"
    assert "pull request from dev to main" in text
    assert "critical hotfix" in text
    assert "before main advances" in text
    assert "workflow_dispatch on dev" in text
    assert "unique" in text
    assert "complete combined manifest" in text
    assert "Director" in text
    assert "backport" in text
    assert "release:prepare-dev-sync" in text
    assert "origin/main" in text and "origin/dev" in text
    assert "tag" in text and "main" in text
    assert "direct push" in text
    assert "tools/promote.sh" in text
    assert "ADR230_exact_head_pr_and_dependabot_policy" in decision["related"]


def test_decision_index_resolves_to_main_qualification_record() -> None:
    index = _mapping(INDEX_PATH)
    entry = index["decisions"][ADR_KEY]

    assert index["meta"]["version"] == "1.87.0"
    assert str(index["meta"]["updated"]) == "2026-08-26"
    assert entry == {
        "title": decision_title(),
        "status": "accepted",
        "date": "2026-08-26",
        "file": ADR_PATH.name,
    }


def decision_title() -> str:
    return (
        "Main advances only through a Director-controlled release or critical-hotfix "
        "PR after an exact combined CI and uniquely named release qualification manifest"
    )
