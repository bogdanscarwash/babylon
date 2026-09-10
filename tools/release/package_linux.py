#!/usr/bin/env python3
"""Package the native Linux observer and its locked launcher without a checkout dependency."""

from __future__ import annotations

import argparse
import gzip
import hashlib
import json
import os
import re
import shutil
import stat
import struct
import subprocess
import sys
import tarfile
import tempfile
import tomllib
import urllib.request
import zipfile
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
BINARIES = ("babylon-client", "babylon-runtime")
LAUNCHER_DEPENDENCIES = ("psycopg", "typing-extensions")
VERSION_PATTERN = r"(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)"


def digest(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def command(args: list[str], *, cwd: Path) -> str:
    return subprocess.run(args, cwd=cwd, check=True, capture_output=True, text=True).stdout.strip()


def check_source(root: Path, source_sha: str) -> int:
    if re.fullmatch(r"[0-9a-f]{40}", source_sha) is None:
        raise ValueError("source SHA must be a complete lowercase Git commit")
    if command(["git", "rev-parse", "HEAD"], cwd=root) != source_sha:
        raise ValueError("the package must use the exact checked-out source commit")
    if command(["git", "status", "--porcelain", "--untracked-files=no"], cwd=root):
        raise ValueError("commit tracked changes before packaging a release")
    return int(command(["git", "show", "-s", "--format=%ct", source_sha], cwd=root))


def copy_file(source: Path, destination: Path, *, executable: bool = False) -> None:
    if not source.is_file() or source.is_symlink():
        raise ValueError(f"package input is not a regular file: {source}")
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source, destination)
    destination.chmod(0o755 if executable else 0o644)


def check_binary(path: Path) -> None:
    with path.open("rb") as stream:
        header = stream.read(20)
    if (
        len(header) != 20
        or header[:6] != b"\x7fELF\x02\x01"
        or struct.unpack("<H", header[18:20])[0] != 62
    ):
        raise ValueError(f"expected a Linux x86_64 ELF binary: {path}")
    dynamic = command(["readelf", "--dynamic", str(path)], cwd=path.parent)
    if "bevy_dylib" in dynamic:
        raise ValueError("development-only Bevy dynamic linking cannot be distributed")


def launcher_wheels(root: Path) -> list[dict[str, str]]:
    lock = tomllib.loads((root / "uv.lock").read_text(encoding="utf-8"))
    result = []
    for name in LAUNCHER_DEPENDENCIES:
        packages = [package for package in lock["package"] if package["name"] == name]
        if len(packages) != 1:
            raise ValueError(f"expected exactly one locked launcher dependency: {name}")
        package = packages[0]
        wheels = [
            wheel for wheel in package["wheels"] if wheel["url"].endswith("-py3-none-any.whl")
        ]
        if len(wheels) != 1 or not wheels[0]["hash"].startswith("sha256:"):
            raise ValueError(f"expected one SHA-pinned pure-Python wheel: {name}")
        result.append({"name": name, "version": package["version"], **wheels[0]})
    return result


def install_wheel(wheel: dict[str, Any], destination: Path, cache: Path) -> None:
    """Extract only a hash-verified pure-Python wheel; never run installation code."""
    expected = wheel["hash"].removeprefix("sha256:")
    if re.fullmatch(r"[0-9a-f]{64}", expected) is None or not wheel["url"].startswith(
        "https://files.pythonhosted.org/packages/"
    ):
        raise ValueError("launcher wheels require an official HTTPS URL and SHA-256")
    cache.mkdir(parents=True, exist_ok=True)
    archive_path = cache / f"{expected}.whl"
    if not archive_path.exists():
        # Only the checked PyPI HTTPS origin is admitted, and bytes are hashed before use.
        with urllib.request.urlopen(wheel["url"], timeout=60) as response:  # noqa: S310
            contents = response.read(10_000_001)
        if len(contents) > 10_000_000 or hashlib.sha256(contents).hexdigest() != expected:
            raise ValueError(f"locked launcher wheel hash or size mismatch: {wheel['name']}")
        archive_path.write_bytes(contents)
    if digest(archive_path) != expected:
        raise ValueError(f"cached launcher wheel hash mismatch: {wheel['name']}")
    destination.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(archive_path) as archive:
        for member in archive.infolist():
            relative = Path(member.filename)
            if (
                relative.is_absolute()
                or ".." in relative.parts
                or stat.S_ISLNK(member.external_attr >> 16)
            ):
                raise ValueError("unsafe path in locked launcher wheel")
            target = destination / relative
            if member.is_dir():
                target.mkdir(parents=True, exist_ok=True)
            else:
                if target.exists():
                    raise ValueError(f"launcher wheels overlap at {relative}")
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(archive.read(member))
                target.chmod(0o644)


def copy_rust_notices(root: Path, destination: Path) -> None:
    metadata = json.loads(
        command(
            [
                "cargo",
                "metadata",
                "--locked",
                "--offline",
                "--format-version",
                "1",
                "--filter-platform",
                "x86_64-unknown-linux-gnu",
            ],
            cwd=root / "rust",
        )
    )
    inventory = []
    for package in sorted(
        metadata["packages"], key=lambda package: (package["name"], package["version"])
    ):
        if package["source"] is None:
            continue
        source = Path(package["manifest_path"]).parent
        candidates = [
            path
            for path in source.iterdir()
            if path.name.lower().startswith(("license", "licence", "copying", "notice"))
        ]
        if package.get("license_file"):
            candidates.append(source / package["license_file"])
        notices: set[Path] = set()
        for candidate in candidates:
            notices.update(candidate.rglob("*") if candidate.is_dir() else [candidate])
        paths = []
        for notice in sorted(notices):
            if not notice.is_file() or notice.is_symlink():
                continue
            relative = notice.relative_to(source)
            packaged = Path("rust") / f"{package['name']}-{package['version']}" / relative
            copy_file(notice, destination / packaged)
            paths.append(packaged.as_posix())
        inventory.append(
            {
                "name": package["name"],
                "version": package["version"],
                "license_expression": package["license"],
                "repository": package["repository"],
                "cargo_source": package["source"],
                "included_notice_files": paths,
            }
        )
    (destination / "rust-dependencies.json").write_text(
        json.dumps(inventory, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )


def assemble(
    root: Path,
    destination: Path,
    *,
    version: str,
    source_sha: str,
    binaries_dir: Path,
    wheel_cache: Path,
) -> None:
    destination.mkdir()
    binary_hashes = {}
    for name in BINARIES:
        source = binaries_dir / name
        check_binary(source)
        copy_file(source, destination / "bin" / name, executable=True)
        binary_hashes[name] = digest(source)
    for path in (
        "tools/run_observer_session.py",
        "content/scenarios/michigan/defines.toml",
        "content/scenarios/michigan/statewide-sources.json",
        "content/scenarios/michigan/statewide-qualification.json.gz",
        "content/scenarios/michigan/statewide-physical.json.gz",
        "content/scenarios/michigan/NOTICE",
        "docker/postgres/Dockerfile",
        "docker/postgres/patch-entrypoint.awk",
        "docker/postgres/initdb/01-babylon-init.sql",
    ):
        copy_file(root / path, destination / path)
    distribution = root / "tools/release/distribution"
    for name in ("compose.yaml", "postgresql.conf"):
        copy_file(distribution / name, destination / "distribution" / name)
    copy_file(distribution / "babylon", destination / "babylon", executable=True)
    copy_file(root / "tools/release/DOWNLOAD.md", destination / "README.md")
    copy_file(
        distribution / "99-preview-durability.sql",
        destination / "docker/postgres/initdb/99-preview-durability.sql",
    )
    for path in (
        "LICENSE",
        "LICENSE-ASSETS",
        "LICENSING.md",
        "rust/Cargo.lock",
        "assets/fonts/SourceSans3-OFL.txt",
        "assets/fonts/BarlowCondensed-OFL.txt",
        "assets/fonts/manifest.toml",
        "assets/licenses/FluidR3-GM.txt",
        "assets/audio-renders.json",
    ):
        copy_file(root / path, destination / "notices" / path)
    wheels = launcher_wheels(root)
    for wheel in wheels:
        install_wheel(wheel, destination / "lib", wheel_cache)
    copy_rust_notices(root, destination / "notices")
    (destination / "release.json").write_text(
        json.dumps(
            {
                "schema": 1,
                "version": version,
                "platform": "linux-x86_64",
                "source_sha": source_sha,
                "source_url": f"https://github.com/percy-raskova/babylon/tree/{source_sha}",
                "binaries_sha256": binary_hashes,
                "launcher_dependencies": wheels,
            },
            indent=2,
            sort_keys=True,
        )
        + "\n",
        encoding="utf-8",
    )
    files = sorted(path for path in destination.rglob("*") if path.is_file())
    (destination / "SHA256SUMS").write_text(
        "".join(f"{digest(path)}  {path.relative_to(destination).as_posix()}\n" for path in files),
        encoding="utf-8",
    )


def archive_tree(source: Path, archive_path: Path, timestamp: int) -> None:
    def normalize(info: tarfile.TarInfo) -> tarfile.TarInfo:
        info.uid = info.gid = 0
        info.uname = info.gname = "root"
        info.mtime = timestamp
        if info.isdir():
            info.mode = 0o755
        return info

    with (
        archive_path.open("xb") as output,
        gzip.GzipFile(
            filename="",
            mode="wb",
            fileobj=output,
            mtime=timestamp,
            compresslevel=6,
        ) as compressed,
        tarfile.open(fileobj=compressed, mode="w") as archive,
    ):
        archive.add(source, arcname=source.name, filter=normalize)
    checksum = archive_path.with_name(archive_path.name + ".sha256")
    checksum.write_text(f"{digest(archive_path)}  {archive_path.name}\n", encoding="ascii")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--version", required=True)
    parser.add_argument("--source-sha", required=True)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--binaries-dir", type=Path)
    mode.add_argument("--build", action="store_true")
    parser.add_argument("--output-dir", type=Path, default=ROOT / "dist/release")
    parser.add_argument("--wheel-cache", type=Path)
    args = parser.parse_args()
    try:
        if re.fullmatch(VERSION_PATTERN, args.version) is None:
            raise ValueError("version must have three canonical numeric components")
        timestamp = check_source(ROOT, args.source_sha)
        if args.build:
            subprocess.run(
                [
                    "cargo",
                    "build",
                    "--locked",
                    "--release",
                    "-p",
                    "babylon-persistence",
                    "--bin",
                    "babylon-runtime",
                    "-p",
                    "babylon-client",
                    "--bin",
                    "babylon-client",
                ],
                cwd=ROOT / "rust",
                check=True,
            )
            target = Path(os.environ.get("CARGO_TARGET_DIR", "target"))
            args.binaries_dir = (
                target if target.is_absolute() else ROOT / "rust" / target
            ) / "release"
        output = args.output_dir.resolve()
        output.mkdir(parents=True, exist_ok=True)
        name = f"babylon-{args.version}-linux-x86_64"
        archive_path = output / f"{name}.tar.gz"
        if archive_path.exists() or archive_path.with_name(archive_path.name + ".sha256").exists():
            raise ValueError("refusing to overwrite an existing release artifact")
        with tempfile.TemporaryDirectory(prefix=".package-", dir=output) as temporary:
            work = Path(temporary)
            package = work / name
            assemble(
                ROOT,
                package,
                version=args.version,
                source_sha=args.source_sha,
                binaries_dir=args.binaries_dir.resolve(),
                wheel_cache=args.wheel_cache or work / "wheels",
            )
            archive_tree(package, archive_path, timestamp)
        print(archive_path)
        return 0
    except (OSError, ValueError, subprocess.CalledProcessError, zipfile.BadZipFile) as error:
        print(f"Native packaging refused: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
