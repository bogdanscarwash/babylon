#!/usr/bin/env python3
"""Launch one persistent Michigan observer session with separate read capabilities."""

from __future__ import annotations

import argparse
import ipaddress
import json
import os
import re
import select
import subprocess
import sys
import time
from collections.abc import Mapping
from dataclasses import dataclass
from pathlib import Path
from typing import Literal
from uuid import UUID, uuid4

import psycopg
from psycopg import sql
from psycopg.conninfo import conninfo_to_dict, make_conninfo

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_DEFINES = ROOT / "content" / "scenarios" / "michigan" / "defines.toml"
DEFAULT_RUNTIME_DSN = "host=127.0.0.1 port=5433 dbname=babylon_test user=test password=test"
OBSERVER_CAPTURE_FILTER = "session=debug,babylon_client=debug"
# The runtime's database statement timeout is 120 seconds. EOF/Stop gets time
# to finish a transaction before any exact-child termination is attempted.
RUNTIME_SHUTDOWN_GRACE_SECONDS = 150
CHILD_SIGNAL_WAIT_SECONDS = 10
READ_LOGINS = (
    ("babylon_observer_game", "babylon_observer", "babylon_observer_game"),
    ("babylon_preview_game", "babylon_reader", "babylon_preview_game"),
)


def distribution_identity(root: Path) -> tuple[str, str]:
    """Read the release identity used to isolate this installation's saved worlds."""
    try:
        manifest_path = root / "release.json"
        if manifest_path.stat().st_size > 65_536:
            raise ValueError("oversized release manifest")
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        version = manifest["version"]
        if (
            not isinstance(version, str)
            or re.fullmatch(r"(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)", version)
            is None
        ):
            raise ValueError("invalid release version")
        if manifest["schema"] != 1 or manifest["platform"] != "linux-x86_64":
            raise ValueError("unsupported release manifest")
    except (OSError, ValueError, TypeError, KeyError) as error:
        raise ObserverLaunchError("cannot read this distribution's release identity") from error
    return version, f"babylon-preview-{os.getuid()}-{version.replace('.', '-')}"


def distribution_compose(root: Path) -> list[str]:
    """Use only this release's compose file, project, and persisted volume."""
    _, project = distribution_identity(root)
    return [
        "docker",
        "compose",
        "--project-name",
        project,
        "--file",
        str(root / "distribution" / "compose.yaml"),
        "--env-file",
        "/dev/null",
    ]


def distribution_docker_environment(root: Path, environment: Mapping[str, str]) -> dict[str, str]:
    """Ignore other compose projects and require a local Docker daemon."""
    common = {
        key: value
        for key, value in _clean_environment(environment).items()
        if not key.startswith("COMPOSE_") and key != "BABYLON_PG_DATA"
    }
    # A remote Docker daemon would create the database on another machine even
    # though every native connection is confined to this machine's loopback.
    try:
        endpoint = common.get("DOCKER_HOST") if not common.get("DOCKER_CONTEXT") else None
        if not endpoint:
            endpoint = subprocess.run(
                ["docker", "context", "inspect", "--format", "{{.Endpoints.docker.Host}}"],
                cwd=root,
                env=common,
                check=True,
                capture_output=True,
                text=True,
                timeout=10,
            ).stdout.strip()
        if not endpoint.startswith("unix://"):
            raise ValueError("Docker must use a local Unix socket")
    except (OSError, subprocess.SubprocessError, ValueError) as error:
        raise ObserverLaunchError(
            "the preview requires a local Docker Unix-socket context"
        ) from error
    return common


def distribution_environment(root: Path, environment: Mapping[str, str]) -> dict[str, str]:
    """Start the packaged local database and resolve its assigned loopback port."""
    version, _ = distribution_identity(root)
    common = distribution_docker_environment(root, environment)
    for variable, default in (
        ("XDG_STATE_HOME", ".local/state"),
        ("XDG_DATA_HOME", ".local/share"),
    ):
        base = Path(environment.get(variable, str(Path.home() / default)))
        if not base.is_absolute():
            raise ObserverLaunchError(f"{variable} must be absolute")
        common[variable] = str(base / "babylon-preview" / version)
    try:
        compose = distribution_compose(root)
        _run(
            [
                *compose,
                "up",
                "--build",
                "--detach",
                "--wait",
                "--wait-timeout",
                "120",
                "babylon-pg",
            ],
            root,
            common,
            "preview database startup",
        )
        address = subprocess.run(
            [*compose, "port", "babylon-pg", "5432"],
            cwd=root,
            env=common,
            check=True,
            capture_output=True,
            text=True,
            timeout=10,
        ).stdout.strip()
        match = re.fullmatch(r"127\.0\.0\.1:([1-9][0-9]{0,4})", address)
        if match is None or int(match[1]) > 65_535:
            raise ValueError("Docker did not return one loopback port")
    except (OSError, subprocess.SubprocessError, ValueError) as error:
        raise ObserverLaunchError("cannot prepare this preview's local Docker database") from error
    parameters = _target_parameters(DEFAULT_RUNTIME_DSN)
    parameters["port"] = match[1]
    common["BABYLON_RUNTIME_DSN"] = make_conninfo(**parameters)
    return common


class ObserverLaunchError(ValueError):
    """A launcher refusal safe to display without credentials."""


@dataclass(frozen=True)
class ReaderCredentials:
    """Separate database capabilities admitted by the native readers."""

    observer_dsn: str
    known_dsn: str


def _campaign(value: str) -> UUID:
    try:
        campaign = UUID(value)
    except ValueError as error:
        raise ObserverLaunchError("campaign must be a canonical UUID") from error
    if str(campaign) != value or campaign.int == 0:
        raise ObserverLaunchError("campaign must be a nonzero canonical UUID")
    return campaign


def preference_path(environment: Mapping[str, str]) -> Path:
    """Locate the user's continuation preference outside the checkout."""
    state_home = environment.get("XDG_STATE_HOME")
    base = Path(state_home) if state_home else Path.home() / ".local" / "state"
    if not base.is_absolute():
        raise ObserverLaunchError("XDG_STATE_HOME must be absolute")
    return base / "babylon" / "observer-campaign"


@dataclass(frozen=True)
class NewCampaignTarget:
    """An explicit request to found one absent campaign after the runtime Hello."""

    campaign: UUID
    preset: Literal[
        "standard",
        "delayed",
        "shared-freight-ample",
        "shared-freight-constrained",
        "statewide-baseline",
        "statewide-freight-constraint",
        "statewide-packaging-shortage",
        "statewide-both",
    ]


@dataclass(frozen=True)
class OpenCampaignTarget:
    """An explicit existing-only campaign request; it never authorizes founding."""

    campaign: UUID


def _new_target(campaign: UUID, preset: str | None) -> NewCampaignTarget:
    if preset is None or preset == "standard":
        return NewCampaignTarget(campaign, "standard")
    if preset == "delayed":
        return NewCampaignTarget(campaign, "delayed")
    if preset == "shared-freight-ample":
        return NewCampaignTarget(campaign, "shared-freight-ample")
    if preset == "shared-freight-constrained":
        return NewCampaignTarget(campaign, "shared-freight-constrained")
    if preset == "statewide-baseline":
        return NewCampaignTarget(campaign, "statewide-baseline")
    if preset == "statewide-freight-constraint":
        return NewCampaignTarget(campaign, "statewide-freight-constraint")
    if preset == "statewide-packaging-shortage":
        return NewCampaignTarget(campaign, "statewide-packaging-shortage")
    if preset == "statewide-both":
        return NewCampaignTarget(campaign, "statewide-both")
    raise ObserverLaunchError("unknown material scenario preset")


def select_initial_target(
    environment: Mapping[str, str],
    *,
    state_file: Path,
    explicit: str | None = None,
    new: bool = False,
    preset: str | None = None,
) -> NewCampaignTarget | OpenCampaignTarget:
    """Choose New or Open without writing the saved continuation pointer."""
    if new:
        if explicit is not None:
            raise ObserverLaunchError("new and existing campaign targets are mutually exclusive")
        return _new_target(uuid4(), preset)
    selected = explicit if explicit is not None else environment.get("BABYLON_CAMPAIGN_ID")
    if selected is None:
        try:
            if state_file.stat().st_size > 64:
                raise ObserverLaunchError("saved campaign preference is oversized")
            selected = state_file.read_text(encoding="ascii").strip()
        except FileNotFoundError:
            return _new_target(uuid4(), preset)
        except (OSError, UnicodeError) as error:
            raise ObserverLaunchError("cannot read saved campaign preference") from error
    if preset is not None:
        raise ObserverLaunchError("--preset applies only to a new campaign; use --new")
    return OpenCampaignTarget(_campaign(selected))


def _clean_environment(environment: Mapping[str, str]) -> dict[str, str]:
    return {
        key: value
        for key, value in environment.items()
        if not key.upper().startswith("PG")
        and key
        not in {
            "BABYLON_RUNTIME_DSN",
            "BABYLON_OBSERVER_DSN",
            "BABYLON_READER_DSN",
            "BABYLON_SESSION_STDIO",
            "BABYLON_CAMPAIGN_ID",
            "BABYLON_DOSSIER_DEMO_PASSWORD",
        }
    }


def child_environments(
    environment: Mapping[str, str],
    credentials: ReaderCredentials,
) -> tuple[dict[str, str], dict[str, str]]:
    """Separate writer and reader authority; only client arguments select a target."""
    common = _clean_environment(environment)
    runtime = {
        **common,
        "BABYLON_RUNTIME_DSN": environment.get("BABYLON_RUNTIME_DSN", DEFAULT_RUNTIME_DSN),
    }
    client = {
        **common,
        # EnvFilter replaces an identical target with its last directive.
        # Keep ambient engine filters; observer capture is always explicit.
        "RUST_LOG": f"{environment.get('RUST_LOG', '').strip() or 'warn'},{OBSERVER_CAPTURE_FILTER}",
        "BABYLON_SESSION_STDIO": "1",
        "BABYLON_OBSERVER_DSN": credentials.observer_dsn,
        "BABYLON_READER_DSN": credentials.known_dsn,
    }
    return runtime, client


def _target_parameters(dsn: str) -> dict[str, str]:
    try:
        parameters: dict[str, str] = {}
        for key, value in conninfo_to_dict(dsn).items():
            if not isinstance(value, str):
                raise ValueError("connection parameter must be text")
            parameters[key] = value
        host = parameters.get("host", "")
        if not (host.startswith("/") or ipaddress.ip_address(host).is_loopback):
            raise ValueError("not loopback")
        if set(parameters) - {"host", "port", "dbname", "user", "password"}:
            raise ValueError("unsupported startup parameter")
        if not all(parameters.get(key) for key in ("port", "dbname", "user", "password")):
            raise ValueError("incomplete explicit connection")
        if not 0 < int(parameters["port"]) <= 65_535:
            raise ValueError("invalid port")
    except (psycopg.Error, ValueError) as error:
        raise ObserverLaunchError(
            "runtime DSN requires one explicit local database target"
        ) from error
    return parameters


def provision_readers(runtime_dsn: str) -> ReaderCredentials:
    """Provision only local LOGIN memberships after Rust installs the reader schemas."""
    parameters = _target_parameters(runtime_dsn)
    try:
        with psycopg.connect(runtime_dsn, connect_timeout=10, options="") as connection:
            for name, group, password in READ_LOGINS:
                if (
                    connection.execute(
                        "SELECT 1 FROM pg_catalog.pg_roles WHERE rolname = %s", (name,)
                    ).fetchone()
                    is None
                ):
                    connection.execute(
                        sql.SQL(
                            "CREATE ROLE {} LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS"
                        ).format(sql.Identifier(name))
                    )
                connection.execute(
                    sql.SQL(
                        "ALTER ROLE {} LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS PASSWORD {}"
                    ).format(sql.Identifier(name), sql.Literal(password))
                )
                connection.execute(
                    sql.SQL("GRANT {} TO {}").format(sql.Identifier(group), sql.Identifier(name))
                )
                connection.execute(
                    sql.SQL("GRANT SET ON PARAMETER event_triggers TO {}").format(
                        sql.Identifier(name)
                    )
                )
    except psycopg.Error as error:
        raise ObserverLaunchError("local observer read-role provisioning failed") from error
    targets = {key: parameters[key] for key in ("host", "port", "dbname")}
    dsns = [
        make_conninfo(**targets, user=name, password=password) for name, _, password in READ_LOGINS
    ]
    return ReaderCredentials(dsns[0], dsns[1])


def _run(args: list[str], root: Path, environment: Mapping[str, str], label: str) -> None:
    try:
        subprocess.run(args, cwd=root, env=dict(environment), check=True)
    except (OSError, subprocess.CalledProcessError) as error:
        raise ObserverLaunchError(f"{label} failed") from error


def database_reachable(runtime_dsn: str) -> bool:
    """Check the exact local database with a short read-only connection."""
    _target_parameters(runtime_dsn)
    try:
        with psycopg.connect(runtime_dsn, connect_timeout=3, options="") as connection:
            connection.execute("SET TRANSACTION READ ONLY")
            connection.execute("SELECT 1")
    except psycopg.Error:
        return False
    return True


def prepare(
    root: Path, environment: Mapping[str, str], *, no_build: bool, distribution: bool = False
) -> tuple[Path, Path, ReaderCredentials]:
    """Start the local DB and install Rust authority before any client connects."""
    runtime_dsn = environment.get("BABYLON_RUNTIME_DSN", DEFAULT_RUNTIME_DSN)
    _target_parameters(runtime_dsn)
    common = _clean_environment(environment)
    target = Path(environment.get("CARGO_TARGET_DIR", "target"))
    if not target.is_absolute():
        target = root / "rust" / target
    common["CARGO_TARGET_DIR"] = str(target)
    binary_dir = root / "bin" if distribution else target / "debug"
    runtime, client = binary_dir / "babylon-runtime", binary_dir / "babylon-client"
    if not database_reachable(runtime_dsn):
        if distribution or _target_parameters(runtime_dsn) != _target_parameters(
            DEFAULT_RUNTIME_DSN
        ):
            raise ObserverLaunchError(
                "requested local database is unavailable; start or create that database and retry"
            )
        _run(["mise", "run", "db:up"], root, common, "local database startup")
        if not database_reachable(runtime_dsn):
            raise ObserverLaunchError("default local database is still unavailable after db:up")
    if not no_build and not distribution:
        _run(
            [
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
            ],
            root / "rust",
            common,
            "observer build",
        )
    writer = {**common, "BABYLON_RUNTIME_DSN": runtime_dsn}
    # Rust classifies and verifies the exact fresh/current schema before any
    # schema, reference, or role mutation. An incompatible database refuses.
    _run([str(runtime), "bootstrap"], root, writer, "Rust current schema bootstrap")
    _run([str(runtime), "provision-readers"], root, writer, "reader role provisioning")
    return runtime, client, provision_readers(runtime_dsn)


def _stop(child: subprocess.Popen[bytes]) -> None:
    if child.poll() is None:
        try:
            child.terminate()
        except ProcessLookupError:
            pass  # The exact child exited between poll and signal; still reap it.
        try:
            child.wait(timeout=CHILD_SIGNAL_WAIT_SECONDS)
        except subprocess.TimeoutExpired:
            try:
                child.kill()
            except ProcessLookupError:
                pass
            try:
                child.wait(timeout=CHILD_SIGNAL_WAIT_SECONDS)
            except subprocess.TimeoutExpired as error:
                raise ObserverLaunchError(
                    "observer child did not exit after bounded shutdown; "
                    "reopen the campaign to reconcile committed progress"
                ) from error


def _finish_runtime(child: subprocess.Popen[bytes]) -> int:
    """After client EOF, allow one graceful commit/close window before signals."""
    try:
        return child.wait(timeout=RUNTIME_SHUTDOWN_GRACE_SECONDS)
    except subprocess.TimeoutExpired as error:
        _stop(child)
        raise ObserverLaunchError(
            "runtime shutdown deadline exceeded; "
            "reopen the campaign to reconcile committed progress"
        ) from error


def _check_session(
    runtime: Path,
    root: Path,
    environment: Mapping[str, str],
    defines: Path,
    target: NewCampaignTarget | OpenCampaignTarget,
) -> tuple[str, dict[str, object]]:
    """Exercise the installed lifecycle protocol with one bounded native process."""
    child = subprocess.Popen(
        [str(runtime), "session", "--stdio", "--defines", str(defines)],
        cwd=root,
        env=dict(environment),
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
    )
    assert child.stdin is not None and child.stdout is not None
    output = child.stdout
    buffer = b""

    def receive(kind: str, request_id: int | None = None) -> dict[str, object]:
        nonlocal buffer
        deadline = time.monotonic() + RUNTIME_SHUTDOWN_GRACE_SECONDS
        while True:
            while b"\n" not in buffer:
                if len(buffer) >= 4096:
                    raise ObserverLaunchError(
                        "installation check received an oversized protocol row"
                    )
                remaining = deadline - time.monotonic()
                if remaining <= 0 or not select.select([output], [], [], remaining)[0]:
                    raise ObserverLaunchError(
                        "installation check timed out waiting for the runtime"
                    )
                chunk = os.read(output.fileno(), 4096)
                if not chunk:
                    raise ObserverLaunchError(
                        "installation check runtime exited before acknowledgement"
                    )
                buffer += chunk
            line, buffer = buffer.split(b"\n", 1)
            if len(line) >= 4096:
                raise ObserverLaunchError("installation check received an oversized protocol row")
            message = json.loads(line)
            if not isinstance(message, dict):
                raise ObserverLaunchError("installation check received an invalid protocol row")
            if message.get("type") == "error":
                raise ObserverLaunchError(
                    f"installation check runtime refused: {message.get('code')}"
                )
            if message.get("type") == kind and message.get("request_id") == request_id:
                return message
            if message.get("type") not in {"switching", "archive_progress"}:
                raise ObserverLaunchError(
                    "installation check received an unexpected protocol response"
                )

    def send(kind: str, request_id: int, scope: object, **fields: object) -> None:
        assert child.stdin is not None
        row = {
            "type": kind,
            "protocol_version": 3,
            "request_id": request_id,
            "scope": scope,
            **fields,
        }
        child.stdin.write(json.dumps(row).encode("ascii") + b"\n")
        child.stdin.flush()

    try:
        hello = receive("hello")
        if hello.get("protocol_version") != 3:
            raise ObserverLaunchError("installation check requires runtime session protocol 3")
        new = isinstance(target, NewCampaignTarget)
        requested = {"type": "new" if new else "open", "campaign_id": str(target.campaign)}
        if isinstance(target, NewCampaignTarget):
            requested["preset"] = target.preset
        send("switch", 1, hello["scope"], target=requested)
        ready = receive("ready", 1)
        tail = ready["tail"]
        scope = ready["scope"]
        if new:
            if tail != {"resolve_tick": 0, "tick_content_hash": None}:
                raise ObserverLaunchError(
                    "installation check New did not return an empty durable tail"
                )
            send("advance", 2, scope, expected_tail=tail)
            committed = receive("committed", 2)
            if committed["scope"] != scope:
                raise ObserverLaunchError("installation check commit changed campaign scope")
            tail = committed["tail"]
        if (
            not isinstance(tail, dict)
            or tail.get("resolve_tick") != 1
            or re.fullmatch(r"[0-9a-f]{64}", str(tail.get("tick_content_hash"))) is None
        ):
            raise ObserverLaunchError(
                "installation check did not observe one hashed committed period"
            )
        stop_id = 3 if new else 2
        send("stop", stop_id, scope)
        receive("stopped", stop_id)
        child.stdin.close()
        if _finish_runtime(child) != 0:
            raise ObserverLaunchError("installation check runtime shutdown failed")
        foundation = ready["foundation_digest"]
        if not isinstance(foundation, str):
            raise ObserverLaunchError("installation check foundation identity was absent")
        return foundation, tail
    finally:
        if not child.stdin.closed:
            child.stdin.close()
        if child.poll() is None:
            _finish_runtime(child)
        child.stdout.close()


def check_installation(
    runtime: Path,
    client: Path,
    root: Path,
    writer: Mapping[str, str],
    reader: Mapping[str, str],
    defines: Path,
    preset: str | None,
) -> int:
    """Prove New, one period, process restart, Open, and the real native reader."""
    campaign = uuid4()
    target = _new_target(campaign, preset)
    first = _check_session(runtime, root, writer, defines, target)
    reopened = _check_session(
        runtime, root, writer, Path("/dev/null"), OpenCampaignTarget(campaign)
    )
    if reopened != first:
        raise ObserverLaunchError(
            "installation check reopened a different foundation or durable tail"
        )
    result = subprocess.run(
        [str(client), "--headless", "--campaign", str(campaign), "tick", "status"],
        cwd=root,
        env=dict(reader),
        stdin=subprocess.DEVNULL,
        capture_output=True,
        text=True,
        check=True,
        timeout=RUNTIME_SHUTDOWN_GRACE_SECONDS,
    )
    status = json.loads(result.stdout)
    if (
        not isinstance(status, dict)
        or status.get("campaign_id") != str(campaign)
        or (
            status.get("durable_tick") != 1
            or status.get("tick_content_hash") != first[1]["tick_content_hash"]
        )
    ):
        raise ObserverLaunchError(
            "installation check native reader disagrees with the committed period"
        )
    print(
        json.dumps(
            {
                "check": "passed",
                "campaign_id": str(campaign),
                "preset": target.preset,
                "periods": 1,
                "foundation_digest": first[0],
                "tail": first[1],
                "reopened_without_defines": True,
            }
        )
    )
    return 0


def run_pair(
    runtime_binary: Path,
    client_binary: Path,
    root: Path,
    runtime_environment: Mapping[str, str],
    client_environment: Mapping[str, str],
    *,
    defines_path: Path,
    initial_target: NewCampaignTarget | OpenCampaignTarget,
) -> int:
    """Cross-connect two anonymous pipes; the parent never reads or forwards protocol bytes."""
    descriptors: list[int] = []
    runtime: subprocess.Popen[bytes] | None = None
    client: subprocess.Popen[bytes] | None = None
    runtime_shutdown_started = False
    try:
        requests = os.pipe()
        descriptors.extend(requests)
        responses = os.pipe()
        descriptors.extend(responses)
        runtime = subprocess.Popen(
            [str(runtime_binary), "session", "--stdio", "--defines", str(defines_path)],
            cwd=root,
            env=dict(runtime_environment),
            stdin=requests[0],
            stdout=responses[1],
            stderr=None,
            close_fds=True,
        )
        if isinstance(initial_target, NewCampaignTarget):
            client_args = [
                str(client_binary),
                "--new-campaign",
                str(initial_target.campaign),
                "--preset",
                initial_target.preset,
            ]
        else:
            client_args = [str(client_binary), "--campaign", str(initial_target.campaign)]
        client = subprocess.Popen(
            client_args,
            cwd=root,
            env=dict(client_environment),
            stdin=responses[0],
            stdout=requests[1],
            stderr=None,
            close_fds=True,
        )
        for descriptor in descriptors:
            os.close(descriptor)
        descriptors.clear()
        # The user may keep the game open indefinitely. The client bounds its
        # explicit Quit handshake; closing it also sends EOF through the pipe.
        client_code = client.wait()
        runtime_shutdown_started = True
        runtime_code = _finish_runtime(runtime)
        return client_code if client_code != 0 else runtime_code
    except OSError as error:
        raise ObserverLaunchError("cannot start observer processes") from error
    finally:
        for descriptor in descriptors:
            os.close(descriptor)
        try:
            if client is not None:
                _stop(client)
        finally:
            if runtime is not None and not runtime_shutdown_started:
                _finish_runtime(runtime)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    identity = parser.add_mutually_exclusive_group()
    identity.add_argument("--campaign", help="open this exact campaign UUID")
    identity.add_argument(
        "--new", action="store_true", help="start another campaign and preserve prior worlds"
    )
    parser.add_argument("--no-build", action="store_true", help="use existing native binaries")
    parser.add_argument("--distribution", action="store_true", help=argparse.SUPPRESS)
    parser.add_argument(
        "--stop-database", action="store_true", help="stop this preview's database and retain saves"
    )
    parser.add_argument(
        "--smoke",
        action="store_true",
        help="verify New, one period, restart, and native reads without a window",
    )
    parser.add_argument(
        "--defines",
        type=Path,
        default=DEFAULT_DEFINES,
        help="TOML values for new campaigns; existing campaigns use their saved parameters",
    )
    parser.add_argument(
        "--preset",
        choices=(
            "standard",
            "delayed",
            "shared-freight-ample",
            "shared-freight-constrained",
            "statewide-baseline",
            "statewide-freight-constraint",
            "statewide-packaging-shortage",
            "statewide-both",
        ),
        help="choose a new world's material preset; requires New rather than Open",
    )
    args = parser.parse_args(argv)
    try:
        environment = dict(os.environ)
        if args.stop_database:
            if not args.distribution:
                raise ObserverLaunchError(
                    "--stop-database is available in the downloadable preview"
                )
            _run(
                [*distribution_compose(ROOT), "stop", "babylon-pg"],
                ROOT,
                distribution_docker_environment(ROOT, environment),
                "preview database stop",
            )
            return 0
        if args.distribution:
            environment = distribution_environment(ROOT, environment)
        initial_target = None
        if args.smoke and args.campaign is not None:
            raise ObserverLaunchError("--smoke creates a new campaign and cannot use --campaign")
        if not args.smoke:
            initial_target = select_initial_target(
                environment,
                state_file=preference_path(environment),
                explicit=args.campaign,
                new=args.new,
                preset=args.preset,
            )
        runtime, client, credentials = prepare(
            ROOT, environment, no_build=args.no_build, distribution=args.distribution
        )
        writer_environment, reader_environment = child_environments(environment, credentials)
        if args.smoke:
            return check_installation(
                runtime,
                client,
                ROOT,
                writer_environment,
                reader_environment,
                args.defines.expanduser().resolve(),
                args.preset,
            )
        assert initial_target is not None
        # Never echo the ambient filter: field selectors may contain private values.
        print(f"Observer log targets enabled: {OBSERVER_CAPTURE_FILTER}", file=sys.stderr)
        return run_pair(
            runtime,
            client,
            ROOT,
            writer_environment,
            reader_environment,
            defines_path=args.defines.expanduser().resolve(),
            initial_target=initial_target,
        )
    except (ObserverLaunchError, OSError, ValueError, subprocess.SubprocessError) as error:
        print(f"Observer launch refused: {error}", file=sys.stderr)
        return 1
    except KeyboardInterrupt:
        return 130


if __name__ == "__main__":
    raise SystemExit(main())
