"""Observer launcher capabilities, direct pipes, and campaign continuity."""

from __future__ import annotations

import os
from pathlib import Path
from typing import Any
from uuid import UUID

import pytest
from tools import run_observer_session as launcher

CAMPAIGN = UUID("81b979ee-a9c1-48fd-8835-06cbfe594675")


def test_child_environments_do_not_inherit_writer_or_pg_authority() -> None:
    environment = {
        "PATH": "/usr/bin",
        "DISPLAY": ":0",
        "BABYLON_RUNTIME_DSN": "writer-secret",
        "PGPASSWORD": "other-secret",
        "PGSERVICEFILE": "/private/service",
        "BABYLON_OBSERVER_DSN": "stale-observer",
        "BABYLON_READER_DSN": "stale-reader",
        "RUST_LOG": "warn",
    }
    credentials = launcher.ReaderCredentials("observer-capability", "known-capability")
    runtime, client = launcher.child_environments(environment, CAMPAIGN, credentials)
    assert runtime["BABYLON_RUNTIME_DSN"] == "writer-secret"
    assert "BABYLON_RUNTIME_DSN" not in client
    assert all(not key.upper().startswith("PG") for key in runtime | client)
    assert "BABYLON_READER_DSN" not in runtime and "BABYLON_OBSERVER_DSN" not in runtime
    assert client["BABYLON_OBSERVER_DSN"] == "observer-capability"
    assert client["BABYLON_READER_DSN"] == "known-capability"
    assert client["BABYLON_SESSION_STDIO"] == "1"
    assert runtime["BABYLON_CAMPAIGN_ID"] == client["BABYLON_CAMPAIGN_ID"] == str(CAMPAIGN)
    assert client["DISPLAY"] == ":0"
    assert client["RUST_LOG"] == "warn,session=debug,babylon_client=debug"
    assert runtime["RUST_LOG"] == "warn"
    assert environment["RUST_LOG"] == "warn"


@pytest.mark.parametrize(
    ("ambient", "effective"),
    [
        (None, "warn,session=debug,babylon_client=debug"),
        ("", "warn,session=debug,babylon_client=debug"),
        ("   ", "warn,session=debug,babylon_client=debug"),
        (
            "warn,wgpu=error,session=off,babylon_client=error,babylon_kernel=trace",
            "warn,wgpu=error,session=off,babylon_client=error,babylon_kernel=trace,"
            "session=debug,babylon_client=debug",
        ),
    ],
)
def test_observer_capture_targets_override_exact_ambient_filters_only_for_client(
    ambient: str | None, effective: str
) -> None:
    environment = {} if ambient is None else {"RUST_LOG": ambient}
    runtime, client = launcher.child_environments(
        environment, CAMPAIGN, launcher.ReaderCredentials("observer", "known")
    )
    assert client["RUST_LOG"] == effective
    assert runtime.get("RUST_LOG") == ambient
    assert environment.get("RUST_LOG") == ambient


def test_campaign_selection_and_new_preserve_existing_preference(tmp_path: Path) -> None:
    state = tmp_path / "campaign"
    first = launcher.select_campaign({}, state_file=state)
    launcher.save_campaign(state, first)
    assert launcher.select_campaign({}, state_file=state) == first
    assert launcher.select_campaign({}, state_file=state, explicit=str(CAMPAIGN)) == CAMPAIGN
    assert launcher.select_campaign({}, state_file=state, new=True) != first
    assert state.read_text().strip() == str(first)


def test_log_capture_notice_is_bounded_and_never_echoes_environment_secrets(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    monkeypatch.setattr(launcher, "run_pair", lambda *_args, **_kwargs: 0)
    assert (
        launcher.run_campaigns(
            tmp_path / "runtime",
            tmp_path / "client",
            tmp_path,
            CAMPAIGN,
            tmp_path / "campaign",
            {
                "BABYLON_RUNTIME_DSN": "writer-secret",
                "RUST_LOG": 'warn,engine[span{key="private-filter"}]=trace',
            },
            launcher.ReaderCredentials("observer-secret", "known-secret"),
        )
        == 0
    )
    captured = capsys.readouterr()
    assert captured.out == ""
    assert captured.err == "Observer log targets enabled: session=debug,babylon_client=debug\n"


@pytest.mark.parametrize("value", ["not-a-uuid", "", "0" * 32, str(CAMPAIGN).upper()])
def test_invalid_explicit_campaign_refuses(value: str, tmp_path: Path) -> None:
    with pytest.raises(launcher.ObserverLaunchError, match="campaign"):
        launcher.select_campaign({}, state_file=tmp_path / "campaign", explicit=value)


def test_corrupt_saved_campaign_refuses_without_replacing_it(tmp_path: Path) -> None:
    state = tmp_path / "campaign"
    state.write_text("damaged")
    with pytest.raises(launcher.ObserverLaunchError, match="campaign"):
        launcher.select_campaign({}, state_file=state)
    assert state.read_text() == "damaged"


def test_two_anonymous_pipes_connect_children_without_parent_forwarding(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    children: list[dict[str, Any]] = []
    fds: list[int] = []
    original_pipe = os.pipe

    def pipe() -> tuple[int, int]:
        pair = original_pipe()
        fds.extend(pair)
        return pair

    class Child:
        def __init__(self, args: list[str], **kwargs: Any) -> None:
            children.append({"args": args, **kwargs})
            self.returncode = 0

        def wait(self, timeout: float | None = None) -> int:
            return self.returncode

        def poll(self) -> int:
            return self.returncode

    monkeypatch.setattr(launcher.os, "pipe", pipe)
    monkeypatch.setattr(launcher.subprocess, "Popen", Child)
    code = launcher.run_pair(
        tmp_path / "runtime",
        tmp_path / "client",
        tmp_path,
        {"BABYLON_RUNTIME_DSN": "writer", "BABYLON_CAMPAIGN_ID": str(CAMPAIGN)},
        {"BABYLON_SESSION_STDIO": "1", "BABYLON_CAMPAIGN_ID": str(CAMPAIGN)},
    )
    assert code == 0
    assert len(fds) == 4 and len(children) == 2
    runtime, client = children
    assert runtime["args"] == [str(tmp_path / "runtime"), "session", "--stdio"]
    assert runtime["stdin"] == fds[0] and client["stdout"] == fds[1]
    assert client["stdin"] == fds[2] and runtime["stdout"] == fds[3]
    assert runtime["stderr"] is None and client["stderr"] is None
    assert runtime["close_fds"] and client["close_fds"]
    for fd in fds:
        with pytest.raises(OSError):
            os.fstat(fd)


class ShutdownChild:
    def __init__(self, *, runtime: bool, behavior: str = "graceful") -> None:
        self.runtime = runtime
        self.behavior = behavior
        self.returncode: int | None = None
        self.calls: list[tuple[str, float | None]] = []
        self.signal: str | None = None

    def poll(self) -> int | None:
        return self.returncode

    def wait(self, timeout: float | None = None) -> int:
        self.calls.append(("wait", timeout))
        if not self.runtime:
            self.returncode = 0
        else:
            assert timeout is not None, "runtime shutdown must have a deadline"
            if self.behavior == "graceful":
                self.returncode = 0
            elif self.signal == "terminate" and self.behavior == "terminate":
                self.returncode = -15
            elif self.signal == "kill" and self.behavior != "unreapable":
                self.returncode = -9
            else:
                raise launcher.subprocess.TimeoutExpired("exact-observer-child", timeout)
        return self.returncode

    def terminate(self) -> None:
        self.calls.append(("terminate", None))
        self.signal = "terminate"

    def kill(self) -> None:
        self.calls.append(("kill", None))
        self.signal = "kill"


@pytest.mark.parametrize("behavior", ["graceful", "terminate", "kill"])
def test_runtime_shutdown_allows_commit_grace_before_bounded_exact_child_stop(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path, behavior: str
) -> None:
    runtime = ShutdownChild(runtime=True, behavior=behavior)
    client = ShutdownChild(runtime=False)
    children = iter([runtime, client])
    monkeypatch.setattr(launcher.subprocess, "Popen", lambda *_args, **_kwargs: next(children))
    arguments = (tmp_path / "runtime", tmp_path / "client", tmp_path, {}, {})
    if behavior == "graceful":
        assert launcher.run_pair(*arguments) == 0
    else:
        with pytest.raises(launcher.ObserverLaunchError, match="runtime shutdown deadline"):
            launcher.run_pair(*arguments)
    # A normal game session has no time limit; shutdown starts after client EOF.
    assert client.calls == [("wait", None)]
    expected: list[tuple[str, float | None]] = [("wait", 150)]
    if behavior != "graceful":
        expected += [("terminate", None), ("wait", 10)]
    if behavior == "kill":
        expected += [("kill", None), ("wait", 10)]
    assert runtime.calls == expected


def test_interrupted_startup_closes_pipes_and_preserves_runtime_grace(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    runtime = ShutdownChild(runtime=True)
    launches = 0
    descriptors: list[int] = []
    original_pipe = os.pipe

    def pipe() -> tuple[int, int]:
        pair = original_pipe()
        descriptors.extend(pair)
        return pair

    def child(*_args: Any, **_kwargs: Any) -> ShutdownChild:
        nonlocal launches
        launches += 1
        if launches == 2:
            raise OSError("client unavailable")
        return runtime

    monkeypatch.setattr(launcher.subprocess, "Popen", child)
    monkeypatch.setattr(launcher.os, "pipe", pipe)
    with pytest.raises(launcher.ObserverLaunchError, match="cannot start observer processes"):
        launcher.run_pair(tmp_path / "runtime", tmp_path / "client", tmp_path, {}, {})
    assert runtime.calls == [("wait", 150)]
    assert len(descriptors) == 4
    for descriptor in descriptors:
        with pytest.raises(OSError):
            os.fstat(descriptor)


def test_child_shutdown_has_a_deadline_even_after_kill() -> None:
    child = ShutdownChild(runtime=True, behavior="unreapable")
    with pytest.raises(launcher.ObserverLaunchError, match="child did not exit"):
        launcher._stop(child)  # type: ignore[arg-type]
    assert child.calls == [
        ("terminate", None),
        ("wait", 10),
        ("kill", None),
        ("wait", 10),
    ]


def test_in_game_new_campaign_restarts_with_fresh_identity(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    campaigns: list[str] = []

    def pair(
        runtime: Path,
        client: Path,
        root: Path,
        writer_env: dict[str, str],
        reader_env: dict[str, str],
        *,
        preset: str | None = None,
    ) -> int:
        assert writer_env["BABYLON_CAMPAIGN_ID"] == reader_env["BABYLON_CAMPAIGN_ID"]
        campaigns.append(writer_env["BABYLON_CAMPAIGN_ID"])
        return 20 if len(campaigns) == 1 else 0

    monkeypatch.setattr(launcher, "run_pair", pair)
    state = tmp_path / "campaign"
    result = launcher.run_campaigns(
        tmp_path / "runtime",
        tmp_path / "client",
        tmp_path,
        CAMPAIGN,
        state,
        {"BABYLON_RUNTIME_DSN": "writer"},
        launcher.ReaderCredentials("observer", "known"),
    )
    assert result == 0
    assert campaigns[0] == str(CAMPAIGN)
    assert len(set(campaigns)) == 2
    assert state.read_text().strip() == campaigns[1]


def test_deliberate_reopen_preserves_campaign_and_failure_does_not_retry(
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
) -> None:
    campaigns: list[str] = []

    def pair(
        runtime: Path,
        client: Path,
        root: Path,
        writer_env: dict[str, str],
        reader_env: dict[str, str],
        *,
        preset: str | None = None,
    ) -> int:
        campaigns.append(writer_env["BABYLON_CAMPAIGN_ID"])
        return 21 if len(campaigns) == 1 else 1

    monkeypatch.setattr(launcher, "run_pair", pair)
    result = launcher.run_campaigns(
        tmp_path / "runtime",
        tmp_path / "client",
        tmp_path,
        CAMPAIGN,
        tmp_path / "campaign",
        {"BABYLON_RUNTIME_DSN": "writer"},
        launcher.ReaderCredentials("observer", "known"),
    )
    assert result == 1
    assert campaigns == [str(CAMPAIGN), str(CAMPAIGN)]


@pytest.mark.parametrize("fresh", [True, False])
def test_preparation_orders_bootstrap_schema_and_restricted_logins_without_ticks(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path, fresh: bool
) -> None:
    calls: list[str] = []

    def run(args: list[str], root: Path, environment: dict[str, str], label: str) -> None:
        calls.append(args[-1])
        if args[0] == "mise":
            assert "BABYLON_RUNTIME_DSN" not in environment
        else:
            assert environment["BABYLON_RUNTIME_DSN"] == launcher.DEFAULT_RUNTIME_DSN
        assert all(not key.upper().startswith("PG") for key in environment)

    def provision(dsn: str) -> launcher.ReaderCredentials:
        calls.append("provision")
        assert dsn == launcher.DEFAULT_RUNTIME_DSN
        return launcher.ReaderCredentials("observer", "known")

    monkeypatch.setattr(launcher, "_run", run)
    monkeypatch.setattr(launcher, "bootstrap_required", lambda _: fresh)
    monkeypatch.setattr(launcher, "database_reachable", lambda _: True, raising=False)
    monkeypatch.setattr(launcher, "provision_readers", provision)
    runtime, client, credentials = launcher.prepare(
        tmp_path, {"PGOPTIONS": "unsafe"}, no_build=True
    )
    assert calls == (
        ["bootstrap", "observer-schema", "provision"] if fresh else ["observer-schema", "provision"]
    )
    assert runtime == tmp_path / "rust/target/debug/babylon-runtime"
    assert client == tmp_path / "rust/target/debug/babylon-client"
    assert credentials.known_dsn == "known"


@pytest.mark.parametrize(
    "dsn",
    [
        "host=localhost port=5433 dbname=babylon_test user=test password=test",
        "host=198.51.100.1 port=5433 dbname=babylon_test user=test password=test",
        "host=127.0.0.1 port=5433 dbname=babylon_test user=test password=test options=unsafe",
        "service=writer",
    ],
)
def test_unapproved_connection_targets_refuse_before_database_access(dsn: str) -> None:
    with pytest.raises(launcher.ObserverLaunchError, match="explicit local"):
        launcher.provision_readers(dsn)


def test_provisioning_grants_only_distinct_reader_memberships(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    statements: list[str] = []

    class Connection:
        def __enter__(self) -> Connection:
            return self

        def __exit__(self, *args: Any) -> None:
            return None

        def execute(self, query: Any, parameters: Any = None) -> Connection:
            statements.append(query if isinstance(query, str) else query.as_string())
            return self

        def fetchone(self) -> None:
            return None

    monkeypatch.setattr(launcher.psycopg, "connect", lambda *_args, **_kwargs: Connection())
    credentials = launcher.provision_readers(launcher.DEFAULT_RUNTIME_DSN)
    assert 'GRANT "babylon_observer" TO "babylon_observer_game"' in statements
    assert 'GRANT "babylon_reader" TO "babylon_preview_game"' in statements
    assert not any("babylon_state" in query or "INSERT" in query for query in statements)
    for statement in statements:
        if statement.startswith("ALTER ROLE"):
            assert "NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS" in statement
    observer = launcher.conninfo_to_dict(credentials.observer_dsn)
    known = launcher.conninfo_to_dict(credentials.known_dsn)
    assert observer["user"] == "babylon_observer_game"
    assert known["user"] == "babylon_preview_game"
    assert observer["dbname"] == known["dbname"] == "babylon_test"


@pytest.mark.parametrize(
    ("exists", "active", "required"),
    [(False, False, True), (True, False, True), (True, True, False)],
)
def test_bootstrap_probe_is_read_only_and_requires_active_marker(
    monkeypatch: pytest.MonkeyPatch,
    exists: bool,
    active: bool,
    required: bool,
) -> None:
    statements: list[str] = []

    class Connection:
        def __enter__(self) -> Connection:
            return self

        def __exit__(self, *args: Any) -> None:
            return None

        def execute(self, query: str) -> Connection:
            statements.append(query)
            return self

        def fetchone(self) -> tuple[Any, ...] | None:
            if "to_regclass" in statements[-1]:
                return ("authority" if exists else None,)
            assert "ordinal = 2 AND state_tag = 2 AND activation_epoch = 11" in statements[-1]
            return (1,) if active else None

    monkeypatch.setattr(launcher.psycopg, "connect", lambda *_args, **_kwargs: Connection())
    assert launcher.bootstrap_required(launcher.DEFAULT_RUNTIME_DSN) is required
    assert statements[0] == "SET TRANSACTION READ ONLY"
    assert all(statement.startswith("SELECT") for statement in statements[1:])


def test_delayed_new_world_and_resume_preserve_durable_preset(
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
) -> None:
    calls: list[tuple[str, str | None]] = []

    def pair(
        runtime: Path,
        client: Path,
        root: Path,
        writer_env: dict[str, str],
        reader_env: dict[str, str],
        *,
        preset: str | None = None,
    ) -> int:
        calls.append((writer_env["BABYLON_CAMPAIGN_ID"], preset))
        return [22, 21, 0][len(calls) - 1]

    monkeypatch.setattr(launcher, "run_pair", pair)
    assert (
        launcher.run_campaigns(
            tmp_path / "runtime",
            tmp_path / "client",
            tmp_path,
            CAMPAIGN,
            tmp_path / "campaign",
            {},
            launcher.ReaderCredentials("observer", "known"),
        )
        == 0
    )
    assert calls[0] == (str(CAMPAIGN), None)
    assert calls[1][0] != str(CAMPAIGN)
    assert calls[1][1] == "delayed"
    assert calls[2] == (calls[1][0], None)


def test_catalog_selection_reloads_exact_saved_uuid_without_environment_override(
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
) -> None:
    selected = UUID("fc7d28a0-a29a-49ea-bf3b-ef07ee163cd4")
    state = tmp_path / "campaign"
    campaigns: list[str] = []

    def pair(
        runtime: Path,
        client: Path,
        root: Path,
        writer_env: dict[str, str],
        reader_env: dict[str, str],
        *,
        preset: str | None = None,
    ) -> int:
        campaigns.append(writer_env["BABYLON_CAMPAIGN_ID"])
        if len(campaigns) == 1:
            launcher.save_campaign(state, selected)
            return 23
        assert preset is None
        return 0

    monkeypatch.setattr(launcher, "run_pair", pair)
    assert (
        launcher.run_campaigns(
            tmp_path / "runtime",
            tmp_path / "client",
            tmp_path,
            CAMPAIGN,
            state,
            {"BABYLON_CAMPAIGN_ID": str(CAMPAIGN)},
            launcher.ReaderCredentials("observer", "known"),
        )
        == 0
    )
    assert campaigns == [str(CAMPAIGN), str(selected)]


@pytest.mark.parametrize("available_after_start", [True, False])
def test_only_unavailable_default_target_starts_compose_and_rechecks(
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
    available_after_start: bool,
) -> None:
    probes = iter([False, available_after_start])
    calls: list[str] = []
    monkeypatch.setattr(launcher, "database_reachable", lambda _: next(probes))
    monkeypatch.setattr(launcher, "_run", lambda args, *_: calls.append(args[-1]))
    monkeypatch.setattr(launcher, "bootstrap_required", lambda _: False)
    monkeypatch.setattr(
        launcher, "provision_readers", lambda _: launcher.ReaderCredentials("observer", "known")
    )
    if available_after_start:
        launcher.prepare(tmp_path, {}, no_build=True)
        assert calls == ["db:up", "observer-schema"]
    else:
        with pytest.raises(launcher.ObserverLaunchError, match="still unavailable"):
            launcher.prepare(tmp_path, {}, no_build=True)
        assert calls == ["db:up"]


def test_unavailable_custom_target_does_not_start_another_database(
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
) -> None:
    calls: list[str] = []
    monkeypatch.setattr(launcher, "database_reachable", lambda _: False)
    monkeypatch.setattr(launcher, "_run", lambda args, *_: calls.append(args[-1]))
    with pytest.raises(
        launcher.ObserverLaunchError, match="requested local database is unavailable"
    ):
        launcher.prepare(
            tmp_path,
            {
                "BABYLON_RUNTIME_DSN": "host=127.0.0.1 port=5433 dbname=observer_review user=test password=test"
            },
            no_build=True,
        )
    assert calls == []


def test_reachability_probe_uses_a_read_only_transaction(monkeypatch: pytest.MonkeyPatch) -> None:
    statements: list[str] = []

    class Connection:
        def __enter__(self) -> Connection:
            return self

        def __exit__(self, *args: Any) -> None:
            return None

        def execute(self, query: str) -> None:
            statements.append(query)

    monkeypatch.setattr(launcher.psycopg, "connect", lambda *_args, **_kwargs: Connection())
    assert launcher.database_reachable(launcher.DEFAULT_RUNTIME_DSN)
    assert statements == ["SET TRANSACTION READ ONLY", "SELECT 1"]


def test_prepare_builds_with_native_rustup_from_the_pinned_workspace(
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
) -> None:
    calls: list[tuple[list[str], Path, dict[str, str]]] = []
    monkeypatch.setattr(launcher, "database_reachable", lambda _: True)
    monkeypatch.setattr(launcher, "bootstrap_required", lambda _: False)
    monkeypatch.setattr(
        launcher, "provision_readers", lambda _: launcher.ReaderCredentials("observer", "known")
    )
    monkeypatch.setattr(
        launcher,
        "_run",
        lambda args, cwd, environment, _label: calls.append((args, cwd, dict(environment))),
    )
    launcher.prepare(tmp_path, {}, no_build=False)
    args, cwd, environment = calls[0]
    assert args == [
        "cargo",
        "build",
        "--locked",
        "-p",
        "babylon-persistence",
        "--bin",
        "babylon-runtime",
        "-p",
        "babylon-client",
        "--bin",
        "babylon-client",
    ]
    assert cwd == tmp_path / "rust"
    assert environment["CARGO_TARGET_DIR"] == str(tmp_path / "rust" / "target")
