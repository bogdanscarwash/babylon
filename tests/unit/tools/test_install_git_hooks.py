"""Shared hooks must resolve the checkout that Git is operating on."""

from __future__ import annotations

import json
import os
import site
import subprocess
import sys
import venv
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
INSTALLER = ROOT / "tools/install_git_hooks.py"


def _run(cwd: Path, environment: dict[str, str], *args: str) -> str:
    result = subprocess.run(  # noqa: S603
        args, cwd=cwd, env=environment, capture_output=True, text=True, check=False
    )
    assert result.returncode == 0, result.stdout + result.stderr
    return result.stdout


def _environment(root: Path) -> None:
    venv.EnvBuilder(with_pip=False).create(root / ".venv")
    site_packages = (
        root
        / ".venv/lib"
        / f"python{sys.version_info.major}.{sys.version_info.minor}"
        / "site-packages"
    )
    # Reuse installed test dependencies, while each interpreter owns its real prefix.
    (site_packages / "test-dependencies.pth").write_text("\n".join(site.getsitepackages()) + "\n")


def test_shared_hooks_use_each_checkout_and_survive_sibling_retirement(tmp_path: Path) -> None:
    primary, sibling, remote = (tmp_path / name for name in ("primary", "sibling", "remote"))
    primary.mkdir()
    capture = tmp_path / "hook-prefixes.jsonl"
    legacy_capture = tmp_path / "legacy-hook.log"
    environment = {
        **os.environ,
        "HOOK_CAPTURE": str(capture),
        "LEGACY_CAPTURE": str(legacy_capture),
    }
    environment.pop("VIRTUAL_ENV", None)
    _run(primary, environment, "git", "init", "--initial-branch=main")
    _run(primary, environment, "git", "config", "user.name", "Hook Contract")
    _run(primary, environment, "git", "config", "user.email", "hooks@example.invalid")
    _run(primary, environment, "git", "config", "commit.gpgsign", "false")
    (primary / ".gitignore").write_text(".venv/\n")
    (primary / "record.py").write_text(
        "import json, os, sys\nfrom pathlib import Path\n"
        "with Path(os.environ['HOOK_CAPTURE']).open('a') as stream:\n"
        "    stream.write(json.dumps([sys.argv[1], sys.prefix]) + '\\n')\n"
    )
    config = "default_install_hook_types: [pre-commit, commit-msg, pre-push]\nrepos:\n- repo: local\n  hooks:\n"
    for hook in ("pre-commit", "commit-msg", "pre-push"):
        config += (
            f"  - id: {hook}\n    name: {hook}\n    entry: python record.py {hook}\n"
            f"    language: system\n    always_run: true\n    pass_filenames: false\n    stages: [{hook}]\n"
        )
    (primary / ".pre-commit-config.yaml").write_text(config)
    _run(primary, environment, "git", "add", ".")
    _run(primary, environment, "git", "-c", "core.hooksPath=/dev/null", "commit", "-m", "initial")
    legacy_hook = primary / ".git/hooks/pre-commit"
    legacy_hook.write_text('#!/bin/sh\nprintf "legacy\\n" >> "$LEGACY_CAPTURE"\n')
    legacy_hook.chmod(0o755)
    _environment(primary)
    _run(primary, environment, str(primary / ".venv/bin/python"), str(INSTALLER))
    _run(primary, environment, "git", "worktree", "add", "-b", "sibling", str(sibling))
    _environment(sibling)
    _run(sibling, environment, str(sibling / ".venv/bin/python"), str(INSTALLER))
    _run(primary, environment, "git", "init", "--bare", str(remote))
    for checkout, branch in ((primary, "main"), (sibling, "sibling")):
        _run(checkout, environment, "git", "commit", "--allow-empty", "-m", "exercise hooks")
        _run(checkout, environment, "git", "push", str(remote), f"HEAD:refs/heads/{branch}")
        rows = [json.loads(line) for line in capture.read_text().splitlines()]
        assert {row[0] for row in rows} == {"pre-commit", "commit-msg", "pre-push"}
        assert {row[1] for row in rows} == {str(checkout / ".venv")}
        capture.unlink()
    _run(primary, environment, "git", "worktree", "remove", "--force", str(sibling))
    _run(primary, environment, "git", "commit", "--allow-empty", "-m", "after retirement")
    _run(primary, environment, "git", "push", str(remote), "HEAD:refs/heads/main")
    rows = [json.loads(line) for line in capture.read_text().splitlines()]
    assert {row[0] for row in rows} == {"pre-commit", "commit-msg", "pre-push"}
    assert {row[1] for row in rows} == {str(primary / ".venv")}

    assert legacy_capture.read_text().splitlines() == ["legacy", "legacy", "legacy"]
    previous_head = _run(primary, environment, "git", "rev-parse", "HEAD")
    (primary / ".venv").rename(primary / "retained-environment")
    refused = subprocess.run(  # noqa: S603
        ["git", "commit", "--allow-empty", "-m", "must not bypass hooks"],  # noqa: S607
        cwd=primary,
        env=environment,
        capture_output=True,
        text=True,
        check=False,
    )
    assert refused.returncode != 0
    assert "checkout-local .venv" in refused.stderr
    assert _run(primary, environment, "git", "rev-parse", "HEAD") == previous_head
