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


def test_source_release_runs_locked_environment_smoke_before_publish() -> None:
    steps = _steps("release.yml", "release")
    bootstrap = next(
        index
        for index, step in enumerate(steps)
        if step.get("uses") == "./.github/actions/bootstrap-python"
    )
    smoke = next(index for index, step in enumerate(steps) if step.get("name") == "Source smoke")
    lock = next(index for index, step in enumerate(steps) if step.get("run") == "uv lock --check")
    publish = next(
        index for index, step in enumerate(steps) if step.get("name") == "Create GitHub Release"
    )
    assert bootstrap < lock < smoke < publish
    assert "uv run --frozen python -c" in steps[smoke]["run"]
    assert "uv run --frozen babylon --help" in steps[smoke]["run"]
    assert "continue-on-error" not in steps[smoke]
    assert not (ROOT / "tools" / "run_regression.py").exists()
    assert all(step.get("run") != "mise run qa:regression" for step in steps)


def test_publication_requires_main_lineage_and_verified_native_download() -> None:
    workflow = yaml.safe_load((ROOT / ".github/workflows/release.yml").read_text())
    triggers = workflow.get("on", workflow.get(True))
    assert triggers["push"] == {"tags": ["v*"]}
    assert workflow["permissions"] == {"contents": "read"}
    jobs = workflow["jobs"]
    identity = jobs["identity"]
    identity_runs = "\n".join(step.get("run", "") for step in identity["steps"])
    for guard in (
        "tools/check_release_tag.py",
        "tools/release_lineage.py verify",
        "tools/release_version.py --release-tag",
    ):
        assert guard in identity_runs
    release = jobs["release"]
    assert release["needs"] == ["identity"]
    assert release["permissions"] == {"contents": "write", "actions": "read"}
    assert not any("continue-on-error" in job for job in jobs.values())
    for job in (identity, release):
        assert not any("continue-on-error" in step for step in job["steps"])
    publish = next(step for step in release["steps"] if step.get("name") == "Create GitHub Release")
    promotion = next(
        step for step in release["steps"] if "tools/release_artifact.py" in step.get("run", "")
    )
    assert release["steps"].index(promotion) < release["steps"].index(publish)
    assert "--output-dir release-download" in promotion["run"]
    assert "--verify-tag --draft" in publish["run"]
    assert "--clobber" not in publish["run"]
    assert publish["run"].index("gh release upload") < publish["run"].index("--draft=false")


def test_native_artifact_upload_requires_unpacked_runtime_exercise() -> None:
    steps = _steps("main.yml", "native-package")
    smoke = next(
        index for index, step in enumerate(steps) if "./babylon --smoke" in step.get("run", "")
    )
    upload = next(
        index
        for index, step in enumerate(steps)
        if step.get("uses", "").startswith("actions/upload-artifact@")
    )
    assert smoke < upload
    assert "sha256sum --check" in steps[smoke]["run"]
    assert "tar --extract" in steps[smoke]["run"]
    assert "if" not in steps[upload]
    assert not any("continue-on-error" in step for step in steps)
    assert steps[upload]["with"]["if-no-files-found"] == "error"


def test_native_package_has_no_arbitrary_checkout_input() -> None:
    assert not (ROOT / ".github/workflows/native-package.yml").exists()
    workflow = yaml.safe_load((ROOT / ".github/workflows/main.yml").read_text())
    job = workflow["jobs"]["native-package"]
    assert "uses" not in job
    assert job["name"] == "Main Qualification / Native Download / Linux x86_64"
    assert workflow["permissions"] == {"contents": "read"}


def test_native_package_separates_pr_heads_from_dispatch_cache_scope() -> None:
    workflow = yaml.safe_load((ROOT / ".github/workflows/main.yml").read_text())
    job = workflow["jobs"]["native-package"]
    guard = " ".join(job["if"].split())
    assert guard == (
        "(github.event_name == 'pull_request' && "
        "github.event.pull_request.head.repo.full_name == github.repository && "
        "github.event.pull_request.base.ref == 'main') || "
        "(github.event_name == 'workflow_dispatch' && github.ref == 'refs/heads/dev')"
    )
    checkouts = [
        step for step in job["steps"] if step.get("uses", "").startswith("actions/checkout@")
    ]
    assert len(checkouts) == 2
    pr, dispatch = checkouts
    assert pr["if"] == "github.event_name == 'pull_request'"
    assert pr["with"]["ref"] == "${{ github.event.pull_request.head.sha }}"
    assert dispatch["if"] == "github.event_name == 'workflow_dispatch'"
    assert "ref" not in dispatch["with"]
    assert all(step["with"]["persist-credentials"] is False for step in checkouts)
    identity = next(step for step in job["steps"] if step.get("id") == "identity")
    assert identity["env"]["SOURCE_SHA"] == (
        "${{ github.event_name == 'pull_request' && github.event.pull_request.head.sha "
        "|| github.sha }}"
    )
    assert 'test "$(git rev-parse HEAD)" = "$SOURCE_SHA"' in identity["run"]
    assert 'echo "sha=$SOURCE_SHA" >> "$GITHUB_OUTPUT"' in identity["run"]


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
