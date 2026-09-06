"""Native release and exact-reference-build checks survive channel retirement."""

from __future__ import annotations

import shutil
import subprocess
from pathlib import Path
from typing import Any

import pytest
import yaml

ROOT = Path(__file__).resolve().parents[3]

PIN_INPUTS = (
    ".mise.toml",
    "mise.lock",
    ".python-version",
    "pyproject.toml",
    "uv.lock",
    "rust/rust-toolchain.toml",
    "rust/Cargo.lock",
    "tools/build_reference_db.py",
    "data-artifacts.yaml",
)


def _pin_fixture(tmp_path: Path) -> Path:
    for name in PIN_INPUTS:
        target = tmp_path / name
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(ROOT / name, target)
    return tmp_path


def _check_pins(root: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["sh", str(ROOT / "tools/check_release_pins.sh")],
        cwd=root,
        check=False,
        capture_output=True,
        text=True,
        timeout=10,
    )


def test_native_release_pins_validate_without_retired_environment_files(tmp_path: Path) -> None:
    result = _check_pins(_pin_fixture(tmp_path))
    assert result.returncode == 0, result.stdout + result.stderr


@pytest.mark.parametrize(
    ("path", "before", "after"),
    [
        (".python-version", "3.12.14", "3.12.13"),
        (".mise.toml", 'uv = "0.9.8"', 'uv = "latest"'),
        ("rust/rust-toolchain.toml", '"clippy"', '"rust-src"'),
        ("mise.lock", 'version = "3.12.14"', 'version = "3.12.13"'),
        ("mise.lock", 'specifiers = ["3.12.14"]', 'specifiers = ["latest"]'),
        ("mise.lock", 'backend = "core:python"', 'backend = "asdf:python"'),
        ("mise.lock", "sha256:72748d", "sha256:invalid72748d"),
        ("mise.lock", 'tools.python."platforms.linux-x64"', 'tools.python."platforms.macos-arm64"'),
        ("mise.lock", "github.com/astral-sh/python-build-standalone", "example.com/python"),
        ("mise.lock", "3.12.14+20260901", "3.12.14+20260902"),
        ("mise.lock", 'provenance = "github-attestations"', 'provenance = "none"'),
        ("data-artifacts.yaml", 'sqlite_version: "3.53.1"', 'sqlite_version: "3.53.0"'),
    ],
)
def test_native_release_pin_drift_refuses(
    tmp_path: Path, path: str, before: str, after: str
) -> None:
    root = _pin_fixture(tmp_path)
    target = root / path
    source = target.read_text()
    assert before in source
    target.write_text(source.replace(before, after, 1))
    result = _check_pins(root)
    assert result.returncode != 0
    assert "REFUSE" in result.stderr


@pytest.mark.parametrize("path", ["uv.lock", "rust/Cargo.lock", "mise.lock"])
def test_missing_native_release_pin_refuses(tmp_path: Path, path: str) -> None:
    root = _pin_fixture(tmp_path)
    (root / path).unlink()
    result = _check_pins(root)
    assert result.returncode != 0


def _steps(name: str, job: str) -> list[dict[str, Any]]:
    workflow = yaml.safe_load((ROOT / ".github" / "workflows" / name).read_text())
    return workflow["jobs"][job]["steps"]


def test_retired_channel_has_no_live_installer_or_workflow() -> None:
    retired = (
        "install.sh",
        "tools/installer_smoke.sh",
        "tests/install/test_install_sh.sh",
        ".github/workflows/installer.yml",
        ".github/workflows/nix-release.yml",
        ".github/workflows/flake-update.yml",
    )
    assert [path for path in retired if (ROOT / path).exists()] == []


def test_source_release_runs_locked_environment_smoke_and_regression_before_publish() -> None:
    steps = _steps("release.yml", "release")
    bootstrap = next(
        index
        for index, step in enumerate(steps)
        if step.get("uses") == "./.github/actions/bootstrap-python"
    )
    regression = next(
        index for index, step in enumerate(steps) if step.get("run") == "mise run qa:regression"
    )
    smoke = next(index for index, step in enumerate(steps) if step.get("name") == "Source smoke")
    lock = next(index for index, step in enumerate(steps) if step.get("run") == "uv lock --check")
    publish = next(
        index for index, step in enumerate(steps) if step.get("name") == "Create GitHub Release"
    )
    assert bootstrap < lock < regression < publish
    assert bootstrap < lock < smoke < publish
    assert "uv run --frozen python -c" in steps[smoke]["run"]
    assert "uv run --frozen babylon --help" in steps[smoke]["run"]
    assert "continue-on-error" not in steps[regression]
    assert "continue-on-error" not in steps[smoke]


def test_weekly_rebuild_uses_exact_native_interpreter_and_compares_product_bytes() -> None:
    path = ROOT / ".github" / "workflows" / "weekly-rebuild-verify.yml"
    workflow = yaml.safe_load(path.read_text())
    job = workflow["jobs"]["rebuild-verify"]
    steps = job["steps"]
    runs = "\n".join(step.get("run", "") for step in steps)
    assert job["if"] == "vars.CI_REFDB_READY == 'true'"
    assert any(step.get("uses") == "./.github/actions/bootstrap-python" for step in steps)
    assert "uv run --frozen python tools/build_reference_db.py --out /tmp/rebuilt.sqlite" in runs
    assert "uv run --frozen python - <<'EOF'" in runs
    assert 'hashlib.sha256(pathlib.Path("/tmp/rebuilt.sqlite").read_bytes()).hexdigest()' in runs
    assert "sys.exit(0 if want == got else 1)" in runs
    assert any(step.get("uses") == "./.github/actions/fetch-reference-db" for step in steps)
    assert "nix " not in path.read_text()
    assert "data:python" not in runs


def test_python_bootstrap_requires_locked_tools_and_caches_exact_interpreter_build() -> None:
    action = yaml.safe_load((ROOT / ".github/actions/bootstrap-python/action.yml").read_text())
    steps = action["runs"]["steps"]
    install = next(step for step in steps if step.get("uses", "").startswith("jdx/mise-action@"))
    assert install["with"]["install_args"] == "--locked"
    cache = next(step for step in steps if step.get("id") == "venv-cache")
    assert "hashFiles('uv.lock', '.mise.toml', 'mise.lock')" in cache["with"]["key"]
