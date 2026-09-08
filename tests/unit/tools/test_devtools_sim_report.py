"""Contracts for the stdlib-only Rust simulation report wrapper."""

from __future__ import annotations

import argparse
import copy
import csv
import hashlib
import json
import os
import struct
import uuid
from pathlib import Path

import pytest
from tools.devtools import sim_report

pytestmark = pytest.mark.unit

CAMPAIGN_ID = "aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee"

SCOPE = {
    "slice_id": "michigan-persistence-slice",
    "scenario": "production/michigan-rust-runtime",
    "fixed_replay_seed": 281,
    "tick_duration_days": 28,
    "parameter_overrides": False,
    "stochastic_draws": False,
    "dynamic_h3_updates": False,
}

FOUNDATION = {
    "foundation_sha256": "61" * 32,
    "defines_sha256": "62" * 32,
    "rules_sha256": "63" * 32,
    "reference_sha256": "64" * 32,
}

PROVENANCE = {
    "git_head_sha": "1" * 40,
    "git_dirty": False,
    "runtime_binary_sha256": "2" * 64,
    "reporter_sha256": "3" * 64,
}

OBSERVABLE_FIELDS = (
    ("territory/median-wage", "configured_input", 21.0),
    ("territory/phi-hour", "configured_input", 1.0),
    ("territory/phi-savings-adjustment", "dynamic", 0.047619047619047616),
    ("territory/rate-accumulation", "dynamic", 0.0),
    ("territory/dist-year", "dynamic", 2010.0),
)


def _digest(value: int) -> str:
    return f"{value:064x}"


def _bits_hex(value: float) -> str:
    return struct.pack(">d", value).hex()


def _observable(
    field: str,
    role: str,
    before_value: float,
    after_value: float,
) -> dict[str, object]:
    return {
        "name": f"{SCOPE['scenario']}::wayne::{field}",
        "entity": "wayne",
        "field": field,
        "role": role,
        "kind": "f64",
        "before_value": before_value,
        "before_bits_hex": _bits_hex(before_value),
        "after_value": after_value,
        "after_bits_hex": _bits_hex(after_value),
    }


def _valid_row(
    resolve_tick: int,
    *,
    reopened_after_commit: bool | None = None,
) -> dict[str, object]:
    reopened = resolve_tick % 13 == 0 if reopened_after_commit is None else reopened_after_commit
    return {
        "schema": "babylon.simulation.tick-report.v2",
        "resolve_tick": resolve_tick,
        "commit_disposition": "committed",
        "scope": copy.deepcopy(SCOPE),
        "foundation": copy.deepcopy(FOUNDATION),
        "persistence": {"reopened_after_commit": reopened},
        "stable_graph": {
            "before_sha256": _digest(resolve_tick + 500),
            "after_sha256": _digest(resolve_tick + 501),
        },
        "graph": {
            "before_sha256": _digest(resolve_tick),
            "after_sha256": _digest(resolve_tick + 1),
        },
        "world": {
            "before_sha256": _digest(resolve_tick + 100),
            "after_sha256": _digest(resolve_tick + 101),
        },
        "rules": {
            "considered": 3,
            "fired": 1,
            "per_rule": [
                {"rule_id": "phase/example-a", "considered": 2, "fired": 1},
                {"rule_id": "phase/example-b", "considered": 1, "fired": 0},
            ],
        },
        "events": {
            "count": 2,
            "digest_sha256": _digest(resolve_tick + 200),
            "per_type": [
                {"event_type": "EventType/ALPHA", "count": 1},
                {"event_type": "EventType/OMEGA", "count": 1},
            ],
        },
        "observables": [
            _observable(field, role, value, value) for field, role, value in OBSERVABLE_FIELDS
        ],
        "audit_receipts": {"count": 2},
        "material_rows": {
            "count": 3,
            "digest_sha256": _digest(resolve_tick + 300),
        },
        "tick_content_hash": _digest(resolve_tick + 400),
    }


def _complete_rows(*resolve_ticks: int) -> list[dict[str, object]]:
    rows = [_valid_row(tick) for tick in resolve_ticks]
    if rows:
        persistence = rows[-1]["persistence"]
        assert isinstance(persistence, dict)
        persistence["reopened_after_commit"] = True
    return rows


def _write_runtime(path: Path, body: str) -> Path:
    path.write_text("#!/usr/bin/env python3\n" + body, encoding="utf-8")
    path.chmod(0o755)
    return path


def _configured_runtime(
    path: Path,
    *,
    rows: list[dict[str, object]] | None,
    exit_code: int = 0,
    stdout: str = "",
    stderr: str = "",
    stdout_env: list[str] | None = None,
    stderr_env: list[str] | None = None,
    capture_campaign: bool = False,
    print_campaign: bool = False,
    sleep_seconds: float = 0.0,
    marker_path: Path | None = None,
) -> Path:
    config = {
        "rows": rows,
        "exit_code": exit_code,
        "stdout": stdout,
        "stderr": stderr,
        "stdout_env": stdout_env or [],
        "stderr_env": stderr_env or [],
        "capture_campaign": capture_campaign,
        "print_campaign": print_campaign,
        "sleep_seconds": sleep_seconds,
        "marker_path": str(marker_path) if marker_path is not None else None,
    }
    _write_runtime(
        path,
        """\
import json
import os
import pathlib
import sys
import time

assert sys.argv[1] == "run"
assert sys.argv[2] == "--ticks"
assert sys.argv[4:6] == ["--restart-every", "13"]
assert sys.argv[6] == "--report-jsonl"
report_path = pathlib.Path(sys.argv[7])
config = json.loads(pathlib.Path(sys.argv[0] + ".config.json").read_text(encoding="utf-8"))
if config["rows"] is not None:
    report_path.write_text(
        "".join(json.dumps(row) + "\\n" for row in config["rows"]),
        encoding="utf-8",
    )
campaign_id = os.environ["BABYLON_CAMPAIGN_ID"]
if config["capture_campaign"]:
    capture_path = pathlib.Path(os.environ["CAMPAIGN_CAPTURE_PATH"])
    with capture_path.open("a", encoding="utf-8") as capture:
        capture.write(campaign_id + "\\n")
sys.stdout.write(config["stdout"])
for key in config["stdout_env"]:
    sys.stdout.write(os.environ[key] + "\\n")
if config["print_campaign"]:
    sys.stdout.write(campaign_id + "\\n")
sys.stdout.flush()
sys.stderr.write(config["stderr"])
for key in config["stderr_env"]:
    sys.stderr.write(os.environ[key] + "\\n")
sys.stderr.flush()
time.sleep(config["sleep_seconds"])
if config["marker_path"] is not None:
    pathlib.Path(config["marker_path"]).write_text("survived\\n", encoding="utf-8")
raise SystemExit(config["exit_code"])
""",
    )
    path.with_name(path.name + ".config.json").write_text(json.dumps(config), encoding="utf-8")
    return path


def _write_jsonl(path: Path, rows: list[dict[str, object]]) -> None:
    path.write_text("".join(json.dumps(row) + "\n" for row in rows), encoding="utf-8")


def _summary(artifact: Path) -> dict[str, object]:
    return json.loads((artifact / "summary.json").read_text(encoding="utf-8"))


def _set_observable(
    row: dict[str, object],
    index: int,
    *,
    before_value: float,
    after_value: float,
) -> None:
    observables = row["observables"]
    assert isinstance(observables, list)
    observable = observables[index]
    assert isinstance(observable, dict)
    observable["before_value"] = before_value
    observable["before_bits_hex"] = _bits_hex(before_value)
    observable["after_value"] = after_value
    observable["after_bits_hex"] = _bits_hex(after_value)


def _load_json(path: Path) -> dict[str, object]:
    loaded = json.loads(path.read_text(encoding="utf-8"))
    assert isinstance(loaded, dict)
    return loaded


class _TrackingReader:
    def __init__(self, lines: list[bytes]) -> None:
        self.lines = list(lines)
        self.readline_sizes: list[int] = []
        self.iteration_used = False

    def __enter__(self) -> _TrackingReader:
        return self

    def __exit__(self, *_args: object) -> None:
        return None

    def __iter__(self) -> _TrackingReader:
        self.iteration_used = True
        return self

    def __next__(self) -> bytes:
        if not self.lines:
            raise StopIteration
        return self.lines.pop(0)

    def readline(self, size: int = -1) -> bytes:
        self.readline_sizes.append(size)
        if not self.lines:
            return b""
        return self.lines.pop(0)[:size]


def test_success_creates_unique_secret_safe_artifacts_summary_and_csv(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    rows = _complete_rows(1, 2)
    runtime = _configured_runtime(
        tmp_path / "babylon-runtime",
        rows=rows,
        stdout="runtime stdout must stay in its artifact\n",
        stderr_env=["VISIBLE_SENTINEL", "DATABASE_URL", "PGDATABASE"],
    )
    output_root = tmp_path / "reports" / "sim-runs"
    monkeypatch.setenv("BABYLON_CAMPAIGN_ID", CAMPAIGN_ID)
    monkeypatch.setenv("VISIBLE_SENTINEL", "inherited-by-runtime")
    monkeypatch.setenv("DATABASE_URL", "postgresql://user:top-secret@example.invalid/babylon")
    monkeypatch.setenv("PGDATABASE", "pgdatabase-secret")
    monkeypatch.setattr(
        sim_report, "_source_provenance", lambda _runtime: copy.deepcopy(PROVENANCE)
    )

    first_exit = sim_report.main(
        [
            "--runtime",
            str(runtime),
            "--ticks",
            "2",
            "--output-root",
            str(output_root),
        ]
    )
    first_console = capsys.readouterr()
    second_exit = sim_report.main(
        [
            "--runtime",
            str(runtime),
            "--ticks",
            "2",
            "--output-root",
            str(output_root),
        ]
    )
    second_console = capsys.readouterr()

    assert first_exit == second_exit == 0
    first_artifact = Path(first_console.out.splitlines()[0])
    second_artifact = Path(second_console.out.splitlines()[0])
    assert first_artifact.parent == output_root
    assert second_artifact.parent == output_root
    assert first_artifact != second_artifact
    assert first_artifact.is_dir()
    assert second_artifact.is_dir()

    first_summary = _summary(first_artifact)
    second_summary = _summary(second_artifact)
    first_campaign = first_summary.pop("campaign_id")
    second_campaign = second_summary.pop("campaign_id")
    assert isinstance(first_campaign, str)
    assert isinstance(second_campaign, str)
    assert first_campaign != second_campaign
    assert CAMPAIGN_ID not in {first_campaign, second_campaign}
    assert uuid.UUID(first_campaign).version == 4
    assert uuid.UUID(second_campaign).version == 4
    assert first_summary == {
        "diagnostics": {
            "notice_codes": [
                "scope.fixed_parameters",
                "scope.no_stochastic_draws",
                "scope.no_dynamic_h3_updates",
                "rule.never_fired",
                "material_rows.constant",
                "observable.dynamic_flat",
            ],
            "notice_count": 8,
        },
        "final": {
            "administrative_graph_sha256": _digest(3),
            "reopened_after_commit": True,
            "stable_graph_sha256": _digest(503),
            "tick_content_hash": _digest(402),
            "world_sha256": _digest(103),
        },
        "foundation": FOUNDATION,
        "first_resolve_tick": 1,
        "last_resolve_tick": 2,
        "provenance": PROVENANCE,
        "runtime_exit_code": 0,
        "schema": "babylon.simulation.run-summary.v2",
        "status": "ok",
        "ticks_reported": 2,
        "ticks_requested": 2,
        "tick_duration_days": 28,
        "totals": {
            "audit_receipts": 4,
            "events": 4,
            "material_rows": 6,
            "rules_considered": 6,
            "rules_fired": 2,
        },
    }
    assert "runtime stdout must stay" in (first_artifact / "stdout.txt").read_text()
    stderr_log = (first_artifact / "stderr.txt").read_text()
    assert "inherited-by-runtime" in stderr_log
    assert "top-secret" not in stderr_log
    assert "pgdatabase-secret" not in stderr_log
    assert "[REDACTED]" in stderr_log
    assert "runtime stdout must stay" not in first_console.out
    assert f"campaign: {first_campaign}" in first_console.out
    assert "ticks: 2 reported / 2 requested" in first_console.out
    assert "final administrative graph:" in first_console.out
    assert "final stable graph:" in first_console.out
    assert "\nfinal graph:" not in first_console.out
    assert "diagnostic notices: 8" in first_console.out
    assert "diagnostic codes:" in first_console.out
    assert first_console.err == ""

    csv_lines = (first_artifact / "ticks.csv").read_text(encoding="utf-8").splitlines()
    assert csv_lines[0].split(",") == list(sim_report.CSV_COLUMNS)
    assert len(csv_lines) == 3
    assert csv_lines[1].startswith("1,committed,")
    assert csv_lines[2].endswith("," + _digest(402))
    assert "graph_before_sha256" not in sim_report.CSV_COLUMNS
    assert "administrative_graph_before_sha256" in sim_report.CSV_COLUMNS
    assert "stable_graph_before_sha256" in sim_report.CSV_COLUMNS
    with (first_artifact / "ticks.csv").open(encoding="utf-8", newline="") as csv_file:
        csv_rows = list(csv.DictReader(csv_file))
    assert csv_rows[0]["reopened_after_commit"] == "False"
    assert csv_rows[1]["reopened_after_commit"] == "True"
    for prefix in (
        "median_wage",
        "phi_hour",
        "phi_savings_adjustment",
        "rate_accumulation",
        "dist_year",
    ):
        for suffix in ("before_value", "before_bits_hex", "after_value", "after_bits_hex"):
            assert f"{prefix}_{suffix}" in sim_report.CSV_COLUMNS

    persisted = "".join(
        file.read_text(encoding="utf-8") for file in first_artifact.iterdir() if file.is_file()
    )
    assert "postgresql://user:top-secret@example.invalid/babylon" not in persisted
    assert "pgdatabase-secret" not in persisted


def test_success_writes_separate_resource_observations_and_artifact_manifest(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    runtime = _configured_runtime(
        tmp_path / "babylon-runtime",
        rows=_complete_rows(1, 2),
    )
    probes = iter(
        [
            sim_report._PostgresProbe(
                snapshot=sim_report._PostgresSnapshot(
                    database_bytes=1_000,
                    babylon_relation_bytes=400,
                    wal_bytes=10_000,
                ),
                error=None,
            ),
            sim_report._PostgresProbe(
                snapshot=sim_report._PostgresSnapshot(
                    database_bytes=1_600,
                    babylon_relation_bytes=850,
                    wal_bytes=10_750,
                ),
                error=None,
            ),
        ]
    )
    monkeypatch.setattr(sim_report, "_probe_postgres_snapshot", lambda _environment: next(probes))

    result = sim_report.run_report(
        runtime=runtime,
        ticks=2,
        output_root=tmp_path / "out",
        database_scope="exclusive",
    )

    assert result.exit_code == 0
    diagnostics = _load_json(result.artifact_dir / "diagnostics.json")
    assert diagnostics["ticks_reported"] == 2
    resources = _load_json(result.artifact_dir / "resources.json")
    assert resources["schema"] == "babylon.simulation.resource-observation.v1"
    assert resources["measurement_scope"] == {
        "compilation": "excluded",
        "filesystem_disk": "filesystem-wide-coarse-unattributed",
        "filesystem_disk_caveat": "includes-concurrent-and-background-allocations",
        "postgresql_cpu_ram": "unobserved",
        "report_wrapper": "excluded",
        "runtime_cpu_rss": "babylon-runtime-process-only",
        "runtime_wall_time": "babylon-runtime-invocation-only",
    }
    runtime_observation = resources["runtime"]
    assert isinstance(runtime_observation, dict)
    assert runtime_observation["status"] == "observed"
    assert runtime_observation["reason"] is None
    for metric in ("wall_time_ns", "user_cpu_time_ns", "system_cpu_time_ns"):
        assert isinstance(runtime_observation[metric], int)
        assert runtime_observation[metric] >= 0
    assert runtime_observation["max_rss_status"] in {"observed", "unavailable"}
    if runtime_observation["max_rss_status"] == "observed":
        assert isinstance(runtime_observation["max_rss_bytes"], int)
        assert runtime_observation["max_rss_bytes"] >= 0
    else:
        assert runtime_observation["max_rss_bytes"] is None
    host = resources["host"]
    assert isinstance(host, dict)
    assert isinstance(host["logical_cpus"], int)
    assert host["logical_cpus"] >= 1
    assert host["total_ram_bytes"] is None or host["total_ram_bytes"] > 0
    assert host["filesystem_free_bytes_before"] >= 0
    assert host["filesystem_free_bytes_after"] >= 0
    assert host["filesystem_free_delta_bytes"] == (
        host["filesystem_free_bytes_after"] - host["filesystem_free_bytes_before"]
    )
    assert host["filesystem_consumed_bytes"] == -host["filesystem_free_delta_bytes"]
    assert resources["database"] == {
        "after": {
            "babylon_relation_bytes": 850,
            "database_bytes": 1_600,
            "wal_bytes": 10_750,
        },
        "before": {
            "babylon_relation_bytes": 400,
            "database_bytes": 1_000,
            "wal_bytes": 10_000,
        },
        "contaminated_by_concurrency": False,
        "delta": {
            "babylon_relation_bytes": 450,
            "database_bytes": 600,
            "wal_bytes": 750,
        },
        "reason": None,
        "scope": "exclusive",
        "scope_assertion": "caller_asserted_unverified",
        "status": "observed",
    }
    artifacts = resources["artifacts"]
    assert isinstance(artifacts, dict)
    files = artifacts["files"]
    assert isinstance(files, list)
    paths = [entry["path"] for entry in files]
    assert paths == sorted(paths)
    assert "resources.json" not in paths
    assert "diagnostics.json" in paths
    assert artifacts["payload_bytes"] == sum(entry["bytes"] for entry in files)
    for entry in files:
        artifact_path = result.artifact_dir / entry["path"]
        assert entry["bytes"] == artifact_path.stat().st_size
        assert entry["sha256"] == hashlib.sha256(artifact_path.read_bytes()).hexdigest()


def test_postgres_probe_keeps_runtime_dsn_out_of_argv_and_persisted_errors(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    secret = "postgresql://sim-user:probe-secret@db.invalid/babylon"
    captured: dict[str, object] = {}

    def fake_bounded_run(
        argv: list[str],
        *,
        environment: dict[str, str],
        timeout_seconds: float,
        stdout_limit: int,
        stderr_limit: int,
    ) -> sim_report._ProcessOutcome:
        captured.update(
            argv=argv,
            environment=environment,
            timeout_seconds=timeout_seconds,
            stdout_limit=stdout_limit,
            stderr_limit=stderr_limit,
        )
        return sim_report._ProcessOutcome(
            returncode=0,
            stdout=b"100\t50\t1000\n",
            stderr=b"",
            wall_time_ns=1,
            user_cpu_time_ns=2,
            system_cpu_time_ns=3,
            max_rss_bytes=4,
        )

    monkeypatch.setattr(sim_report, "_bounded_process_run", fake_bounded_run)
    environment = {"PATH": os.environ.get("PATH", ""), "BABYLON_RUNTIME_DSN": secret}

    probe = sim_report._probe_postgres_snapshot(environment)

    assert probe == sim_report._PostgresProbe(
        snapshot=sim_report._PostgresSnapshot(
            database_bytes=100,
            babylon_relation_bytes=50,
            wal_bytes=1_000,
        ),
        error=None,
    )
    argv = captured["argv"]
    assert isinstance(argv, list)
    assert secret not in " ".join(argv)
    child_environment = captured["environment"]
    assert isinstance(child_environment, dict)
    assert child_environment["PGHOST"] == "db.invalid"
    assert child_environment["PGUSER"] == "sim-user"
    assert child_environment["PGPASSWORD"] == "probe-secret"
    assert child_environment["PGDATABASE"] == "babylon"
    assert "BABYLON_RUNTIME_DSN" not in child_environment
    assert captured["timeout_seconds"] == sim_report.POSTGRES_PROBE_TIMEOUT_SECONDS
    assert captured["stdout_limit"] == sim_report.MAX_POSTGRES_PROBE_OUTPUT_BYTES
    assert captured["stderr_limit"] == sim_report.MAX_POSTGRES_PROBE_OUTPUT_BYTES


def test_postgres_probe_translates_keyword_dsn_without_inheriting_pg_state(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    captured_environment: dict[str, str] = {}

    def fake_bounded_run(
        _argv: list[str],
        *,
        environment: dict[str, str],
        timeout_seconds: float,
        stdout_limit: int,
        stderr_limit: int,
    ) -> sim_report._ProcessOutcome:
        del timeout_seconds, stdout_limit, stderr_limit
        captured_environment.update(environment)
        return sim_report._ProcessOutcome(
            returncode=0,
            stdout=b"100\t50\t1000\n",
            stderr=b"",
            wall_time_ns=1,
            user_cpu_time_ns=2,
            system_cpu_time_ns=3,
            max_rss_bytes=4,
        )

    monkeypatch.setattr(sim_report, "_bounded_process_run", fake_bounded_run)
    environment = {
        "PATH": os.environ.get("PATH", ""),
        "PGHOST": "wrong.invalid",
        "PGPASSWORD": "inherited-secret",
        "BABYLON_RUNTIME_DSN": (
            "host=127.0.0.1 port=5433 dbname=babylon_test "
            "user=test password='probe secret' sslmode=disable"
        ),
    }

    probe = sim_report._probe_postgres_snapshot(environment)

    assert probe.error is None
    assert captured_environment["PGHOST"] == "127.0.0.1"
    assert captured_environment["PGPORT"] == "5433"
    assert captured_environment["PGDATABASE"] == "babylon_test"
    assert captured_environment["PGUSER"] == "test"
    assert captured_environment["PGPASSWORD"] == "probe secret"
    assert captured_environment["PGSSLMODE"] == "disable"
    assert captured_environment["PGCONNECT_TIMEOUT"] == "10"
    assert "BABYLON_RUNTIME_DSN" not in captured_environment


def test_source_provenance_hashes_the_executed_runtime_and_reporter(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    runtime = _write_runtime(tmp_path / "babylon-runtime", "raise SystemExit(0)\n")
    monkeypatch.setattr(sim_report, "_git_source_state", lambda: ("a" * 40, True))

    provenance = sim_report._source_provenance(runtime)

    assert provenance == {
        "git_head_sha": "a" * 40,
        "git_dirty": True,
        "runtime_binary_sha256": hashlib.sha256(runtime.read_bytes()).hexdigest(),
        "reporter_sha256": hashlib.sha256(
            Path(sim_report.__file__).resolve().read_bytes()
        ).hexdigest(),
    }


def test_postgres_probe_failure_is_nonfatal_and_shared_deltas_are_contaminated(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    runtime = _configured_runtime(tmp_path / "babylon-runtime", rows=_complete_rows(1))
    monkeypatch.setattr(
        sim_report,
        "_probe_postgres_snapshot",
        lambda _environment: sim_report._PostgresProbe(
            snapshot=None,
            error="probe_failed",
        ),
    )

    result = sim_report.run_report(runtime=runtime, ticks=1, output_root=tmp_path / "out")

    assert result.exit_code == 0
    database = _load_json(result.artifact_dir / "resources.json")["database"]
    assert database == {
        "after": None,
        "before": None,
        "contaminated_by_concurrency": True,
        "delta": None,
        "reason": "before:probe_failed; after:probe_failed",
        "scope": "shared",
        "scope_assertion": "caller_asserted_unverified",
        "status": "unavailable",
    }


def test_resource_schema_validation_rejects_noninteger_measurements(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    runtime = _configured_runtime(tmp_path / "babylon-runtime", rows=_complete_rows(1))
    monkeypatch.setattr(
        sim_report,
        "_probe_postgres_snapshot",
        lambda _environment: sim_report._PostgresProbe(None, "dsn_unavailable"),
    )
    result = sim_report.run_report(runtime=runtime, ticks=1, output_root=tmp_path / "out")
    payload = _load_json(result.artifact_dir / "resources.json")
    artifacts = payload["artifacts"]
    assert isinstance(artifacts, dict)
    files = artifacts["files"]
    assert isinstance(files, list)
    files[0]["bytes"] = "1"

    with pytest.raises(sim_report.ReportError, match=r"artifacts\.files\[0\]\.bytes"):
        sim_report._validate_resources_payload(payload)


def _invalid_row(case: str) -> dict[str, object]:  # noqa: C901 - table-driven mutations
    row = copy.deepcopy(_valid_row(41))
    if case == "missing_top":
        del row["world"]
    elif case == "extra_top":
        row["unexpected"] = "value"
    elif case == "missing_nested":
        del row["graph"]["after_sha256"]  # type: ignore[index]
    elif case == "zero_tick":
        row["resolve_tick"] = 0
    elif case == "bad_disposition":
        row["commit_disposition"] = "maybe_committed"
    elif case == "nontext_disposition":
        row["commit_disposition"] = []
    elif case == "uppercase_digest":
        row["tick_content_hash"] = "A" * 64
    elif case == "uppercase_stable_graph_digest":
        row["stable_graph"]["after_sha256"] = "A" * 64  # type: ignore[index]
    elif case == "foundation_extra":
        row["foundation"]["extra"] = _digest(999)  # type: ignore[index]
    elif case == "persistence_nonboolean":
        row["persistence"]["reopened_after_commit"] = 1  # type: ignore[index]
    elif case == "bool_count":
        row["events"]["count"] = True  # type: ignore[index]
    elif case == "scope_extra":
        row["scope"]["extra"] = False  # type: ignore[index]
    elif case == "scope_seed":
        row["scope"]["fixed_replay_seed"] = 282  # type: ignore[index]
    elif case == "scope_weekly_duration":
        row["scope"]["tick_duration_days"] = 7  # type: ignore[index]
    elif case == "scope_duration_float":
        row["scope"]["tick_duration_days"] = 28.0  # type: ignore[index]
    elif case == "scope_duration_missing":
        del row["scope"]["tick_duration_days"]  # type: ignore[index]
    elif case == "scope_capability":
        row["scope"]["stochastic_draws"] = True  # type: ignore[index]
    elif case == "event_types_unsorted":
        row["events"]["per_type"].reverse()  # type: ignore[index]
    elif case == "event_type_sum":
        row["events"]["per_type"][0]["count"] = 2  # type: ignore[index]
    elif case == "observable_missing":
        row["observables"].pop()  # type: ignore[union-attr]
    elif case == "observable_order":
        row["observables"][0], row["observables"][1] = (  # type: ignore[index]
            row["observables"][1],  # type: ignore[index]
            row["observables"][0],  # type: ignore[index]
        )
    elif case == "observable_role":
        row["observables"][0]["role"] = "dynamic"  # type: ignore[index]
    elif case == "observable_nonfinite":
        row["observables"][2]["after_value"] = float("nan")  # type: ignore[index]
    elif case == "observable_bits":
        row["observables"][2]["after_bits_hex"] = "0" * 16  # type: ignore[index]
    elif case == "top_fired_gt_considered":
        row["rules"]["considered"] = 0  # type: ignore[index]
    elif case == "empty_rule_id":
        row["rules"]["per_rule"][0]["rule_id"] = ""  # type: ignore[index]
    elif case == "per_rule_fired_gt_considered":
        row["rules"]["per_rule"][0]["fired"] = 3  # type: ignore[index]
    elif case == "per_rule_sum_mismatch":
        row["rules"]["considered"] = 4  # type: ignore[index]
    else:
        raise AssertionError(f"unknown case {case}")
    return row


@pytest.mark.parametrize(
    ("case", "message"),
    [
        ("missing_top", "top-level fields"),
        ("extra_top", "top-level fields"),
        ("missing_nested", "graph fields"),
        ("zero_tick", "positive resolve_tick"),
        ("bad_disposition", "commit_disposition"),
        ("nontext_disposition", "commit_disposition"),
        ("uppercase_digest", "lowercase 64-hex"),
        ("uppercase_stable_graph_digest", "stable_graph.after_sha256"),
        ("foundation_extra", "foundation fields"),
        ("persistence_nonboolean", "persistence.reopened_after_commit"),
        ("bool_count", "events.count"),
        ("scope_extra", "scope fields"),
        ("scope_seed", "scope.fixed_replay_seed"),
        ("scope_capability", "scope.stochastic_draws"),
        ("scope_weekly_duration", "scope.tick_duration_days"),
        ("scope_duration_float", "scope.tick_duration_days"),
        ("scope_duration_missing", "scope fields"),
        ("event_types_unsorted", "events.per_type must be sorted"),
        ("event_type_sum", "per-type event count sum"),
        ("observable_missing", "exactly 5 observables"),
        ("observable_order", "observable 0 field"),
        ("observable_role", "observable 0 role"),
        ("observable_nonfinite", "observable 2 after_value must be finite"),
        ("observable_bits", "observable 2 after_bits_hex does not match after_value"),
        ("top_fired_gt_considered", "rules.considered must be >= rules.fired"),
        ("empty_rule_id", "nonempty rule_id"),
        ("per_rule_fired_gt_considered", "considered must be >= fired"),
        ("per_rule_sum_mismatch", "per-rule considered sum"),
    ],
)
def test_strict_v2_row_validation_rejects_wrong_shapes_and_values(
    tmp_path: Path,
    case: str,
    message: str,
) -> None:
    report = tmp_path / "ticks.jsonl"
    _write_jsonl(report, [_invalid_row(case)])

    with pytest.raises(sim_report.JsonlValidationError, match=message):
        sim_report._validate_jsonl(report, expected_rows=1)


def test_partial_rows_must_be_contiguous_but_may_start_after_one(
    tmp_path: Path,
) -> None:
    report = tmp_path / "ticks.jsonl"
    _write_jsonl(report, [_valid_row(41), _valid_row(43)])

    with pytest.raises(sim_report.JsonlValidationError, match="contiguous resolve_tick 42"):
        sim_report._validate_jsonl(report, expected_rows=2)

    _write_jsonl(report, [_valid_row(41), _valid_row(42)])
    rows = sim_report._validate_jsonl(report, expected_rows=2)
    assert [row["resolve_tick"] for row in rows] == [41, 42]


def test_durable_rows_accept_continuous_graph_and_world_hash_chains(
    tmp_path: Path,
) -> None:
    report = tmp_path / "ticks.jsonl"
    _write_jsonl(report, [_valid_row(1), _valid_row(2), _valid_row(3)])

    rows = sim_report._validate_jsonl(report, expected_rows=3)

    assert [row["resolve_tick"] for row in rows] == [1, 2, 3]


def test_v2_scope_and_ordered_rule_inventory_cannot_drift_between_ticks(
    tmp_path: Path,
) -> None:
    rows = [_valid_row(1), _valid_row(2)]
    rules = rows[1]["rules"]
    assert isinstance(rules, dict)
    per_rule = rules["per_rule"]
    assert isinstance(per_rule, list)
    per_rule.reverse()
    report = tmp_path / "ticks.jsonl"
    _write_jsonl(report, rows)

    with pytest.raises(sim_report.JsonlValidationError, match="rule inventory differs"):
        sim_report._validate_jsonl(report, expected_rows=2)


def test_v2_foundation_and_observable_state_cannot_drift_between_ticks(
    tmp_path: Path,
) -> None:
    report = tmp_path / "ticks.jsonl"
    rows = [_valid_row(1), _valid_row(2)]
    foundation = rows[1]["foundation"]
    assert isinstance(foundation, dict)
    foundation["rules_sha256"] = _digest(999)
    _write_jsonl(report, rows)
    with pytest.raises(sim_report.JsonlValidationError, match="foundation differs"):
        sim_report._validate_jsonl(report, expected_rows=2)

    rows = [_valid_row(1), _valid_row(2)]
    _set_observable(rows[1], 2, before_value=9.0, after_value=9.0)
    _write_jsonl(report, rows)
    with pytest.raises(
        sim_report.JsonlValidationError,
        match="observable 2 before_bits_hex does not equal the previous row's after_bits_hex",
    ):
        sim_report._validate_jsonl(report, expected_rows=2)


def test_restart_schedule_requires_intervals_and_only_complete_final_readback(
    tmp_path: Path,
) -> None:
    report = tmp_path / "ticks.jsonl"
    _write_jsonl(report, [_valid_row(13, reopened_after_commit=False)])
    with pytest.raises(sim_report.JsonlValidationError, match="resolve_tick 13 must be reopened"):
        sim_report._validate_jsonl(report, expected_rows=1)

    _write_jsonl(
        report,
        [
            _valid_row(1, reopened_after_commit=True),
            _valid_row(2, reopened_after_commit=True),
        ],
    )
    with pytest.raises(
        sim_report.JsonlValidationError, match="unexpected reopen at resolve_tick 1"
    ):
        sim_report._validate_jsonl(report, expected_rows=2, complete_success=True)

    partial = [_valid_row(1), _valid_row(2)]
    _write_jsonl(report, partial)
    assert len(sim_report._validate_jsonl(report, expected_rows=2)) == 2
    with pytest.raises(
        sim_report.JsonlValidationError,
        match="final resolve_tick 2 must be reopened",
    ):
        sim_report._validate_jsonl(report, expected_rows=2, complete_success=True)

    persistence = partial[-1]["persistence"]
    assert isinstance(persistence, dict)
    persistence["reopened_after_commit"] = True
    _write_jsonl(report, partial)
    assert len(sim_report._validate_jsonl(report, expected_rows=2, complete_success=True)) == 2


def test_zero_events_requires_an_empty_per_type_breakdown(tmp_path: Path) -> None:
    row = _valid_row(1)
    events = row["events"]
    assert isinstance(events, dict)
    events["count"] = 0
    events["per_type"] = []
    report = tmp_path / "ticks.jsonl"
    _write_jsonl(report, [row])

    assert sim_report._validate_jsonl(report, expected_rows=1)[0]["events"] == events

    events["per_type"] = [{"event_type": "EventType/ALPHA", "count": 0}]
    _write_jsonl(report, [row])
    with pytest.raises(sim_report.JsonlValidationError, match="positive integer"):
        sim_report._validate_jsonl(report, expected_rows=1)


def test_diagnostics_exposes_plateaus_rule_event_material_and_observable_trends() -> None:
    rows = [_valid_row(1), _valid_row(2), _valid_row(3)]
    for row in rows[1:]:
        graph = row["graph"]
        assert isinstance(graph, dict)
        graph["before_sha256"] = _digest(2)
        graph["after_sha256"] = _digest(2)
    second_stable = rows[1]["stable_graph"]
    third_stable = rows[2]["stable_graph"]
    assert isinstance(second_stable, dict)
    assert isinstance(third_stable, dict)
    second_stable["after_sha256"] = _digest(502)
    third_stable["before_sha256"] = _digest(502)
    third_stable["after_sha256"] = _digest(502)

    second_events = rows[1]["events"]
    assert isinstance(second_events, dict)
    second_events["count"] = 0
    second_events["per_type"] = []
    _set_observable(rows[1], 3, before_value=0.0, after_value=1e-12)
    _set_observable(rows[2], 3, before_value=1e-12, after_value=2e-12)
    _set_observable(rows[2], 4, before_value=2010.0, after_value=2011.0)
    third_material = rows[2]["material_rows"]
    assert isinstance(third_material, dict)
    third_material["count"] = 4

    diagnostics = sim_report._diagnostics(rows)

    assert diagnostics["schema"] == "babylon.simulation.diagnostics.v1"
    assert diagnostics["scope"] == SCOPE
    assert diagnostics["ticks_reported"] == 3
    assert diagnostics["administrative_graph"] == {
        "changed_ticks": 1,
        "distinct_after_hashes": 1,
        "first_change_tick": 1,
        "last_change_tick": 1,
        "longest_unchanged_run": {"end_tick": 3, "start_tick": 2, "ticks": 2},
        "unchanged_ticks": 2,
    }
    assert diagnostics["stable_graph"] == {
        "changed_ticks": 1,
        "distinct_hashes": 2,
        "final_sha256": _digest(502),
        "first_change_tick": 1,
        "initial_sha256": _digest(501),
        "last_change_tick": 1,
        "longest_unchanged_run": {"end_tick": 3, "start_tick": 2, "ticks": 2},
        "unchanged_ticks": 2,
    }
    assert diagnostics["world"]["changed_ticks"] == 3  # type: ignore[index]
    assert diagnostics["commits"] == {
        "committed": 3,
        "reconciled_after_ambiguous_commit": 0,
    }
    rules = diagnostics["rules"]
    assert isinstance(rules, list)
    assert rules[0] == {
        "considered": 6,
        "fired": 3,
        "fired_on_unchanged_administrative_graph_ticks": 2,
        "first_fired_tick": 1,
        "last_fired_tick": 3,
        "maximum_firing_interval_ticks": 1,
        "minimum_firing_interval_ticks": 1,
        "firing_interval_gcd_ticks": 1,
        "rule_id": "phase/example-a",
        "ticks_considered": 3,
        "ticks_fired": 3,
    }
    assert rules[1]["fired"] == 0
    assert rules[1]["first_fired_tick"] is None
    assert diagnostics["events"] == {
        "active_ticks": 2,
        "change_count": 2,
        "first_change_tick": 2,
        "first_event_tick": 1,
        "last_change_tick": 3,
        "last_event_tick": 3,
        "maximum_per_tick": 2,
        "minimum_per_tick": 0,
        "per_type": [
            {
                "active_ticks": 2,
                "change_count": 2,
                "count": 2,
                "event_type": "EventType/ALPHA",
                "first_change_tick": 2,
                "first_tick": 1,
                "last_change_tick": 3,
                "last_tick": 3,
                "maximum_per_tick": 1,
                "minimum_per_tick": 0,
                "unique_counts": 2,
            },
            {
                "active_ticks": 2,
                "change_count": 2,
                "count": 2,
                "event_type": "EventType/OMEGA",
                "first_change_tick": 2,
                "first_tick": 1,
                "last_change_tick": 3,
                "last_tick": 3,
                "maximum_per_tick": 1,
                "minimum_per_tick": 0,
                "unique_counts": 2,
            },
        ],
        "total": 4,
        "unique_counts": 2,
    }
    assert diagnostics["material_rows"] == {
        "change_count": 1,
        "final_per_tick": 4,
        "first_change_tick": 3,
        "initial_per_tick": 3,
        "last_change_tick": 3,
        "maximum_per_tick": 4,
        "minimum_per_tick": 3,
        "total": 10,
        "unique_counts": 2,
    }
    observables = diagnostics["observables"]
    assert isinstance(observables, list)
    assert observables[0]["role"] == "configured_input"
    assert observables[0]["unique_values"] == 1
    assert observables[2]["change_count"] == 0
    assert observables[2]["first_change_tick"] is None
    assert observables[3]["minimum"] == 0.0
    assert observables[3]["maximum"] == 2e-12
    assert observables[3]["final"] == 2e-12
    assert observables[3]["unique_values"] == 3
    assert observables[3]["change_count"] == 2
    assert observables[3]["first_change_tick"] == 2
    assert observables[3]["last_change_tick"] == 3
    assert observables[4]["change_count"] == 1
    notice_codes = [notice["code"] for notice in diagnostics["notices"]]
    assert notice_codes[:3] == [
        "scope.fixed_parameters",
        "scope.no_stochastic_draws",
        "scope.no_dynamic_h3_updates",
    ]
    assert "rule.never_fired" in notice_codes
    assert "rule.fired_on_unchanged_administrative_graph" in notice_codes
    assert "observable.dynamic_flat" in notice_codes
    assert "observable.dynamic_near_flat" in notice_codes


def test_tick_one_stable_and_observable_movement_is_not_reported_flat() -> None:
    row = _valid_row(1)
    _set_observable(row, 2, before_value=0.0, after_value=0.25)

    diagnostics = sim_report._diagnostics([row])

    assert diagnostics["stable_graph"]["changed_ticks"] == 1  # type: ignore[index]
    observable = diagnostics["observables"][2]  # type: ignore[index]
    assert observable["change_count"] == 1
    assert observable["first_change_tick"] == 1
    flat_subjects = {
        notice["subject"]
        for notice in diagnostics["notices"]
        if notice["code"] in {"stable_graph.flat", "observable.dynamic_flat"}
    }
    assert observable["name"] not in flat_subjects


def test_dynamic_observable_that_changes_once_then_stalls_is_reported_plateaued() -> None:
    rows = [_valid_row(tick) for tick in range(1, 105)]
    _set_observable(rows[0], 2, before_value=0.0, after_value=0.25)
    for row in rows[1:]:
        _set_observable(row, 2, before_value=0.25, after_value=0.25)

    diagnostics = sim_report._diagnostics(rows)

    observable = diagnostics["observables"][2]  # type: ignore[index]
    assert observable["last_change_tick"] == 1
    assert {(notice["code"], notice["subject"]) for notice in diagnostics["notices"]} >= {
        ("observable.dynamic_plateau", observable["name"])
    }
    assert all(notice["code"] != "stable_graph.flat" for notice in diagnostics["notices"])


@pytest.mark.parametrize("state_name", ["graph", "stable_graph", "world"])
def test_durable_rows_reject_hash_chain_discontinuities(
    tmp_path: Path,
    state_name: str,
) -> None:
    rows = [_valid_row(1), _valid_row(2)]
    state = rows[1][state_name]
    assert isinstance(state, dict)
    state["before_sha256"] = _digest(999)
    report = tmp_path / "ticks.jsonl"
    _write_jsonl(report, rows)

    with pytest.raises(
        sim_report.JsonlValidationError,
        match=(
            rf"line 2 {state_name}\.before_sha256 does not equal "
            rf"the previous durable row's {state_name}\.after_sha256"
        ),
    ):
        sim_report._validate_jsonl(report, expected_rows=2)


@pytest.mark.parametrize(
    ("row_limit", "message"),
    [
        ({"expected_rows": 1}, "exceeds expected row count 1"),
        ({"maximum_rows": 1}, "exceeds maximum row count 1"),
    ],
)
def test_jsonl_aborts_as_soon_as_a_row_count_bound_is_exceeded(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    row_limit: dict[str, int],
    message: str,
) -> None:
    report = tmp_path / "ticks.jsonl"
    report.write_bytes(b"placeholder\n")
    lines = [(json.dumps(_valid_row(tick)) + "\n").encode("utf-8") for tick in (1, 2, 3)]
    reader = _TrackingReader(lines)
    monkeypatch.setattr(Path, "open", lambda *_args, **_kwargs: reader)

    with pytest.raises(sim_report.JsonlValidationError, match=message):
        sim_report._validate_jsonl(report, **row_limit)

    assert reader.readline_sizes == [sim_report.MAX_JSONL_ROW_BYTES + 1] * 2
    assert reader.iteration_used is False
    assert len(reader.lines) == 1


def test_jsonl_caps_each_read_before_rejecting_an_oversize_row(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    report = tmp_path / "ticks.jsonl"
    report.write_bytes(b"placeholder\n")
    monkeypatch.setattr(sim_report, "MAX_JSONL_ROW_BYTES", 32)
    reader = _TrackingReader([b"x" * 33])
    monkeypatch.setattr(Path, "open", lambda *_args, **_kwargs: reader)

    with pytest.raises(
        sim_report.JsonlValidationError,
        match="line 1 exceeds the 32-byte bound",
    ):
        sim_report._validate_jsonl(report)

    assert reader.readline_sizes == [33]
    assert reader.iteration_used is False


def test_successful_runtime_rejects_reused_nonfresh_tick_sequence(tmp_path: Path) -> None:
    runtime = _configured_runtime(
        tmp_path / "babylon-runtime",
        rows=[_valid_row(41), _valid_row(42)],
    )

    result = sim_report.run_report(runtime=runtime, ticks=2, output_root=tmp_path / "out")

    assert result.exit_code == sim_report.VALIDATION_EXIT_CODE
    assert result.summary["status"] == "validation_failed"
    assert "must start at resolve_tick 1" in result.summary["error"]


def test_successful_runtime_must_report_exactly_the_requested_row_count(
    tmp_path: Path,
) -> None:
    runtime = _configured_runtime(tmp_path / "babylon-runtime", rows=[_valid_row(1)])

    result = sim_report.run_report(runtime=runtime, ticks=2, output_root=tmp_path / "out")

    assert result.exit_code == sim_report.VALIDATION_EXIT_CODE
    assert result.summary["status"] == "validation_failed"
    assert "reports 1 rows; expected exactly 2" in result.summary["error"]


def test_nonzero_runtime_preserves_valid_partial_evidence_and_actual_status(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    rows = [_valid_row(1), _valid_row(2)]
    runtime = _configured_runtime(
        tmp_path / "babylon-runtime",
        rows=rows,
        exit_code=17,
        stderr="specific runtime failure\n",
    )
    monkeypatch.setattr(
        sim_report,
        "_probe_postgres_snapshot",
        lambda _environment: sim_report._PostgresProbe(None, "dsn_unavailable"),
    )

    result = sim_report.run_report(runtime=runtime, ticks=5, output_root=tmp_path / "out")

    assert result.exit_code == 17
    assert result.summary["status"] == "runtime_failed"
    assert result.summary["runtime_exit_code"] == 17
    assert result.summary["ticks_reported"] == 2
    assert result.summary["first_resolve_tick"] == 1
    assert result.summary["last_resolve_tick"] == 2
    assert result.summary["totals"] == {
        "audit_receipts": 4,
        "events": 4,
        "material_rows": 6,
        "rules_considered": 6,
        "rules_fired": 2,
    }
    assert (result.artifact_dir / "ticks.csv").is_file()
    assert (result.artifact_dir / "diagnostics.json").is_file()
    assert (result.artifact_dir / "resources.json").is_file()
    assert (result.artifact_dir / "stderr.txt").read_text() == "specific runtime failure\n"


def test_nonzero_runtime_records_invalid_partial_report_without_hiding_status(
    tmp_path: Path,
) -> None:
    runtime = _configured_runtime(
        tmp_path / "babylon-runtime",
        rows=[_valid_row(1), _valid_row(3)],
        exit_code=19,
    )

    result = sim_report.run_report(runtime=runtime, ticks=5, output_root=tmp_path / "out")

    assert result.exit_code == 19
    assert result.summary["status"] == "runtime_failed"
    assert result.summary["runtime_exit_code"] == 19
    assert result.summary["ticks_reported"] == 0
    assert "contiguous resolve_tick 2" in result.summary["report_validation_error"]
    assert "runtime exited with status 19" in result.error
    assert "report validation failed" in result.error
    assert not (result.artifact_dir / "ticks.csv").exists()


def test_nonzero_runtime_rejects_partial_evidence_from_a_reused_campaign(
    tmp_path: Path,
) -> None:
    runtime = _configured_runtime(
        tmp_path / "babylon-runtime",
        rows=[_valid_row(41), _valid_row(42)],
        exit_code=29,
    )

    result = sim_report.run_report(runtime=runtime, ticks=5, output_root=tmp_path / "out")

    assert result.exit_code == 29
    assert result.summary["status"] == "runtime_failed"
    assert result.summary["ticks_reported"] == 0
    assert "must start at resolve_tick 1" in result.summary["report_validation_error"]
    assert not (result.artifact_dir / "ticks.csv").exists()


def test_runtime_failure_without_rows_preserves_logs_and_actual_status(tmp_path: Path) -> None:
    runtime = _configured_runtime(
        tmp_path / "babylon-runtime",
        rows=None,
        exit_code=23,
        stdout="before failure\n",
        stderr="specific runtime failure\n",
    )

    result = sim_report.run_report(runtime=runtime, ticks=1, output_root=tmp_path / "out")

    assert result.exit_code == 23
    assert (result.artifact_dir / "stdout.txt").read_text() == "before failure\n"
    assert (result.artifact_dir / "stderr.txt").read_text() == "specific runtime failure\n"
    assert result.summary["status"] == "runtime_failed"
    assert result.summary["runtime_exit_code"] == 23
    assert result.summary["ticks_reported"] == 0


def test_noisy_runtime_is_killed_at_bounded_capture_without_secret_leak(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    marker = tmp_path / "survived.txt"
    secret = "postgresql://user:noisy-secret@example.invalid/babylon"
    monkeypatch.setattr(sim_report, "MAX_STDOUT_BYTES", 128)
    monkeypatch.setenv("DATABASE_URL", secret)
    runtime = _configured_runtime(
        tmp_path / "babylon-runtime",
        rows=[_valid_row(1)],
        stdout=secret + ("x" * 4096),
        sleep_seconds=5.0,
        marker_path=marker,
    )

    result = sim_report.run_report(runtime=runtime, ticks=1, output_root=tmp_path / "out")

    assert result.exit_code == sim_report.VALIDATION_EXIT_CODE
    assert result.summary["status"] == "runtime_output_limit_exceeded"
    assert "stdout exceeded the 128-byte capture bound" in result.summary["error"]
    assert result.summary["ticks_reported"] == 1
    assert result.summary["first_resolve_tick"] == 1
    assert (result.artifact_dir / "ticks.csv").is_file()
    stdout = (result.artifact_dir / "stdout.txt").read_bytes()
    assert len(stdout) <= 128
    assert secret.encode() not in stdout
    assert not marker.exists()


def test_hung_runtime_is_killed_at_explicit_timeout_without_secret_leak(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    marker = tmp_path / "survived.txt"
    secret = "host=db.invalid password=timeout-secret"
    monkeypatch.setenv("BABYLON_RUNTIME_DSN", secret)
    monkeypatch.setattr(
        sim_report,
        "_probe_postgres_snapshot",
        lambda _environment: sim_report._PostgresProbe(None, "probe_timed_out"),
    )
    runtime = _configured_runtime(
        tmp_path / "babylon-runtime",
        rows=[_valid_row(1)],
        stderr_env=["BABYLON_RUNTIME_DSN"],
        sleep_seconds=5.0,
        marker_path=marker,
    )

    result = sim_report.run_report(
        runtime=runtime,
        ticks=1,
        output_root=tmp_path / "out",
        timeout_seconds=0.1,
    )

    assert result.exit_code == sim_report.VALIDATION_EXIT_CODE
    assert result.summary["status"] == "runtime_timed_out"
    assert "exceeded the 0.1-second timeout" in result.summary["error"]
    assert result.summary["ticks_reported"] == 1
    assert result.summary["first_resolve_tick"] == 1
    assert (result.artifact_dir / "ticks.csv").is_file()
    assert (result.artifact_dir / "diagnostics.json").is_file()
    resources = _load_json(result.artifact_dir / "resources.json")
    assert resources["runtime"]["status"] == "observed"  # type: ignore[index]
    assert resources["runtime"]["wall_time_ns"] > 0  # type: ignore[index]
    stderr = (result.artifact_dir / "stderr.txt").read_text(encoding="utf-8")
    assert "timeout-secret" not in stderr
    assert "[REDACTED]" in stderr
    assert not marker.exists()


def test_absent_campaign_id_generates_fresh_child_only_uuid_and_surfaces_it(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    capture_path = tmp_path / "campaigns.txt"
    runtime = _configured_runtime(
        tmp_path / "babylon-runtime",
        rows=_complete_rows(1),
        capture_campaign=True,
        print_campaign=True,
    )
    monkeypatch.delenv("BABYLON_CAMPAIGN_ID", raising=False)
    monkeypatch.setenv("CAMPAIGN_CAPTURE_PATH", str(capture_path))

    results = [
        sim_report.run_report(runtime=runtime, ticks=1, output_root=tmp_path / "out")
        for _ in range(2)
    ]

    campaign_ids = capture_path.read_text(encoding="utf-8").splitlines()
    assert len(set(campaign_ids)) == 2
    for result, campaign_id in zip(results, campaign_ids, strict=True):
        parsed = uuid.UUID(campaign_id)
        assert parsed.version == 4
        assert str(parsed) == campaign_id
        assert result.summary["campaign_id"] == campaign_id
        assert (result.artifact_dir / "stdout.txt").read_text() == campaign_id + "\n"
        assert f"campaign: {campaign_id}" in (result.artifact_dir / "summary.txt").read_text()
    assert "BABYLON_CAMPAIGN_ID" not in os.environ


def test_inherited_campaign_id_is_replaced_before_the_runtime_starts(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    capture_path = tmp_path / "campaigns.txt"
    runtime = _configured_runtime(
        tmp_path / "babylon-runtime",
        rows=_complete_rows(1),
        capture_campaign=True,
    )
    monkeypatch.setenv("BABYLON_CAMPAIGN_ID", CAMPAIGN_ID)
    monkeypatch.setenv("CAMPAIGN_CAPTURE_PATH", str(capture_path))

    result = sim_report.run_report(runtime=runtime, ticks=1, output_root=tmp_path / "out")

    assert result.exit_code == 0
    child_campaign_id = capture_path.read_text(encoding="utf-8").strip()
    assert child_campaign_id != CAMPAIGN_ID
    assert uuid.UUID(child_campaign_id).version == 4
    assert result.summary["campaign_id"] == child_campaign_id


def test_help_explains_tick_count_and_fresh_campaign_semantics() -> None:
    help_text = sim_report._parser().format_help()

    assert "count of four-week periods" in help_text
    assert "BABYLON_CAMPAIGN_ID" in help_text
    assert "fresh campaign" in " ".join(help_text.split())
    assert "inherited value is ignored" in help_text
    assert "maximum 10000" in help_text
    assert "--database-scope {shared,exclusive}" in help_text
    assert "shared" in help_text


def test_tick_count_bound_accepts_10000_and_rejects_10001() -> None:
    assert sim_report._positive_ticks("10000") == 10_000
    with pytest.raises(argparse.ArgumentTypeError, match="between 1 and 10000"):
        sim_report._positive_ticks("10001")


def test_runtime_path_must_be_an_executable_file(tmp_path: Path) -> None:
    runtime = tmp_path / "babylon-runtime"
    runtime.write_text("not executable\n", encoding="utf-8")

    with pytest.raises(sim_report.ReportError, match="not executable"):
        sim_report.run_report(runtime=runtime, ticks=1, output_root=tmp_path / "out")
