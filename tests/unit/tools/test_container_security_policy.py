"""Static contracts for the Babylon PostgreSQL container boundary."""

from __future__ import annotations

import json
import os
import stat
import subprocess
import sys
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[3]
DOCKERFILE_PATH = ROOT / "docker" / "postgres" / "Dockerfile"
ENTRYPOINT_PATCH_PATH = ROOT / "docker" / "postgres" / "patch-entrypoint.awk"
COMPOSE_PATH = ROOT / "docker-compose.yml"
GOTCHAS_PATH = ROOT / "docs" / "agents" / "gotchas.md"
MISE_PATH = ROOT / ".mise.toml"
TRIVYIGNORE_PATH = ROOT / ".trivyignore"
POSTGRES_RUNNER_PATH = ROOT / "tools" / "run_rust_postgres.sh"


def _run_with_mock_image(
    tmp_path: Path, requested_id: str | None, tag_id: str
) -> tuple[subprocess.CompletedProcess[str], list[str]]:
    """Stop before container creation; no command can reach the real Docker daemon."""
    log = tmp_path / "docker.log"
    docker = tmp_path / "docker"
    docker.write_text(
        "#!/usr/bin/env bash\n"
        'printf "%s\\n" "$*" >> "$MOCK_DOCKER_LOG"\n'
        'case "$1 $2" in\n'
        '  "container inspect") exit 1 ;;\n'
        '  "image inspect")\n'
        '    [ -n "$MOCK_TAG_IMAGE_ID" ] || exit 1\n'
        '    printf "%s\\n" "$MOCK_TAG_IMAGE_ID" ;;\n'
        '  "build --tag") exit 0 ;;\n'
        '  "run --detach") exit 70 ;;\n'
        '  "inspect --format") exit 1 ;;\n'
        "  *) exit 99 ;;\n"
        "esac\n",
        encoding="utf-8",
    )
    docker.chmod(0o755)
    psql = tmp_path / "psql"
    psql.write_text("#!/usr/bin/env bash\nexit 99\n", encoding="utf-8")
    psql.chmod(0o755)
    env = {
        **os.environ,
        "PATH": f"{tmp_path}:{os.environ['PATH']}",
        "MOCK_DOCKER_LOG": str(log),
        "MOCK_TAG_IMAGE_ID": tag_id,
        "BABYLON_POSTGRES_LIVE_FOCUS": "runtime_smoke",
    }
    env.pop("BABYLON_POSTGRES_IMAGE_ID", None)
    if requested_id is not None:
        env["BABYLON_POSTGRES_IMAGE_ID"] = requested_id
    result = subprocess.run(
        ["bash", str(POSTGRES_RUNNER_PATH)],
        env=env,
        capture_output=True,
        text=True,
        timeout=10,
        check=False,
    )
    return result, log.read_text().splitlines() if log.exists() else []


@pytest.mark.parametrize("requested_id", ["", "latest", "sha256:1234", "sha256:" + "A" * 64])
def test_prebuilt_postgres_rejects_malformed_identity_before_docker(
    tmp_path: Path, requested_id: str
) -> None:
    result, calls = _run_with_mock_image(tmp_path, requested_id, "sha256:" + "a" * 64)

    assert result.returncode == 2
    assert "BABYLON_POSTGRES_IMAGE_ID must be a complete sha256 image ID" in result.stderr
    assert calls == []


@pytest.mark.parametrize("tag_id", ["", "sha256:1234", "sha256:" + "b" * 64])
def test_prebuilt_postgres_rejects_missing_invalid_or_retagged_image(
    tmp_path: Path, tag_id: str
) -> None:
    result, calls = _run_with_mock_image(tmp_path, "sha256:" + "a" * 64, tag_id)

    assert result.returncode == 2
    assert "task-owned container did not start" not in result.stderr
    assert any(
        call == "image inspect --format {{.Id}} babylon-postgres-runtime:local" for call in calls
    )
    assert not any(call.startswith(("build ", "run ")) for call in calls)


def test_local_postgres_rejects_invalid_built_image_identity(tmp_path: Path) -> None:
    result, calls = _run_with_mock_image(tmp_path, None, "sha256:1234")

    assert result.returncode == 2
    assert "PostgreSQL image tag did not resolve to a complete sha256 image ID" in result.stderr
    assert len([call for call in calls if call.startswith("build ")]) == 1
    assert not any(call.startswith("run ") for call in calls)


@pytest.mark.parametrize("prebuilt", [False, True])
def test_postgres_starts_by_verified_immutable_image_id(tmp_path: Path, prebuilt: bool) -> None:
    image_id = "sha256:" + "a" * 64
    result, calls = _run_with_mock_image(tmp_path, image_id if prebuilt else None, image_id)

    assert result.returncode == 2
    assert "task-owned container did not start" in result.stderr
    builds = [call for call in calls if call.startswith("build ")]
    assert len(builds) == (0 if prebuilt else 1)
    starts = [call for call in calls if call.startswith("run ")]
    assert len(starts) == 1
    assert f" {image_id} postgres " in starts[0]
    assert " babylon-postgres-runtime:local postgres " not in starts[0]


def _run_mock_runtime(
    tmp_path: Path, focus: str, fault: str = ""
) -> tuple[subprocess.CompletedProcess[str], list[dict[str, object]]]:
    """Exercise runner control flow with no real database, compiler, or Docker access."""
    command = r"""
import json
import os
import sys
from pathlib import Path

name = Path(sys.argv[0]).name
args = sys.argv[1:]
state_path = Path(os.environ["MOCK_STATE"])
state = json.loads(state_path.read_text()) if state_path.exists() else {}
fault = os.environ["MOCK_FAULT"]
with Path(os.environ["MOCK_LOG"]).open("a") as log:
    log.write(json.dumps({"name": name, "args": args}) + "\n")

def save():
    state_path.write_text(json.dumps(state))

if name == "docker":
    if args[:2] == ["container", "inspect"]:
        sys.exit(0 if state.get("container") else 1)
    if args[:2] == ["volume", "inspect"]:
        sys.exit(0 if state.get("volume") else 1)
    if args[:2] == ["image", "inspect"]:
        print("sha256:" + "a" * 64)
    elif args[0] == "run":
        state.update(container=True, volume=True, canary=args[args.index("--label") + 1].split("=", 1)[1])
        save()
        print("b" * 64)
    elif args[0] == "inspect":
        if "Config.Labels" in args[2]:
            canary = "foreign" if fault == "foreign_owner" else state["canary"]
            print("b" * 64 + "|" + canary)
        else:
            print("owned-anonymous-volume")
    elif args[0] == "port":
        print("0.0.0.0:54321" if fault == "wildcard_port" else "127.0.0.1:54321")
    elif args[0] == "exec":
        if "|| '|' || pg_catalog.current_setting('server_version')" in args[-1]:
            print("16|16.9" if fault == "wrong_version" else "17|17.9")
        else:
            print("t")
    elif args[0] == "logs":
        print("mock PostgreSQL diagnostic")
    elif args[0] == "rm":
        assert args[-1] == "b" * 64, "cleanup must use the proved immutable container ID"
        if fault == "cleanup_failure":
            sys.exit(47)
        state["container"] = False
        state["volume"] = fault == "volume_survives"
        save()
    else:
        sys.exit(99)
elif name == "psql":
    query = args[-1]
    if query.startswith("SELECT 1,"):
        print("1|" + state["canary"])
    elif "babylon_meta.current_schema" in query:
        print("false|0|true|true" if fault == "invalid_schema_marker" else "true|0|true|true")
    elif "pg_catalog.pg_database" in query:
        print("0")
    elif "pg_catalog.pg_class" in query:
        print("100")
elif name == "babylon-runtime":
    assert "?options=" in os.environ["BABYLON_RUNTIME_DSN"]
    sys.exit(1)
elif name == "mise":
    if fault == "bootstrap_failure" and args == ["run", "db:bootstrap"]:
        sys.exit(42)
elif name == "cargo":
    assert os.environ["BABYLON_POSTGRES_DISPOSABLE_CANARY"] == state["canary"]
    assert os.environ["BABYLON_POSTGRES_TEST_DSN"] == "postgresql://test:test@127.0.0.1:54321/postgres"
    if any("material_runtime::writer_bounds_tests::" in arg for arg in args):
        assert os.environ["BABYLON_RUNTIME_DSN"] == "postgresql://test:test@127.0.0.1:54321/babylon_test"
else:
    sys.exit(99)
"""
    commands = tmp_path / "bin"
    commands.mkdir()
    for name in ("docker", "psql", "cargo", "mise", "babylon-runtime"):
        executable = commands / name
        executable.write_text(f"#!{sys.executable}\n" + command)
        executable.chmod(0o755)
    log = tmp_path / "commands.jsonl"
    result = subprocess.run(
        ["bash", str(POSTGRES_RUNNER_PATH)],
        env={
            **os.environ,
            "PATH": f"{commands}:{os.environ['PATH']}",
            "CARGO_TARGET_DIR": str(tmp_path / "target"),
            "BABYLON_POSTGRES_IMAGE_ID": "sha256:" + "a" * 64,
            "BABYLON_POSTGRES_LIVE_FOCUS": focus,
            "MOCK_STATE": str(tmp_path / "state.json"),
            "MOCK_LOG": str(log),
            "MOCK_FAULT": fault,
        },
        capture_output=True,
        text=True,
        timeout=20,
        check=False,
    )
    return result, [
        json.loads(line) for line in log.read_text().splitlines()
    ] if log.exists() else []


@pytest.mark.parametrize(
    "focus",
    [
        "runtime_smoke",
        "reference_integrity",
        "runtime",
        "archive",
        "reader",
        "statewide_synthetic",
        "statewide_qualified",
        "client",
    ],
)
def test_current_runtime_focuses_finish_with_checked_owned_cleanup(
    tmp_path: Path, focus: str
) -> None:
    result, calls = _run_mock_runtime(tmp_path, focus)
    assert result.returncode == 0, result.stderr
    assert "Rust PostgreSQL cleanup verified:" in result.stdout
    assert "phase complete:" in result.stdout
    removals = [call for call in calls if call["name"] == "docker" and call["args"][0] == "rm"]
    assert len(removals) == 1
    assert removals[0]["args"][-1] == "b" * 64
    assert any(call["args"][:2] == ["volume", "inspect"] for call in calls)
    writer_probes = [
        call
        for call in calls
        if call["name"] == "cargo"
        and "material_runtime::writer_bounds_tests::live_bounded_writer_verifies_authority_and_timeouts_in_read_only_transaction"
        in call["args"]
    ]
    assert len(writer_probes) == (1 if focus == "runtime" else 0)
    if focus in {"statewide_synthetic", "statewide_qualified"}:
        selected = "statewide::" if focus == "statewide_synthetic" else "statewide_qualified::"
        tests = [
            call["args"] for call in calls if call["name"] == "cargo" and call["args"][0] == "test"
        ]
        assert len(tests) == 1
        assert selected in tests[0] and "--ignored" in tests[0]
    if focus == "reference_integrity":
        assert not any(call["name"] == "mise" for call in calls)
        assert any(
            "reference_integrity" in call["args"] for call in calls if call["name"] == "cargo"
        )


@pytest.mark.parametrize(
    "fault",
    [
        "wildcard_port",
        "wrong_version",
        "invalid_schema_marker",
        "bootstrap_failure",
        "cleanup_failure",
        "volume_survives",
    ],
)
def test_runtime_refusals_preserve_failure_and_cleanup_only_the_owned_id(
    tmp_path: Path, fault: str
) -> None:
    result, calls = _run_mock_runtime(tmp_path, "runtime", fault)
    assert result.returncode != 0
    assert (
        "cleanup verified:" not in result.stdout
        or "status=42" in result.stdout
        or "status=1" in result.stdout
    )
    removals = [call for call in calls if call["name"] == "docker" and call["args"][0] == "rm"]
    assert removals
    assert all(call["args"][-1] == "b" * 64 for call in removals)


def test_unproved_container_owner_is_never_removed(tmp_path: Path) -> None:
    result, calls = _run_mock_runtime(tmp_path, "runtime", "foreign_owner")
    assert result.returncode != 0
    assert "created container identity was not proved" in result.stderr
    assert not any(call["args"][0] == "rm" for call in calls if call["name"] == "docker")


@pytest.mark.parametrize(
    "focus", ["", "pr", "schema_epoch_matrix", "runtime_census_v2", "h3_shadow_backfill", "unknown"]
)
def test_retired_or_unknown_focus_is_refused_before_docker(tmp_path: Path, focus: str) -> None:
    result, calls = _run_mock_runtime(tmp_path, focus)
    assert result.returncode == 2
    assert "unsupported live focus" in result.stderr
    assert calls == []


POSTGIS_ALPINE = (
    "postgis/postgis:17-3.5-alpine@"
    "sha256:08f4b1e1f4a571008c60272ceb9e0d1f9f8f643792d006b74a35b1bec44c2218"
)
LINEAGE = (
    "babylon-postgres-lineage-v1|postgres=17|locale-provider=builtin|"
    "locale=C.UTF-8|encoding=UTF8|postgis=3.5.7|h3=4.5.0|"
    "h3_postgis=4.5.0|vector=0.8.5"
)
NEW_DEFAULT_VOLUME = "babylon-pg-alpine-c-utf8-v1"

UPSTREAM_ENTRYPOINT_SHAPE = "\n".join(
    (
        "_main() {",
        '\tif [ "$1" = \'postgres\' ] && ! _pg_want_help "$@"; then',
        "\t\tdocker_setup_env",
        "\t\t# setup data directories and permissions (when run as root)",
        "\t\tdocker_create_db_directories",
        "\t\tif [ \"$(id -u)\" = '0' ]; then",
        "\t\t\t# then restart script as postgres user",
        '\t\t\texec gosu postgres "$BASH_SOURCE" "$@"',
        "\t\tfi",
        '\t\tif [ -z "$DATABASE_ALREADY_EXISTS" ]; then',
        "\t\t\tdocker_verify_minimum_env",
        "\t\t\tdocker_error_old_databases",
        "\t\t\tls /docker-entrypoint-initdb.d/ > /dev/null",
        "\t\t\tdocker_init_database_dir",
        '\t\t\tpg_setup_hba_conf "$@"',
        '\t\t\texport PGPASSWORD="${PGPASSWORD:-$POSTGRES_PASSWORD}"',
        '\t\t\tdocker_temp_server_start "$@"',
        "\t\t\tdocker_setup_db",
        "\t\t\tdocker_process_init_files /docker-entrypoint-initdb.d/*",
        "\t\t\tdocker_temp_server_stop",
        "\t\t\tunset PGPASSWORD",
        "\t\tfi",
        '\t\tunset "${!POSTGRES_@}"',
        "\tfi",
        '\texec "$@"',
        "}",
        "",
    )
)


def _dockerfile() -> str:
    return DOCKERFILE_PATH.read_text(encoding="utf-8")


def _patch_entrypoint(source: str = UPSTREAM_ENTRYPOINT_SHAPE) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["awk", "-f", str(ENTRYPOINT_PATCH_PATH)],
        input=source,
        check=False,
        capture_output=True,
        text=True,
    )


def _write_patched_entrypoint(tmp_path: Path) -> Path:
    result = _patch_entrypoint()
    assert result.returncode == 0, result.stderr
    entrypoint = tmp_path / "docker-entrypoint.sh"
    entrypoint.write_text(result.stdout, encoding="utf-8")
    entrypoint.chmod(0o755)
    return entrypoint


def test_postgres_extensions_share_one_pinned_alpine_lineage() -> None:
    dockerfile = _dockerfile()
    from_lines = [line for line in dockerfile.splitlines() if line.startswith("FROM ")]

    assert from_lines == [
        f"FROM {POSTGIS_ALPINE} AS extension-builder",
        f"FROM {POSTGIS_ALPINE}",
    ]
    for forbidden in ("bullseye", "gcc:", "apt-archive", ".deb", "dpkg"):
        assert forbidden not in dockerfile


def test_postgres_extensions_use_checksum_pinned_official_sources() -> None:
    dockerfile = _dockerfile()
    expected_inputs = {
        "https://github.com/postgis/h3-pg/archive/refs/tags/v4.5.0.tar.gz": (
            "sha256:c54c119e1d9a578d5cbcce22f6c66dab2b5a45219fc2b260619807f7f061e53a"
        ),
        "https://github.com/uber/h3/archive/refs/tags/v4.5.0.tar.gz": (
            "sha256:0da8a392a6ff77e76b60e6a331a49497d0935b6b7b6899da7a3e2786139b0441"
        ),
        "https://github.com/pgvector/pgvector/archive/refs/tags/v0.8.5.tar.gz": (
            "sha256:6f88a5cbdde31666f4b6c1a6b75c51dcbeffe58f9a7d2b26e502d5a6e5e14d44"
        ),
    }

    for source, checksum in expected_inputs.items():
        assert f"ADD --checksum={checksum}" in dockerfile
        assert source in dockerfile
    assert dockerfile.count("ADD --checksum=sha256:") == len(expected_inputs)


def test_final_image_copies_only_staged_extension_artifacts() -> None:
    copy_lines = [line for line in _dockerfile().splitlines() if line.startswith("COPY --from=")]

    assert copy_lines == [
        "COPY --from=extension-builder \\",
        "COPY --from=extension-builder \\",
    ]
    dockerfile = _dockerfile()
    assert (
        "/tmp/extension-install/usr/local/lib/postgresql/ /usr/local/lib/postgresql/"
    ) in dockerfile
    assert (
        "/tmp/extension-install/usr/local/share/postgresql/extension/ "
        "/usr/local/share/postgresql/extension/"
    ) in dockerfile


def test_entrypoint_switches_directly_to_su_exec_after_root_bootstrap() -> None:
    dockerfile = _dockerfile()

    assert "apk add --no-cache su-exec=0.3-r0" in dockerfile
    assert (
        "--mount=type=bind,source=patch-entrypoint.awk,"
        "target=/tmp/patch-entrypoint.awk,readonly" in dockerfile
    )
    assert 'awk -f /tmp/patch-entrypoint.awk "$entrypoint" > "$patched_entrypoint"' in dockerfile
    assert 'expected_gosu=\'exec gosu postgres "$BASH_SOURCE" "$@"\'' in dockerfile
    assert 'expected_su_exec=\'exec su-exec postgres "$BASH_SOURCE" "$@"\'' in dockerfile
    assert "sed -i" not in dockerfile
    assert "! grep -Fq 'gosu' \"$entrypoint\"" in dockerfile
    assert "rm /usr/local/bin/gosu" in dockerfile
    assert "test ! -e /usr/local/bin/gosu" in dockerfile
    assert "ENTRYPOINT" not in dockerfile
    assert not any(line.startswith("USER ") for line in dockerfile.splitlines())


def test_entrypoint_rejects_unmarked_pgdata_before_any_directory_mutation() -> None:
    result = _patch_entrypoint()

    assert result.returncode == 0, result.stderr
    patched = result.stdout
    setup = patched.index("\t\tdocker_setup_env")
    marker_check = patched.index(
        f'\t\tlocal babylon_lineage="{LINEAGE}"',
        setup,
    )
    create_directories = patched.index("\t\tdocker_create_db_directories")
    assert setup < marker_check < create_directories
    for required in (
        'local babylon_lineage_marker="$PGDATA/.babylon-postgres-lineage-v1"',
        'if [ -e "$PGDATA" ] && [ ! -d "$PGDATA" ]; then',
        'elif [ -e "$babylon_lineage_marker" ]; then',
        '[ "$(stat -c %a "$babylon_lineage_marker")" = "444" ]',
        '[ "$(cat "$PGDATA/PG_VERSION")" = "17" ]',
        'find "$PGDATA" -mindepth 1 -maxdepth 1 -print -quit',
        'if [ "$babylon_lineage_state" = "refuse" ]; then',
        "No PGDATA ownership, mode, or content mutation was attempted.",
        "exit 1",
    ):
        assert required in patched


def test_entrypoint_forces_builtin_locale_and_marks_only_completed_init() -> None:
    result = _patch_entrypoint()

    assert result.returncode == 0, result.stderr
    patched = result.stdout
    init_branch = patched.index('\t\tif [ -z "$DATABASE_ALREADY_EXISTS" ]; then')
    init_args = patched.index(
        "--locale-provider=builtin --builtin-locale=C.UTF-8 --encoding=UTF8",
        init_branch,
    )
    init_files = patched.index(
        "\t\t\tdocker_process_init_files /docker-entrypoint-initdb.d/*",
        init_args,
    )
    temp_stop = patched.index("\t\t\tdocker_temp_server_stop", init_files)
    unset_password = patched.index("\t\t\tunset PGPASSWORD", temp_stop)
    marker_temp = patched.index(
        '\t\t\tlocal babylon_lineage_tmp="${babylon_lineage_marker}.tmp.$$"',
        unset_password,
    )
    marker_write = patched.index(
        'printf "%s\\n" "$babylon_lineage" > "$babylon_lineage_tmp"',
        marker_temp,
    )
    marker_chmod = patched.index('chmod 0444 "$babylon_lineage_tmp"', marker_write)
    marker_move = patched.index(
        'mv "$babylon_lineage_tmp" "$babylon_lineage_marker"',
        marker_chmod,
    )
    assert init_branch < init_args < init_files < temp_stop < unset_password
    assert unset_password < marker_temp < marker_write < marker_chmod < marker_move


def test_entrypoint_patch_fails_closed_when_upstream_shape_drifts() -> None:
    missing_anchor = UPSTREAM_ENTRYPOINT_SHAPE.replace(
        "\t\tdocker_create_db_directories\n",
        "",
        1,
    )
    duplicate_anchor = UPSTREAM_ENTRYPOINT_SHAPE.replace(
        "\t\tdocker_setup_env\n",
        "\t\tdocker_setup_env\n\t\tdocker_setup_env\n",
        1,
    )

    for source in (missing_anchor, duplicate_anchor):
        result = _patch_entrypoint(source)
        assert result.returncode == 42
        assert "refusing unexpected upstream entrypoint shape" in result.stderr


def test_unmarked_pgdata_rejection_preserves_content_mode_and_skips_mutator(
    tmp_path: Path,
) -> None:
    entrypoint = _write_patched_entrypoint(tmp_path)
    pgdata = tmp_path / "legacy-pgdata"
    pgdata.mkdir(mode=0o751)
    version = pgdata / "PG_VERSION"
    sentinel = pgdata / "sentinel.bin"
    version.write_bytes(b"17\n")
    sentinel.write_bytes(b"legacy-bytes\x00must-not-change")
    sentinel.chmod(0o640)
    mutator_receipt = tmp_path / "mutator-called"
    before = (
        stat.S_IMODE(pgdata.stat().st_mode),
        stat.S_IMODE(version.stat().st_mode),
        stat.S_IMODE(sentinel.stat().st_mode),
        version.read_bytes(),
        sentinel.read_bytes(),
    )

    result = subprocess.run(
        [
            "bash",
            "-c",
            """
source "$1"
_pg_want_help() { return 1; }
docker_setup_env() { DATABASE_ALREADY_EXISTS=true; }
docker_create_db_directories() { printf 'called\n' > "$2"; }
export PGDATA="$3" PG_MAJOR=17
_main postgres
""",
            "per272-lineage-test",
            str(entrypoint),
            str(mutator_receipt),
            str(pgdata),
        ],
        check=False,
        capture_output=True,
        text=True,
    )

    after = (
        stat.S_IMODE(pgdata.stat().st_mode),
        stat.S_IMODE(version.stat().st_mode),
        stat.S_IMODE(sentinel.stat().st_mode),
        version.read_bytes(),
        sentinel.read_bytes(),
    )
    assert result.returncode == 1
    assert "refusing unrecognized PostgreSQL data directory" in result.stderr
    assert "No PGDATA ownership, mode, or content mutation was attempted." in result.stderr
    assert not mutator_receipt.exists()
    assert after == before


def test_fresh_init_appends_fixed_locale_args_and_writes_exact_marker_last(
    tmp_path: Path,
) -> None:
    entrypoint = _write_patched_entrypoint(tmp_path)
    pgdata = tmp_path / "fresh-pgdata"
    pgdata.mkdir(mode=0o700)
    fake_bin = tmp_path / "bin"
    fake_bin.mkdir()
    fake_postgres = fake_bin / "postgres"
    fake_postgres.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
    fake_postgres.chmod(0o755)
    events = tmp_path / "events"
    init_args = tmp_path / "init-args"

    result = subprocess.run(
        [
            "bash",
            "-c",
            """
source "$1"
_pg_want_help() { return 1; }
docker_setup_env() {
    DATABASE_ALREADY_EXISTS=
    POSTGRES_INITDB_ARGS="--data-checksums"
    POSTGRES_PASSWORD=test
    POSTGRES_USER=test
    POSTGRES_DB=test
    POSTGRES_HOST_AUTH_METHOD=
}
docker_create_db_directories() { printf 'create-directories\n' >> "$EVENTS"; }
docker_verify_minimum_env() { printf 'verify-env\n' >> "$EVENTS"; }
docker_error_old_databases() { printf 'verify-layout\n' >> "$EVENTS"; }
ls() { :; }
docker_init_database_dir() {
    printf '%s\n' "$POSTGRES_INITDB_ARGS" > "$INIT_ARGS"
    printf 'init-database\n' >> "$EVENTS"
}
pg_setup_hba_conf() { printf 'hba\n' >> "$EVENTS"; }
docker_temp_server_start() { printf 'temp-start\n' >> "$EVENTS"; }
docker_setup_db() { printf 'setup-db\n' >> "$EVENTS"; }
docker_process_init_files() { printf 'init-files\n' >> "$EVENTS"; }
docker_temp_server_stop() { printf 'temp-stop\n' >> "$EVENTS"; }
export PGDATA="$2" PG_MAJOR=17 EVENTS="$3" INIT_ARGS="$4"
export PATH="$5:$PATH"
_main postgres
""",
            "per272-lineage-test",
            str(entrypoint),
            str(pgdata),
            str(events),
            str(init_args),
            str(fake_bin),
        ],
        check=False,
        capture_output=True,
        text=True,
    )

    marker = pgdata / ".babylon-postgres-lineage-v1"
    assert result.returncode == 0, result.stderr
    assert init_args.read_text(encoding="utf-8") == (
        "--data-checksums --locale-provider=builtin --builtin-locale=C.UTF-8 --encoding=UTF8\n"
    )
    assert events.read_text(encoding="utf-8").splitlines() == [
        "create-directories",
        "verify-env",
        "verify-layout",
        "init-database",
        "hba",
        "temp-start",
        "setup-db",
        "init-files",
        "temp-stop",
    ]
    assert marker.read_bytes() == f"{LINEAGE}\n".encode()
    assert stat.S_IMODE(marker.stat().st_mode) == 0o444
    assert list(pgdata.glob(".babylon-postgres-lineage-v1.tmp.*")) == []


def test_compose_rotates_the_default_volume_without_touching_the_legacy_name() -> None:
    compose = COMPOSE_PATH.read_text(encoding="utf-8")

    assert f"${{BABYLON_PG_DATA:-{NEW_DEFAULT_VOLUME}}}:/var/lib/postgresql/data" in compose
    assert f"  {NEW_DEFAULT_VOLUME}:" in compose
    assert "${BABYLON_PG_DATA:-babylon-pg-data}" not in compose
    assert "\n  babylon-pg-data:\n" not in compose


def test_local_start_tasks_always_build_the_declared_postgres_image() -> None:
    mise = MISE_PATH.read_text(encoding="utf-8")

    for task_name in ("db:up", "db:start"):
        task_start = mise.index(f'[tasks."{task_name}"]')
        task_end = mise.index("\n[tasks.", task_start + 1)
        task = mise[task_start:task_end]
        assert 'run = "docker compose up --build -d --wait babylon-pg"' in task


def test_compose_healthcheck_proves_the_exact_runtime_and_cluster_lineage() -> None:
    compose = COMPOSE_PATH.read_text(encoding="utf-8")

    for required in (
        "$${PGDATA}/.babylon-postgres-lineage-v1",
        LINEAGE,
        "server_version_num') = '170011'",
        "pg_encoding_to_char(database_row.encoding) = 'UTF8'",
        "database_row.datlocprovider = 'b'",
        "database_row.datlocale = 'C.UTF-8'",
        "extension_row.extversion = '3.5.7'",
        "extension_row.extversion = '0.8.5'",
        "available.default_version = '4.5.0'",
        "available.name = 'h3'",
        "available.name = 'h3_postgis'",
    ):
        assert required in compose
    assert "pg_isready -U test -d babylon_test" in compose


def test_operational_contract_is_fresh_init_or_offline_logical_restore_only() -> None:
    dockerfile = _dockerfile()
    gotchas = GOTCHAS_PATH.read_text(encoding="utf-8")
    mise = MISE_PATH.read_text(encoding="utf-8")

    assert NEW_DEFAULT_VOLUME in gotchas
    assert "babylon-pg-data" in gotchas
    assert "offline logical dump/restore" in gotchas
    assert "never attach" in gotchas
    assert NEW_DEFAULT_VOLUME in mise
    for forbidden in (
        "REINDEX DATABASE",
        "REFRESH COLLATION VERSION",
        "ALTER EXTENSION",
        "postgis_extensions_upgrade",
    ):
        assert forbidden not in dockerfile


def test_lineage_marker_is_a_recoverability_fence_not_security_attestation() -> None:
    dockerfile = _dockerfile()
    gotchas = GOTCHAS_PATH.read_text(encoding="utf-8")
    normalized_dockerfile = " ".join(dockerfile.split())
    normalized_gotchas = " ".join(gotchas.split())

    assert "accidental/recoverability lineage fence" in normalized_gotchas
    assert "not an adversarial attestation" in normalized_gotchas
    assert "fail-before-chown" in normalized_gotchas
    assert "No wrapper or fallback" in normalized_gotchas
    assert "recoverability fence" in normalized_dockerfile
    assert "security attestation" in normalized_dockerfile


def test_build_contract_is_pinned_and_verified_but_not_byte_reproducible() -> None:
    dockerfile = _dockerfile()
    normalized = " ".join(line.removeprefix("#").strip() for line in dockerfile.splitlines())

    for claim in (
        "digest-pinned base",
        "checksum-pinned source archives",
        "exact final-stage runtime package revisions",
        "behavioral version checks",
        "does not claim byte-identical image rebuilds",
        "Builder-only APK packages are repository-resolved",
    ):
        assert claim in normalized
    assert "apk add --no-cache --virtual .extension-build-deps" in dockerfile
    assert "$DOCKER_PG_LLVM_DEPS" in dockerfile


def test_final_image_pins_available_high_severity_security_fixes() -> None:
    dockerfile = _dockerfile()

    for package in (
        "giflib=5.2.2-r2",
        "libcrypto3=3.5.8-r0",
        "libssl3=3.5.8-r0",
    ):
        assert package in dockerfile


def test_trivy_policy_retains_only_the_root_bootstrap_shape_exception() -> None:
    active_entries = [
        line.strip()
        for line in TRIVYIGNORE_PATH.read_text(encoding="utf-8").splitlines()
        if line.strip() and not line.lstrip().startswith("#")
    ]

    assert active_entries == ["DS-0002"]
