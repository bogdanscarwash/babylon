#!/usr/bin/env python3
"""Fail closed on Cargo configuration that can escape the managed host policy."""

from __future__ import annotations

import os
import shlex
import sys
import tomllib
from pathlib import Path
from typing import Any

MAX_ALIASES = 256
MAX_ALIAS_BYTES = 8192
MAX_ALIAS_TOKENS = 128
MAX_ANCESTORS = 128
MAX_CONFIG_FILES = 512
MAX_ENVIRONMENT_ENTRIES = 4096
MAX_INCLUDES_PER_FILE = 128


class PolicyError(Exception):
    """A Cargo configuration value can bypass the managed resource boundary."""


def add_active_config(configs: list[Path], cargo_dir: Path) -> None:
    """Append the one config Cargo selects from a `.cargo` directory."""
    legacy = cargo_dir / "config"
    modern = cargo_dir / "config.toml"
    configs.append(legacy if legacy.exists() else modern)


def initial_configs(working_directory: Path) -> list[Path]:
    """Return Cargo's bounded ancestor config chain and its home config."""
    configs: list[Path] = []
    current = working_directory
    reached_root = False
    for _ in range(MAX_ANCESTORS):
        add_active_config(configs, current / ".cargo")
        parent = current.parent
        if parent == current:
            reached_root = True
            break
        current = parent
    if not reached_root:
        raise PolicyError("Cargo config ancestry exceeds the host-policy bound")

    cargo_home_value = os.environ.get("CARGO_HOME")
    cargo_home = Path(cargo_home_value) if cargo_home_value else Path.home() / ".cargo"
    if not cargo_home.is_absolute():
        cargo_home = working_directory / cargo_home
    add_active_config(configs, cargo_home)
    return configs


def load_config(path: Path) -> dict[str, Any]:
    """Read one exact TOML mapping with a path-specific refusal."""
    try:
        with path.open("rb") as stream:
            value = tomllib.load(stream)
    except OSError as error:
        raise PolicyError(f"cannot read Cargo config {path}: {error}") from error
    except tomllib.TOMLDecodeError as error:
        raise PolicyError(f"cannot parse Cargo config {path}: {error}") from error
    if not isinstance(value, dict):
        raise PolicyError(f"Cargo config {path} is not a table")
    return value


def add_includes(configs: list[Path], path: Path, value: dict[str, Any]) -> None:
    """Add a bounded Cargo `include` graph to the pending config list."""
    includes = value.get("include", [])
    if not isinstance(includes, list):
        raise PolicyError(f"Cargo config {path} has a non-list include value")
    if len(includes) > MAX_INCLUDES_PER_FILE:
        raise PolicyError(f"Cargo config {path} exceeds the include bound")

    for index in range(MAX_INCLUDES_PER_FILE):
        if index >= len(includes):
            break
        entry = includes[index]
        optional = False
        if isinstance(entry, str):
            include_value = entry
        elif isinstance(entry, dict):
            include_value = entry.get("path")
            optional = entry.get("optional", False)
            if not isinstance(include_value, str) or not isinstance(optional, bool):
                raise PolicyError(f"Cargo config {path} has an invalid include table")
        else:
            raise PolicyError(f"Cargo config {path} has an invalid include entry")

        include_path = Path(include_value)
        if not include_path.is_absolute():
            include_path = path.parent / include_path
        include_path = include_path.resolve()
        if include_path.exists():
            configs.append(include_path)
        elif not optional:
            raise PolicyError(f"Cargo config {path} requires missing include {include_path}")


def alias_tokens(name: str, value: object, origin: str) -> list[str]:
    """Normalize one Cargo alias while bounding parser and process input."""
    if isinstance(value, str):
        if len(value.encode()) > MAX_ALIAS_BYTES:
            raise PolicyError(f"Cargo alias {name!r} from {origin} exceeds the byte bound")
        if value.lstrip().startswith("!"):
            raise PolicyError(
                f"Cargo alias {name!r} from {origin} executes an external command; "
                "managed repositories require an inspectable Cargo expansion"
            )
        try:
            tokens = shlex.split(value)
        except ValueError as error:
            raise PolicyError(f"cannot parse Cargo alias {name!r} from {origin}: {error}") from error
    elif isinstance(value, list) and all(isinstance(token, str) for token in value):
        tokens = list(value)
    else:
        raise PolicyError(f"Cargo alias {name!r} from {origin} is not a string or string array")

    if len(tokens) > MAX_ALIAS_TOKENS:
        raise PolicyError(f"Cargo alias {name!r} from {origin} exceeds the token bound")
    if tokens and tokens[0].startswith("!"):
        raise PolicyError(
            f"Cargo alias {name!r} from {origin} executes an external command; "
            "managed repositories require an inspectable Cargo expansion"
        )
    return tokens


def validate_job_value(name: str, value: str, origin: str) -> None:
    """Accept only an alias job count inside the four-job ceiling."""
    if value not in {"1", "2", "3", "4"}:
        raise PolicyError(f"Cargo alias {name!r} from {origin} exceeds the four-job host limit")


def validate_alias(name: str, value: object, origin: str) -> None:
    """Reject post-dispatch alias tokens that can replace managed resources."""
    tokens = alias_tokens(name, value, origin)
    expect_jobs = False
    before_child_arguments = True
    for index in range(MAX_ALIAS_TOKENS):
        if index >= len(tokens):
            break
        token = tokens[index]
        if not before_child_arguments:
            continue
        if expect_jobs:
            validate_job_value(name, token, origin)
            expect_jobs = False
            continue
        if token == "--":
            before_child_arguments = False
        elif token in {"-j", "--jobs"}:
            expect_jobs = True
        elif token.startswith("-j") and token != "-j":
            validate_job_value(name, token[2:], origin)
        elif token.startswith("--jobs="):
            validate_job_value(name, token.removeprefix("--jobs="), origin)
        elif token in {
            "--config",
            "--target-dir",
            "--manifest-path",
            "-C",
            "--directory",
        } or token.startswith(
            ("--config=", "--target-dir=", "--manifest-path=", "-C", "--directory=")
        ):
            raise PolicyError(
                f"Cargo alias {name!r} from {origin} changes dispatcher-owned configuration or repository selection"
            )
    if expect_jobs:
        raise PolicyError(f"Cargo alias {name!r} from {origin} has --jobs without a value")


def validate_config(path: Path, config: dict[str, Any]) -> None:
    """Validate the resource-bearing tables and aliases in one config."""
    build = config.get("build", {})
    if not isinstance(build, dict):
        raise PolicyError(f"Cargo config {path} has an invalid build table")
    for forbidden in ("target-dir", "build-dir", "rustc-wrapper", "rustc-workspace-wrapper"):
        if forbidden in build:
            raise PolicyError(
                f"Cargo config {path} sets build.{forbidden}; "
                "the repository dispatcher owns that boundary"
            )
    if "jobs" in build:
        jobs = build["jobs"]
        if not isinstance(jobs, int) or isinstance(jobs, bool) or jobs not in range(1, 5):
            raise PolicyError(f"Cargo config {path} exceeds the four-job host limit")

    configured_environment = config.get("env", {})
    if not isinstance(configured_environment, dict):
        raise PolicyError(f"Cargo config {path} has an invalid env table")
    for cache_key in (
        "SCCACHE_DIR",
        "SCCACHE_SERVER_UDS",
        "SCCACHE_CACHE_SIZE",
        "SCCACHE_CLIENT_SIDE",
    ):
        if cache_key in configured_environment:
            raise PolicyError(
                f"Cargo config {path} sets env.{cache_key}; "
                "the repository dispatcher owns the compiler-cache boundary"
            )

    aliases = config.get("alias", {})
    if not isinstance(aliases, dict):
        raise PolicyError(f"Cargo config {path} has an invalid alias table")
    if len(aliases) > MAX_ALIASES:
        raise PolicyError(f"Cargo config {path} exceeds the alias bound")
    for index, (name, value) in enumerate(aliases.items()):
        if index >= MAX_ALIASES:
            raise PolicyError(f"Cargo config {path} exceeds the alias bound")
        validate_alias(name, value, str(path))


def validate_environment_aliases() -> None:
    """Apply the same rule to Cargo aliases supplied through the environment."""
    environment = list(os.environ.items())
    if len(environment) > MAX_ENVIRONMENT_ENTRIES:
        raise PolicyError("process environment exceeds the host-policy bound")
    for index in range(MAX_ENVIRONMENT_ENTRIES):
        if index >= len(environment):
            break
        name, value = environment[index]
        if name.startswith("CARGO_ALIAS_"):
            validate_alias(name.removeprefix("CARGO_ALIAS_").lower(), value, name)


def validate(working_directory: Path) -> None:
    """Validate every active config and bounded include exactly once."""
    configs = initial_configs(working_directory)
    seen: set[Path] = set()
    for index in range(MAX_CONFIG_FILES):
        if index >= len(configs):
            break
        config_path = configs[index].resolve()
        if config_path in seen or not config_path.exists():
            continue
        seen.add(config_path)
        config = load_config(config_path)
        validate_config(config_path, config)
        add_includes(configs, config_path, config)
    else:
        raise PolicyError("Cargo config include graph exceeds the host-policy bound")
    if len(configs) > MAX_CONFIG_FILES:
        raise PolicyError("Cargo config include graph exceeds the host-policy bound")
    validate_environment_aliases()


def main() -> int:
    """Validate the effective Cargo working directory named by the dispatcher."""
    if len(sys.argv) != 2:
        print(f"usage: {sys.argv[0]} <effective-cargo-directory>", file=sys.stderr)
        return 64
    working_directory = Path(sys.argv[1])
    if not working_directory.is_absolute() or not working_directory.is_dir():
        print(f"cargo: invalid effective working directory: {working_directory}", file=sys.stderr)
        return 64
    try:
        validate(working_directory.resolve())
    except PolicyError as error:
        print(f"cargo: {error}", file=sys.stderr)
        return 64
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
