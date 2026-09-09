#!/usr/bin/env python3
"""Run the Rust simulation and retain a bounded, validated report bundle."""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import math
import os
import re
import resource
import shlex
import shutil
import struct
import subprocess
import sys
import tempfile
import threading
import time
from collections.abc import Mapping, Sequence
from dataclasses import dataclass, field
from datetime import UTC, datetime
from pathlib import Path
from typing import BinaryIO, Final, cast
from urllib.parse import parse_qsl, unquote, urlsplit
from uuid import uuid4

TICK_REPORT_SCHEMA: Final = "babylon.simulation.tick-report.v2"
SUMMARY_SCHEMA: Final = "babylon.simulation.run-summary.v2"
DIAGNOSTICS_SCHEMA: Final = "babylon.simulation.diagnostics.v1"
RESOURCES_SCHEMA: Final = "babylon.simulation.resource-observation.v1"
RESOURCE_MEASUREMENT_SCOPE: Final = {
    "runtime_wall_time": "babylon-runtime-invocation-only",
    "runtime_cpu_rss": "babylon-runtime-process-only",
    "postgresql_cpu_ram": "unobserved",
    "compilation": "excluded",
    "report_wrapper": "excluded",
    "filesystem_disk": "filesystem-wide-coarse-unattributed",
    "filesystem_disk_caveat": "includes-concurrent-and-background-allocations",
}
VALIDATION_EXIT_CODE: Final = 2
MAX_TICKS: Final = 10_000
MAX_JSONL_BYTES: Final = 256 * 1024 * 1024
MAX_JSONL_ROW_BYTES: Final = 4 * 1024 * 1024
MAX_STDOUT_BYTES: Final = 2 * 1024 * 1024
MAX_STDERR_BYTES: Final = 2 * 1024 * 1024
RUNTIME_TIMEOUT_SECONDS: Final = 15 * 60.0
CAPTURE_CHUNK_BYTES: Final = 64 * 1024
PROCESS_POLL_SECONDS: Final = 0.02
PROCESS_JOIN_SECONDS: Final = 2.0
TICK_DURATION_DAYS: Final = 28
RESTART_INTERVAL_TICKS: Final = 13
POSTGRES_PROBE_TIMEOUT_SECONDS: Final = 10.0
MAX_POSTGRES_PROBE_OUTPUT_BYTES: Final = 4 * 1024
SOURCE_PROBE_TIMEOUT_SECONDS: Final = 5.0
MAX_SOURCE_PROBE_OUTPUT_BYTES: Final = 64 * 1024
LIBPQ_ENV_BY_DSN_KEY: Final = {
    "application_name": "PGAPPNAME",
    "channel_binding": "PGCHANNELBINDING",
    "client_encoding": "PGCLIENTENCODING",
    "connect_timeout": "PGCONNECT_TIMEOUT",
    "dbname": "PGDATABASE",
    "gssencmode": "PGGSSENCMODE",
    "host": "PGHOST",
    "hostaddr": "PGHOSTADDR",
    "keepalives": "PGKEEPALIVES",
    "keepalives_count": "PGKEEPALIVESCOUNT",
    "keepalives_idle": "PGKEEPALIVESIDLE",
    "keepalives_interval": "PGKEEPALIVESINTERVAL",
    "load_balance_hosts": "PGLOADBALANCEHOSTS",
    "options": "PGOPTIONS",
    "passfile": "PGPASSFILE",
    "password": "PGPASSWORD",
    "port": "PGPORT",
    "requirepeer": "PGREQUIREPEER",
    "service": "PGSERVICE",
    "servicefile": "PGSERVICEFILE",
    "sslcert": "PGSSLCERT",
    "sslcrl": "PGSSLCRL",
    "sslcrldir": "PGSSLCRLDIR",
    "sslkey": "PGSSLKEY",
    "sslmode": "PGSSLMODE",
    "sslpassword": "PGSSLPASSWORD",
    "sslrootcert": "PGSSLROOTCERT",
    "sslsni": "PGSSLSNI",
    "ssl_max_protocol_version": "PGSSLMAXPROTOCOLVERSION",
    "ssl_min_protocol_version": "PGSSLMINPROTOCOLVERSION",
    "target_session_attrs": "PGTARGETSESSIONATTRS",
    "tcp_user_timeout": "PGTCPUSER_TIMEOUT",
    "user": "PGUSER",
}

EXPECTED_SCOPE: Final = {
    "slice_id": "michigan-persistence-slice",
    "scenario": "production/michigan-observer-v1",
    "fixed_replay_seed": 319,
    "tick_duration_days": TICK_DURATION_DAYS,
    "parameter_overrides": False,
    "stochastic_draws": False,
    "dynamic_h3_updates": False,
}
# The native Michigan observer emits four QCEW facts for each of its 83
# counties, ordered by GEOID then by the governed economics field inventory.
OBSERVABLE_SPECS: Final = tuple(
    (f"county-26{county:03}", f"territory/{field}")
    for county in range(1, 166, 2)
    for field in (
        "qcew-establishments",
        "qcew-employment",
        "qcew-total-annual-wages",
        "qcew-average-weekly-wage",
    )
)
OBSERVABLE_CSV_PREFIXES: Final = tuple(
    f"{entity}_{field.removeprefix('territory/')}".replace("-", "_")
    for entity, field in OBSERVABLE_SPECS
)

COMMIT_DISPOSITIONS: Final = frozenset({"committed", "reconciled_after_ambiguous_commit"})
TOP_LEVEL_FIELDS: Final = frozenset(
    {
        "schema",
        "resolve_tick",
        "commit_disposition",
        "scope",
        "foundation",
        "persistence",
        "graph",
        "stable_graph",
        "world",
        "rules",
        "events",
        "audit_receipts",
        "choice_receipts",
        "material_rows",
        "observables",
        "tick_content_hash",
    }
)
HASH_PAIR_FIELDS: Final = frozenset({"before_sha256", "after_sha256"})
RULES_FIELDS: Final = frozenset({"considered", "fired", "per_rule"})
PER_RULE_FIELDS: Final = frozenset({"rule_id", "considered", "fired"})
SCOPE_FIELDS: Final = frozenset(EXPECTED_SCOPE)
EVENT_FIELDS: Final = frozenset({"count", "digest_sha256", "per_type"})
EVENT_TYPE_FIELDS: Final = frozenset({"event_type", "count"})
OBSERVABLE_FIELDS: Final = frozenset(
    {
        "name",
        "entity",
        "field",
        "role",
        "kind",
        "before_value",
        "before_bits_hex",
        "after_value",
        "after_bits_hex",
    }
)
PERSISTENCE_FIELDS: Final = frozenset({"reopened_after_commit"})
FOUNDATION_FIELDS: Final = frozenset(
    {"foundation_sha256", "defines_sha256", "rules_sha256", "reference_sha256"}
)
AUDIT_RECEIPT_FIELDS: Final = frozenset({"count"})
CHOICE_RECEIPT_FIELDS: Final = frozenset({"count", "digest_sha256"})
MATERIAL_ROW_FIELDS: Final = frozenset({"count", "digest_sha256"})
LOWER_HEX_DIGEST: Final = re.compile(r"[0-9a-f]{64}")
LOWER_HEX_F64_BITS: Final = re.compile(r"[0-9a-f]{16}")
LOWER_GIT_OBJECT_ID: Final = re.compile(r"[0-9a-f]{40}|[0-9a-f]{64}")

POSTGRES_SIZE_SQL: Final = """\
SELECT pg_catalog.pg_database_size(pg_catalog.current_database())::pg_catalog.int8,
       coalesce((
           SELECT pg_catalog.sum(pg_catalog.pg_total_relation_size(c.oid))::pg_catalog.int8
           FROM pg_catalog.pg_class AS c
           JOIN pg_catalog.pg_namespace AS n ON n.oid = c.relnamespace
           WHERE n.nspname IN ('babylon_state', 'babylon_meta')
             AND c.relkind IN ('r', 'p')
       ), 0::pg_catalog.int8)::pg_catalog.int8,
       pg_catalog.pg_wal_lsn_diff(
           pg_catalog.pg_current_wal_insert_lsn(), '0/0'
       )::pg_catalog.int8;
"""

CSV_COLUMNS: Final = (
    "resolve_tick",
    "commit_disposition",
    "reopened_after_commit",
    "administrative_graph_before_sha256",
    "administrative_graph_after_sha256",
    "stable_graph_before_sha256",
    "stable_graph_after_sha256",
    "world_before_sha256",
    "world_after_sha256",
    "rules_considered",
    "rules_fired",
    "event_count",
    "event_digest_sha256",
    *(
        f"{prefix}_{suffix}"
        for prefix in OBSERVABLE_CSV_PREFIXES
        for suffix in ("before_value", "before_bits_hex", "after_value", "after_bits_hex")
    ),
    "audit_receipt_count",
    "choice_receipt_count",
    "choice_receipt_digest_sha256",
    "material_row_count",
    "material_row_digest_sha256",
    "tick_content_hash",
)


class ReportError(RuntimeError):
    """Raised when a report run cannot safely start."""


class JsonlValidationError(ReportError):
    """Raised when the runtime report is missing or violates its wire contract."""


class _DuplicateJsonField(ValueError):
    """Raised by the JSON object hook for a duplicate member name."""

    def __init__(self, field_name: str) -> None:
        super().__init__(field_name)
        self.field_name = field_name


@dataclass(frozen=True)
class RunResult:
    """Completed wrapper result, including its durable artifact location."""

    artifact_dir: Path
    summary: dict[str, object]
    exit_code: int
    error: str | None = None


@dataclass
class _StreamCapture:
    label: str
    limit: int
    content: bytearray = field(default_factory=bytearray)
    exceeded: bool = False
    error: OSError | None = None


@dataclass(frozen=True)
class _ProcessOutcome:
    returncode: int
    stdout: bytes
    stderr: bytes
    wall_time_ns: int
    user_cpu_time_ns: int
    system_cpu_time_ns: int
    max_rss_bytes: int | None
    wrapper_status: str | None = None
    wrapper_error: str | None = None


@dataclass(frozen=True)
class _PostgresSnapshot:
    database_bytes: int
    babylon_relation_bytes: int
    wal_bytes: int


@dataclass(frozen=True)
class _PostgresProbe:
    snapshot: _PostgresSnapshot | None
    error: str | None


def _positive_ticks(value: str) -> int:
    try:
        ticks = int(value)
    except ValueError as error:
        raise argparse.ArgumentTypeError("ticks must be an integer") from error
    if ticks < 1 or ticks > MAX_TICKS:
        raise argparse.ArgumentTypeError(f"ticks must be between 1 and {MAX_TICKS}")
    return ticks


def _positive_timeout(value: str) -> float:
    try:
        timeout = float(value)
    except ValueError as error:
        raise argparse.ArgumentTypeError("timeout must be a number") from error
    if not math.isfinite(timeout) or timeout <= 0:
        raise argparse.ArgumentTypeError("timeout must be a positive finite number")
    return timeout


def _validate_runtime(runtime: Path) -> Path:
    try:
        resolved = runtime.resolve(strict=True)
    except OSError as error:
        raise ReportError(f"runtime does not exist: {runtime}") from error
    if not resolved.is_file():
        raise ReportError(f"runtime is not a file: {runtime}")
    if not os.access(resolved, os.X_OK):
        raise ReportError(f"runtime is not executable: {runtime}")
    return resolved


def _create_artifact_dir(output_root: Path) -> Path:
    try:
        root = output_root.resolve()
        root.mkdir(parents=True, exist_ok=True)
        if not root.is_dir():
            raise ReportError(f"output root is not a directory: {output_root}")
        timestamp = datetime.now(UTC).strftime("%Y%m%dT%H%M%S.%fZ-")
        return Path(tempfile.mkdtemp(prefix=timestamp, dir=root))
    except ReportError:
        raise
    except OSError as error:
        raise ReportError(f"cannot create output root: {output_root}") from error


def _child_environment() -> tuple[dict[str, str], str]:
    environment = dict(os.environ)
    campaign_id = str(uuid4())
    environment["BABYLON_CAMPAIGN_ID"] = campaign_id
    return environment, campaign_id


def _redacted_environment_values(environment: Mapping[str, str]) -> tuple[bytes, ...]:
    secrets: set[bytes] = set()
    for key, value in environment.items():
        upper_key = key.upper()
        is_connection_secret = (
            "DSN" in upper_key
            or "DATABASE_URL" in upper_key
            or upper_key == "PGDATABASE"
            or "PASSWORD" in upper_key
            or "PASSWD" in upper_key
        )
        if not is_connection_secret or not value:
            continue
        secrets.add(value.encode(errors="surrogateescape"))
    return tuple(sorted(secrets, key=len, reverse=True))


def _redact_secrets(content: bytes, secrets: Sequence[bytes]) -> bytes:
    redacted = content
    for secret in secrets:
        redacted = redacted.replace(secret, b"[REDACTED]")
    return redacted


def _write_process_logs(
    artifact_dir: Path,
    *,
    stdout: bytes,
    stderr: bytes,
    secrets: Sequence[bytes],
) -> None:
    safe_stdout = _redact_secrets(stdout, secrets)[:MAX_STDOUT_BYTES]
    safe_stderr = _redact_secrets(stderr, secrets)[:MAX_STDERR_BYTES]
    (artifact_dir / "stdout.txt").write_bytes(safe_stdout)
    (artifact_dir / "stderr.txt").write_bytes(safe_stderr)


def _capture_pipe(
    pipe: BinaryIO,
    state: _StreamCapture,
    attention: threading.Event,
) -> None:
    try:
        while True:
            chunk = os.read(pipe.fileno(), CAPTURE_CHUNK_BYTES)
            if not chunk:
                break
            remaining = state.limit - len(state.content)
            if remaining > 0:
                state.content.extend(chunk[:remaining])
            if len(chunk) > remaining:
                state.exceeded = True
                attention.set()
    except OSError as error:
        state.error = error
        attention.set()
    finally:
        pipe.close()


def _kill_and_reap(process: subprocess.Popen[bytes]) -> None:
    if process.poll() is None:
        try:
            process.kill()
        except ProcessLookupError:
            pass
    try:
        process.wait(timeout=PROCESS_JOIN_SECONDS)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait()


def _process_peak_rss_bytes(process_id: int) -> int:
    """Read Linux's per-process high-water RSS without retaining process output."""
    try:
        status = Path(f"/proc/{process_id}/status").read_text(encoding="ascii")
    except (FileNotFoundError, OSError, UnicodeError):
        return 0
    for line in status.splitlines():
        if not line.startswith("VmHWM:"):
            continue
        fields = line.split()
        if len(fields) == 3 and fields[2] == "kB" and fields[1].isdigit():
            return int(fields[1]) * 1024
    return 0


def _bounded_process_run(
    argv: Sequence[str],
    *,
    environment: Mapping[str, str],
    timeout_seconds: float,
    stdout_limit: int | None = None,
    stderr_limit: int | None = None,
) -> _ProcessOutcome:
    effective_stdout_limit = MAX_STDOUT_BYTES if stdout_limit is None else stdout_limit
    effective_stderr_limit = MAX_STDERR_BYTES if stderr_limit is None else stderr_limit
    started_ns = time.monotonic_ns()
    usage_before = resource.getrusage(resource.RUSAGE_CHILDREN)
    process = subprocess.Popen(
        list(argv),
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        env=dict(environment),
    )
    if process.stdout is None or process.stderr is None:
        _kill_and_reap(process)
        raise ReportError("runtime capture pipes were not created")

    peak_rss_bytes = _process_peak_rss_bytes(process.pid)
    stdout = _StreamCapture("stdout", effective_stdout_limit)
    stderr = _StreamCapture("stderr", effective_stderr_limit)
    attention = threading.Event()
    threads = [
        threading.Thread(
            target=_capture_pipe,
            args=(process.stdout, stdout, attention),
            daemon=True,
        ),
        threading.Thread(
            target=_capture_pipe,
            args=(process.stderr, stderr, attention),
            daemon=True,
        ),
    ]
    for thread in threads:
        thread.start()

    deadline = time.monotonic() + timeout_seconds
    wrapper_status: str | None = None
    wrapper_error: str | None = None
    while process.poll() is None:
        peak_rss_bytes = max(peak_rss_bytes, _process_peak_rss_bytes(process.pid))
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            wrapper_status = "runtime_timed_out"
            rendered_timeout = f"{timeout_seconds:g}"
            wrapper_error = f"runtime exceeded the {rendered_timeout}-second timeout"
            _kill_and_reap(process)
            break
        if attention.wait(timeout=min(PROCESS_POLL_SECONDS, remaining)):
            if stdout.exceeded:
                wrapper_status = "runtime_output_limit_exceeded"
                wrapper_error = f"runtime stdout exceeded the {stdout.limit}-byte capture bound"
            elif stderr.exceeded:
                wrapper_status = "runtime_output_limit_exceeded"
                wrapper_error = f"runtime stderr exceeded the {stderr.limit}-byte capture bound"
            elif stdout.error is not None:
                wrapper_status = "runtime_capture_failed"
                wrapper_error = "runtime stdout capture failed"
            elif stderr.error is not None:
                wrapper_status = "runtime_capture_failed"
                wrapper_error = "runtime stderr capture failed"
            if wrapper_status is not None:
                _kill_and_reap(process)
                break
            attention.clear()

    if process.poll() is None:
        _kill_and_reap(process)
    for thread in threads:
        thread.join(timeout=PROCESS_JOIN_SECONDS)

    if wrapper_status is None:
        if stdout.exceeded:
            wrapper_status = "runtime_output_limit_exceeded"
            wrapper_error = f"runtime stdout exceeded the {stdout.limit}-byte capture bound"
        elif stderr.exceeded:
            wrapper_status = "runtime_output_limit_exceeded"
            wrapper_error = f"runtime stderr exceeded the {stderr.limit}-byte capture bound"
        elif stdout.error is not None:
            wrapper_status = "runtime_capture_failed"
            wrapper_error = "runtime stdout capture failed"
        elif stderr.error is not None:
            wrapper_status = "runtime_capture_failed"
            wrapper_error = "runtime stderr capture failed"
        elif any(thread.is_alive() for thread in threads):
            wrapper_status = "runtime_capture_failed"
            wrapper_error = "runtime capture did not close after process exit"

    returncode = process.returncode
    if returncode is None:
        _kill_and_reap(process)
        returncode = process.returncode
    if returncode is None:
        raise ReportError("runtime process could not be reaped")
    finished_ns = time.monotonic_ns()
    usage_after = resource.getrusage(resource.RUSAGE_CHILDREN)
    return _ProcessOutcome(
        returncode=returncode,
        stdout=bytes(stdout.content),
        stderr=bytes(stderr.content),
        wall_time_ns=max(0, finished_ns - started_ns),
        user_cpu_time_ns=max(
            0,
            int(round((usage_after.ru_utime - usage_before.ru_utime) * 1_000_000_000)),
        ),
        system_cpu_time_ns=max(
            0,
            int(round((usage_after.ru_stime - usage_before.ru_stime) * 1_000_000_000)),
        ),
        max_rss_bytes=peak_rss_bytes or None,
        wrapper_status=wrapper_status,
        wrapper_error=wrapper_error,
    )


def _duplicate_rejecting_object(
    pairs: list[tuple[str, object]],
) -> dict[str, object]:
    result: dict[str, object] = {}
    for key, value in pairs:
        if key in result:
            raise _DuplicateJsonField(key)
        result[key] = value
    return result


def _require_fields(
    value: object,
    expected: frozenset[str],
    *,
    line_number: int,
    location: str,
) -> dict[str, object]:
    if not isinstance(value, dict):
        raise JsonlValidationError(f"line {line_number} {location} must be a JSON object")
    actual = set(value)
    if actual != expected:
        details: list[str] = []
        missing = sorted(expected - actual)
        unexpected = sorted(actual - expected)
        if missing:
            details.append(f"missing {', '.join(missing)}")
        if unexpected:
            details.append(f"unexpected {', '.join(unexpected)}")
        raise JsonlValidationError(
            f"line {line_number} {location} fields differ ({'; '.join(details)})"
        )
    return value


def _require_nonnegative_integer(
    value: object,
    *,
    line_number: int,
    location: str,
) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or value < 0:
        raise JsonlValidationError(f"line {line_number} {location} must be a nonnegative integer")
    return value


def _require_positive_integer(
    value: object,
    *,
    line_number: int,
    location: str,
) -> int:
    parsed = _require_nonnegative_integer(
        value,
        line_number=line_number,
        location=location,
    )
    if parsed == 0:
        raise JsonlValidationError(f"line {line_number} {location} must be a positive integer")
    return parsed


def _require_digest(
    value: object,
    *,
    line_number: int,
    location: str,
) -> str:
    if not isinstance(value, str) or LOWER_HEX_DIGEST.fullmatch(value) is None:
        raise JsonlValidationError(
            f"line {line_number} {location} must be a lowercase 64-hex digest"
        )
    return value


def _validate_scope(value: object, *, line_number: int) -> None:
    scope = _require_fields(
        value,
        SCOPE_FIELDS,
        line_number=line_number,
        location="scope",
    )
    for field_name, expected in EXPECTED_SCOPE.items():
        actual = scope[field_name]
        if type(actual) is not type(expected) or actual != expected:
            raise JsonlValidationError(
                f"line {line_number} scope.{field_name} must be {expected!r}, got {actual!r}"
            )


def _validate_events(value: object, *, line_number: int) -> None:
    events = _require_fields(
        value,
        EVENT_FIELDS,
        line_number=line_number,
        location="events",
    )
    count = _require_nonnegative_integer(
        events["count"],
        line_number=line_number,
        location="events.count",
    )
    _require_digest(
        events["digest_sha256"],
        line_number=line_number,
        location="events.digest_sha256",
    )
    per_type = events["per_type"]
    if not isinstance(per_type, list):
        raise JsonlValidationError(f"line {line_number} events.per_type must be a list")
    event_types: list[str] = []
    per_type_sum = 0
    for index, candidate in enumerate(per_type):
        location = f"events.per_type[{index}]"
        event = _require_fields(
            candidate,
            EVENT_TYPE_FIELDS,
            line_number=line_number,
            location=location,
        )
        event_type = event["event_type"]
        if not isinstance(event_type, str) or not event_type.strip():
            raise JsonlValidationError(
                f"line {line_number} {location}.event_type must be a nonempty string"
            )
        event_types.append(event_type)
        per_type_sum += _require_positive_integer(
            event["count"],
            line_number=line_number,
            location=f"{location}.count",
        )
    if event_types != sorted(event_types) or len(event_types) != len(set(event_types)):
        raise JsonlValidationError(
            f"line {line_number} events.per_type must be sorted by unique event_type"
        )
    if per_type_sum != count:
        raise JsonlValidationError(
            f"line {line_number} per-type event count sum {per_type_sum} "
            f"does not equal events.count {count}"
        )


def _validate_observables(value: object, *, line_number: int) -> None:
    if not isinstance(value, list) or len(value) != len(OBSERVABLE_SPECS):
        raise JsonlValidationError(
            f"line {line_number} must contain exactly {len(OBSERVABLE_SPECS)} observables"
        )
    scenario = cast("str", EXPECTED_SCOPE["scenario"])
    for index, (candidate, specification) in enumerate(zip(value, OBSERVABLE_SPECS, strict=True)):
        location = f"observable {index}"
        observable = _require_fields(
            candidate,
            OBSERVABLE_FIELDS,
            line_number=line_number,
            location=location,
        )
        expected_entity, expected_field = specification
        exact_values = {
            "field": expected_field,
            "role": "observed_baseline",
            "entity": expected_entity,
            "kind": "f64",
            "name": f"{scenario}::{expected_entity}::{expected_field}",
        }
        for field_name, expected in exact_values.items():
            if observable[field_name] != expected:
                raise JsonlValidationError(
                    f"line {line_number} {location} {field_name} must be {expected!r}"
                )
        for state_name in ("before", "after"):
            value_field = f"{state_name}_value"
            bits_field = f"{state_name}_bits_hex"
            raw_value = observable[value_field]
            if (
                not isinstance(raw_value, (int, float))
                or isinstance(raw_value, bool)
                or not math.isfinite(raw_value)
            ):
                raise JsonlValidationError(
                    f"line {line_number} {location} {value_field} must be finite"
                )
            bits_hex = observable[bits_field]
            if not isinstance(bits_hex, str) or LOWER_HEX_F64_BITS.fullmatch(bits_hex) is None:
                raise JsonlValidationError(
                    f"line {line_number} {location} {bits_field} must be 16 lowercase hex characters"
                )
            expected_bits = struct.pack(">d", float(raw_value)).hex()
            if bits_hex != expected_bits:
                raise JsonlValidationError(
                    f"line {line_number} {location} {bits_field} does not match {value_field}"
                )


def _validate_persistence(value: object, *, line_number: int) -> None:
    persistence = _require_fields(
        value,
        PERSISTENCE_FIELDS,
        line_number=line_number,
        location="persistence",
    )
    if not isinstance(persistence["reopened_after_commit"], bool):
        raise JsonlValidationError(
            f"line {line_number} persistence.reopened_after_commit must be boolean"
        )


def _validate_foundation(value: object, *, line_number: int) -> None:
    foundation = _require_fields(
        value,
        FOUNDATION_FIELDS,
        line_number=line_number,
        location="foundation",
    )
    for field_name in sorted(FOUNDATION_FIELDS):
        _require_digest(
            foundation[field_name],
            line_number=line_number,
            location=f"foundation.{field_name}",
        )


def _validate_hash_pair(
    value: object,
    *,
    line_number: int,
    location: str,
) -> None:
    pair = _require_fields(
        value,
        HASH_PAIR_FIELDS,
        line_number=line_number,
        location=location,
    )
    _require_digest(
        pair["before_sha256"],
        line_number=line_number,
        location=f"{location}.before_sha256",
    )
    _require_digest(
        pair["after_sha256"],
        line_number=line_number,
        location=f"{location}.after_sha256",
    )


def _validate_rules(value: object, *, line_number: int) -> None:
    rules = _require_fields(
        value,
        RULES_FIELDS,
        line_number=line_number,
        location="rules",
    )
    considered = _require_nonnegative_integer(
        rules["considered"],
        line_number=line_number,
        location="rules.considered",
    )
    fired = _require_nonnegative_integer(
        rules["fired"],
        line_number=line_number,
        location="rules.fired",
    )
    if considered < fired:
        raise JsonlValidationError(f"line {line_number} rules.considered must be >= rules.fired")

    per_rule = rules["per_rule"]
    if not isinstance(per_rule, list):
        raise JsonlValidationError(f"line {line_number} rules.per_rule must be a list")
    considered_sum = 0
    fired_sum = 0
    seen_rule_ids: set[str] = set()
    for index, candidate in enumerate(per_rule):
        location = f"rules.per_rule[{index}]"
        rule = _require_fields(
            candidate,
            PER_RULE_FIELDS,
            line_number=line_number,
            location=location,
        )
        rule_id = rule["rule_id"]
        if not isinstance(rule_id, str) or not rule_id.strip():
            raise JsonlValidationError(
                f"line {line_number} {location} must have a nonempty rule_id"
            )
        if rule_id in seen_rule_ids:
            raise JsonlValidationError(
                f"line {line_number} {location} duplicates rule_id {rule_id!r}"
            )
        seen_rule_ids.add(rule_id)
        rule_considered = _require_nonnegative_integer(
            rule["considered"],
            line_number=line_number,
            location=f"{location}.considered",
        )
        rule_fired = _require_nonnegative_integer(
            rule["fired"],
            line_number=line_number,
            location=f"{location}.fired",
        )
        if rule_considered < rule_fired:
            raise JsonlValidationError(f"line {line_number} {location} considered must be >= fired")
        considered_sum += rule_considered
        fired_sum += rule_fired
    if considered_sum != considered:
        raise JsonlValidationError(
            f"line {line_number} per-rule considered sum {considered_sum} "
            f"does not equal rules.considered {considered}"
        )
    if fired_sum != fired:
        raise JsonlValidationError(
            f"line {line_number} per-rule fired sum {fired_sum} does not equal rules.fired {fired}"
        )


def _validate_tick_row(
    candidate: object,
    *,
    line_number: int,
    previous_tick: int | None,
) -> dict[str, object]:
    row = _require_fields(
        candidate,
        TOP_LEVEL_FIELDS,
        line_number=line_number,
        location="top-level",
    )
    if row["schema"] != TICK_REPORT_SCHEMA:
        raise JsonlValidationError(
            f"line {line_number} has schema {row['schema']!r}; expected {TICK_REPORT_SCHEMA!r}"
        )
    resolve_tick = row["resolve_tick"]
    if not isinstance(resolve_tick, int) or isinstance(resolve_tick, bool) or resolve_tick < 1:
        raise JsonlValidationError(f"line {line_number} must have a positive resolve_tick")
    if previous_tick is not None and resolve_tick != previous_tick + 1:
        raise JsonlValidationError(
            f"line {line_number} expected contiguous resolve_tick "
            f"{previous_tick + 1}, got {resolve_tick}"
        )
    disposition = row["commit_disposition"]
    if not isinstance(disposition, str) or disposition not in COMMIT_DISPOSITIONS:
        raise JsonlValidationError(
            f"line {line_number} has invalid commit_disposition {disposition!r}"
        )

    _validate_scope(row["scope"], line_number=line_number)
    _validate_foundation(row["foundation"], line_number=line_number)
    _validate_persistence(row["persistence"], line_number=line_number)
    _validate_hash_pair(row["graph"], line_number=line_number, location="graph")
    _validate_hash_pair(
        row["stable_graph"],
        line_number=line_number,
        location="stable_graph",
    )
    _validate_hash_pair(row["world"], line_number=line_number, location="world")
    _validate_rules(row["rules"], line_number=line_number)

    _validate_events(row["events"], line_number=line_number)
    _validate_observables(row["observables"], line_number=line_number)

    receipts = _require_fields(
        row["audit_receipts"],
        AUDIT_RECEIPT_FIELDS,
        line_number=line_number,
        location="audit_receipts",
    )
    _require_nonnegative_integer(
        receipts["count"],
        line_number=line_number,
        location="audit_receipts.count",
    )

    choices = _require_fields(
        row["choice_receipts"],
        CHOICE_RECEIPT_FIELDS,
        line_number=line_number,
        location="choice_receipts",
    )
    _require_nonnegative_integer(
        choices["count"],
        line_number=line_number,
        location="choice_receipts.count",
    )
    _require_digest(
        choices["digest_sha256"],
        line_number=line_number,
        location="choice_receipts.digest_sha256",
    )

    material_rows = _require_fields(
        row["material_rows"],
        MATERIAL_ROW_FIELDS,
        line_number=line_number,
        location="material_rows",
    )
    _require_nonnegative_integer(
        material_rows["count"],
        line_number=line_number,
        location="material_rows.count",
    )
    _require_digest(
        material_rows["digest_sha256"],
        line_number=line_number,
        location="material_rows.digest_sha256",
    )
    _require_digest(
        row["tick_content_hash"],
        line_number=line_number,
        location="tick_content_hash",
    )
    return row


def _validate_durable_hash_continuity(
    previous_row: Mapping[str, object],
    row: Mapping[str, object],
    *,
    line_number: int,
) -> None:
    previous_disposition = previous_row["commit_disposition"]
    disposition = row["commit_disposition"]
    if previous_disposition not in COMMIT_DISPOSITIONS or disposition not in COMMIT_DISPOSITIONS:
        return

    for state_name in ("graph", "stable_graph", "world"):
        previous_state = cast("Mapping[str, object]", previous_row[state_name])
        state = cast("Mapping[str, object]", row[state_name])
        if state["before_sha256"] != previous_state["after_sha256"]:
            raise JsonlValidationError(
                f"line {line_number} {state_name}.before_sha256 does not equal "
                f"the previous durable row's {state_name}.after_sha256"
            )


def _validate_observable_continuity(
    previous_row: Mapping[str, object],
    row: Mapping[str, object],
    *,
    line_number: int,
) -> None:
    previous_observables = cast(
        "Sequence[Mapping[str, object]]",
        previous_row["observables"],
    )
    observables = cast("Sequence[Mapping[str, object]]", row["observables"])
    for index, (previous, current) in enumerate(
        zip(previous_observables, observables, strict=True)
    ):
        if current["before_bits_hex"] != previous["after_bits_hex"]:
            raise JsonlValidationError(
                f"line {line_number} observable {index} before_bits_hex does not equal "
                "the previous row's after_bits_hex"
            )


def _validate_persistence_schedule(
    rows: Sequence[Mapping[str, object]],
    *,
    complete_success: bool,
) -> None:
    final_tick = cast("int", rows[-1]["resolve_tick"])
    for row in rows:
        tick = cast("int", row["resolve_tick"])
        persistence = _nested_mapping(row, "persistence")
        reopened = cast("bool", persistence["reopened_after_commit"])
        interval_reopen = tick % RESTART_INTERVAL_TICKS == 0
        final_reopen = complete_success and tick == final_tick
        if interval_reopen and not reopened:
            raise JsonlValidationError(
                f"resolve_tick {tick} must be reopened at the {RESTART_INTERVAL_TICKS}-tick interval"
            )
        if reopened and not interval_reopen and not final_reopen:
            raise JsonlValidationError(f"unexpected reopen at resolve_tick {tick}")
    if complete_success:
        final_persistence = _nested_mapping(rows[-1], "persistence")
        if final_persistence["reopened_after_commit"] is not True:
            raise JsonlValidationError(
                f"final resolve_tick {final_tick} must be reopened for durable readback"
            )


def _validate_jsonl(
    path: Path,
    *,
    expected_rows: int | None = None,
    maximum_rows: int | None = None,
    first_resolve_tick: int | None = None,
    complete_success: bool = False,
) -> list[dict[str, object]]:
    try:
        size = path.stat().st_size
    except FileNotFoundError as error:
        raise JsonlValidationError("runtime did not create ticks.jsonl") from error
    except OSError as error:
        raise JsonlValidationError("cannot inspect ticks.jsonl") from error
    if not path.is_file():
        raise JsonlValidationError("ticks.jsonl is not a regular file")
    if size > MAX_JSONL_BYTES:
        raise JsonlValidationError(
            f"ticks.jsonl exceeds the {MAX_JSONL_BYTES}-byte validation bound"
        )

    rows: list[dict[str, object]] = []
    previous_tick: int | None = None
    previous_row: dict[str, object] | None = None
    expected_rule_ids: tuple[str, ...] | None = None
    expected_foundation: Mapping[str, object] | None = None
    try:
        with path.open("rb") as report_file:
            line_number = 0
            while raw_line := report_file.readline(MAX_JSONL_ROW_BYTES + 1):
                line_number += 1
                if len(raw_line) > MAX_JSONL_ROW_BYTES:
                    raise JsonlValidationError(
                        f"line {line_number} exceeds the {MAX_JSONL_ROW_BYTES}-byte bound"
                    )
                next_row_count = len(rows) + 1
                if expected_rows is not None and next_row_count > expected_rows:
                    raise JsonlValidationError(
                        f"ticks.jsonl exceeds expected row count {expected_rows}"
                    )
                if maximum_rows is not None and next_row_count > maximum_rows:
                    raise JsonlValidationError(
                        f"ticks.jsonl exceeds maximum row count {maximum_rows}"
                    )
                if not raw_line.strip():
                    raise JsonlValidationError(f"line {line_number} is blank")
                try:
                    decoded = raw_line.decode("utf-8")
                except UnicodeDecodeError as error:
                    raise JsonlValidationError(f"line {line_number} is not UTF-8") from error
                try:
                    candidate = json.loads(
                        decoded,
                        object_pairs_hook=_duplicate_rejecting_object,
                    )
                except _DuplicateJsonField as error:
                    raise JsonlValidationError(
                        f"line {line_number} contains duplicate JSON field {error.field_name!r}"
                    ) from error
                except json.JSONDecodeError as error:
                    raise JsonlValidationError(
                        f"line {line_number} is not valid JSON: {error.msg}"
                    ) from error
                row = _validate_tick_row(
                    candidate,
                    line_number=line_number,
                    previous_tick=previous_tick,
                )
                rules = cast("Mapping[str, object]", row["rules"])
                per_rule = cast("Sequence[Mapping[str, object]]", rules["per_rule"])
                rule_ids = tuple(cast("str", rule["rule_id"]) for rule in per_rule)
                if expected_rule_ids is None:
                    expected_rule_ids = rule_ids
                elif rule_ids != expected_rule_ids:
                    raise JsonlValidationError(
                        f"line {line_number} ordered rule inventory differs from line 1"
                    )
                foundation = cast("Mapping[str, object]", row["foundation"])
                if expected_foundation is None:
                    expected_foundation = foundation
                elif foundation != expected_foundation:
                    raise JsonlValidationError(f"line {line_number} foundation differs from line 1")
                current_tick = cast("int", row["resolve_tick"])
                if (
                    previous_tick is None
                    and first_resolve_tick is not None
                    and current_tick != first_resolve_tick
                ):
                    raise JsonlValidationError(
                        "ticks.jsonl must start at resolve_tick "
                        f"{first_resolve_tick}, got {current_tick}"
                    )
                if previous_row is not None:
                    _validate_durable_hash_continuity(
                        previous_row,
                        row,
                        line_number=line_number,
                    )
                    _validate_observable_continuity(
                        previous_row,
                        row,
                        line_number=line_number,
                    )
                previous_tick = current_tick
                previous_row = row
                rows.append(row)
    except JsonlValidationError:
        raise
    except OSError as error:
        raise JsonlValidationError("cannot read ticks.jsonl") from error

    if not rows:
        raise JsonlValidationError("ticks.jsonl contains no tick reports")
    if expected_rows is not None and len(rows) != expected_rows:
        raise JsonlValidationError(
            f"ticks.jsonl reports {len(rows)} rows; expected exactly {expected_rows}"
        )
    _validate_persistence_schedule(rows, complete_success=complete_success)
    return rows


def _nested_mapping(row: Mapping[str, object], key: str) -> Mapping[str, object]:
    return cast("Mapping[str, object]", row[key])


def _state_change_diagnostics(
    rows: Sequence[Mapping[str, object]],
    state_name: str,
) -> dict[str, object]:
    changed_ticks: list[int] = []
    unchanged_ticks: list[int] = []
    after_hashes: set[object] = set()
    longest_start: int | None = None
    longest_end: int | None = None
    longest_count = 0
    current_start: int | None = None
    current_end: int | None = None
    current_count = 0
    for row in rows:
        tick = cast("int", row["resolve_tick"])
        state = _nested_mapping(row, state_name)
        after_hashes.add(state["after_sha256"])
        if state["before_sha256"] != state["after_sha256"]:
            changed_ticks.append(tick)
            current_start = None
            current_end = None
            current_count = 0
            continue
        unchanged_ticks.append(tick)
        if current_start is None:
            current_start = tick
        current_end = tick
        current_count += 1
        if current_count > longest_count:
            longest_start = current_start
            longest_end = current_end
            longest_count = current_count
    return {
        "changed_ticks": len(changed_ticks),
        "unchanged_ticks": len(unchanged_ticks),
        "first_change_tick": changed_ticks[0] if changed_ticks else None,
        "last_change_tick": changed_ticks[-1] if changed_ticks else None,
        "distinct_after_hashes": len(after_hashes),
        "longest_unchanged_run": {
            "start_tick": longest_start,
            "end_tick": longest_end,
            "ticks": longest_count,
        },
    }


def _stable_graph_diagnostics(
    rows: Sequence[Mapping[str, object]],
) -> dict[str, object]:
    diagnostics = _state_change_diagnostics(rows, "stable_graph")
    first = _nested_mapping(rows[0], "stable_graph")
    states = [first["before_sha256"]]
    states.extend(_nested_mapping(row, "stable_graph")["after_sha256"] for row in rows)
    diagnostics["distinct_hashes"] = len(set(states))
    diagnostics["initial_sha256"] = states[0]
    diagnostics["final_sha256"] = states[-1]
    del diagnostics["distinct_after_hashes"]
    return diagnostics


def _firing_interval_metrics(
    fired_ticks: Sequence[int],
) -> tuple[int | None, int | None, int | None]:
    intervals = [right - left for left, right in zip(fired_ticks, fired_ticks[1:], strict=False)]
    if not intervals:
        return None, None, None
    interval_gcd = intervals[0]
    for interval in intervals[1:]:
        interval_gcd = math.gcd(interval_gcd, interval)
    return min(intervals), max(intervals), interval_gcd


def _diagnostics(rows: Sequence[Mapping[str, object]]) -> dict[str, object]:
    """Derive deterministic diagnostic signals from already validated tick rows."""
    if not rows:
        raise ReportError("diagnostics require at least one validated tick row")

    notices: list[dict[str, str]] = [
        {"code": "scope.fixed_parameters", "subject": "parameter_overrides=false"},
        {"code": "scope.no_stochastic_draws", "subject": "stochastic_draws=false"},
        {"code": "scope.no_dynamic_h3_updates", "subject": "dynamic_h3_updates=false"},
    ]
    graph = _state_change_diagnostics(rows, "graph")
    stable_graph = _stable_graph_diagnostics(rows)
    world = _state_change_diagnostics(rows, "world")
    if graph["changed_ticks"] == 0:
        notices.append(
            {"code": "administrative_graph.flat", "subject": "administrative graph state"}
        )
    stable_plateau = cast("Mapping[str, object]", stable_graph["longest_unchanged_run"])
    if stable_graph["changed_ticks"] == 0:
        notices.append({"code": "stable_graph.flat", "subject": "stable graph digest"})
    elif cast("int", stable_plateau["ticks"]) > 1:
        notices.append(
            {
                "code": "stable_graph.plateau",
                "subject": (f"ticks {stable_plateau['start_tick']}..{stable_plateau['end_tick']}"),
            }
        )

    disposition_counts = dict.fromkeys(sorted(COMMIT_DISPOSITIONS), 0)
    for row in rows:
        disposition = cast("str", row["commit_disposition"])
        disposition_counts[disposition] += 1

    first_rules = _nested_mapping(rows[0], "rules")
    first_per_rule = cast("Sequence[Mapping[str, object]]", first_rules["per_rule"])
    rule_diagnostics: list[dict[str, object]] = []
    for rule_index, first_rule in enumerate(first_per_rule):
        rule_id = cast("str", first_rule["rule_id"])
        considered = 0
        fired = 0
        considered_ticks: list[int] = []
        fired_ticks: list[int] = []
        fired_on_unchanged_administrative_graph_ticks = 0
        for row in rows:
            tick = cast("int", row["resolve_tick"])
            per_rule = cast(
                "Sequence[Mapping[str, object]]",
                _nested_mapping(row, "rules")["per_rule"],
            )
            rule = per_rule[rule_index]
            rule_considered = cast("int", rule["considered"])
            rule_fired = cast("int", rule["fired"])
            considered += rule_considered
            fired += rule_fired
            if rule_considered > 0:
                considered_ticks.append(tick)
            if rule_fired > 0:
                fired_ticks.append(tick)
                graph_state = _nested_mapping(row, "graph")
                if graph_state["before_sha256"] == graph_state["after_sha256"]:
                    fired_on_unchanged_administrative_graph_ticks += 1
        minimum_interval, maximum_interval, interval_gcd = _firing_interval_metrics(fired_ticks)
        rule_diagnostics.append(
            {
                "rule_id": rule_id,
                "considered": considered,
                "fired": fired,
                "ticks_considered": len(considered_ticks),
                "ticks_fired": len(fired_ticks),
                "first_fired_tick": fired_ticks[0] if fired_ticks else None,
                "last_fired_tick": fired_ticks[-1] if fired_ticks else None,
                "minimum_firing_interval_ticks": minimum_interval,
                "maximum_firing_interval_ticks": maximum_interval,
                "firing_interval_gcd_ticks": interval_gcd,
                "fired_on_unchanged_administrative_graph_ticks": (
                    fired_on_unchanged_administrative_graph_ticks
                ),
            }
        )
        if fired == 0:
            notices.append({"code": "rule.never_fired", "subject": rule_id})
        if fired_on_unchanged_administrative_graph_ticks > 0:
            notices.append(
                {
                    "code": "rule.fired_on_unchanged_administrative_graph",
                    "subject": rule_id,
                }
            )

    event_total = 0
    event_ticks: list[int] = []
    event_types: dict[str, dict[str, object]] = {}
    event_counts: list[int] = []
    reported_ticks = [cast("int", row["resolve_tick"]) for row in rows]
    for row in rows:
        tick = cast("int", row["resolve_tick"])
        events = _nested_mapping(row, "events")
        count = cast("int", events["count"])
        event_counts.append(count)
        event_total += count
        if count > 0:
            event_ticks.append(tick)
        for event in cast("Sequence[Mapping[str, object]]", events["per_type"]):
            event_type = cast("str", event["event_type"])
            aggregate = event_types.setdefault(
                event_type,
                {"count": 0, "ticks": [], "counts": {}},
            )
            aggregate["count"] = cast("int", aggregate["count"]) + cast("int", event["count"])
            cast("list[int]", aggregate["ticks"]).append(tick)
            cast("dict[int, int]", aggregate["counts"])[tick] = cast("int", event["count"])
    per_type_diagnostics = []
    for event_type in sorted(event_types):
        aggregate = event_types[event_type]
        active_ticks = cast("list[int]", aggregate["ticks"])
        counts_by_tick = cast("Mapping[int, int]", aggregate["counts"])
        count_series = [counts_by_tick.get(tick, 0) for tick in reported_ticks]
        change_ticks = [
            reported_ticks[index]
            for index in range(1, len(count_series))
            if count_series[index] != count_series[index - 1]
        ]
        per_type_diagnostics.append(
            {
                "event_type": event_type,
                "count": aggregate["count"],
                "active_ticks": len(active_ticks),
                "first_tick": active_ticks[0],
                "last_tick": active_ticks[-1],
                "minimum_per_tick": min(count_series),
                "maximum_per_tick": max(count_series),
                "unique_counts": len(set(count_series)),
                "change_count": len(change_ticks),
                "first_change_tick": change_ticks[0] if change_ticks else None,
                "last_change_tick": change_ticks[-1] if change_ticks else None,
            }
        )
    event_change_ticks = [
        reported_ticks[index]
        for index in range(1, len(event_counts))
        if event_counts[index] != event_counts[index - 1]
    ]
    events_diagnostics = {
        "total": event_total,
        "active_ticks": len(event_ticks),
        "first_event_tick": event_ticks[0] if event_ticks else None,
        "last_event_tick": event_ticks[-1] if event_ticks else None,
        "minimum_per_tick": min(event_counts),
        "maximum_per_tick": max(event_counts),
        "unique_counts": len(set(event_counts)),
        "change_count": len(event_change_ticks),
        "first_change_tick": event_change_ticks[0] if event_change_ticks else None,
        "last_change_tick": event_change_ticks[-1] if event_change_ticks else None,
        "per_type": per_type_diagnostics,
    }
    if event_total == 0:
        notices.append({"code": "events.none", "subject": "all reported ticks"})

    material_counts = [cast("int", _nested_mapping(row, "material_rows")["count"]) for row in rows]
    material_change_ticks = [
        reported_ticks[index]
        for index in range(1, len(material_counts))
        if material_counts[index] != material_counts[index - 1]
    ]
    material_diagnostics = {
        "total": sum(material_counts),
        "initial_per_tick": material_counts[0],
        "final_per_tick": material_counts[-1],
        "minimum_per_tick": min(material_counts),
        "maximum_per_tick": max(material_counts),
        "unique_counts": len(set(material_counts)),
        "change_count": len(material_change_ticks),
        "first_change_tick": material_change_ticks[0] if material_change_ticks else None,
        "last_change_tick": material_change_ticks[-1] if material_change_ticks else None,
    }
    if len(set(material_counts)) == 1:
        notices.append({"code": "material_rows.constant", "subject": str(material_counts[0])})

    observable_diagnostics: list[dict[str, object]] = []
    first_observables = cast("Sequence[Mapping[str, object]]", rows[0]["observables"])
    for observable_index, first_observable in enumerate(first_observables):
        values = [float(cast("int | float", first_observable["before_value"]))]
        bits = [cast("str", first_observable["before_bits_hex"])]
        observable_change_ticks: list[int] = []
        for row in rows:
            tick_observables = cast("Sequence[Mapping[str, object]]", row["observables"])
            observable = tick_observables[observable_index]
            before_bits = cast("str", observable["before_bits_hex"])
            after_bits = cast("str", observable["after_bits_hex"])
            if before_bits != after_bits:
                observable_change_ticks.append(cast("int", row["resolve_tick"]))
            values.append(float(cast("int | float", observable["after_value"])))
            bits.append(after_bits)
        role = cast("str", first_observable["role"])
        observable_diagnostics.append(
            {
                "name": first_observable["name"],
                "entity": first_observable["entity"],
                "field": first_observable["field"],
                "role": role,
                "kind": first_observable["kind"],
                "initial": values[0],
                "final": values[-1],
                "minimum": min(values),
                "maximum": max(values),
                "unique_values": len(set(bits)),
                "change_count": len(observable_change_ticks),
                "first_change_tick": (
                    observable_change_ticks[0] if observable_change_ticks else None
                ),
                "last_change_tick": (
                    observable_change_ticks[-1] if observable_change_ticks else None
                ),
            }
        )
        name = cast("str", first_observable["name"])
        if observable_change_ticks:
            notices.append({"code": "observable.observed_baseline_changed", "subject": name})

    return {
        "schema": DIAGNOSTICS_SCHEMA,
        "scope": dict(cast("Mapping[str, object]", rows[0]["scope"])),
        "ticks_reported": len(rows),
        "stable_graph": stable_graph,
        "administrative_graph": graph,
        "world": world,
        "commits": disposition_counts,
        "rules": rule_diagnostics,
        "events": events_diagnostics,
        "material_rows": material_diagnostics,
        "observables": observable_diagnostics,
        "notices": notices,
    }


def _write_diagnostics(
    artifact_dir: Path,
    rows: Sequence[Mapping[str, object]],
) -> None:
    content = json.dumps(
        _diagnostics(rows),
        indent=2,
        sort_keys=True,
        ensure_ascii=False,
        allow_nan=False,
    )
    (artifact_dir / "diagnostics.json").write_text(content + "\n", encoding="utf-8")


def _git_source_state() -> tuple[str, bool]:
    repository = Path(__file__).resolve().parents[2]
    environment = dict(os.environ)
    base_argv = ["git", "-C", str(repository)]
    try:
        head_outcome = _bounded_process_run(
            [*base_argv, "rev-parse", "--verify", "HEAD"],
            environment=environment,
            timeout_seconds=SOURCE_PROBE_TIMEOUT_SECONDS,
            stdout_limit=MAX_SOURCE_PROBE_OUTPUT_BYTES,
            stderr_limit=MAX_SOURCE_PROBE_OUTPUT_BYTES,
        )
        status_outcome = _bounded_process_run(
            [*base_argv, "status", "--porcelain=v1", "--untracked-files=normal"],
            environment=environment,
            timeout_seconds=SOURCE_PROBE_TIMEOUT_SECONDS,
            stdout_limit=MAX_SOURCE_PROBE_OUTPUT_BYTES,
            stderr_limit=MAX_SOURCE_PROBE_OUTPUT_BYTES,
        )
    except (OSError, ReportError) as error:
        raise ReportError("cannot inspect git source identity") from error
    if head_outcome.wrapper_status is not None or head_outcome.returncode != 0:
        raise ReportError("cannot inspect git HEAD")
    try:
        git_head = head_outcome.stdout.decode("ascii").strip()
    except UnicodeDecodeError as error:
        raise ReportError("git HEAD is not ASCII") from error
    if LOWER_GIT_OBJECT_ID.fullmatch(git_head) is None:
        raise ReportError("git HEAD is not a lowercase object ID")
    if status_outcome.wrapper_status == "runtime_output_limit_exceeded":
        return git_head, True
    if status_outcome.wrapper_status is not None or status_outcome.returncode != 0:
        raise ReportError("cannot inspect git dirty status")
    return git_head, bool(status_outcome.stdout.strip())


def _source_provenance(runtime_path: Path) -> dict[str, object]:
    git_head, git_dirty = _git_source_state()
    return {
        "git_head_sha": git_head,
        "git_dirty": git_dirty,
        "runtime_binary_sha256": _sha256_file(runtime_path),
        "reporter_sha256": _sha256_file(Path(__file__).resolve()),
    }


def _summary_diagnostics(rows: Sequence[Mapping[str, object]]) -> dict[str, object]:
    diagnostics = _diagnostics(rows)
    notices = cast("Sequence[Mapping[str, object]]", diagnostics["notices"])
    codes = list(dict.fromkeys(cast("str", notice["code"]) for notice in notices))
    return {
        "notice_count": len(notices),
        "notice_codes": codes,
    }


def _evidence_summary(
    rows: Sequence[Mapping[str, object]],
    *,
    campaign_id: str,
    ticks_requested: int,
    status: str,
    runtime_exit_code: int,
    provenance: Mapping[str, object],
    error: str | None = None,
) -> dict[str, object]:
    final = rows[-1]
    final_graph = _nested_mapping(final, "graph")
    final_stable_graph = _nested_mapping(final, "stable_graph")
    final_world = _nested_mapping(final, "world")
    final_persistence = _nested_mapping(final, "persistence")
    totals = {
        "rules_considered": sum(
            cast("int", _nested_mapping(row, "rules")["considered"]) for row in rows
        ),
        "rules_fired": sum(cast("int", _nested_mapping(row, "rules")["fired"]) for row in rows),
        "events": sum(cast("int", _nested_mapping(row, "events")["count"]) for row in rows),
        "audit_receipts": sum(
            cast("int", _nested_mapping(row, "audit_receipts")["count"]) for row in rows
        ),
        "choice_receipts": sum(
            cast("int", _nested_mapping(row, "choice_receipts")["count"]) for row in rows
        ),
        "material_rows": sum(
            cast("int", _nested_mapping(row, "material_rows")["count"]) for row in rows
        ),
    }
    summary: dict[str, object] = {
        "schema": SUMMARY_SCHEMA,
        "status": status,
        "campaign_id": campaign_id,
        "runtime_exit_code": runtime_exit_code,
        "ticks_requested": ticks_requested,
        "ticks_reported": len(rows),
        "tick_duration_days": _nested_mapping(final, "scope")["tick_duration_days"],
        "first_resolve_tick": rows[0]["resolve_tick"],
        "last_resolve_tick": final["resolve_tick"],
        "provenance": dict(provenance),
        "foundation": dict(_nested_mapping(rows[0], "foundation")),
        "diagnostics": _summary_diagnostics(rows),
        "totals": totals,
        "final": {
            "administrative_graph_sha256": final_graph["after_sha256"],
            "stable_graph_sha256": final_stable_graph["after_sha256"],
            "world_sha256": final_world["after_sha256"],
            "reopened_after_commit": final_persistence["reopened_after_commit"],
            "tick_content_hash": final["tick_content_hash"],
        },
    }
    if error is not None:
        summary["error"] = error
    return summary


def _failure_summary(
    *,
    status: str,
    campaign_id: str,
    ticks_requested: int,
    runtime_exit_code: int | None,
    error: str,
    provenance: Mapping[str, object],
    report_validation_error: str | None = None,
) -> dict[str, object]:
    summary: dict[str, object] = {
        "schema": SUMMARY_SCHEMA,
        "status": status,
        "campaign_id": campaign_id,
        "runtime_exit_code": runtime_exit_code,
        "ticks_requested": ticks_requested,
        "ticks_reported": 0,
        "provenance": dict(provenance),
        "error": error,
    }
    if report_validation_error is not None:
        summary["report_validation_error"] = report_validation_error
    return summary


def render_summary(summary: Mapping[str, object]) -> str:
    provenance = cast("Mapping[str, object]", summary["provenance"])
    dirty_label = "dirty" if provenance["git_dirty"] else "clean"
    lines = [
        f"status: {summary['status']}",
        f"campaign: {summary['campaign_id']}",
        f"ticks: {summary['ticks_reported']} reported / {summary['ticks_requested']} requested",
        f"source git: {provenance['git_head_sha']} ({dirty_label})",
        f"runtime binary: {provenance['runtime_binary_sha256']}",
        f"reporter: {provenance['reporter_sha256']}",
    ]
    if cast("int", summary["ticks_reported"]) > 0:
        lines.append(f"simulation days per tick: {summary['tick_duration_days']}")
        lines.append(
            f"resolve ticks: {summary['first_resolve_tick']}..{summary['last_resolve_tick']}"
        )
        totals = cast("Mapping[str, object]", summary["totals"])
        final = cast("Mapping[str, object]", summary["final"])
        lines.extend(
            [
                f"rules: {totals['rules_fired']} fired / {totals['rules_considered']} considered",
                (
                    f"events: {totals['events']}; "
                    f"audit receipts: {totals['audit_receipts']}; "
                    f"choice receipts: {totals['choice_receipts']}; "
                    f"material rows: {totals['material_rows']}"
                ),
                f"final administrative graph: {final['administrative_graph_sha256']}",
                f"final stable graph: {final['stable_graph_sha256']}",
                f"final world: {final['world_sha256']}",
                f"final reopened after commit: {final['reopened_after_commit']}",
                f"tick content hash: {final['tick_content_hash']}",
            ]
        )
        foundation = cast("Mapping[str, object]", summary["foundation"])
        diagnostics = cast("Mapping[str, object]", summary["diagnostics"])
        notice_codes = cast("Sequence[str]", diagnostics["notice_codes"])
        lines.extend(
            [
                f"foundation: {foundation['foundation_sha256']}",
                f"defines: {foundation['defines_sha256']}",
                f"rules identity: {foundation['rules_sha256']}",
                f"reference: {foundation['reference_sha256']}",
                f"diagnostic notices: {diagnostics['notice_count']}",
                "diagnostic codes: " + (", ".join(notice_codes) if notice_codes else "none"),
            ]
        )
    if "error" in summary:
        lines.append(f"error: {summary['error']}")
    if "report_validation_error" in summary:
        lines.append(f"report validation: {summary['report_validation_error']}")
    return "\n".join(lines) + "\n"


def _write_summary(artifact_dir: Path, summary: Mapping[str, object]) -> None:
    json_text = json.dumps(summary, indent=2, sort_keys=True, ensure_ascii=False) + "\n"
    (artifact_dir / "summary.json").write_text(json_text, encoding="utf-8")
    (artifact_dir / "summary.txt").write_text(render_summary(summary), encoding="utf-8")


def _csv_row(row: Mapping[str, object]) -> dict[str, object]:
    graph = _nested_mapping(row, "graph")
    stable_graph = _nested_mapping(row, "stable_graph")
    world = _nested_mapping(row, "world")
    rules = _nested_mapping(row, "rules")
    events = _nested_mapping(row, "events")
    receipts = _nested_mapping(row, "audit_receipts")
    choices = _nested_mapping(row, "choice_receipts")
    material_rows = _nested_mapping(row, "material_rows")
    persistence = _nested_mapping(row, "persistence")
    rendered: dict[str, object] = {
        "resolve_tick": row["resolve_tick"],
        "commit_disposition": row["commit_disposition"],
        "reopened_after_commit": persistence["reopened_after_commit"],
        "administrative_graph_before_sha256": graph["before_sha256"],
        "administrative_graph_after_sha256": graph["after_sha256"],
        "stable_graph_before_sha256": stable_graph["before_sha256"],
        "stable_graph_after_sha256": stable_graph["after_sha256"],
        "world_before_sha256": world["before_sha256"],
        "world_after_sha256": world["after_sha256"],
        "rules_considered": rules["considered"],
        "rules_fired": rules["fired"],
        "event_count": events["count"],
        "event_digest_sha256": events["digest_sha256"],
        "audit_receipt_count": receipts["count"],
        "choice_receipt_count": choices["count"],
        "choice_receipt_digest_sha256": choices["digest_sha256"],
        "material_row_count": material_rows["count"],
        "material_row_digest_sha256": material_rows["digest_sha256"],
        "tick_content_hash": row["tick_content_hash"],
    }
    observables = cast("Sequence[Mapping[str, object]]", row["observables"])
    for prefix, observable in zip(OBSERVABLE_CSV_PREFIXES, observables, strict=True):
        for field_name in (
            "before_value",
            "before_bits_hex",
            "after_value",
            "after_bits_hex",
        ):
            rendered[f"{prefix}_{field_name}"] = observable[field_name]
    return rendered


def _write_csv(
    artifact_dir: Path,
    rows: Sequence[Mapping[str, object]],
) -> None:
    with (artifact_dir / "ticks.csv").open("x", encoding="utf-8", newline="") as output:
        writer = csv.DictWriter(
            output,
            fieldnames=CSV_COLUMNS,
            lineterminator="\n",
        )
        writer.writeheader()
        for row in rows:
            writer.writerow(_csv_row(row))


def _libpq_dsn_parameters(dsn: str) -> dict[str, str]:
    if dsn.startswith(("postgres://", "postgresql://")):
        try:
            parsed = urlsplit(dsn)
            parameters = dict(parse_qsl(parsed.query, keep_blank_values=True, strict_parsing=True))
            if parsed.username is not None:
                parameters["user"] = unquote(parsed.username)
            if parsed.password is not None:
                parameters["password"] = unquote(parsed.password)
            if parsed.hostname is not None:
                parameters["host"] = parsed.hostname
            if parsed.port is not None:
                parameters["port"] = str(parsed.port)
            if parsed.path not in {"", "/"}:
                parameters["dbname"] = unquote(parsed.path.removeprefix("/"))
            if parsed.fragment:
                raise ValueError("connection URI fragments are unsupported")
        except ValueError as error:
            raise ValueError("invalid PostgreSQL connection URI") from error
    else:
        try:
            tokens = shlex.split(dsn, posix=True)
        except ValueError as error:
            raise ValueError("invalid PostgreSQL keyword connection string") from error
        parameters = {}
        for token in tokens:
            key, separator, value = token.partition("=")
            if not separator or not key:
                raise ValueError("invalid PostgreSQL keyword connection string")
            parameters[key] = value
    if not parameters:
        raise ValueError("empty PostgreSQL connection string")
    unsupported = sorted(set(parameters).difference(LIBPQ_ENV_BY_DSN_KEY))
    if unsupported:
        raise ValueError(f"unsupported PostgreSQL connection keys: {', '.join(unsupported)}")
    return parameters


def _postgres_probe_environment(
    environment: Mapping[str, str],
    dsn: str,
) -> dict[str, str]:
    parameters = _libpq_dsn_parameters(dsn)
    probe_environment = {
        key: value
        for key, value in environment.items()
        if key != "BABYLON_RUNTIME_DSN" and not key.upper().startswith("PG")
    }
    for key, value in parameters.items():
        probe_environment[LIBPQ_ENV_BY_DSN_KEY[key]] = value
    probe_environment["PGCONNECT_TIMEOUT"] = str(math.ceil(POSTGRES_PROBE_TIMEOUT_SECONDS))
    return probe_environment


def _probe_postgres_snapshot(environment: Mapping[str, str]) -> _PostgresProbe:
    dsn = environment.get("BABYLON_RUNTIME_DSN")
    if not dsn:
        return _PostgresProbe(snapshot=None, error="dsn_unavailable")
    try:
        probe_environment = _postgres_probe_environment(environment, dsn)
    except ValueError:
        return _PostgresProbe(snapshot=None, error="dsn_unsupported")
    argv = [
        "psql",
        "--no-psqlrc",
        "--quiet",
        "--tuples-only",
        "--no-align",
        "--field-separator",
        "\t",
        "--set",
        "ON_ERROR_STOP=1",
        "--command",
        POSTGRES_SIZE_SQL,
    ]
    try:
        outcome = _bounded_process_run(
            argv,
            environment=probe_environment,
            timeout_seconds=POSTGRES_PROBE_TIMEOUT_SECONDS,
            stdout_limit=MAX_POSTGRES_PROBE_OUTPUT_BYTES,
            stderr_limit=MAX_POSTGRES_PROBE_OUTPUT_BYTES,
        )
    except FileNotFoundError:
        return _PostgresProbe(snapshot=None, error="psql_not_found")
    except (OSError, ReportError):
        return _PostgresProbe(snapshot=None, error="probe_failed")
    if outcome.wrapper_status == "runtime_timed_out":
        return _PostgresProbe(snapshot=None, error="probe_timed_out")
    if outcome.wrapper_status is not None:
        return _PostgresProbe(snapshot=None, error="probe_output_or_capture_failed")
    if outcome.returncode != 0:
        return _PostgresProbe(snapshot=None, error="probe_failed")
    try:
        decoded = outcome.stdout.decode("ascii").strip()
        fields = decoded.split("\t")
        if len(fields) != 3:
            raise ValueError
        values = [int(field) for field in fields]
        if any(value < 0 for value in values):
            raise ValueError
    except (UnicodeDecodeError, ValueError):
        return _PostgresProbe(snapshot=None, error="invalid_output")
    return _PostgresProbe(
        snapshot=_PostgresSnapshot(
            database_bytes=values[0],
            babylon_relation_bytes=values[1],
            wal_bytes=values[2],
        ),
        error=None,
    )


def _host_total_ram_bytes() -> int | None:
    try:
        for line in Path("/proc/meminfo").read_text(encoding="ascii").splitlines():
            if not line.startswith("MemTotal:"):
                continue
            fields = line.split()
            if len(fields) == 3 and fields[2] == "kB" and fields[1].isdigit():
                return int(fields[1]) * 1024
    except (OSError, UnicodeError):
        return None
    return None


def _snapshot_payload(snapshot: _PostgresSnapshot | None) -> dict[str, int] | None:
    if snapshot is None:
        return None
    return {
        "database_bytes": snapshot.database_bytes,
        "babylon_relation_bytes": snapshot.babylon_relation_bytes,
        "wal_bytes": snapshot.wal_bytes,
    }


def _database_resource_observation(
    before: _PostgresProbe,
    after: _PostgresProbe,
    *,
    database_scope: str,
) -> dict[str, object]:
    if database_scope not in {"shared", "exclusive"}:
        raise ReportError("database scope must be shared or exclusive")
    errors: list[str] = []
    if before.error is not None:
        errors.append(f"before:{before.error}")
    if after.error is not None:
        errors.append(f"after:{after.error}")
    if before.snapshot is not None and after.snapshot is not None:
        status = "observed"
        delta: dict[str, int] | None = {
            "database_bytes": after.snapshot.database_bytes - before.snapshot.database_bytes,
            "babylon_relation_bytes": (
                after.snapshot.babylon_relation_bytes - before.snapshot.babylon_relation_bytes
            ),
            "wal_bytes": after.snapshot.wal_bytes - before.snapshot.wal_bytes,
        }
    else:
        status = (
            "partial"
            if before.snapshot is not None or after.snapshot is not None
            else "unavailable"
        )
        delta = None
    return {
        "status": status,
        "reason": "; ".join(errors) if errors else None,
        "scope": database_scope,
        "scope_assertion": "caller_asserted_unverified",
        "contaminated_by_concurrency": database_scope == "shared",
        "before": _snapshot_payload(before.snapshot),
        "after": _snapshot_payload(after.snapshot),
        "delta": delta,
    }


def _sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        while chunk := source.read(128 * 1024):
            digest.update(chunk)
    return digest.hexdigest()


def _artifact_resource_observation(artifact_dir: Path) -> dict[str, object]:
    files: list[dict[str, object]] = []
    for path in sorted(artifact_dir.iterdir(), key=lambda candidate: candidate.name):
        if path.name == "resources.json":
            continue
        if path.is_symlink() or not path.is_file():
            raise ReportError(f"unexpected non-file report artifact: {path.name}")
        size = path.stat().st_size
        files.append(
            {
                "path": path.name,
                "bytes": size,
                "sha256": _sha256_file(path),
            }
        )
    return {
        "files": files,
        "payload_bytes": sum(cast("int", file["bytes"]) for file in files),
    }


def _resource_require_fields(
    value: object,
    expected: frozenset[str],
    *,
    location: str,
) -> Mapping[str, object]:
    if not isinstance(value, dict) or set(value) != expected:
        raise ReportError(f"resources schema violation at {location}")
    return value


def _resource_integer(
    value: object,
    *,
    location: str,
    nonnegative: bool,
) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or (nonnegative and value < 0):
        qualifier = "nonnegative " if nonnegative else ""
        raise ReportError(f"resources {location} must be a {qualifier}integer")
    return value


def _validate_snapshot_payload(value: object, *, location: str) -> None:
    snapshot = _resource_require_fields(
        value,
        frozenset({"database_bytes", "babylon_relation_bytes", "wal_bytes"}),
        location=location,
    )
    for field_name in snapshot:
        _resource_integer(
            snapshot[field_name],
            location=f"{location}.{field_name}",
            nonnegative=True,
        )


def _validate_resources_payload(payload: object) -> None:
    root = _resource_require_fields(
        payload,
        frozenset({"schema", "measurement_scope", "runtime", "host", "database", "artifacts"}),
        location="top-level",
    )
    if root["schema"] != RESOURCES_SCHEMA:
        raise ReportError("resources schema identifier is invalid")
    measurement_scope = _resource_require_fields(
        root["measurement_scope"],
        frozenset(RESOURCE_MEASUREMENT_SCOPE),
        location="measurement_scope",
    )
    if measurement_scope != RESOURCE_MEASUREMENT_SCOPE:
        raise ReportError("resources measurement scope is invalid")

    runtime = _resource_require_fields(
        root["runtime"],
        frozenset(
            {
                "status",
                "reason",
                "wall_time_ns",
                "user_cpu_time_ns",
                "system_cpu_time_ns",
                "max_rss_status",
                "max_rss_bytes",
            }
        ),
        location="runtime",
    )
    if runtime["status"] not in {"observed", "unavailable"}:
        raise ReportError("resources runtime.status is invalid")
    runtime_metrics: list[object] = []
    for field_name in ("wall_time_ns", "user_cpu_time_ns", "system_cpu_time_ns"):
        value = runtime[field_name]
        runtime_metrics.append(value)
        if value is not None:
            _resource_integer(value, location=f"runtime.{field_name}", nonnegative=True)
    if runtime["max_rss_status"] not in {"observed", "unavailable"}:
        raise ReportError("resources runtime.max_rss_status is invalid")
    max_rss_bytes = runtime["max_rss_bytes"]
    if max_rss_bytes is not None:
        _resource_integer(max_rss_bytes, location="runtime.max_rss_bytes", nonnegative=True)
    if (runtime["max_rss_status"] == "observed") != (max_rss_bytes is not None):
        raise ReportError("resources runtime max RSS status disagrees with its value")
    if runtime["reason"] is not None and not isinstance(runtime["reason"], str):
        raise ReportError("resources runtime.reason must be text or null")
    if runtime["status"] == "observed" and (
        runtime["reason"] is not None or any(value is None for value in runtime_metrics)
    ):
        raise ReportError("resources observed runtime must have metrics and no reason")
    if runtime["status"] == "unavailable" and (
        not isinstance(runtime["reason"], str)
        or not runtime["reason"]
        or any(value is not None for value in runtime_metrics)
        or runtime["max_rss_status"] != "unavailable"
        or max_rss_bytes is not None
    ):
        raise ReportError("resources unavailable runtime must have a reason and null metrics")

    host = _resource_require_fields(
        root["host"],
        frozenset(
            {
                "logical_cpus",
                "total_ram_bytes",
                "filesystem_free_bytes_before",
                "filesystem_free_bytes_after",
                "filesystem_free_delta_bytes",
                "filesystem_consumed_bytes",
            }
        ),
        location="host",
    )
    logical_cpus = _resource_integer(
        host["logical_cpus"],
        location="host.logical_cpus",
        nonnegative=True,
    )
    if logical_cpus < 1:
        raise ReportError("resources host.logical_cpus must be positive")
    if host["total_ram_bytes"] is not None:
        total_ram = _resource_integer(
            host["total_ram_bytes"],
            location="host.total_ram_bytes",
            nonnegative=True,
        )
        if total_ram < 1:
            raise ReportError("resources host.total_ram_bytes must be positive")
    for field_name in (
        "filesystem_free_bytes_before",
        "filesystem_free_bytes_after",
    ):
        _resource_integer(host[field_name], location=f"host.{field_name}", nonnegative=True)
    free_delta = _resource_integer(
        host["filesystem_free_delta_bytes"],
        location="host.filesystem_free_delta_bytes",
        nonnegative=False,
    )
    consumed = _resource_integer(
        host["filesystem_consumed_bytes"],
        location="host.filesystem_consumed_bytes",
        nonnegative=False,
    )
    before_free = cast("int", host["filesystem_free_bytes_before"])
    after_free = cast("int", host["filesystem_free_bytes_after"])
    if free_delta != after_free - before_free or consumed != before_free - after_free:
        raise ReportError("resources host filesystem deltas do not match free-byte observations")

    database = _resource_require_fields(
        root["database"],
        frozenset(
            {
                "status",
                "reason",
                "scope",
                "scope_assertion",
                "contaminated_by_concurrency",
                "before",
                "after",
                "delta",
            }
        ),
        location="database",
    )
    if database["status"] not in {"observed", "partial", "unavailable"}:
        raise ReportError("resources database.status is invalid")
    if database["scope"] not in {"shared", "exclusive"}:
        raise ReportError("resources database.scope is invalid")
    if database["scope_assertion"] != "caller_asserted_unverified":
        raise ReportError("resources database.scope_assertion is invalid")
    if not isinstance(database["contaminated_by_concurrency"], bool):
        raise ReportError("resources database.contaminated_by_concurrency must be boolean")
    if database["reason"] is not None and not isinstance(database["reason"], str):
        raise ReportError("resources database.reason must be text or null")
    for field_name in ("before", "after"):
        if database[field_name] is not None:
            _validate_snapshot_payload(database[field_name], location=f"database.{field_name}")
    if database["delta"] is not None:
        delta = _resource_require_fields(
            database["delta"],
            frozenset({"database_bytes", "babylon_relation_bytes", "wal_bytes"}),
            location="database.delta",
        )
        for field_name in delta:
            _resource_integer(
                delta[field_name],
                location=f"database.delta.{field_name}",
                nonnegative=False,
            )
    expected_contamination = database["scope"] == "shared"
    if database["contaminated_by_concurrency"] is not expected_contamination:
        raise ReportError("resources database contamination label disagrees with scope")
    snapshots_present = sum(database[field_name] is not None for field_name in ("before", "after"))
    reason_present = isinstance(database["reason"], str) and bool(database["reason"])
    if database["status"] == "observed" and (
        snapshots_present != 2 or database["delta"] is None or database["reason"] is not None
    ):
        raise ReportError("resources observed database state is incomplete")
    if database["status"] == "partial" and (
        snapshots_present != 1 or database["delta"] is not None or not reason_present
    ):
        raise ReportError("resources partial database state is inconsistent")
    if database["status"] == "unavailable" and (
        snapshots_present != 0 or database["delta"] is not None or not reason_present
    ):
        raise ReportError("resources unavailable database state is inconsistent")

    artifacts = _resource_require_fields(
        root["artifacts"],
        frozenset({"files", "payload_bytes"}),
        location="artifacts",
    )
    files = artifacts["files"]
    if not isinstance(files, list):
        raise ReportError("resources artifacts.files must be a list")
    paths: list[str] = []
    total = 0
    for index, candidate in enumerate(files):
        location = f"artifacts.files[{index}]"
        file = _resource_require_fields(
            candidate,
            frozenset({"path", "bytes", "sha256"}),
            location=location,
        )
        path = file["path"]
        if (
            not isinstance(path, str)
            or not path
            or Path(path).name != path
            or path == "resources.json"
        ):
            raise ReportError(f"resources {location}.path is invalid")
        paths.append(path)
        total += _resource_integer(
            file["bytes"],
            location=f"{location}.bytes",
            nonnegative=True,
        )
        digest = file["sha256"]
        if not isinstance(digest, str) or LOWER_HEX_DIGEST.fullmatch(digest) is None:
            raise ReportError(f"resources {location}.sha256 is invalid")
    if paths != sorted(paths) or len(paths) != len(set(paths)):
        raise ReportError("resources artifacts.files must have unique sorted paths")
    payload_bytes = _resource_integer(
        artifacts["payload_bytes"],
        location="artifacts.payload_bytes",
        nonnegative=True,
    )
    if payload_bytes != total:
        raise ReportError("resources artifacts.payload_bytes does not match file sizes")


def _write_resources(
    artifact_dir: Path,
    *,
    outcome: _ProcessOutcome | None,
    runtime_reason: str | None,
    before_probe: _PostgresProbe,
    after_probe: _PostgresProbe,
    database_scope: str,
    filesystem_free_before: int,
) -> None:
    if outcome is None:
        runtime_observation: dict[str, object] = {
            "status": "unavailable",
            "reason": runtime_reason or "runtime_not_observed",
            "wall_time_ns": None,
            "user_cpu_time_ns": None,
            "system_cpu_time_ns": None,
            "max_rss_status": "unavailable",
            "max_rss_bytes": None,
        }
    else:
        runtime_observation = {
            "status": "observed",
            "reason": None,
            "wall_time_ns": outcome.wall_time_ns,
            "user_cpu_time_ns": outcome.user_cpu_time_ns,
            "system_cpu_time_ns": outcome.system_cpu_time_ns,
            "max_rss_status": ("observed" if outcome.max_rss_bytes is not None else "unavailable"),
            "max_rss_bytes": outcome.max_rss_bytes,
        }
    filesystem_free_after = shutil.disk_usage(artifact_dir).free
    payload: dict[str, object] = {
        "schema": RESOURCES_SCHEMA,
        "measurement_scope": dict(RESOURCE_MEASUREMENT_SCOPE),
        "runtime": runtime_observation,
        "host": {
            "logical_cpus": os.cpu_count() or 1,
            "total_ram_bytes": _host_total_ram_bytes(),
            "filesystem_free_bytes_before": filesystem_free_before,
            "filesystem_free_bytes_after": filesystem_free_after,
            "filesystem_free_delta_bytes": filesystem_free_after - filesystem_free_before,
            "filesystem_consumed_bytes": filesystem_free_before - filesystem_free_after,
        },
        "database": _database_resource_observation(
            before_probe,
            after_probe,
            database_scope=database_scope,
        ),
        "artifacts": _artifact_resource_observation(artifact_dir),
    }
    _validate_resources_payload(payload)
    rendered = json.dumps(
        payload,
        indent=2,
        sort_keys=True,
        ensure_ascii=False,
        allow_nan=False,
    )
    (artifact_dir / "resources.json").write_text(rendered + "\n", encoding="utf-8")


def _finalize_wrapper_failure(
    *,
    artifact_dir: Path,
    report_path: Path,
    campaign_id: str,
    ticks: int,
    outcome: _ProcessOutcome,
    provenance: Mapping[str, object],
) -> RunResult:
    message = outcome.wrapper_error or "runtime wrapper failed"
    if report_path.exists():
        try:
            rows = _validate_jsonl(
                report_path,
                maximum_rows=ticks,
                first_resolve_tick=1,
            )
        except JsonlValidationError as error:
            validation_message = f"report validation failed: {error}"
            combined = f"{message}; {validation_message}"
            summary = _failure_summary(
                status=outcome.wrapper_status or "runtime_wrapper_failed",
                campaign_id=campaign_id,
                ticks_requested=ticks,
                runtime_exit_code=outcome.returncode,
                error=message,
                provenance=provenance,
                report_validation_error=validation_message,
            )
            _write_summary(artifact_dir, summary)
            return RunResult(
                artifact_dir,
                summary,
                VALIDATION_EXIT_CODE,
                combined,
            )

        summary = _evidence_summary(
            rows,
            campaign_id=campaign_id,
            ticks_requested=ticks,
            status=outcome.wrapper_status or "runtime_wrapper_failed",
            runtime_exit_code=outcome.returncode,
            provenance=provenance,
            error=message,
        )
        _write_csv(artifact_dir, rows)
        _write_diagnostics(artifact_dir, rows)
        _write_summary(artifact_dir, summary)
        return RunResult(
            artifact_dir,
            summary,
            VALIDATION_EXIT_CODE,
            message,
        )

    summary = _failure_summary(
        status=outcome.wrapper_status or "runtime_wrapper_failed",
        campaign_id=campaign_id,
        ticks_requested=ticks,
        runtime_exit_code=outcome.returncode,
        error=message,
        provenance=provenance,
    )
    _write_summary(artifact_dir, summary)
    return RunResult(
        artifact_dir,
        summary,
        VALIDATION_EXIT_CODE,
        message,
    )


def _finalize_runtime_failure(
    *,
    artifact_dir: Path,
    report_path: Path,
    campaign_id: str,
    ticks: int,
    runtime_exit_code: int,
    provenance: Mapping[str, object],
) -> RunResult:
    message = f"runtime exited with status {runtime_exit_code}"
    if not report_path.exists():
        summary = _failure_summary(
            status="runtime_failed",
            campaign_id=campaign_id,
            ticks_requested=ticks,
            runtime_exit_code=runtime_exit_code,
            error=message,
            provenance=provenance,
        )
        _write_summary(artifact_dir, summary)
        return RunResult(artifact_dir, summary, runtime_exit_code, message)

    try:
        rows = _validate_jsonl(
            report_path,
            maximum_rows=ticks,
            first_resolve_tick=1,
        )
    except JsonlValidationError as error:
        validation_message = f"report validation failed: {error}"
        combined = f"{message}; {validation_message}"
        summary = _failure_summary(
            status="runtime_failed",
            campaign_id=campaign_id,
            ticks_requested=ticks,
            runtime_exit_code=runtime_exit_code,
            error=message,
            provenance=provenance,
            report_validation_error=validation_message,
        )
        _write_summary(artifact_dir, summary)
        return RunResult(artifact_dir, summary, runtime_exit_code, combined)

    summary = _evidence_summary(
        rows,
        campaign_id=campaign_id,
        ticks_requested=ticks,
        status="runtime_failed",
        runtime_exit_code=runtime_exit_code,
        provenance=provenance,
        error=message,
    )
    _write_csv(artifact_dir, rows)
    _write_diagnostics(artifact_dir, rows)
    _write_summary(artifact_dir, summary)
    return RunResult(artifact_dir, summary, runtime_exit_code, message)


def run_report(
    *,
    runtime: Path,
    ticks: int,
    output_root: Path,
    timeout_seconds: float | None = None,
    database_scope: str = "shared",
) -> RunResult:
    """Run one fresh-campaign simulation and return its durable wrapper result."""
    if ticks < 1 or ticks > MAX_TICKS:
        raise ReportError(f"ticks must be between 1 and {MAX_TICKS}")
    effective_timeout = RUNTIME_TIMEOUT_SECONDS if timeout_seconds is None else timeout_seconds
    if not math.isfinite(effective_timeout) or effective_timeout <= 0:
        raise ReportError("runtime timeout must be a positive finite number")
    if database_scope not in {"shared", "exclusive"}:
        raise ReportError("database scope must be shared or exclusive")

    runtime_path = _validate_runtime(runtime)
    provenance = _source_provenance(runtime_path)
    child_environment, campaign_id = _child_environment()
    artifact_dir = _create_artifact_dir(output_root)
    filesystem_free_before = shutil.disk_usage(artifact_dir).free
    before_probe = _probe_postgres_snapshot(child_environment)
    report_path = artifact_dir / "ticks.jsonl"
    argv = [
        str(runtime_path),
        "run",
        "--ticks",
        str(ticks),
        "--restart-every",
        str(RESTART_INTERVAL_TICKS),
        "--report-jsonl",
        str(report_path),
    ]
    secrets = _redacted_environment_values(child_environment)
    try:
        outcome = _bounded_process_run(
            argv,
            environment=child_environment,
            timeout_seconds=effective_timeout,
        )
    except (OSError, ReportError) as error:
        after_probe = _probe_postgres_snapshot(child_environment)
        if isinstance(error, OSError):
            status = "launch_failed"
            message = f"could not launch runtime: {error.strerror or type(error).__name__}"
        else:
            status = "runtime_wrapper_failed"
            message = f"could not observe runtime: {error}"
        _write_process_logs(
            artifact_dir,
            stdout=b"",
            stderr=b"",
            secrets=secrets,
        )
        summary = _failure_summary(
            status=status,
            campaign_id=campaign_id,
            ticks_requested=ticks,
            runtime_exit_code=None,
            error=message,
            provenance=provenance,
        )
        _write_summary(artifact_dir, summary)
        result = RunResult(
            artifact_dir,
            summary,
            VALIDATION_EXIT_CODE,
            message,
        )
        _write_resources(
            artifact_dir,
            outcome=None,
            runtime_reason=status,
            before_probe=before_probe,
            after_probe=after_probe,
            database_scope=database_scope,
            filesystem_free_before=filesystem_free_before,
        )
        return result

    after_probe = _probe_postgres_snapshot(child_environment)
    _write_process_logs(
        artifact_dir,
        stdout=outcome.stdout,
        stderr=outcome.stderr,
        secrets=secrets,
    )
    if outcome.wrapper_status is not None:
        result = _finalize_wrapper_failure(
            artifact_dir=artifact_dir,
            report_path=report_path,
            campaign_id=campaign_id,
            ticks=ticks,
            outcome=outcome,
            provenance=provenance,
        )
        _write_resources(
            artifact_dir,
            outcome=outcome,
            runtime_reason=None,
            before_probe=before_probe,
            after_probe=after_probe,
            database_scope=database_scope,
            filesystem_free_before=filesystem_free_before,
        )
        return result
    if outcome.returncode != 0:
        result = _finalize_runtime_failure(
            artifact_dir=artifact_dir,
            report_path=report_path,
            campaign_id=campaign_id,
            ticks=ticks,
            runtime_exit_code=outcome.returncode,
            provenance=provenance,
        )
        _write_resources(
            artifact_dir,
            outcome=outcome,
            runtime_reason=None,
            before_probe=before_probe,
            after_probe=after_probe,
            database_scope=database_scope,
            filesystem_free_before=filesystem_free_before,
        )
        return result

    try:
        rows = _validate_jsonl(
            report_path,
            expected_rows=ticks,
            first_resolve_tick=1,
            complete_success=True,
        )
    except JsonlValidationError as error:
        message = f"report validation failed: {error}"
        summary = _failure_summary(
            status="validation_failed",
            campaign_id=campaign_id,
            ticks_requested=ticks,
            runtime_exit_code=0,
            error=message,
            provenance=provenance,
        )
        _write_summary(artifact_dir, summary)
        result = RunResult(
            artifact_dir,
            summary,
            VALIDATION_EXIT_CODE,
            message,
        )
        _write_resources(
            artifact_dir,
            outcome=outcome,
            runtime_reason=None,
            before_probe=before_probe,
            after_probe=after_probe,
            database_scope=database_scope,
            filesystem_free_before=filesystem_free_before,
        )
        return result

    summary = _evidence_summary(
        rows,
        campaign_id=campaign_id,
        ticks_requested=ticks,
        status="ok",
        runtime_exit_code=0,
        provenance=provenance,
    )
    _write_csv(artifact_dir, rows)
    _write_diagnostics(artifact_dir, rows)
    _write_summary(artifact_dir, summary)
    result = RunResult(artifact_dir, summary, 0)
    _write_resources(
        artifact_dir,
        outcome=outcome,
        runtime_reason=None,
        before_probe=before_probe,
        after_probe=after_probe,
        database_scope=database_scope,
        filesystem_free_before=filesystem_free_before,
    )
    return result


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description=(
            "Run babylon-runtime and retain a bounded, validated simulation report bundle."
        ),
        epilog=(
            "The wrapper always supplies a fresh child-only BABYLON_CAMPAIGN_ID; "
            "an inherited value is ignored so report runs cannot resume an old campaign."
        ),
    )
    parser.add_argument(
        "--runtime",
        required=True,
        type=Path,
        help="babylon-runtime executable",
    )
    parser.add_argument(
        "--ticks",
        required=True,
        type=_positive_ticks,
        help=(
            f"count of four-week periods to execute for the fresh campaign (maximum {MAX_TICKS})"
        ),
    )
    parser.add_argument(
        "--output-root",
        type=Path,
        default=Path(__file__).resolve().parents[2] / "reports" / "sim-runs",
        help="parent directory for collision-safe run artifacts",
    )
    parser.add_argument(
        "--timeout-seconds",
        type=_positive_timeout,
        default=RUNTIME_TIMEOUT_SECONDS,
        help="wall-clock runtime timeout in seconds",
    )
    parser.add_argument(
        "--database-scope",
        choices=("shared", "exclusive"),
        default="shared",
        help=(
            "database measurement scope; shared marks database and WAL deltas as "
            "contaminated by possible concurrent activity"
        ),
    )
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    arguments = _parser().parse_args(argv)
    try:
        result = run_report(
            runtime=arguments.runtime,
            ticks=arguments.ticks,
            output_root=arguments.output_root,
            timeout_seconds=arguments.timeout_seconds,
            database_scope=arguments.database_scope,
        )
    except ReportError as error:
        print(f"sim-report: {error}", file=sys.stderr)
        return VALIDATION_EXIT_CODE

    print(result.artifact_dir)
    print(render_summary(result.summary), end="")
    if result.error is not None:
        print(f"sim-report: {result.error}", file=sys.stderr)
    return result.exit_code


if __name__ == "__main__":
    raise SystemExit(main())
