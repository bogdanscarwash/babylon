#!/bin/sh
# Offline release consistency check. Native language pins and lock files,
# immutable tool downloads, and the SQLite byte contract must agree.
# Runs before project dependency setup; Python 3.11+ stdlib only.
# Exit 0 consistent / 1 drifted / 2 unreadable or malformed input.
set -eu

exec python3 - <<'PY'
import pathlib
import re
import sys
import tomllib

class PinDrift(ValueError):
    pass


def require(condition, message):
    if not condition:
        raise PinDrift(message)


def read_toml(path):
    return tomllib.loads(pathlib.Path(path).read_text(encoding="utf-8"))


def exact_version(value, label):
    require(isinstance(value, str) and re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", value),
            f"{label} requires an exact numeric patch version")
    return value


def digest(value, label):
    require(isinstance(value, str) and re.fullmatch(r"[0-9a-f]{64}", value)
            and value != "0" * 64, f"{label} requires a nonzero SHA-256 digest")


def locked_tool(lock, name, version, backend):
    entries = lock["tools"][name]
    require(isinstance(entries, list) and len(entries) == 1,
            f"mise.lock {name} must have one version")
    entry = entries[0]
    require(entry["version"] == version and entry["specifiers"] == [version],
            f"mise.lock {name} differs from its exact mise pin")
    require(entry["backend"] == backend, f"mise.lock {name} backend differs")
    platform = entry.get("platforms.linux-x64")
    require(isinstance(platform, dict), f"mise.lock {name} requires linux-x64")
    checksum = platform.get("checksum", "")
    require(isinstance(checksum, str) and checksum.startswith("sha256:"),
            f"mise.lock {name} requires SHA-256")
    digest(checksum.removeprefix("sha256:"), f"mise.lock {name}")
    return platform


try:
    mise = read_toml(".mise.toml")
    python = exact_version(mise["tools"]["python"], "mise Python")
    uv = exact_version(mise["tools"]["uv"], "mise uv")
    node = exact_version(mise["tools"]["node"], "mise Node.js")
    require(pathlib.Path(".python-version").read_text().strip() == python,
            ".python-version differs from mise Python")
    for variable in ("OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS", "MKL_NUM_THREADS",
                     "NUMEXPR_NUM_THREADS", "RAYON_NUM_THREADS"):
        require(mise["env"][variable] == "1", f"{variable} must remain one")

    toolchain = read_toml("rust/rust-toolchain.toml")["toolchain"]
    rust = exact_version(toolchain["channel"], "Rust toolchain")
    require(set(toolchain["components"]) == {"rustfmt", "clippy"},
            "Rust toolchain must include exactly rustfmt and clippy")
    cargo = read_toml("rust/Cargo.lock")
    require(cargo["version"] > 0 and bool(cargo["package"]), "Cargo.lock is empty")
    for package in cargo["package"]:
        if package.get("source", "").startswith("registry+"):
            digest(package["checksum"], f"Cargo.lock {package['name']}")
    lock = read_toml("uv.lock")
    require(lock["version"] > 0 and bool(lock["package"]), "uv.lock is empty")
    project = read_toml("pyproject.toml")["project"]
    require(lock["requires-python"].replace(" ", "")
            == project["requires-python"].replace(" ", ""),
            "uv.lock Python range differs from pyproject.toml")

    tools_lock = read_toml("mise.lock")
    require(tools_lock["lockfile_version"] == 1, "unsupported mise.lock version")
    locked_python = locked_tool(tools_lock, "python", python, "core:python")
    python_url = (r"https://github\.com/astral-sh/python-build-standalone/releases/download/"
                  r"([0-9]{8})/cpython-" + re.escape(python)
                  + r"\+\1-x86_64-unknown-linux-gnu-install_only_stripped\.tar\.gz")
    require(re.fullmatch(python_url, locked_python["url"]),
            "mise.lock Python requires a matching official dated standalone build")
    require(locked_python.get("provenance") == "github-attestations",
            "mise.lock Python requires GitHub attestations")
    locked_node = locked_tool(tools_lock, "node", node, "core:node")
    require(locked_node["url"] == f"https://nodejs.org/dist/v{node}/node-v{node}-linux-x64.tar.gz",
            "mise.lock Node.js URL differs from its official version")
    locked_uv = locked_tool(tools_lock, "uv", uv, "aqua:astral-sh/uv")
    require(locked_uv["url"] == f"https://github.com/astral-sh/uv/releases/download/{uv}/"
            "uv-x86_64-unknown-linux-gnu.tar.gz", "mise.lock uv URL differs from its official version")
    require(locked_uv.get("provenance") == "github-attestations",
            "mise.lock uv requires GitHub attestations")

    builder = pathlib.Path("tools/build_reference_db.py").read_text()
    pins = re.findall(r'^PINNED_SQLITE_VERSION = "([0-9.]+)"', builder, re.MULTILINE)
    require(len(pins) == 1, "reference builder must declare one SQLite pin")
    manifest = pathlib.Path("data-artifacts.yaml").read_text()
    products = re.findall(r"^product:\n((?:[ \t].*\n|\n)+)", manifest, re.MULTILINE)
    require(len(products) == 1, "manifest must contain one product block")
    versions = re.findall(r'^  sqlite_version: "([0-9.]+)"$', products[0], re.MULTILINE)
    require(versions == pins, "SQLite builder and registry product pins differ")
    hashes = re.findall(r"^  sha256: ([0-9a-f]{64})$", products[0], re.MULTILINE)
    require(len(hashes) == 1, "manifest product requires one SHA-256 digest")
    digest(hashes[0], "manifest product")
except PinDrift as error:
    print(f"check_release_pins: REFUSE: {error}", file=sys.stderr)
    sys.exit(1)
except (OSError, ValueError, KeyError, TypeError) as error:
    print(f"check_release_pins: FATAL: {error}", file=sys.stderr)
    sys.exit(2)

print(f"check_release_pins: OK: Python {python}, uv {uv}, Rust {rust}; SQLite {pins[0]} lockstep.")
PY
