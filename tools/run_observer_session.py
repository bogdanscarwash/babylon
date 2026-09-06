#!/usr/bin/env python3
"""Launch one persistent Michigan observer session with separate read capabilities."""

from __future__ import annotations

import argparse
import ipaddress
import os
import subprocess
import sys
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
    preset: Literal["standard", "delayed"]


@dataclass(frozen=True)
class OpenCampaignTarget:
    """An explicit existing-only campaign request; it never authorizes founding."""

    campaign: UUID


def _new_target(campaign: UUID, preset: str | None) -> NewCampaignTarget:
    if preset is None or preset == "standard":
        return NewCampaignTarget(campaign, "standard")
    if preset == "delayed":
        return NewCampaignTarget(campaign, "delayed")
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


def bootstrap_required(runtime_dsn: str) -> bool:
    """Distinguish initial activation from an already active observer database.

    Rust validates the full ledger when opening a session. Re-running the
    pre-activation catalog census after installing observer views is invalid.
    """
    _target_parameters(runtime_dsn)
    try:
        with psycopg.connect(runtime_dsn, connect_timeout=10, options="") as connection:
            connection.execute("SET TRANSACTION READ ONLY")
            relation = connection.execute(
                "SELECT pg_catalog.to_regclass('babylon_meta.committed_tick_v2_authority_ledger')"
            ).fetchone()
            if relation is None or relation[0] is None:
                return True
            active = connection.execute(
                "SELECT 1 FROM babylon_meta.committed_tick_v2_authority_ledger "
                "WHERE ordinal = 2 AND state_tag = 2 AND activation_epoch = 11"
            ).fetchone()
            return active is None
    except psycopg.Error as error:
        raise ObserverLaunchError("cannot inspect local Rust activation status") from error


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
    root: Path, environment: Mapping[str, str], *, no_build: bool
) -> tuple[Path, Path, ReaderCredentials]:
    """Start the local DB and install Rust authority before any client connects."""
    runtime_dsn = environment.get("BABYLON_RUNTIME_DSN", DEFAULT_RUNTIME_DSN)
    _target_parameters(runtime_dsn)
    common = _clean_environment(environment)
    target = Path(environment.get("CARGO_TARGET_DIR", "target"))
    if not target.is_absolute():
        target = root / "rust" / target
    common["CARGO_TARGET_DIR"] = str(target)
    runtime, client = target / "debug" / "babylon-runtime", target / "debug" / "babylon-client"
    if not database_reachable(runtime_dsn):
        if _target_parameters(runtime_dsn) != _target_parameters(DEFAULT_RUNTIME_DSN):
            raise ObserverLaunchError(
                "requested local database is unavailable; start or create that database and retry"
            )
        _run(["mise", "run", "db:up"], root, common, "local database startup")
        if not database_reachable(runtime_dsn):
            raise ObserverLaunchError("default local database is still unavailable after db:up")
    if not no_build:
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
    if bootstrap_required(runtime_dsn):
        _run([str(runtime), "bootstrap"], root, writer, "Rust database bootstrap")
    _run([str(runtime), "observer-schema"], root, writer, "observer schema installation")
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
            cwd=root / "rust",
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
            cwd=root / "rust",
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
    parser.add_argument(
        "--defines",
        type=Path,
        default=DEFAULT_DEFINES,
        help="TOML values for new campaigns; existing campaigns use their saved parameters",
    )
    parser.add_argument(
        "--preset",
        choices=("standard", "delayed"),
        help="choose a new world's delivery preset; requires New rather than Open",
    )
    args = parser.parse_args(argv)
    try:
        environment = dict(os.environ)
        state_file = preference_path(environment)
        initial_target = select_initial_target(
            environment,
            state_file=state_file,
            explicit=args.campaign,
            new=args.new,
            preset=args.preset,
        )
        runtime, client, credentials = prepare(ROOT, environment, no_build=args.no_build)
        writer_environment, reader_environment = child_environments(environment, credentials)
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
    except ObserverLaunchError as error:
        print(f"Observer launch refused: {error}", file=sys.stderr)
        return 1
    except KeyboardInterrupt:
        return 130


if __name__ == "__main__":
    raise SystemExit(main())
