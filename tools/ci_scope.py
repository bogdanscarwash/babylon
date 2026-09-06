#!/usr/bin/env python3
"""Select changed-input CI jobs and reject incomplete execution receipts."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
from pathlib import Path


def scope(paths: list[str], *, full: bool) -> dict[str, bool]:
    """Unknown executable inputs run every lane; prose alone needs static checks."""
    selected = dict.fromkeys(("rust", "python", "postgres"), full)
    for path in paths:
        if path.startswith((".github/", ".mise/")) or path in {
            ".mise.toml",
            "mise.lock",
            ".pre-commit-config.yaml",
            "tools/ci_scope.py",
        }:
            selected = dict.fromkeys(selected, True)
        elif path.startswith("rust/"):
            selected["rust"] = True
            selected["postgres"] = True
        elif path.startswith(
            ("docker/", "contracts/", "content/", "src/babylon/data/")
        ) or path.startswith("docker-compose"):
            selected = dict.fromkeys(selected, True)
        elif path.endswith(".py") or path in {"pyproject.toml", "uv.lock", ".python-version"}:
            selected["python"] = True
            if path.startswith("tools/"):
                selected["postgres"] = True
        elif path.endswith((".md", ".rst")):
            continue
        elif path.startswith(("docs/", "reports/", "project/", "specs/", "openwiki/")) and (
            path.endswith((".png", ".jpg", ".jpeg", ".webp", ".pdf"))
        ):
            continue
        else:
            selected = dict.fromkeys(selected, True)
    return {"full": full, **selected}


def verify_results(plan: dict[str, bool], results: dict[str, str]) -> None:
    """Only explicitly unselected jobs may skip; failed prerequisites never pass."""
    expected = {
        "scope": True,
        "fast-gate": True,
        "ceremony-gate": True,
        "gitleaks": True,
        "trivy-config": True,
        "rust-gate": plan["rust"],
        "test-unit": plan["python"],
        "security": plan["python"],
        "pg-integration-shards": plan["postgres"],
    }
    unexpected = results.keys() - expected.keys()
    if unexpected:
        raise ValueError(f"unexpected job receipts: {', '.join(sorted(unexpected))}")
    failures = [
        f"{job}: expected {'success' if required else 'skipped'}, got {results.get(job, 'missing')}"
        for job, required in expected.items()
        if results.get(job) != ("success" if required else "skipped")
    ]
    if failures:
        raise ValueError("; ".join(failures))


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--verify", action="store_true")
    args = parser.parse_args()
    if args.verify:
        needs = json.loads(os.environ["CI_NEEDS"])
        plan = json.loads(needs["scope"]["outputs"]["plan"])
        verify_results(plan, {job: result["result"] for job, result in needs.items()})
        print("Every selected CI job passed.")
        return
    event = json.loads(Path(os.environ["GITHUB_EVENT_PATH"]).read_text())
    event_name = os.environ["GITHUB_EVENT_NAME"]
    full = (
        event_name == "workflow_dispatch"
        or event.get("pull_request", {}).get("base", {}).get("ref") == "main"
    )
    paths: list[str] = []
    if not full:
        if event_name == "pull_request":
            base = event["pull_request"]["base"]["sha"]
            head = event["pull_request"]["head"]["sha"]
            revision = f"{base}...{head}"
        elif event_name == "push":
            base, head = event["before"], event["after"]
            if set(base) == {"0"}:
                full = True
            revision = f"{base}..{head}"
        else:
            raise ValueError(f"unsupported CI event {event_name!r}")
        if not full:
            changed = subprocess.run(
                ["git", "diff", "--name-only", "--no-renames", "-z", revision],
                check=True,
                capture_output=True,
            )
            paths = [os.fsdecode(path) for path in changed.stdout.split(b"\0") if path]
    plan = scope(paths, full=full)
    focus = ["runtime_smoke"]
    if full:
        focus += [
            "reference_integrity",
            "runtime",
            "archive",
            "reader",
            "client",
        ]
    outputs = {**plan, "plan": json.dumps(plan), "pg-matrix": json.dumps({"focus": focus})}
    with Path(os.environ["GITHUB_OUTPUT"]).open("a") as output:
        for key, value in outputs.items():
            output.write(f"{key}={json.dumps(value) if isinstance(value, bool) else value}\n")
    print(json.dumps({"plan": plan, "changed_paths": paths, "postgres_focus": focus}, indent=2))


if __name__ == "__main__":
    main()
