"""Observer launcher capabilities, direct pipes, and campaign continuity."""

from __future__ import annotations

import json
import os
from pathlib import Path
from typing import Any
from uuid import UUID

import pytest
from tools import run_observer_session as launcher

CAMPAIGN = UUID("81b979ee-a9c1-48fd-8835-06cbfe594675")


def test_packaged_preparation_uses_only_the_bundled_binaries(
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
) -> None:
    calls: list[list[str]] = []
    monkeypatch.setattr(launcher, "database_reachable", lambda _: True)
    monkeypatch.setattr(launcher, "_run", lambda args, *_: calls.append(args))
    monkeypatch.setattr(
        launcher, "provision_readers", lambda _: launcher.ReaderCredentials("observer", "known")
    )
    runtime, client, _ = launcher.prepare(
        tmp_path, {"CARGO_TARGET_DIR": "/unrelated/build"}, no_build=False, distribution=True
    )
    assert runtime == tmp_path / "bin/babylon-runtime"
    assert client == tmp_path / "bin/babylon-client"
    assert calls == [[str(runtime), "bootstrap"], [str(runtime), "provision-readers"]]


def test_missing_packaged_database_cannot_start_the_developer_service(
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
) -> None:
    monkeypatch.setattr(launcher, "database_reachable", lambda _: False)
    calls: list[object] = []
    monkeypatch.setattr(launcher, "_run", lambda *args: calls.append(args))
    with pytest.raises(launcher.ObserverLaunchError, match="database is unavailable"):
        launcher.prepare(tmp_path, {}, no_build=True, distribution=True)
    assert not calls


def test_packaged_database_owns_its_loopback_port_project_and_state(
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
) -> None:
    (tmp_path / "release.json").write_text(
        json.dumps(
            {
                "schema": 1,
                "version": "0.4.0",
                "platform": "linux-x86_64",
            }
        )
    )
    calls: list[tuple[list[str], dict[str, str]]] = []

    def run(args: list[str], **kwargs: Any) -> Any:
        calls.append((args, kwargs["env"]))
        return launcher.subprocess.CompletedProcess(args, 0, stdout="127.0.0.1:49177\n")

    monkeypatch.setattr(launcher.subprocess, "run", run)
    monkeypatch.setattr(launcher.os, "getuid", lambda: 4242)
    environment = launcher.distribution_environment(
        tmp_path,
        {
            "DOCKER_HOST": "unix:///run/docker.sock",
            "COMPOSE_FILE": "/private/compose.yaml",
            "COMPOSE_PROJECT_NAME": "unrelated",
            "BABYLON_PG_DATA": "/private/saves",
            "BABYLON_RUNTIME_DSN": "private credentials",
            "PGPASSWORD": "secret",
            "XDG_STATE_HOME": str(tmp_path / "state"),
            "XDG_DATA_HOME": str(tmp_path / "data"),
        },
    )
    assert launcher._target_parameters(environment["BABYLON_RUNTIME_DSN"])["port"] == "49177"
    assert environment["XDG_STATE_HOME"] == str(tmp_path / "state/babylon-preview/0.4.0")
    assert environment["XDG_DATA_HOME"] == str(tmp_path / "data/babylon-preview/0.4.0")
    assert len(calls) == 2
    assert calls[0][0][:5] == [
        "docker",
        "compose",
        "--project-name",
        "babylon-preview-4242-0-4-0",
        "--file",
    ]
    for _, child_environment in calls:
        assert "COMPOSE_FILE" not in child_environment
        assert "COMPOSE_PROJECT_NAME" not in child_environment
        assert "BABYLON_PG_DATA" not in child_environment
        assert "PGPASSWORD" not in child_environment


@pytest.mark.parametrize("endpoint", ["tcp://host.example:2376", "ssh://host.example"])
def test_packaged_start_refuses_remote_docker_without_mutation(
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
    endpoint: str,
) -> None:
    calls: list[object] = []
    monkeypatch.setattr(launcher.subprocess, "run", lambda *args, **_kwargs: calls.append(args))
    with pytest.raises(launcher.ObserverLaunchError, match="local Docker"):
        launcher.distribution_docker_environment(tmp_path, {"DOCKER_HOST": endpoint})
    assert not calls


def test_selected_remote_context_cannot_hide_behind_a_local_docker_host(
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
) -> None:
    monkeypatch.setattr(
        launcher.subprocess,
        "run",
        lambda *args, **_kwargs: launcher.subprocess.CompletedProcess(
            args, 0, stdout="ssh://remote.example\n"
        ),
    )
    with pytest.raises(launcher.ObserverLaunchError, match="local Docker"):
        launcher.distribution_docker_environment(
            tmp_path,
            {
                "DOCKER_HOST": "unix:///run/docker.sock",
                "DOCKER_CONTEXT": "remote",
            },
        )


def test_installation_check_refuses_a_different_reopened_tail(
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
) -> None:
    observations = iter(
        [
            ("foundation", {"resolve_tick": 1, "tick_content_hash": "a" * 64}),
            ("foundation", {"resolve_tick": 1, "tick_content_hash": "b" * 64}),
        ]
    )
    monkeypatch.setattr(launcher, "_check_session", lambda *_args, **_kwargs: next(observations))
    with pytest.raises(launcher.ObserverLaunchError, match="different foundation or durable tail"):
        launcher.check_installation(
            tmp_path / "runtime", tmp_path / "client", tmp_path, {}, {}, tmp_path / "defines", None
        )


@pytest.mark.parametrize("exit_code", [20, 21, 22, 23])
def test_session_exit_never_cycles_the_native_process_pair(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path, exit_code: int
) -> None:
    monkeypatch.setenv("XDG_STATE_HOME", str(tmp_path))
    monkeypatch.delenv("BABYLON_CAMPAIGN_ID", raising=False)
    calls: list[dict[str, Any]] = []

    def pair(*_args: Any, **kwargs: Any) -> int:
        calls.append(kwargs)
        return exit_code if len(calls) == 1 else 0

    monkeypatch.setattr(launcher, "run_pair", pair)
    monkeypatch.setattr(
        launcher,
        "prepare",
        lambda *_args, **_kwargs: (
            tmp_path / "runtime",
            tmp_path / "client",
            launcher.ReaderCredentials("observer", "known"),
        ),
    )
    assert launcher.main(["--new", "--no-build"]) == exit_code
    assert len(calls) == 1


def test_unadmitted_initial_target_never_replaces_the_saved_pointer(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    monkeypatch.setenv("XDG_STATE_HOME", str(tmp_path))
    state = tmp_path / "babylon" / "observer-campaign"
    state.parent.mkdir()
    state.write_text(f"{CAMPAIGN}\n")
    monkeypatch.setattr(launcher, "run_pair", lambda *_args, **_kwargs: 1)
    monkeypatch.setattr(
        launcher,
        "prepare",
        lambda *_args, **_kwargs: (
            tmp_path / "runtime",
            tmp_path / "client",
            launcher.ReaderCredentials("observer", "known"),
        ),
    )
    assert launcher.main(["--new", "--no-build"]) == 1
    assert state.read_text() == f"{CAMPAIGN}\n"


def test_child_environments_do_not_inherit_writer_or_pg_authority() -> None:
    environment = {
        "PATH": "/usr/bin",
        "DISPLAY": ":0",
        "BABYLON_RUNTIME_DSN": "writer-secret",
        "BABYLON_CAMPAIGN_ID": str(CAMPAIGN),
        "PGPASSWORD": "other-secret",
        "PGSERVICEFILE": "/private/service",
        "BABYLON_OBSERVER_DSN": "stale-observer",
        "BABYLON_READER_DSN": "stale-reader",
        "RUST_LOG": "warn",
    }
    credentials = launcher.ReaderCredentials("observer-capability", "known-capability")
    runtime, client = launcher.child_environments(environment, credentials)
    assert runtime["BABYLON_RUNTIME_DSN"] == "writer-secret"
    assert "BABYLON_RUNTIME_DSN" not in client
    assert all(not key.upper().startswith("PG") for key in runtime | client)
    assert "BABYLON_READER_DSN" not in runtime and "BABYLON_OBSERVER_DSN" not in runtime
    assert client["BABYLON_OBSERVER_DSN"] == "observer-capability"
    assert client["BABYLON_READER_DSN"] == "known-capability"
    assert client["BABYLON_SESSION_STDIO"] == "1"
    assert "BABYLON_CAMPAIGN_ID" not in runtime and "BABYLON_CAMPAIGN_ID" not in client
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
        environment, launcher.ReaderCredentials("observer", "known")
    )
    assert client["RUST_LOG"] == effective
    assert runtime.get("RUST_LOG") == ambient
    assert environment.get("RUST_LOG") == ambient


def test_campaign_selection_and_new_preserve_existing_preference(tmp_path: Path) -> None:
    state = tmp_path / "campaign"
    first = launcher.select_initial_target({}, state_file=state)
    assert isinstance(first, launcher.NewCampaignTarget)
    assert first.preset == "standard" and first.campaign.int != 0
    assert not state.exists()
    state.write_text(f"{first.campaign}\n")
    assert launcher.select_initial_target({}, state_file=state) == launcher.OpenCampaignTarget(
        first.campaign
    )
    assert launcher.select_initial_target(
        {}, state_file=state, explicit=str(CAMPAIGN)
    ) == launcher.OpenCampaignTarget(CAMPAIGN)
    second = launcher.select_initial_target({}, state_file=state, new=True, preset="delayed")
    assert isinstance(second, launcher.NewCampaignTarget)
    assert second.campaign != first.campaign and second.preset == "delayed"
    assert state.read_text() == f"{first.campaign}\n"


def test_log_capture_notice_is_bounded_and_never_echoes_environment_secrets(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    monkeypatch.setenv("XDG_STATE_HOME", str(tmp_path))
    monkeypatch.setenv("BABYLON_RUNTIME_DSN", "writer-secret")
    monkeypatch.setenv("RUST_LOG", 'warn,engine[span{key="private-filter"}]=trace')
    monkeypatch.setattr(launcher, "run_pair", lambda *_args, **_kwargs: 0)
    monkeypatch.setattr(
        launcher,
        "prepare",
        lambda *_args, **_kwargs: (
            tmp_path / "runtime",
            tmp_path / "client",
            launcher.ReaderCredentials("observer-secret", "known-secret"),
        ),
    )
    assert launcher.main(["--new", "--no-build"]) == 0
    captured = capsys.readouterr()
    assert captured.out == ""
    assert captured.err == "Observer log targets enabled: session=debug,babylon_client=debug\n"


@pytest.mark.parametrize("value", ["not-a-uuid", "", "0" * 32, str(CAMPAIGN).upper()])
def test_invalid_explicit_campaign_refuses(value: str, tmp_path: Path) -> None:
    with pytest.raises(launcher.ObserverLaunchError, match="campaign"):
        launcher.select_initial_target({}, state_file=tmp_path / "campaign", explicit=value)


def test_corrupt_saved_campaign_refuses_without_replacing_it(tmp_path: Path) -> None:
    state = tmp_path / "campaign"
    state.write_text("damaged")
    with pytest.raises(launcher.ObserverLaunchError, match="campaign"):
        launcher.select_initial_target({}, state_file=state)
    assert state.read_text() == "damaged"


@pytest.mark.parametrize("saved_pointer", [b"damaged", b"x" * 65])
def test_smoke_ignores_damaged_saved_campaign_preferences(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path, saved_pointer: bytes
) -> None:
    monkeypatch.setenv("XDG_STATE_HOME", str(tmp_path))
    state = launcher.preference_path(dict(os.environ))
    state.parent.mkdir(parents=True, exist_ok=True)
    state.write_bytes(saved_pointer)
    runtime, client = tmp_path / "runtime", tmp_path / "client"
    calls: list[tuple[Any, ...]] = []
    monkeypatch.setattr(
        launcher,
        "prepare",
        lambda *_args, **_kwargs: (
            runtime,
            client,
            launcher.ReaderCredentials("observer", "known"),
        ),
    )

    def installation(*args: Any) -> int:
        calls.append(args)
        return 0

    monkeypatch.setattr(launcher, "check_installation", installation)
    assert launcher.main(["--smoke", "--no-build"]) == 0
    assert len(calls) == 1
    assert calls[0][:2] == (runtime, client)
    assert state.read_bytes() == saved_pointer


@pytest.mark.parametrize(
    "preset", [None, "standard", "delayed", "shared-freight-ample", "shared-freight-constrained"]
)
def test_two_anonymous_pipes_connect_children_without_parent_forwarding(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path, preset: str | None
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
        {"BABYLON_RUNTIME_DSN": "writer"},
        {"BABYLON_SESSION_STDIO": "1"},
        defines_path=tmp_path / "custom values.toml",
        initial_target=(
            launcher.OpenCampaignTarget(CAMPAIGN)
            if preset is None
            else launcher._new_target(CAMPAIGN, preset)
        ),
    )
    assert code == 0
    assert len(fds) == 4 and len(children) == 2
    runtime, client = children
    assert runtime["args"] == [
        str(tmp_path / "runtime"),
        "session",
        "--stdio",
        "--defines",
        str(tmp_path / "custom values.toml"),
    ]
    assert client["args"] == (
        [str(tmp_path / "client"), "--campaign", str(CAMPAIGN)]
        if preset is None
        else [str(tmp_path / "client"), "--new-campaign", str(CAMPAIGN), "--preset", preset]
    )
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
        assert (
            launcher.run_pair(
                *arguments,
                defines_path=tmp_path / "defines.toml",
                initial_target=launcher.OpenCampaignTarget(CAMPAIGN),
            )
            == 0
        )
    else:
        with pytest.raises(launcher.ObserverLaunchError, match="runtime shutdown deadline"):
            launcher.run_pair(
                *arguments,
                defines_path=tmp_path / "defines.toml",
                initial_target=launcher.OpenCampaignTarget(CAMPAIGN),
            )
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
        launcher.run_pair(
            tmp_path / "runtime",
            tmp_path / "client",
            tmp_path,
            {},
            {},
            defines_path=tmp_path / "defines.toml",
            initial_target=launcher.OpenCampaignTarget(CAMPAIGN),
        )
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


def test_preparation_verifies_current_schema_before_restricted_logins_without_ticks(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
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
    monkeypatch.setattr(launcher, "database_reachable", lambda _: True, raising=False)
    monkeypatch.setattr(launcher, "provision_readers", provision)
    runtime, client, credentials = launcher.prepare(
        tmp_path, {"PGOPTIONS": "unsafe"}, no_build=True
    )
    assert calls == ["bootstrap", "provision-readers", "provision"]
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
    "preset",
    [
        "standard",
        "delayed",
        "shared-freight-ample",
        "shared-freight-constrained",
        "statewide-baseline",
        "statewide-freight-constraint",
        "statewide-packaging-shortage",
        "statewide-both",
    ],
)
def test_first_launch_has_an_explicit_new_preset_but_saved_resume_cannot_override_it(
    tmp_path: Path, preset: str
) -> None:
    state = tmp_path / "campaign"
    target = launcher.select_initial_target({}, state_file=state, preset=preset)
    assert isinstance(target, launcher.NewCampaignTarget) and target.preset == preset
    state.write_text(f"{CAMPAIGN}\n")
    with pytest.raises(launcher.ObserverLaunchError, match="--preset applies only"):
        launcher.select_initial_target({}, state_file=state, preset=preset)
    assert launcher.select_initial_target({}, state_file=state) == launcher.OpenCampaignTarget(
        CAMPAIGN
    )
    assert state.read_text() == f"{CAMPAIGN}\n"


def test_explicit_open_overrides_environment_and_saved_target_without_writing(
    tmp_path: Path,
) -> None:
    selected = UUID("fc7d28a0-a29a-49ea-bf3b-ef07ee163cd4")
    state = tmp_path / "campaign"
    state.write_text(f"{selected}\n")
    environment = {"BABYLON_CAMPAIGN_ID": str(CAMPAIGN)}
    assert launcher.select_initial_target(
        environment, state_file=state
    ) == launcher.OpenCampaignTarget(CAMPAIGN)
    assert launcher.select_initial_target(
        environment, state_file=state, explicit=str(selected)
    ) == launcher.OpenCampaignTarget(selected)
    assert state.read_text() == f"{selected}\n"


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
    monkeypatch.setattr(
        launcher, "provision_readers", lambda _: launcher.ReaderCredentials("observer", "known")
    )
    if available_after_start:
        launcher.prepare(tmp_path, {}, no_build=True)
        assert calls == ["db:up", "bootstrap", "provision-readers"]
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


def _smoke_transcript_children(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path, *, refused: bool = False
) -> list[list[str]]:
    """Keep the real launcher/session code; replace only native process boundaries."""
    calls: list[list[str]] = []
    tail = {"resolve_tick": 1, "tick_content_hash": "a" * 64}
    scope = {"session_id": "fixture", "generation": 1}

    class RuntimeChild:
        def __init__(self, args: list[str], **_kwargs: Any) -> None:
            new = not calls
            calls.append(args)
            self.stdin = (tmp_path / f"requests-{len(calls)}.jsonl").open("wb")
            rows: list[dict[str, Any]] = [{"type": "hello", "protocol_version": 3, "scope": scope}]
            if refused:
                rows.append({"type": "error", "request_id": 1, "code": "invalid_defines"})
            else:
                rows.append(
                    {
                        "type": "ready",
                        "request_id": 1,
                        "scope": scope,
                        "foundation_digest": "b" * 64,
                        "tail": {"resolve_tick": 0, "tick_content_hash": None} if new else tail,
                    }
                )
                if new:
                    rows.append(
                        {"type": "committed", "request_id": 2, "scope": scope, "tail": tail}
                    )
                rows.append({"type": "stopped", "request_id": 3 if new else 2})
            read_fd, write_fd = os.pipe()
            self.stdout = os.fdopen(read_fd, "rb")
            with os.fdopen(write_fd, "wb") as output:
                output.write(b"".join(json.dumps(row).encode("ascii") + b"\n" for row in rows))

        def wait(self, timeout: float | None = None) -> int:
            return 0

        def poll(self) -> int:
            return 0

    def readback(args: list[str], **_kwargs: Any) -> Any:
        assert not refused, "a refused New cannot proceed to native readback"
        assert args[1:] == ["--headless", "--campaign", str(CAMPAIGN), "tick", "status"]
        return launcher.subprocess.CompletedProcess(
            args,
            0,
            stdout=json.dumps(
                {"campaign_id": str(CAMPAIGN), "durable_tick": 1, "tick_content_hash": "a" * 64}
            ),
        )

    monkeypatch.setattr(launcher, "uuid4", lambda: CAMPAIGN)
    monkeypatch.setattr(
        launcher,
        "prepare",
        lambda *_args, **_kwargs: (
            tmp_path / "runtime",
            tmp_path / "client",
            launcher.ReaderCredentials("observer", "known"),
        ),
    )
    monkeypatch.setattr(launcher.subprocess, "Popen", RuntimeChild)
    monkeypatch.setattr(launcher.subprocess, "run", readback)
    return calls


@pytest.mark.parametrize(
    "preset",
    [
        None,
        "standard",
        "delayed",
        "shared-freight-ample",
        "shared-freight-constrained",
        "statewide-baseline",
        "statewide-freight-constraint",
        "statewide-packaging-shortage",
        "statewide-both",
    ],
)
def test_smoke_request_preserves_selected_preset_through_new_restart_and_readback(
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
    preset: str | None,
) -> None:
    calls = _smoke_transcript_children(monkeypatch, tmp_path)
    defines = tmp_path / "captured defines.toml"
    args = ["--smoke", "--no-build", "--defines", str(defines)]
    if preset is not None:
        args.extend(["--preset", preset])
    assert launcher.main(args) == 0
    assert len(calls) == 2
    requests = [
        [
            json.loads(line)
            for line in (tmp_path / f"requests-{index}.jsonl").read_text().splitlines()
        ]
        for index in (1, 2)
    ]
    selected = preset or "standard"
    assert requests[0][0]["target"] == {
        "type": "new",
        "campaign_id": str(CAMPAIGN),
        "preset": selected,
    }
    assert requests[1][0]["target"] == {"type": "open", "campaign_id": str(CAMPAIGN)}
    assert [request["type"] for request in requests[0]] == ["switch", "advance", "stop"]
    assert [request["type"] for request in requests[1]] == ["switch", "stop"]
    assert calls[0][-2:] == ["--defines", str(defines)]
    assert calls[1][-2:] == ["--defines", "/dev/null"]
    report = json.loads(capsys.readouterr().out)
    assert report["preset"] == selected
    assert report["reopened_without_defines"] and report["periods"] == 1


def test_smoke_request_refusal_never_falls_back_to_another_preset(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    calls = _smoke_transcript_children(monkeypatch, tmp_path, refused=True)
    assert launcher.main(["--smoke", "--no-build", "--preset", "statewide-both"]) == 1
    assert len(calls) == 1
    request = json.loads((tmp_path / "requests-1.jsonl").read_text())
    assert request["target"]["preset"] == "statewide-both"
    output = capsys.readouterr()
    assert "invalid_defines" in output.err
    assert not output.out


def test_smoke_request_rejects_existing_campaign_before_preparing_services(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    calls: list[object] = []
    monkeypatch.setattr(launcher, "prepare", lambda *_args, **_kwargs: calls.append("prepare"))
    assert launcher.main(["--smoke", "--campaign", str(CAMPAIGN)]) == 1
    assert not calls
    assert "--smoke creates a new campaign and cannot use --campaign" in capsys.readouterr().err


@pytest.mark.parametrize("refused_phase", ["bootstrap", "provision-readers"])
def test_preparation_refuses_database_before_reader_mutations(
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
    refused_phase: str,
) -> None:
    calls: list[str] = []
    monkeypatch.setattr(launcher, "database_reachable", lambda _: True)

    def no_python_schema_admission(*_args: Any, **_kwargs: Any) -> None:
        raise AssertionError("schema admission belongs to the Rust verifier")

    def run(args: list[str], *_args: Any) -> None:
        calls.append(args[-1])
        if args[-1] == refused_phase:
            raise launcher.ObserverLaunchError(f"refused {refused_phase}")

    def provision(_dsn: str) -> launcher.ReaderCredentials:
        calls.append("login-mutation")
        return launcher.ReaderCredentials("observer", "known")

    monkeypatch.setattr(launcher.psycopg, "connect", no_python_schema_admission)
    monkeypatch.setattr(launcher, "_run", run)
    monkeypatch.setattr(launcher, "provision_readers", provision)
    with pytest.raises(launcher.ObserverLaunchError, match=f"refused {refused_phase}"):
        launcher.prepare(tmp_path, {}, no_build=True)
    assert calls == (
        ["bootstrap"] if refused_phase == "bootstrap" else ["bootstrap", "provision-readers"]
    )
