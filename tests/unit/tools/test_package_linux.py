"""Native distribution byte integrity and relocatable launcher contracts."""

from __future__ import annotations

import hashlib
import io
import os
import subprocess
import sys
import tarfile
import zipfile
from pathlib import Path

import pytest
from tools.release import package_linux as package


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
    tree = tmp_path / "babylon-0.4.0-linux-x86_64"
    tree.mkdir()
    executable = tree / "babylon"
    executable.write_text("#!/usr/bin/env python3\n")
    executable.chmod(0o755)
    first, second = tmp_path / "first.tar.gz", tmp_path / "second.tar.gz"
    package.archive_tree(tree, first, 1_700_000_000)
    package.archive_tree(tree, second, 1_700_000_000)
    assert first.read_bytes() == second.read_bytes()
    assert (
        first.with_name("first.tar.gz.sha256").read_text()
        == f"{package.digest(first)}  first.tar.gz\n"
    )
    with tarfile.open(first) as archive:
        entry = archive.getmember("babylon-0.4.0-linux-x86_64/babylon")
        assert entry.mode == 0o755
        assert entry.mtime == 1_700_000_000
