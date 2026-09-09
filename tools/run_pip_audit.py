#!/usr/bin/env python3
"""pip-audit policy wrapper — hard-fails on unreasoned or expired ignores.

Security stack (Program 14/15 Phase 3, owner item 41). ``pip-audit`` supports
suppressing known vulnerabilities via ``--ignore-vuln``, but a bare CLI flag
has no memory of *why* it was added or *when* it should be revisited —
ignores rot silently. This wrapper reads a policy file
(``security/pip-audit-ignores.toml``) where every ignore MUST carry an
``id``, a non-empty ``reason``, and an ISO ``expires`` date, and HARD-FAILS
(exit 2) rather than silently degrading (Constitution III.11 Loud Failure):

* a malformed entry (missing/empty ``id``/``reason``, unparseable
  ``expires``) fails validation before anything runs;
* an entry whose ``expires`` date has passed fails too, forcing the
  quarterly re-review the policy promises — an ignore can never coast
  unreviewed.

Only once every entry is valid and unexpired does the wrapper build
``--ignore-vuln`` flags and hand off to the real ``pip-audit`` (via
``uv run pip-audit`` by default — ADR095 swept the Poetry toolchain — or
plain ``pip-audit`` with ``--no-uv``), streaming its output and
propagating its exit code.

Usage:
    uv run python tools/run_pip_audit.py                # full run
    uv run python tools/run_pip_audit.py --check-only    # policy lint only
    uv run python tools/run_pip_audit.py --no-uv         # plain pip-audit
"""

from __future__ import annotations

import argparse
import subprocess
import sys
import tomllib
from datetime import date
from pathlib import Path
from typing import Any

#: Canonical ignore policy file (Program 14/15 Phase 3).
DEFAULT_IGNORES_FILE: Path = (
    Path(__file__).resolve().parent.parent / "security" / "pip-audit-ignores.toml"
)

#: Fixed upper bound on ignore entries (Power-of-10 rule 2 — a policy file
#: needing more entries than this is itself a signal the ignore list needs
#: pruning, not a reason to lift the bound).
MAX_IGNORE_ENTRIES: int = 500


def load_ignores_file(path: Path) -> dict[str, Any]:
    """Parse the pip-audit ignore policy TOML file.

    :param path: Path to the policy file.
    :returns: The parsed TOML document.
    :raises FileNotFoundError: If ``path`` does not exist.
    :raises tomllib.TOMLDecodeError: If ``path`` is not valid TOML.
    """
    with path.open("rb") as handle:
        return tomllib.load(handle)


def get_ignore_entries(policy: dict[str, Any]) -> list[dict[str, Any]]:
    """Extract the ``[[ignore]]`` array of tables from a parsed policy doc.

    :param policy: Parsed TOML document (see :func:`load_ignores_file`).
    :returns: The ``ignore`` entries, or ``[]`` if the key is absent.
    :raises ValueError: If the ``ignore`` key is present but not a list.
    """
    entries = policy.get("ignore", [])
    if not isinstance(entries, list):
        raise ValueError("'ignore' key must be an array of tables ([[ignore]])")
    return entries[:MAX_IGNORE_ENTRIES]


def validate_entry(entry: dict[str, Any], index: int) -> str | None:
    """Validate a single ignore entry.

    :param entry: One ``[[ignore]]`` table.
    :param index: Its position in the array, used to label the error when
        the entry has no usable ``id`` to name itself by.
    :returns: ``None`` if valid, else a human-readable error message naming
        the offending entry.
    """
    entry_id = entry.get("id")
    has_id = isinstance(entry_id, str) and entry_id.strip() != ""
    label = entry_id if has_id else f"entry #{index}"
    if not has_id:
        return f"ignore {label}: missing or empty 'id'"

    reason = entry.get("reason")
    if not isinstance(reason, str) or reason.strip() == "":
        return f"ignore '{label}': missing or empty 'reason'"

    expires = entry.get("expires")
    if not isinstance(expires, str) or expires.strip() == "":
        return f"ignore '{label}': missing 'expires' date"
    try:
        date.fromisoformat(expires)
    except ValueError:
        return f"ignore '{label}': 'expires' is not a valid ISO date (YYYY-MM-DD): {expires!r}"
    return None


def validate_entries(entries: list[dict[str, Any]]) -> list[str]:
    """Validate every ignore entry.

    :param entries: Parsed ``[[ignore]]`` entries.
    :returns: Error messages for offending entries (``[]`` if all valid).
    """
    errors: list[str] = []
    for index, entry in enumerate(entries[:MAX_IGNORE_ENTRIES]):
        if not isinstance(entry, dict):
            errors.append(f"entry #{index}: not a table (expected id/reason/expires)")
            continue
        error = validate_entry(entry, index)
        if error is not None:
            errors.append(error)
    return errors


def find_expired_entries(entries: list[dict[str, Any]], today: date) -> list[dict[str, Any]]:
    """Return entries whose ``expires`` date has passed.

    Precondition: ``entries`` has already passed :func:`validate_entries`
    (every entry has a parseable ``expires`` string) — callers must validate
    first.

    :param entries: Parsed ``[[ignore]]`` entries.
    :param today: The date to compare against (injected for testability).
    :returns: Entries where ``today`` is strictly after ``expires`` — the
        ``expires`` date itself is the last valid day for the ignore.
    """
    expired: list[dict[str, Any]] = []
    for entry in entries[:MAX_IGNORE_ENTRIES]:
        expires_date = date.fromisoformat(entry["expires"])
        if today > expires_date:
            expired.append(entry)
    return expired


def build_ignore_vuln_args(entries: list[dict[str, Any]]) -> list[str]:
    """Build one ``--ignore-vuln <id>`` pair per entry, in file order.

    :param entries: Parsed (and validated) ``[[ignore]]`` entries.
    :returns: Flattened ``--ignore-vuln`` argument list for pip-audit.
    """
    args: list[str] = []
    for entry in entries[:MAX_IGNORE_ENTRIES]:
        args.append("--ignore-vuln")
        args.append(str(entry["id"]))
    return args


def build_pip_audit_command(entries: list[dict[str, Any]], no_uv: bool) -> list[str]:
    """Build the full pip-audit invocation.

    :param entries: Parsed (and validated) ``[[ignore]]`` entries.
    :param no_uv: If ``True``, invoke plain ``pip-audit``; else
        ``uv run --frozen --extra ops pip-audit`` (ADR095: uv is the
        dependency toolchain).
    :returns: The full argv to hand to :func:`subprocess.run`.
    """
    base = ["pip-audit"] if no_uv else ["uv", "run", "--frozen", "--extra", "ops", "pip-audit"]
    return base + build_ignore_vuln_args(entries)


def _format_expired(entry: dict[str, Any]) -> str:
    """Format one expired entry for the exit-2 report.

    :param entry: An entry returned by :func:`find_expired_entries`.
    :returns: A one-line ``"  <id> (expired <date>): <reason>"`` string.
    """
    return f"  {entry.get('id')} (expired {entry.get('expires')}): {entry.get('reason')}"


def build_arg_parser() -> argparse.ArgumentParser:
    """Build the CLI parser for this wrapper.

    :returns: A configured :class:`argparse.ArgumentParser`.
    """
    parser = argparse.ArgumentParser(
        description="pip-audit policy wrapper (hard-fails on unreasoned/expired ignores)"
    )
    parser.add_argument(
        "--ignores-file",
        type=Path,
        default=DEFAULT_IGNORES_FILE,
        help=f"Path to the ignore policy TOML file (default: {DEFAULT_IGNORES_FILE})",
    )
    parser.add_argument(
        "--no-uv",
        action="store_true",
        help="Invoke plain 'pip-audit' instead of 'uv run pip-audit'",
    )
    parser.add_argument(
        "--check-only",
        action="store_true",
        help="Validate the ignore policy file and exit without running pip-audit",
    )
    return parser


def main(argv: list[str] | None = None) -> int:
    """Validate the ignore policy, then (unless --check-only) run pip-audit.

    :param argv: Command-line arguments (defaults to ``sys.argv[1:]``).
    :returns: Process exit code — 2 for policy violations or infrastructure
        failures, otherwise pip-audit's own exit code.
    """
    args = build_arg_parser().parse_args(argv)

    try:
        policy = load_ignores_file(args.ignores_file)
    except FileNotFoundError as exc:
        print(f"pip-audit policy error: ignores file not found: {exc}", file=sys.stderr)
        return 2
    except tomllib.TOMLDecodeError as exc:
        print(
            f"pip-audit policy error: ignores file is not valid TOML: {exc}",
            file=sys.stderr,
        )
        return 2

    try:
        entries = get_ignore_entries(policy)
    except ValueError as exc:
        print(f"pip-audit policy error: {exc}", file=sys.stderr)
        return 2

    errors = validate_entries(entries)
    if errors:
        print("pip-audit policy error: invalid ignore entries:", file=sys.stderr)
        for error in errors:
            print(f"  {error}", file=sys.stderr)
        return 2

    expired = find_expired_entries(entries, date.today())
    if expired:
        print(
            "pip-audit policy error: EXPIRED ignore entries need re-review "
            "(Constitution III.11 — ignores can never rot silently):",
            file=sys.stderr,
        )
        for entry in expired:
            print(_format_expired(entry), file=sys.stderr)
        return 2

    if args.check_only:
        count = len(entries)
        plural = "y" if count == 1 else "ies"
        print(f"pip-audit policy OK: {count} ignore entr{plural} valid.")
        return 0

    command = build_pip_audit_command(entries, args.no_uv)
    try:
        result = subprocess.run(command, check=False)
    except FileNotFoundError as exc:
        print(f"pip-audit wrapper: could not execute {command[0]!r}: {exc}", file=sys.stderr)
        return 2
    return result.returncode


if __name__ == "__main__":
    sys.exit(main())
