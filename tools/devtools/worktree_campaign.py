#!/usr/bin/env python3
"""Choose a bounded campaign UUID for one Babylon worktree task.

An explicit ``BABYLON_CAMPAIGN_ID`` always wins after canonical UUID
validation. Otherwise, the default is a UUIDv5 derived from the resolved Git
worktree root and a caller-supplied purpose. ``--fresh`` selects UUIDv4 for
one-shot tasks that must not resume an earlier campaign.
"""

from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
import uuid
from collections.abc import Mapping, Sequence
from pathlib import Path
from typing import Final, TextIO

CAMPAIGN_ENV: Final = "BABYLON_CAMPAIGN_ID"
COMMAND_TIMEOUT_SECONDS: Final = 5
MAX_GIT_OUTPUT_BYTES: Final = 4096
MAX_PATH_BYTES: Final = 4096
MAX_PURPOSE_BYTES: Final = 128
_PURPOSE_PATTERN: Final = re.compile(r"[a-z0-9][a-z0-9._:-]*")
_CAMPAIGN_NAMESPACE: Final = uuid.UUID("f3e5d2be-6bc2-5cf0-b10e-5b746ecf60e8")


class CampaignIdentityError(RuntimeError):
    """Raised when a safe campaign identity cannot be selected."""


def _canonical_uuid(value: str) -> str:
    if len(value.encode("utf-8")) > 64:
        raise CampaignIdentityError(f"{CAMPAIGN_ENV} exceeds the 64-byte bound")
    try:
        parsed = uuid.UUID(value)
    except (ValueError, AttributeError) as error:
        raise CampaignIdentityError(f"{CAMPAIGN_ENV} must be a canonical UUID") from error
    if str(parsed) != value:
        raise CampaignIdentityError(f"{CAMPAIGN_ENV} must be a canonical UUID")
    return value


def _validated_purpose(purpose: str) -> str:
    encoded = purpose.encode("utf-8")
    if not encoded:
        raise CampaignIdentityError("campaign purpose must not be empty")
    if len(encoded) > MAX_PURPOSE_BYTES:
        raise CampaignIdentityError(f"campaign purpose exceeds the {MAX_PURPOSE_BYTES}-byte bound")
    if _PURPOSE_PATTERN.fullmatch(purpose) is None:
        raise CampaignIdentityError(
            "campaign purpose must use lowercase letters, digits, '.', '_', ':', or '-'"
        )
    return purpose


def _resolved_worktree_root(repository: Path, git_executable: str = "git") -> Path:
    repository_text = str(repository)
    if len(repository_text.encode("utf-8")) > MAX_PATH_BYTES:
        raise CampaignIdentityError(f"repository path exceeds the {MAX_PATH_BYTES}-byte bound")
    try:
        completed = subprocess.run(  # noqa: S603 -- explicit read-only Git query
            [git_executable, "-C", repository_text, "rev-parse", "--show-toplevel"],
            check=False,
            capture_output=True,
            timeout=COMMAND_TIMEOUT_SECONDS,
        )
    except FileNotFoundError as error:
        raise CampaignIdentityError("git executable was not found") from error
    except subprocess.TimeoutExpired as error:
        raise CampaignIdentityError(
            f"git worktree-root query timed out after {COMMAND_TIMEOUT_SECONDS} seconds"
        ) from error
    except OSError as error:
        raise CampaignIdentityError(
            f"git worktree-root query could not start: {error.strerror or type(error).__name__}"
        ) from error

    if len(completed.stdout) > MAX_GIT_OUTPUT_BYTES or len(completed.stderr) > MAX_GIT_OUTPUT_BYTES:
        raise CampaignIdentityError(
            f"git worktree-root output exceeds the {MAX_GIT_OUTPUT_BYTES}-byte bound"
        )
    if completed.returncode != 0:
        raise CampaignIdentityError(
            f"git worktree-root query exited with status {completed.returncode}"
        )
    try:
        root_text = completed.stdout.decode("utf-8").strip()
    except UnicodeDecodeError as error:
        raise CampaignIdentityError("git worktree-root query emitted non-UTF-8 output") from error
    if not root_text:
        raise CampaignIdentityError("git worktree-root query emitted no path")
    if "\n" in root_text or "\r" in root_text:
        raise CampaignIdentityError("git worktree-root query emitted more than one line")
    if len(root_text.encode("utf-8")) > MAX_PATH_BYTES:
        raise CampaignIdentityError(
            f"resolved worktree path exceeds the {MAX_PATH_BYTES}-byte bound"
        )

    root = Path(root_text)
    if not root.is_absolute():
        raise CampaignIdentityError("git worktree-root query emitted a relative path")
    try:
        resolved = root.resolve(strict=True)
    except OSError as error:
        raise CampaignIdentityError(
            f"resolved Git worktree root is unavailable: {error.strerror or type(error).__name__}"
        ) from error
    if not resolved.is_dir():
        raise CampaignIdentityError("resolved Git worktree root is not a directory")
    return resolved


def select_campaign_id(
    *,
    worktree_root: Path,
    purpose: str,
    fresh: bool,
    environment: Mapping[str, str],
) -> str:
    """Return an explicit, stable, or fresh canonical campaign UUID."""
    configured = environment.get(CAMPAIGN_ENV)
    if configured is not None:
        return _canonical_uuid(configured)

    validated_purpose = _validated_purpose(purpose)
    if fresh:
        return str(uuid.uuid4())

    if not worktree_root.is_absolute():
        raise CampaignIdentityError("resolved Git worktree root must be absolute")
    name = f"{worktree_root}\0{validated_purpose}"
    return str(uuid.uuid5(_CAMPAIGN_NAMESPACE, name))


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description=(
            "Emit a campaign UUID. An explicit BABYLON_CAMPAIGN_ID wins; "
            "otherwise derive a worktree-local UUIDv5 or use --fresh for UUIDv4."
        )
    )
    parser.add_argument("--purpose", required=True, help="bounded task-purpose identity")
    parser.add_argument(
        "--repository",
        type=Path,
        default=Path.cwd(),
        help="path inside the target Git worktree (default: current directory)",
    )
    parser.add_argument(
        "--fresh",
        action="store_true",
        help="emit a UUIDv4 when BABYLON_CAMPAIGN_ID is unset",
    )
    return parser


def main(
    argv: Sequence[str] | None = None,
    *,
    environment: Mapping[str, str] | None = None,
    stdout: TextIO = sys.stdout,
    stderr: TextIO = sys.stderr,
) -> int:
    """Run the campaign-identity CLI."""
    arguments = _parser().parse_args(argv)
    selected_environment = os.environ if environment is None else environment
    try:
        purpose = _validated_purpose(arguments.purpose)
        configured = selected_environment.get(CAMPAIGN_ENV)
        if configured is not None:
            campaign_id = _canonical_uuid(configured)
        else:
            root = _resolved_worktree_root(arguments.repository)
            campaign_id = select_campaign_id(
                worktree_root=root,
                purpose=purpose,
                fresh=arguments.fresh,
                environment=selected_environment,
            )
    except CampaignIdentityError as error:
        print(f"worktree-campaign: {error}", file=stderr)
        return 2
    print(campaign_id, file=stdout)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
