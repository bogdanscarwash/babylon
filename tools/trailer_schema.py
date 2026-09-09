#!/usr/bin/env python3
"""Trailer schema — the coordination-database grammar (git doctrine §5.9 item 1).

The v2 roadmap draft (``ai/_inbox/archive/tui-roadmap-update.md`` §5.3, "Trailers:
the coordination database") specifies a fixed set of structured git trailers every
agent commit carries, so that PR bodies, plan progress, and the ceremony ledger can
all be *derived* from ``git log`` instead of hand-maintained beside it::

    Task: 2026-07-18-track1-organizers-map#10
    Train: train/archive-keel
    Lane: pages          Safety: P
    Pinned: dev@72def41c
    Baselines: untouched
    Session: <run id>
    Co-Authored-By: …

This module is the single machine-checkable definition of that grammar — the
canonical reference doc (``docs/reference/trailer-schema.rst``) describes it in
prose; this file is what parses and validates it. :mod:`tools.generate_pr_body`
imports it to surface trailers in generated PR bodies.

The ``Baselines`` trailer's ``blessed(<slug>)`` variant is governed jointly with
this module and ``tools/check_baseline_ceremony.py`` (§6.5 provenance & ceremony,
adoption item 3) — the regex here is intentionally identical to that gate's so the
two never drift apart.
"""

from __future__ import annotations

import re
import subprocess
from dataclasses import dataclass
from pathlib import Path

# ---------------------------------------------------------------------------
# Grammar: one compiled pattern per trailer key, plus a human-readable example
# and description for the reference doc / error messages to cite.
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class TrailerSpec:
    """One trailer's grammar: its value pattern, an example, and its meaning."""

    key: str
    pattern: re.Pattern[str]
    example: str
    description: str
    required: bool = True


TRAILER_SCHEMA: dict[str, TrailerSpec] = {
    "Task": TrailerSpec(
        key="Task",
        pattern=re.compile(r"^[a-z0-9][a-z0-9-]*#[0-9]+$"),
        example="2026-07-18-track1-organizers-map#10",
        description="Plan-slug '#' task-number this commit discharges.",
    ),
    "Train": TrailerSpec(
        key="Train",
        pattern=re.compile(r"^train/[a-z0-9][a-z0-9./-]*$"),
        example="train/archive-keel",
        description="The train branch this task's work integrates into.",
    ),
    "Lane": TrailerSpec(
        key="Lane",
        pattern=re.compile(r"^[a-z][a-z0-9-]*$"),
        example="pages",
        description="Work-order lane label (plan-defined slug).",
    ),
    "Safety": TrailerSpec(
        key="Safety",
        pattern=re.compile(r"^[PS]$"),
        example="P",
        description="Parallel-safe (P) or must-run-serial (S) — the WORK-ORDERS.md legend.",
    ),
    "Pinned": TrailerSpec(
        key="Pinned",
        pattern=re.compile(r"^[A-Za-z0-9_./-]+@[0-9a-f]{6,40}$"),
        example="dev@72def41c",
        description="The base ref + SHA this task branched from (the base-branch trap guard).",
    ),
    "Baselines": TrailerSpec(
        key="Baselines",
        # Mirrors tools/check_baseline_ceremony.py's _TRAILER_RE exactly; kept
        # as a sibling literal rather than an import so this module has no
        # runtime dependency on the ceremony gate (or vice versa).
        pattern=re.compile(r"^(untouched|blessed\([a-z0-9][a-z0-9-]*\))$"),
        example="untouched",
        description=(
            "'untouched' for ordinary commits; 'blessed(<slug>)' when the commit "
            "is a declared baseline ceremony (§6.5)."
        ),
    ),
    "Session": TrailerSpec(
        key="Session",
        pattern=re.compile(r"^\S+$"),
        example="89821c94-b151-4c44-84ad-ec019f7e1ec5",
        description="Opaque run/session id of the agent (or human) that made the commit.",
    ),
    "Co-Authored-By": TrailerSpec(
        key="Co-Authored-By",
        pattern=re.compile(r"^[^<>]+ <[^<>@\s]+@[^<>@\s]+>$"),
        example="Claude Fable 5 <noreply@anthropic.com>",
        description="Standard git co-author trailer (GitHub-recognized, not doctrine-specific).",
        required=False,
    ),
}

# Trailer keys that are unconditionally required on every agent commit per
# §5.3. ``Co-Authored-By`` is a real trailer this repo uses constantly but is
# not itself part of the coordination-database schema, so it is documented
# above (for grammar checking, since it *does* appear) without being demanded.
REQUIRED_TRAILER_KEYS: tuple[str, ...] = tuple(
    spec.key for spec in TRAILER_SCHEMA.values() if spec.required
)


def validate_trailer_value(key: str, value: str) -> bool:
    """True iff ``value`` matches the declared grammar for trailer ``key``.

    Unknown keys are not this schema's concern and always validate — they may
    be a real git trailer (``Signed-off-by``, ``Reviewed-by``, …) this
    doctrine simply doesn't govern.
    """
    spec = TRAILER_SCHEMA.get(key)
    if spec is None:
        return True
    return spec.pattern.match(value) is not None


def validate_trailers(trailers: dict[str, str]) -> list[str]:
    """Return one violation string per malformed or missing required trailer.

    Empty list means the trailer set is fully schema-conformant. This checks
    shape only — it does not know whether a ``Task`` id actually exists in a
    plan, only that it has the right grammar.
    """
    violations: list[str] = []
    for key in REQUIRED_TRAILER_KEYS:
        if key not in trailers:
            violations.append(f"missing required trailer: {key}")
    for key, value in trailers.items():
        if not validate_trailer_value(key, value):
            spec = TRAILER_SCHEMA[key]
            violations.append(
                f"{key}: {value!r} does not match schema (expected shape like {spec.example!r})"
            )
    return violations


# ---------------------------------------------------------------------------
# Parsing: reuse git's own trailer machinery (git interpret-trailers) rather
# than hand-rolling the "trailer block at the end of the message" heuristic.
# ---------------------------------------------------------------------------

# Field/trailer separators chosen to never collide with commit-message text;
# \x01 and \x02 are non-printable control bytes git will never emit itself.
_FIELD_SEP = "\x01"
_TRAILER_SEP = "\x02"

_TRAILER_LINE_RE = re.compile(r"^([A-Za-z][A-Za-z0-9-]*): (.*)$")


def parse_trailer_block(raw: str) -> dict[str, str]:
    """Parse a ``%(trailers:only,unfold,separator=...)`` block into a dict.

    Repeated keys (e.g. multiple ``Co-Authored-By`` lines) keep the *last*
    value under that key for the aggregate dict; callers that need every
    occurrence should split on ``_TRAILER_SEP`` themselves. In practice each
    schema key in §5.3 appears at most once per commit.
    """
    trailers: dict[str, str] = {}
    for line in raw.split(_TRAILER_SEP):
        line = line.strip()
        if not line:
            continue
        match = _TRAILER_LINE_RE.match(line)
        if match is None:
            continue
        trailers[match.group(1)] = match.group(2)
    return trailers


def git_log_trailers(range_spec: str, repo_root: Path) -> list[tuple[str, str, dict[str, str]]]:
    """Return ``(sha, subject, trailers)`` for every non-merge commit in range.

    Uses git's built-in trailer parser (``%(trailers:...)``) instead of
    reimplementing RFC-822-style trailer folding — Reuse Over Recreation.
    Records are NUL-terminated (``%x00``) so a commit whose trailer block is
    empty or multi-line can never be confused with a record boundary.
    """
    fmt = f"%H{_FIELD_SEP}%s{_FIELD_SEP}%(trailers:only,unfold,separator={_TRAILER_SEP})%x00"
    result = subprocess.run(
        ["git", "log", "--no-merges", f"--format={fmt}", range_spec],
        cwd=repo_root,
        capture_output=True,
        text=True,
        check=True,
    )
    entries: list[tuple[str, str, dict[str, str]]] = []
    for record in result.stdout.split("\x00"):
        record = record.strip("\n")
        if not record:
            continue
        parts = record.split(_FIELD_SEP)
        sha, subject = parts[0], parts[1]
        raw_trailers = parts[2] if len(parts) > 2 else ""
        entries.append((sha, subject, parse_trailer_block(raw_trailers)))
    return entries
