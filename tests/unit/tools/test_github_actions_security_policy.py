"""Security contracts for third-party GitHub Actions references."""

from __future__ import annotations

import re
from pathlib import Path
from typing import Any

import yaml

ROOT = Path(__file__).resolve().parents[3]
AUTOMATION_ROOTS = (ROOT / ".github" / "actions", ROOT / ".github" / "workflows")
USES_LINE = re.compile(r"^\s*(?:-\s*)?uses:\s*(?P<value>.*?)\s*$")
COMMENT_START = re.compile(r"\s+#")
PINNED_ACTION = re.compile(r"^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+(?:/[A-Za-z0-9_.-]+)*@[0-9a-f]{40}$")


def _action_use(line: str) -> tuple[str, str | None] | None:
    match = USES_LINE.match(line)
    if match is None:
        return None
    value = match.group("value")
    parts = COMMENT_START.split(value, maxsplit=1)
    reference = parts[0].strip()
    comment = parts[1].strip() if len(parts) == 2 else ""
    tag = comment.split(maxsplit=1)[0] if comment else None
    return reference, tag


def test_action_use_parser_keeps_version_tag_when_comment_has_details() -> None:
    line = "      - uses: actions/checkout@" + "a" * 40 + " # v7 (pinned release)"

    assert _action_use(line) == ("actions/checkout@" + "a" * 40, "v7")


def _automation_paths() -> list[Path]:
    return sorted(
        path
        for root in AUTOMATION_ROOTS
        for pattern in ("*.yml", "*.yaml")
        for path in root.rglob(pattern)
    )


def _workflow(name: str) -> dict[str, Any]:
    payload = yaml.safe_load((ROOT / ".github" / "workflows" / name).read_text(encoding="utf-8"))
    assert isinstance(payload, dict)
    return payload


def _triggers(workflow: dict[str, Any]) -> dict[str, Any]:
    payload = workflow.get("on", workflow.get(True))
    assert isinstance(payload, dict)
    return payload


def test_every_external_action_uses_a_documented_immutable_commit() -> None:
    violations: list[str] = []
    for path in _automation_paths():
        for line_number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
            action_use = _action_use(line)
            if action_use is None:
                continue
            reference, tag = action_use
            if reference.startswith(("./", "docker://")):
                continue
            if PINNED_ACTION.fullmatch(reference) is None or tag is None or not tag.startswith("v"):
                violations.append(f"{path.relative_to(ROOT)}:{line_number}: {line.strip()}")

    assert violations == []


def test_codeql_covers_protected_branch_pull_requests_with_least_privilege() -> None:
    workflow = _workflow("codeql.yml")
    triggers = _triggers(workflow)
    analyze = workflow["jobs"]["analyze"]
    matrix = analyze["strategy"]["matrix"]["include"]
    init = next(step for step in analyze["steps"] if step["name"] == "Initialize CodeQL")
    upload = next(step for step in analyze["steps"] if step["name"] == "Perform CodeQL Analysis")
    codeql_config = yaml.safe_load(init["with"]["config"])
    ignored_paths = codeql_config["paths-ignore"]

    assert triggers["pull_request"] == {"branches": ["main", "dev"]}
    assert triggers["push"] == {"branches": ["main", "dev"]}
    assert workflow["concurrency"]["cancel-in-progress"] is True
    assert analyze["permissions"] == {
        "actions": "read",
        "contents": "read",
        "security-events": "write",
    }
    assert matrix == [
        {"language": "python", "build-mode": "none"},
        {"language": "rust", "build-mode": "none"},
        {"language": "actions", "build-mode": "none"},
        {"language": "javascript-typescript", "build-mode": "none"},
    ]
    assert init["with"]["languages"] == "${{ matrix.language }}"
    assert init["with"]["build-mode"] == "${{ matrix.build-mode }}"
    assert init["with"]["queries"] == "security-extended"
    assert "security-and-quality" not in str(init["with"])
    assert ignored_paths == [
        ".design-sync",
        "design",
        "rust/crates/bsl-lint/tests/fixtures/sfs_non_authorability/reserved-rust/rust",
        "rust/crates/bsl-lint/tests/fixtures/sfs_non_authorability/reserved-two-hop-helper/rust",
    ]
    assert "src/frontend" not in ignored_paths
    assert all((ROOT / path).exists() for path in ignored_paths)
    assert upload["with"] == {
        "category": "/language:${{ matrix.language }}",
        "upload": "always",
    }


def test_pages_write_permissions_exist_only_on_the_deploy_job() -> None:
    workflow = _workflow("docs.yml")

    assert workflow["permissions"] == {"contents": "read"}
    assert "permissions" not in workflow["jobs"]["build"]
    assert workflow["jobs"]["deploy"]["permissions"] == {
        "pages": "write",
        "id-token": "write",
    }


def test_pip_audit_jobs_install_the_operator_dependency_set() -> None:
    violations: list[str] = []
    for name in ("ci.yml", "weekly-security.yml"):
        security = _workflow(name)["jobs"]["security"]
        bootstrap = next(
            step
            for step in security["steps"]
            if step.get("uses") == "./.github/actions/bootstrap-python"
        )
        if bootstrap.get("with") != {"ops": "true"}:
            violations.append(name)

    assert violations == []


def test_source_release_has_no_distribution_secrets_or_inline_secret_shell_expansion() -> None:
    workflow = _workflow("release.yml")
    steps = workflow["jobs"]["release"]["steps"]
    for step in steps:
        assert "${{ secrets." not in step.get("run", "")
    secret_environments = [
        step["env"]
        for step in steps
        if any("${{ secrets." in str(value) for value in step.get("env", {}).values())
    ]
    assert secret_environments == [{"GITHUB_TOKEN": "${{ secrets.GITHUB_TOKEN }}"}]
