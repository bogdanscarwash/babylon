#!/usr/bin/env python3
"""Launch one durable Michigan observer campaign with separate read capabilities."""

from __future__ import annotations

import argparse
import ipaddress
import os
import subprocess
import sys
from collections.abc import Mapping
from dataclasses import dataclass
from pathlib import Path
from tempfile import NamedTemporaryFile
from uuid import UUID, uuid4

import psycopg
from psycopg import sql
from psycopg.conninfo import conninfo_to_dict, make_conninfo

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_RUNTIME_DSN = "host=127.0.0.1 port=5433 dbname=babylon_test user=test password=test"
NEW_CAMPAIGN_EXIT = 20
REOPEN_CAMPAIGN_EXIT = 21
NEW_DELAYED_CAMPAIGN_EXIT = 22
OPEN_SELECTED_CAMPAIGN_EXIT = 23
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


def select_campaign(
    environment: Mapping[str, str],
    *,
    state_file: Path,
    explicit: str | None = None,
    new: bool = False,
) -> UUID:
    """Select an exact campaign without replacing a prior campaign or preference."""
    if new:
        return uuid4()
    selected = explicit if explicit is not None else environment.get("BABYLON_CAMPAIGN_ID")
    if selected is not None:
        return _campaign(selected)
    try:
        if state_file.stat().st_size > 64:
            raise ObserverLaunchError("saved campaign preference is oversized")
        return _campaign(state_file.read_text(encoding="ascii").strip())
    except FileNotFoundError:
        return uuid4()
    except (OSError, UnicodeError) as error:
        raise ObserverLaunchError("cannot read saved campaign preference") from error


def save_campaign(state_file: Path, campaign: UUID) -> None:
    """Atomically replace the continuation pointer; campaign data stays in Postgres."""
    temporary: Path | None = None
    try:
        state_file.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
        with NamedTemporaryFile(
            mode="w", encoding="ascii", dir=state_file.parent, delete=False
        ) as output:
            temporary = Path(output.name)
            output.write(f"{campaign}\n")
            output.flush()
            os.fsync(output.fileno())
        os.replace(temporary, state_file)
    except OSError as error:
        raise ObserverLaunchError("cannot save campaign continuation preference") from error
    finally:
        if temporary is not None:
            temporary.unlink(missing_ok=True)


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
            "BABYLON_DOSSIER_DEMO_PASSWORD",
        }
    }


def child_environments(
    environment: Mapping[str, str],
    campaign: UUID,
    credentials: ReaderCredentials,
) -> tuple[dict[str, str], dict[str, str]]:
    """Bind the same campaign to a writer child and a separate read-only client."""
    common = _clean_environment(environment)
    common["BABYLON_CAMPAIGN_ID"] = str(campaign)
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
    preset: str | None = None,
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
        runtime_args = [str(runtime_binary), "session", "--stdio"]
        if preset is not None:
            if preset not in {"standard", "delayed"}:
                raise ObserverLaunchError("unknown material scenario preset")
            runtime_args.extend(["--preset", preset])
        runtime = subprocess.Popen(
            runtime_args,
            cwd=root / "rust",
            env=dict(runtime_environment),
            stdin=requests[0],
            stdout=responses[1],
            stderr=None,
            close_fds=True,
        )
        client = subprocess.Popen(
            [str(client_binary)],
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


def run_campaigns(
    runtime_binary: Path,
    client_binary: Path,
    root: Path,
    campaign: UUID,
    state_file: Path,
    environment: Mapping[str, str],
    credentials: ReaderCredentials,
    *,
    preset: str | None = None,
) -> int:
    """Honor in-game New Campaign without overwriting any prior campaign."""
    while True:
        save_campaign(state_file, campaign)
        runtime, client = child_environments(environment, campaign, credentials)
        # Never echo the raw ambient filter: field selectors may contain private values.
        print(f"Observer log targets enabled: {OBSERVER_CAPTURE_FILTER}", file=sys.stderr)
        result = run_pair(runtime_binary, client_binary, root, runtime, client, preset=preset)
        if result in {NEW_CAMPAIGN_EXIT, NEW_DELAYED_CAMPAIGN_EXIT}:
            campaign = uuid4()
            preset = "delayed" if result == NEW_DELAYED_CAMPAIGN_EXIT else "standard"
        elif result == REOPEN_CAMPAIGN_EXIT:
            # The runtime discovers the durable preset and reconciles its tail.
            preset = None
        elif result == OPEN_SELECTED_CAMPAIGN_EXIT:
            # The native catalog writes only this user-local continuation pointer.
            # Ignore a launch-time campaign environment override after selection.
            if not state_file.is_file():
                raise ObserverLaunchError("selected campaign preference is absent")
            campaign = select_campaign({}, state_file=state_file)
            preset = None
        else:
            return result


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    identity = parser.add_mutually_exclusive_group()
    identity.add_argument("--campaign", help="open this exact campaign UUID")
    identity.add_argument(
        "--new", action="store_true", help="start another campaign and preserve prior worlds"
    )
    parser.add_argument("--no-build", action="store_true", help="use existing native binaries")
    parser.add_argument(
        "--preset",
        choices=("standard", "delayed"),
        help="choose a new world's delivery preset; an existing world must match exactly",
    )
    args = parser.parse_args(argv)
    try:
        environment = dict(os.environ)
        state_file = preference_path(environment)
        campaign = select_campaign(
            environment, state_file=state_file, explicit=args.campaign, new=args.new
        )
        runtime, client, credentials = prepare(ROOT, environment, no_build=args.no_build)
        return run_campaigns(
            runtime,
            client,
            ROOT,
            campaign,
            state_file,
            environment,
            credentials,
            preset=args.preset,
        )
    except ObserverLaunchError as error:
        print(f"Observer launch refused: {error}", file=sys.stderr)
        return 1
    except KeyboardInterrupt:
        return 130


if __name__ == "__main__":
    raise SystemExit(main())
