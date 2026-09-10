"""Native distribution byte integrity and relocatable launcher contracts."""

from __future__ import annotations

import hashlib
import io
import json
import os
import subprocess
import sys
import tarfile
import zipfile
from pathlib import Path

import pytest
from tools.release import package_linux as package


def test_distribution_contains_the_complete_pinned_statewide_authority(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    binaries = tmp_path / "binaries"
    binaries.mkdir()
    for name in package.BINARIES:
        (binaries / name).write_bytes(b"fixture executable")
    monkeypatch.setattr(package, "check_binary", lambda _path: None)
    monkeypatch.setattr(package, "launcher_wheels", lambda _root: [])
    monkeypatch.setattr(package, "copy_rust_notices", lambda _root, _destination: None)
    destination = tmp_path / "distribution"
    package.assemble(
        package.ROOT,
        destination,
        version="0.4.0",
        source_sha="a" * 40,
        binaries_dir=binaries,
        wheel_cache=tmp_path / "wheels",
    )
    content = destination / "content/scenarios/michigan"
    manifest = json.loads((content / "statewide-sources.json").read_text())
    for filename, pin in (
        ("defines.toml", "defines_sha256"),
        ("statewide-qualification.json.gz", "qualification_sha256"),
        ("statewide-physical.json.gz", "physical_network_sha256"),
    ):
        assert package.digest(content / filename) == manifest[pin]
    assert (content / "NOTICE").read_bytes() == (
        package.ROOT / "content/scenarios/michigan/NOTICE"
    ).read_bytes()


@pytest.mark.parametrize("ambient_adapter", [None, "c", "binary"])
def test_download_entrypoint_selects_its_bundled_adapter_before_import(
    tmp_path: Path, ambient_adapter: str | None
) -> None:
    entrypoint = tmp_path / "babylon"
    entrypoint.write_bytes((package.ROOT / "tools/release/distribution/babylon").read_bytes())
    (tmp_path / "tools").mkdir()
    (tmp_path / "tools/run_observer_session.py").write_text(
        "import os\n"
        "if os.environ.get('PSYCOPG_IMPL') != 'python':\n"
        "    raise ImportError('an unbundled adapter was selected')\n"
        "def main(arguments):\n"
        "    return 0 if arguments == ['--distribution', '--help'] else 2\n"
    )
    environment = dict(os.environ)
    if ambient_adapter is None:
        environment.pop("PSYCOPG_IMPL", None)
    else:
        environment["PSYCOPG_IMPL"] = ambient_adapter

    result = subprocess.run(
        [sys.executable, "-I", "-B", str(entrypoint), "--help"],
        cwd=tmp_path,
        env=environment,
        capture_output=True,
        text=True,
        check=False,
        timeout=10,
    )

    assert result.returncode == 0, result.stderr


def test_locked_launcher_bytes_are_checked_before_extraction(tmp_path: Path) -> None:
    expected = "a" * 64
    cache = tmp_path / "cache"
    cache.mkdir()
    (cache / f"{expected}.whl").write_bytes(b"tampered")
    with pytest.raises(ValueError, match="hash mismatch"):
        package.install_wheel(
            {
                "hash": f"sha256:{expected}",
                "name": "psycopg",
                "url": "https://files.pythonhosted.org/packages/example.whl",
            },
            tmp_path / "lib",
            cache,
        )
    assert not (tmp_path / "lib").exists()


def test_even_a_pinned_wheel_cannot_write_outside_its_package(tmp_path: Path) -> None:
    content = io.BytesIO()
    with zipfile.ZipFile(content, "w") as archive:
        archive.writestr("../outside", b"unexpected")
    expected = hashlib.sha256(content.getvalue()).hexdigest()
    cache = tmp_path / "cache"
    cache.mkdir()
    (cache / f"{expected}.whl").write_bytes(content.getvalue())
    with pytest.raises(ValueError, match="unsafe path"):
        package.install_wheel(
            {
                "hash": f"sha256:{expected}",
                "name": "psycopg",
                "url": "https://files.pythonhosted.org/packages/example.whl",
            },
            tmp_path / "lib",
            cache,
        )
    assert not (tmp_path / "outside").exists()


def test_archive_is_repeatable_and_preserves_the_executable_entrypoint(tmp_path: Path) -> None:
    first, second = tmp_path / "first.tar.gz", tmp_path / "second.tar.gz"
    for umask, output in ((0o022, first), (0o077, second)):
        previous_umask = os.umask(umask)
        try:
            tree = tmp_path / str(umask) / "babylon-0.4.0-linux-x86_64"
            (tree / "assets").mkdir(parents=True)
            executable = tree / "babylon"
            executable.write_text("#!/usr/bin/env python3\n")
            executable.chmod(0o755)
            package.archive_tree(tree, output, 1_700_000_000)
        finally:
            os.umask(previous_umask)
    assert first.read_bytes() == second.read_bytes()
    assert package.digest(first) == package.digest(second)
    assert (
        first.with_name("first.tar.gz.sha256").read_text()
        == f"{package.digest(first)}  first.tar.gz\n"
    )
    with tarfile.open(first) as archive:
        assert all(member.mode == 0o755 for member in archive.getmembers() if member.isdir())
        entry = archive.getmember("babylon-0.4.0-linux-x86_64/babylon")
        assert entry.mode == 0o755
        assert entry.mtime == 1_700_000_000
