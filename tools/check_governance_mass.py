#!/usr/bin/env python3
"""Measure Babylon governance mass and enforce only recurring-context budgets."""

from __future__ import annotations

import argparse
import itertools
import json
import re
import sys
from pathlib import Path
from typing import Final

import yaml

MAX_FILES: Final = 2048
MAX_REPO_SKILLS: Final = 8
MAX_SKILL_BODY_BYTES: Final = 5000
MAX_SKILL_DISCOVERY_CHARS: Final = 2500
MAX_ROUTING_LINES: Final = 2048
ROUTING_WINDOW_RADIUS_LINES: Final = 4
MAX_ROUTING_ACTIONS_PER_WINDOW: Final = 64
MAX_ROUTING_CLAUSES_PER_WINDOW: Final = 64

_INDEX_REFERENCE = "ai/decisions/index.yaml"
_INDEX_REFERENCE_PATTERN = re.compile(re.escape(_INDEX_REFERENCE), re.IGNORECASE)
_INDEX_ACTION = re.compile(r"\b(?:read|consult|load|open|review)\b", re.IGNORECASE)
_NEGATED_INDEX_ACTION = re.compile(
    r"\b(?:do\s+not|don't|never)\s+(?:need\s+to\s+)?"
    r"(?P<actions>(?:read|consult|load|open|review)\b"
    r"(?:\s+(?:or|and)\s+(?:read|consult|load|open|review)\b)*)",
    re.IGNORECASE,
)
_SHARED_OBJECT_NEGATION_GAP = re.compile(
    r"^[\s,]*(?:(?:but|and|or)\s+)?"
    r"(?:do\s+not|don't|never)\s+(?:need\s+to\s+)?$",
    re.IGNORECASE,
)
_ROUTING_CLAUSE_BOUNDARY = re.compile(r"(?<=[.!?;])\s+")
_REQUIRED_ROUTING_FILES: Final = (
    "CLAUDE.md",
    "AGENTS.md",
    "docs/agents/governance.md",
    "ai/README.md",
)


class GovernanceBudgetError(Exception):
    """One or more recurring-context budgets were exceeded."""


def _bounded_files(root: Path, pattern: str) -> list[Path]:
    matches = itertools.islice(root.glob(pattern), MAX_FILES + 1)
    files = sorted(path for path in matches if path.is_file())
    if len(files) > MAX_FILES:
        raise GovernanceBudgetError(
            f"file-count: {pattern} matched {len(files)}, above {MAX_FILES}"
        )
    return files


def _mass(paths: list[Path], repo_root: Path) -> dict[str, int | float | str | None]:
    sizes = [(path.stat().st_size, path) for path in paths[:MAX_FILES]]
    total = sum(size for size, _ in sizes[:MAX_FILES])
    largest_size, largest_path = max(sizes, default=(0, None))
    return {
        "files": len(paths),
        "bytes": total,
        "estimated_tokens_bytes_div_4": total / 4,
        "largest_bytes": largest_size,
        "largest_path": largest_path.relative_to(repo_root).as_posix() if largest_path else None,
    }


def _frontmatter(path: Path) -> tuple[str, str]:
    text = path.read_text(encoding="utf-8")
    lines = text.splitlines()
    if len(lines) < 3 or lines[0] != "---":
        raise GovernanceBudgetError(f"skill-frontmatter: {path} has no YAML header")
    try:
        end = lines[1:128].index("---") + 1
    except ValueError as error:
        raise GovernanceBudgetError(
            f"skill-frontmatter: {path} has no closing delimiter"
        ) from error
    try:
        metadata = yaml.safe_load("\n".join(lines[1:end]))
    except yaml.YAMLError as error:
        raise GovernanceBudgetError(
            f"skill-frontmatter: {path} has invalid YAML: {error}"
        ) from error
    if not isinstance(metadata, dict):
        raise GovernanceBudgetError(f"skill-frontmatter: {path} must be a mapping")
    name = metadata.get("name")
    description = metadata.get("description")
    if not isinstance(name, str) or not isinstance(description, str):
        raise GovernanceBudgetError(f"skill-frontmatter: {path} needs string name and description")
    return name, description


def _skill_report(skill_files: list[Path]) -> tuple[dict[str, int], list[str]]:
    discovery_chars = 0
    largest_body = 0
    violations: list[str] = []
    if len(skill_files) > MAX_REPO_SKILLS:
        violations.append(f"repo-skill-count: {len(skill_files)} exceeds {MAX_REPO_SKILLS}")
    for path in skill_files[:MAX_REPO_SKILLS]:
        name, description = _frontmatter(path)
        discovery_chars += len(name) + len(description)
        body_bytes = path.stat().st_size
        largest_body = max(largest_body, body_bytes)
        if body_bytes > MAX_SKILL_BODY_BYTES:
            violations.append(
                f"skill-body-bytes: {path} is {body_bytes}, above {MAX_SKILL_BODY_BYTES}"
            )
    if discovery_chars > MAX_SKILL_DISCOVERY_CHARS:
        violations.append(
            f"skill-discovery-chars: {discovery_chars} exceeds {MAX_SKILL_DISCOVERY_CHARS}"
        )
    return (
        {
            "files": len(skill_files),
            "discovery_chars": discovery_chars,
            "largest_body_bytes": largest_body,
        },
        violations,
    )


def _normalized_index_windows(text: str) -> tuple[tuple[str, ...], bool]:
    bounded_lines = tuple(itertools.islice(text.splitlines(), MAX_ROUTING_LINES + 1))
    lines = bounded_lines[:MAX_ROUTING_LINES]
    windows: list[str] = []
    for index, line in enumerate(lines[:MAX_ROUTING_LINES]):
        if _INDEX_REFERENCE_PATTERN.search(line) is None:
            continue
        start = max(0, index - ROUTING_WINDOW_RADIUS_LINES)
        stop = min(len(lines), index + ROUTING_WINDOW_RADIUS_LINES + 1)
        normalized = " ".join(" ".join(part.split()) for part in lines[start:stop])
        windows.append(normalized)
    return tuple(windows), len(bounded_lines) > MAX_ROUTING_LINES


def _negated_action_spans(clause: str) -> tuple[tuple[int, int], ...] | None:
    sequences = tuple(
        itertools.islice(
            _NEGATED_INDEX_ACTION.finditer(clause),
            MAX_ROUTING_ACTIONS_PER_WINDOW + 1,
        )
    )
    if len(sequences) > MAX_ROUTING_ACTIONS_PER_WINDOW:
        return None
    spans: list[tuple[int, int]] = []
    for sequence in sequences[:MAX_ROUTING_ACTIONS_PER_WINDOW]:
        base = sequence.start("actions")
        actions = itertools.islice(
            _INDEX_ACTION.finditer(sequence.group("actions")),
            MAX_ROUTING_ACTIONS_PER_WINDOW + 1,
        )
        for action in actions:
            spans.append((base + action.start(), base + action.end()))
            if len(spans) > MAX_ROUTING_ACTIONS_PER_WINDOW:
                return None
    return tuple(spans)


def _reference_gap(action: re.Match[str], reference: re.Match[str]) -> tuple[int, bool, int]:
    if action.end() <= reference.start():
        return reference.start() - action.end(), False, -action.start()
    if reference.end() <= action.start():
        return action.start() - reference.end(), True, action.start()
    return 0, False, -action.start()


def _clause_has_unnegated_index_action(clause: str) -> bool:
    actions = tuple(
        itertools.islice(_INDEX_ACTION.finditer(clause), MAX_ROUTING_ACTIONS_PER_WINDOW + 1)
    )
    references = tuple(
        itertools.islice(
            _INDEX_REFERENCE_PATTERN.finditer(clause),
            MAX_ROUTING_ACTIONS_PER_WINDOW + 1,
        )
    )
    negated_spans = _negated_action_spans(clause)
    if (
        len(actions) > MAX_ROUTING_ACTIONS_PER_WINDOW
        or len(references) > MAX_ROUTING_ACTIONS_PER_WINDOW
    ):
        return True
    if negated_spans is None:
        return True
    if not actions:
        return False
    for reference in references[:MAX_ROUTING_ACTIONS_PER_WINDOW]:
        nearest = min(actions, key=lambda action: _reference_gap(action, reference))
        if nearest.span() not in negated_spans:
            return True
        for action in actions:
            if action.start() >= nearest.start() or action.span() in negated_spans:
                continue
            gap = clause[action.end() : nearest.start()]
            if _SHARED_OBJECT_NEGATION_GAP.fullmatch(gap) is not None:
                return True
    return False


def _has_unnegated_index_action(window: str) -> bool:
    clauses = tuple(
        itertools.islice(
            _ROUTING_CLAUSE_BOUNDARY.split(window),
            MAX_ROUTING_CLAUSES_PER_WINDOW + 1,
        )
    )
    if len(clauses) > MAX_ROUTING_CLAUSES_PER_WINDOW:
        return True
    return any(
        _INDEX_REFERENCE_PATTERN.search(clause) is not None
        and _clause_has_unnegated_index_action(clause)
        for clause in clauses
    )


def _routing_violations(repo_root: Path, skill_files: list[Path]) -> list[str]:
    violations: list[str] = []
    routing_files = [(relative, repo_root / relative) for relative in _REQUIRED_ROUTING_FILES]
    routing_files.extend(
        (path.relative_to(repo_root).as_posix(), path) for path in skill_files[:MAX_FILES]
    )
    seen_targets: set[Path] = set()
    for relative, path in routing_files:
        if not path.is_file():
            violations.append(f"routing-file-missing: {relative}")
            continue
        resolved = path.resolve()
        if resolved in seen_targets:
            continue
        seen_targets.add(resolved)
        windows, lines_exceeded = _normalized_index_windows(path.read_text(encoding="utf-8"))
        if lines_exceeded:
            violations.append(f"routing-lines: {relative} exceeds {MAX_ROUTING_LINES} lines")
        directs_full_read = any(
            _has_unnegated_index_action(window) for window in windows[:MAX_ROUTING_LINES]
        )
        if directs_full_read:
            violations.append(
                f"full-index-routing: {relative} directs agents to load the full index"
            )
    return violations


def measure_governance(repo_root: Path, *, check: bool = False) -> dict[str, object]:
    """Return deterministic mass metrics and optionally enforce lean routing."""
    root = repo_root.resolve()
    adr_files = _bounded_files(root, "ai/decisions/ADR[0-9][0-9][0-9]_*.yaml")
    plan_files = _bounded_files(root, "docs/superpowers/plans/*.md")
    skill_files = _bounded_files(root, ".agents/skills/*/SKILL.md")
    adr_mass = _mass(adr_files, root)
    plan_mass = _mass(plan_files, root)
    skill_report, violations = _skill_report(skill_files)
    violations.extend(_routing_violations(root, skill_files))
    index_path = root / "ai/decisions/index.yaml"
    index_bytes = index_path.stat().st_size if index_path.is_file() else 0
    adr_bytes_value = adr_mass["bytes"]
    if not isinstance(adr_bytes_value, int):
        raise GovernanceBudgetError("internal metric error: ADR bytes are not an integer")
    adr_bytes = adr_bytes_value
    report: dict[str, object] = {
        "adr_corpus": adr_mass,
        "adr_index": {
            "bytes": index_bytes,
            "estimated_tokens_bytes_div_4": index_bytes / 4,
            "index_to_corpus_ratio": index_bytes / adr_bytes if adr_bytes else None,
        },
        "plan_history": plan_mass,
        "repo_skills": skill_report,
        "budgets": {
            "max_repo_skills": MAX_REPO_SKILLS,
            "max_skill_body_bytes": MAX_SKILL_BODY_BYTES,
            "max_skill_discovery_chars": MAX_SKILL_DISCOVERY_CHARS,
            "max_routing_lines": MAX_ROUTING_LINES,
        },
        "violations": violations,
    }
    if check and violations:
        raise GovernanceBudgetError("; ".join(violations))
    return report


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args(argv)
    try:
        report = measure_governance(args.repo, check=args.check)
    except (GovernanceBudgetError, OSError) as error:
        print(f"governance-mass: {error}", file=sys.stderr)
        return 2
    if args.check:
        print("governance-mass: ok")
    else:
        print(json.dumps(report, sort_keys=True, separators=(",", ":")))
    return 0


if __name__ == "__main__":
    sys.exit(main())
