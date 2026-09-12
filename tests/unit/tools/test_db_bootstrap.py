"""Current Rust bootstrap, reference-input, and disposable-host contracts."""

from __future__ import annotations

import tomllib
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[3]
BOOTSTRAP_CALLERS = (
    ".github/workflows/ci.yml",
    ".github/workflows/main.yml",
    ".github/workflows/nightly-michigan-smoke.yml",
    ".github/workflows/weekly-pg-integration.yml",
)
DIRECT_BOOTSTRAP_CALLERS = (
    ".github/workflows/main.yml",
    ".github/workflows/nightly-michigan-smoke.yml",
)
LIBPQ_TARGET_ENV = (
    "PGHOST",
    "PGHOSTADDR",
    "PGPORT",
    "PGDATABASE",
    "PGOPTIONS",
    "PGSERVICE",
    "PGSERVICEFILE",
    "PGSYSCONFDIR",
)
LOCAL_BOOTSTRAP_DSN = "host=127.0.0.1 port=5433 dbname=babylon_test user=test password=test"


def _mise_task(name: str) -> str:
    text = (ROOT / ".mise.toml").read_text(encoding="utf-8")
    for header in (f'[tasks."{name}"]', f"[tasks.{name}]"):
        if header in text:
            return text.split(header, maxsplit=1)[1].split("\n[tasks.", maxsplit=1)[0]
    raise AssertionError(f"missing Mise task: {name}")


def test_db_bootstrap_has_one_rust_owned_construction_root() -> None:
    task = _mise_task("db:bootstrap")

    build = task.index("cargo build -p babylon-persistence --bin babylon-runtime --locked")
    bootstrap = task.index("babylon-runtime bootstrap")
    assert build < bootstrap
    assert task.count("babylon-runtime bootstrap") == 1
    assert "BABYLON_RUNTIME_DSN" in task
    assert "host=127.0.0.1" in task
    assert "host=localhost" not in task
    for retired_root in (
        "BABYLON_SCHEMA_EPOCH_DSN",
        "babylon-schema-epoch",
        "POSTGRES_SCHEMA_DDL",
        "ensure_ddl_applied",
        "_apply_migrations",
        "uv run python",
    ):
        assert retired_root not in task


def test_db_bootstrap_clears_inherited_libpq_targets_before_any_database_access() -> None:
    task = _mise_task("db:bootstrap")

    unset_lines = [line.strip() for line in task.splitlines() if line.strip().startswith("unset ")]
    assert unset_lines == [f"unset {' '.join(LIBPQ_TARGET_ENV)}"]
    sanitization = task.index(unset_lines[0])
    build = task.index("cargo build -p babylon-persistence --bin babylon-runtime --locked")
    bootstrap = task.index("babylon-runtime bootstrap")
    assert sanitization < build < bootstrap


def test_repository_cargo_concurrency_matches_the_host_contract() -> None:
    mise = (ROOT / ".mise.toml").read_text(encoding="utf-8")

    assert 'CARGO_BUILD_JOBS = "4"' in mise
    assert 'CARGO_BUILD_JOBS = "8"' not in mise


def test_sccache_uses_the_repository_local_policy_cache() -> None:
    mise = (ROOT / ".mise.toml").read_text(encoding="utf-8")
    environment = (ROOT / ".codex/environments/environment.toml").read_text(encoding="utf-8")
    policy = (ROOT / ".codex/host/policy.sh").read_text(encoding="utf-8")

    assert "SCCACHE_DIR" not in mise
    assert "codex_rust_dispatcher_bin" in environment
    assert (
        'export PATH="$codex_rust_dispatcher_bin:$codex_rust_cargo_home/bin:$PATH"' in environment
    )
    assert "CODEX_RUST_SCCACHE_POLICY_KEY=0.17.0-p2" in policy
    assert "printf '%s/sccache/%s/%s\\n'" in policy
    assert "/media/user/data/sccache" not in mise


def test_native_ci_and_setup_document_the_bevy_development_dependencies() -> None:
    workflow = (ROOT / ".github/workflows/ci.yml").read_text(encoding="utf-8")
    guide = (ROOT / "SETUP_GUIDE.md").read_text(encoding="utf-8")
    assert "tools/install_ci_apt_packages.sh" in workflow
    for package in ("libasound2-dev", "libudev-dev", "libwayland-dev", "libxkbcommon-dev"):
        assert package in workflow
        assert package in guide


def test_local_bootstrap_callers_expose_the_single_rust_authority_root() -> None:
    setup = _mise_task("setup")
    assert "PostgreSQL 17" in setup
    assert "Postgres 16" not in setup

    for task_name in ("setup", "clean:testdb"):
        task = _mise_task(task_name)
        assert "cargo build -p babylon-persistence --bin babylon-runtime --locked" in task
        assert f'BABYLON_RUNTIME_DSN="{LOCAL_BOOTSTRAP_DSN}" babylon-runtime bootstrap' in task
        assert "mise run db:bootstrap" not in task

    integration = _mise_task("test:int-pg")
    assert f'export BABYLON_RUNTIME_DSN="{LOCAL_BOOTSTRAP_DSN}"' in integration
    assert (
        "python3 tools/devtools/worktree_campaign.py --purpose test-int-pg --fresh" in integration
    )
    assert "babylon-runtime bootstrap" in integration
    assert "babylon-runtime michigan-smoke" in integration
    assert "uv run pytest" not in integration

    michigan_smoke = _mise_task("qa:michigan-rollover-smoke")
    assert 'export PATH="$PWD/rust/target/debug:$PATH"' in michigan_smoke
    assert (
        "python3 tools/devtools/worktree_campaign.py --purpose qa-michigan-rollover-smoke --fresh"
    ) in michigan_smoke
    assert "babylon-runtime michigan-smoke" in michigan_smoke

    direct_bootstrap = _mise_task("db:bootstrap")
    assert (
        f'export BABYLON_RUNTIME_DSN="${{BABYLON_RUNTIME_DSN:-{LOCAL_BOOTSTRAP_DSN}}}"'
    ) in direct_bootstrap

    for relative in ("README.md", "SETUP_GUIDE.md"):
        guide = (ROOT / relative).read_text(encoding="utf-8")
        assert "rustup" in guide
        assert "mise run setup" in guide
        assert "requires [Nix]" not in guide


def test_runtime_cli_uses_current_preflight_and_bootstrap() -> None:
    source = (ROOT / "rust/crates/babylon-persistence/src/bin/babylon-runtime.rs").read_text(
        encoding="utf-8"
    )

    assert 'const DSN_ENV: &str = "BABYLON_RUNTIME_DSN";' in source
    assert "std::env::args_os().skip(1)" in source
    assert "Command::Preflight =>" in source
    assert "preflight_current_schema(config)" in source
    assert "Command::Bootstrap" in source
    assert "bootstrap_current_runtime(config)" in source


def test_bootstrap_validates_exact_reference_inputs_before_database_construction() -> None:
    persistence = ROOT / "rust/crates/babylon-persistence"
    bootstrap = (persistence / "src/bootstrap.rs").read_text(encoding="utf-8")
    source_validation = bootstrap.index("representative_h3_reference_cohort()")
    foundation_validation = bootstrap.index("michigan_dynamic_hex_foundation()")
    construction = bootstrap.index("install_current_schema(config)")
    install = bootstrap.index("install_michigan_h3_reference_bundle(config, cohort, foundation)")
    assert source_validation < foundation_validation < construction < install
    assert bootstrap.count("install_current_schema(config)") == 1


def test_h3_source_fixture_has_one_production_owner() -> None:
    persistence = ROOT / "rust/crates/babylon-persistence"
    source_fixture = persistence / "src/fixtures/h3_reference_source_v1.bin"
    retired_test_fixture = persistence / "tests/fixtures/h3_reference_source_v1.bin"
    assert source_fixture.is_file()
    assert not retired_test_fixture.exists()

    include_sites = []
    for path in persistence.rglob("*.rs"):
        text = path.read_text(encoding="utf-8")
        if 'include_bytes!("fixtures/h3_reference_source_v1.bin")' in text:
            include_sites.append(path.relative_to(persistence).as_posix())
    assert include_sites == ["src/h3_reference_cohort.rs"]


def test_every_checked_in_bootstrap_caller_provisions_pinned_rust() -> None:
    action = (ROOT / ".github/actions/bootstrap-persistence/action.yml").read_text(encoding="utf-8")
    assert "rust/rust-toolchain.toml" in action
    assert 'rustup toolchain install "$CHANNEL" --profile minimal --no-self-update' in action

    for relative in BOOTSTRAP_CALLERS:
        workflow = (ROOT / relative).read_text(encoding="utf-8")
        assert "uses: ./.github/actions/bootstrap-persistence" in workflow

    for relative in DIRECT_BOOTSTRAP_CALLERS:
        workflow = (ROOT / relative).read_text(encoding="utf-8")
        assert "mise run db:bootstrap" in workflow

    ci = (ROOT / ".github/workflows/ci.yml").read_text(encoding="utf-8")
    weekly = (ROOT / ".github/workflows/weekly-pg-integration.yml").read_text(encoding="utf-8")
    assert "tools/run_rust_postgres.sh" in ci
    assert "tools/run_rust_postgres.sh" in weekly
    assert "mise run test:rust-postgres" not in weekly


def test_persistence_bootstrap_provisions_mise_for_runtime_tasks() -> None:
    action = (ROOT / ".github/actions/bootstrap-persistence/action.yml").read_text(encoding="utf-8")

    mise_setup = action.index("uses: jdx/mise-action@3c2e0cf82a5b2e5249f0d3635a4d83d0ae861518 # v4")
    rust_setup = action.index("- name: Install pinned Rust toolchain")
    assert mise_setup < rust_setup
    required = tomllib.loads((ROOT / ".mise.toml").read_text(encoding="utf-8"))["min_version"]
    assert f"version: {required}" in action[mise_setup:rust_setup]


def test_pr_pg_lane_runs_the_rust_live_matrix_without_python_reference_data() -> None:
    workflow = (ROOT / ".github/workflows/ci.yml").read_text(encoding="utf-8")
    pg_lane = workflow.split("\n  pg-integration-shards:", maxsplit=1)[1].split(
        "\n  security:", maxsplit=1
    )[0]
    aggregator = workflow.split("\n  ci-gate:", maxsplit=1)[1]

    bootstrap = pg_lane.index("uses: ./.github/actions/bootstrap-persistence")
    matrix_focus = pg_lane.index("BABYLON_POSTGRES_LIVE_FOCUS: ${{ matrix.focus }}")
    runner = pg_lane.index("tools/run_rust_postgres.sh")
    assert bootstrap < matrix_focus < runner
    assert "matrix: ${{ fromJSON(needs.scope.outputs.pg-matrix) }}" in pg_lane
    assert "fetch-reference-db" not in pg_lane
    assert "bootstrap-python" not in pg_lane
    assert "uv run pytest" not in pg_lane
    assert "pg-integration-shards" in aggregator
    assert "if: always()" in aggregator
    assert "CI_NEEDS: ${{ toJSON(needs) }}" in aggregator
    assert "python3 tools/ci_scope.py --verify" in aggregator


def test_nightly_michigan_job_covers_bootstrap_smoke_and_hosted_setup_cleanup() -> None:
    workflow = (ROOT / ".github/workflows/nightly-michigan-smoke.yml").read_text(encoding="utf-8")
    job = workflow.split("\n  michigan-smoke:", maxsplit=1)[1]
    # The native diagnostic embeds its exact QCEW source; external SQLite
    # readiness must not disable the daily rollover crash canary.
    assert "CI_REFDB_READY" not in job
    assert "fetch-reference-db" not in job
    assert "mise run qa:michigan-rollover-smoke" in job
    assert "if: always()" in job
    assert "tools/ci_postgres_compose.sh down" in job
    timeout_minutes = int(
        next(
            line.split(":", maxsplit=1)[1].strip()
            for line in job.splitlines()
            if line.strip().startswith("timeout-minutes:")
        )
    )

    bootstrap_envelope_seconds = 600 + 30
    michigan_envelope_seconds = 1800 + 30
    hosted_setup_and_cleanup_headroom_seconds = 15 * 60
    assert timeout_minutes == 69
    assert timeout_minutes * 60 >= (
        bootstrap_envelope_seconds
        + michigan_envelope_seconds
        + hosted_setup_and_cleanup_headroom_seconds
    )


def _postgres_runner() -> str:
    return (ROOT / "tools/run_rust_postgres.sh").read_text(encoding="utf-8")


def test_fresh_runtime_focuses_cover_current_live_consumers() -> None:
    runner = _postgres_runner()
    for target in (
        "current_schema::live_tests::",
        "--test reference_integrity",
        "runtime::live_tests::live_",
        "archive_revision::worker::live_tests::",
        "for archive_group in bounds revisions wakeup",
        "for archive_producer in place_producer_live county_producer_live",
        "reader_role_live observer_material_live",
        "--test dossier_cli_live",
    ):
        assert target in runner
    assert 'readonly LIVE_FOCUS="${BABYLON_POSTGRES_LIVE_FOCUS-runtime_smoke}"' in runner
    assert "--ignored" in runner
    assert "--test-threads=1" in runner
    assert "reader_threads=4" in runner
    assert "--skip statewide:: --skip statewide_qualified::" in runner
    assert "run_phase statewide_synthetic 600 cargo test" in runner
    assert not (ROOT / "tools/run_rust_legacy_adopter_pg.sh").exists()
    for retired in (
        "schema_epoch_matrix",
        "runtime_census_v2",
        "h3_shadow_backfill",
        "legacy_adopter_postgres",
    ):
        assert retired not in runner


def test_disposable_pg_runner_proves_fresh_schema_and_michigan_smoke() -> None:
    runner = _postgres_runner()
    before = runner.index('before="$(fresh_relation_count)"')
    hostile = runner.index("run_phase hostile_dsn 30", before)
    after = runner.index('after="$(fresh_relation_count)"', hostile)
    bootstrap = runner.index("run_phase fresh_bootstrap 180", after)
    smoke = runner.index("run_phase michigan_rollover 600", bootstrap)
    assert before < hostile < after < bootstrap < smoke
    assert '[ "$hostile_status" -eq 1 ]' in runner
    assert "options-bearing DSN changed the fresh database before refusal" in runner
    assert "mise run db:bootstrap" in runner[bootstrap:smoke]
    assert "mise run qa:michigan-rollover-smoke" in runner[smoke:]
    for field in LIBPQ_TARGET_ENV:
        assert field in runner[bootstrap:smoke]
    assert 'readonly CLEAN_RUNTIME="true|0|true|true"' in runner
    assert "FROM babylon_meta.current_schema" in runner
    assert "pg_catalog.count(*) = 1" in runner
    assert (
        "pg_catalog.bool_and(singleton AND pg_catalog.octet_length(schema_sha256) = 32)" in runner
    )
    assert "pg_catalog.to_regclass('public.hex_spatial_map') IS NULL" in runner
    assert "pg_catalog.to_regclass('babylon_state.campaign_foundation') IS NOT NULL" in runner


def test_runtime_contracts_clone_one_pristine_current_authority_template() -> None:
    runner = _postgres_runner()
    bootstrap = runner.index("run_phase fresh_bootstrap")
    template = runner.index("create_runtime_template || status=$?", bootstrap)
    dispatch = runner.index('case "$LIVE_FOCUS" in', template)
    verify = runner.index(
        "verify_runtime_template_and_clone_cleanup || template_status=$?", dispatch
    )
    drop = runner.index("drop_runtime_template || template_status=$?", verify)
    assert bootstrap < template < dispatch < verify < drop
    assert 'BABYLON_RUNTIME_TEMPLATE_DB="$RUNTIME_TEMPLATE"' in runner
    assert 'runtime_observation "$RUNTIME_TEMPLATE"' in runner
    assert "babylon_meta.campaign" in runner
    assert "pg_catalog.pg_database" in runner
    assert "datname LIKE 'per281_runtime_%'" in runner
    runtime = (ROOT / "rust/crates/babylon-persistence/src/runtime.rs").read_text(encoding="utf-8")
    assert 'const TEMPLATE_DB_ENV: &str = "BABYLON_RUNTIME_TEMPLATE_DB";' in runtime
    assert "TestDatabase::create_from_template" in runtime


def test_heavy_children_and_enclosing_ci_have_truthful_deadlines() -> None:
    runner = _postgres_runner()
    assert 'timeout --signal=TERM --kill-after=10s "${limit}s" "$@"' in runner
    for invocation in (
        "run_phase runtime_build 600 cargo build",
        "run_phase reference_integrity 900 cargo test",
        "run_phase reference_catalog 600 cargo test",
        "run_phase runtime 600 cargo test",
        'run_phase material_writer_bounds 180 env BABYLON_RUNTIME_DSN="$BOOTSTRAP_DSN"',
        "run_phase archive_worker 600 cargo test",
        'run_phase "archive_$archive_group" 600 cargo test',
        'run_phase "$archive_producer" 600 cargo test',
        'run_phase "$reader_suite" 600 cargo test',
        "run_phase client 900 cargo test",
    ):
        assert invocation in runner
    # Reserve600 for bounded Docker/SQL probes, readiness and cleanup. The
    # weekly/manual workflow also runs the full actual-source statewide case.
    for relative, job_name, contract_seconds in (
        (".github/workflows/ci.yml", "pg-integration-shards", 2 * 600),
        (".github/workflows/weekly-pg-integration.yml", "runtime-contracts", 3600),
    ):
        workflow = yaml.safe_load((ROOT / relative).read_text())
        job = workflow["jobs"][job_name]
        step = next(
            step for step in job["steps"] if step.get("run") == "tools/run_rust_postgres.sh"
        )
        if job_name == "pg-integration-shards":
            assert (
                step["timeout-minutes"]
                == "${{ matrix.focus == 'archive' && 85 || matrix.focus == 'reader' && 60 || 45 }}"
            )
            assert (
                job["timeout-minutes"]
                == "${{ matrix.focus == 'archive' && 95 || matrix.focus == 'reader' && 70 || 55 }}"
            )
            assert 600 + 180 + contract_seconds + 600 <= 45 * 60
            # Reader roles, material observations and statewide synthetic proofs each get a phase.
            assert 60 * 60 >= 600 + 180 + 3 * 600 + 600
            # Six serial Archive groups retain separate ten-minute ceilings.
            assert 85 * 60 >= 600 + 180 + 6 * 600 + 600
        else:
            assert step["timeout-minutes"] * 60 >= 600 + 180 + contract_seconds + 600
            assert job["timeout-minutes"] >= step["timeout-minutes"] + 10


def test_disposable_pg_runner_keeps_bounded_logs_phase_timings_and_runtime_identity() -> None:
    runner = _postgres_runner()
    assert 'docker logs --timestamps --tail 200 "$CONTAINER_ID"' in runner
    assert (
        'die_with_runtime_logs "pinned PostgreSQL runtime was not ready within 90 seconds"'
        in runner
    )
    assert "current_setting('server_version_num')" in runner
    assert "current_setting('server_version')" in runner
    assert '[ "$RUNTIME_MAJOR" = "17" ]' in runner
    assert "elapsed_seconds=%s status=%s" in runner
    assert "cleanup_checked" in runner
