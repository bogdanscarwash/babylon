"""Contracts for the PER-287 clean-bootstrap Rust epoch handoff."""

from __future__ import annotations

import tomllib
from pathlib import Path

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


def test_db_bootstrap_has_one_rust_owned_activation_root() -> None:
    task = _mise_task("db:bootstrap")

    build = task.index("cargo build -p babylon-persistence --bin babylon-runtime --locked")
    activation = task.index("babylon-runtime bootstrap")
    assert build < activation
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
    activation = task.index("babylon-runtime bootstrap")
    assert sanitization < build < activation


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


def test_runtime_cli_has_one_exact_connecting_schema_preflight_mode() -> None:
    source = (ROOT / "rust/crates/babylon-persistence/src/bin/babylon-runtime.rs").read_text(
        encoding="utf-8"
    )

    assert 'const DSN_ENV: &str = "BABYLON_RUNTIME_DSN";' in source
    assert "std::env::args_os().skip(1)" in source
    assert "Command::Preflight =>" in source
    assert "preflight_schema_epoch(config)" in source
    assert "Command::Activate | Command::Bootstrap =>" in source
    assert "activate_rust_persistence_v2(config)" in source
    assert "DurableReplayRuntimeV2" in source
    assert "activate_rust_persistence_v1" not in source
    assert "DurableReplayRuntimeV1" not in source
    assert "babylon-schema-epoch" not in source
    assert "BABYLON_SCHEMA_EPOCH_DSN" not in source


def test_runtime_cli_owns_one_restart_safe_activation_sequence() -> None:
    persistence = ROOT / "rust/crates/babylon-persistence"
    cli = (persistence / "src/bin/babylon-runtime.rs").read_text(encoding="utf-8")
    lib = (persistence / "src/lib.rs").read_text(encoding="utf-8")
    bootstrap = (persistence / "src/bootstrap.rs").read_text(encoding="utf-8")
    runtime = (persistence / "src/runtime.rs").read_text(encoding="utf-8")
    cohort = (persistence / "src/h3_reference_cohort.rs").read_text(encoding="utf-8")

    assert "activate_rust_persistence_v2(config)" in cli
    assert "activate_rust_persistence_v1" not in cli
    activation = runtime.split("pub fn activate_rust_persistence_v2", maxsplit=1)[1].split(
        "\nfn activate_v2_under_lock", maxsplit=1
    )[0]
    assert (
        activation.index("preflight_v2_activation_before_mutation(config)")
        < activation.index("establish_predecessor_authority_v2(config)")
        < activation.index("acquire_lock(&mut client)")
        < activation.index("activate_v2_under_lock(&mut client)")
    )

    predecessor = runtime.split("fn establish_predecessor_authority_v2", maxsplit=1)[1].split(
        "\nconst SERIALIZABLE_ACTIVATION_SETTINGS_V2", maxsplit=1
    )[0]
    assert (
        predecessor.index("bootstrap_h3_reader_epoch_v1(config)")
        < predecessor.index("MIGRATION_0008_SQL")
        < predecessor.index("MIGRATION_0009_SQL")
    )

    v2_activation = runtime.split("fn activate_v2_under_lock", maxsplit=1)[1].split(
        "\nfn execute_v2_activation_migration", maxsplit=1
    )[0]
    assert v2_activation.count("execute_v2_activation_migration(") == 2
    assert (
        v2_activation.index("compiled_committed_tick_v2_activation_migrations()")
        < v2_activation.index("migrations[0]")
        < v2_activation.index("migrations[1]")
    )

    for call in (
        "representative_h3_reference_cohort_v1()",
        "michigan_dynamic_hex_foundation_v1()",
        "migrate_schema_epoch_to_h3_handoff(config)",
        "install_michigan_h3_reference_bundle_v1(config, cohort, foundation)",
        "backfill_legacy_h3_shadow_keys(config)",
        "migrate_schema_epoch(config)",
    ):
        assert call in bootstrap
    source_validation = bootstrap.index("representative_h3_reference_cohort_v1()")
    foundation_validation = bootstrap.index("michigan_dynamic_hex_foundation_v1()")
    handoff = bootstrap.index("migrate_schema_epoch_to_h3_handoff(config)")
    install = bootstrap.index("install_michigan_h3_reference_bundle_v1(config, cohort, foundation)")
    backfill = bootstrap.index("backfill_legacy_h3_shadow_keys(config)")
    terminal = bootstrap.rindex("migrate_schema_epoch(config)")
    assert source_validation < foundation_validation < handoff < install < backfill < terminal
    assert "install_representative_h3_cohort" not in bootstrap
    assert "pub fn bootstrap_h3_reader_epoch_v1" in bootstrap
    assert "mod bootstrap;" in lib
    assert "pub mod bootstrap;" not in lib
    assert "bootstrap_h3_reader_epoch_v1" in lib

    source_fixture = persistence / "src/fixtures/h3_reference_source_v1.bin"
    retired_test_fixture = persistence / "tests/fixtures/h3_reference_source_v1.bin"
    assert source_fixture.is_file()
    assert not retired_test_fixture.exists()
    assert 'include_bytes!("fixtures/h3_reference_source_v1.bin")' in cohort

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
    assert "tools/run_rust_legacy_adopter_pg.sh" in ci
    assert "tools/run_rust_legacy_adopter_pg.sh" in weekly
    assert "mise run test:rust-legacy-adopter-pg" not in weekly


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
        "\n  pg-integration:", maxsplit=1
    )[0]
    aggregator = workflow.split("\n  pg-integration:", maxsplit=1)[1].split(
        "\n  security:", maxsplit=1
    )[0]

    bootstrap = pg_lane.index("uses: ./.github/actions/bootstrap-persistence")
    matrix_focus = pg_lane.index("BABYLON_LEGACY_ADOPTER_LIVE_FOCUS: ${{ matrix.focus }}")
    runner = pg_lane.index("tools/run_rust_legacy_adopter_pg.sh")
    assert bootstrap < matrix_focus < runner
    for focus in [
        "clean_bootstrap",
        "h3_atomicity",
        "rust_persistence_runtime",
        "installed_mutation",
        "archive_worker",
        "reader_role",
    ]:
        assert f"          - {focus}\n" in pg_lane
    assert "fetch-reference-db" not in pg_lane
    assert "bootstrap-python" not in pg_lane
    assert "uv run pytest" not in pg_lane
    assert "needs: pg-integration-shards" in aggregator
    assert "if: always()" in aggregator
    assert "needs.pg-integration-shards.result" in aggregator


def test_nightly_michigan_job_covers_bootstrap_smoke_and_hosted_setup_cleanup() -> None:
    workflow = (ROOT / ".github/workflows/nightly-michigan-smoke.yml").read_text(encoding="utf-8")
    job = workflow.split("\n  michigan-smoke:", maxsplit=1)[1]
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


def test_archive_worker_focus_covers_place_producer_live_target() -> None:
    """The archive_worker shard runs every Archive worker live test binary."""
    runner = (ROOT / "tools/run_rust_legacy_adopter_pg.sh").read_text(encoding="utf-8")

    focus = runner.split('[ "$LIVE_FOCUS" = "archive_worker" ]; then', maxsplit=1)[1]
    invocation = focus.split("|| status=$?", maxsplit=1)[0]
    assert "cargo test -p babylon-persistence" in invocation
    assert "--test archive_worker_live" in invocation
    assert "--test place_producer_live" in invocation
    assert "--locked" in invocation
    assert "--ignored --test-threads=1" in invocation


def test_archive_worker_focus_covers_county_producer_live_target() -> None:
    """The archive_worker shard runs every Archive worker live test binary."""
    runner = (ROOT / "tools/run_rust_legacy_adopter_pg.sh").read_text(encoding="utf-8")

    focus = runner.split('[ "$LIVE_FOCUS" = "archive_worker" ]; then', maxsplit=1)[1]
    invocation = focus.split("|| status=$?", maxsplit=1)[0]
    assert "cargo test -p babylon-persistence" in invocation
    assert "--test archive_worker_live" in invocation
    assert "--test county_producer_live" in invocation
    assert "--locked" in invocation
    assert "--ignored --test-threads=1" in invocation


def test_disposable_pg_runner_proves_bootstrap_and_michigan_smoke() -> None:
    runner = (ROOT / "tools/run_rust_legacy_adopter_pg.sh").read_text(encoding="utf-8")

    assert "clean_bootstrap" in runner
    assert 'if [ "$LIVE_FOCUS" = "clean_bootstrap" ] || [ "$LIVE_FOCUS" = "pr" ]; then' in runner
    options_refusal = runner.index('HOSTILE_OPTIONS_DSN="${BOOTSTRAP_DSN}?options=')
    before_snapshot = runner.index('before_options_refusal="$(timeout', options_refusal)
    refused_bootstrap = runner.index('BABYLON_RUNTIME_DSN="$HOSTILE_OPTIONS_DSN"', before_snapshot)
    after_snapshot = runner.index('after_options_refusal="$(timeout', refused_bootstrap)
    assert "mise run db:bootstrap" in runner
    normal_bootstrap = runner.index('BABYLON_RUNTIME_DSN="$BOOTSTRAP_DSN"', after_snapshot)
    assert options_refusal < before_snapshot < refused_bootstrap < after_snapshot < normal_bootstrap
    assert "options-bearing DSN unexpectedly passed schema preflight" in runner
    assert "options-bearing DSN changed the fresh database before refusal" in runner
    bootstrap_prefix = runner[
        normal_bootstrap : runner.index("mise run db:bootstrap", normal_bootstrap)
    ]
    for assignment in (
        'PGHOST="host.invalid"',
        'PGHOSTADDR="203.0.113.1"',
        'PGPORT="1"',
        'PGDATABASE="redirected"',
        "PGOPTIONS='-c search_path=redirected,public'",
        'PGSERVICE="redirected"',
        'PGSERVICEFILE="/nonexistent/babylon-pg-service.conf"',
        'PGSYSCONFDIR="/nonexistent/babylon-pg-service.d"',
    ):
        assert assignment in bootstrap_prefix
    assert "mise run qa:michigan-rollover-smoke" in runner
    assert '"1:1:8,2:2:9|true|true"' in runner
    assert "babylon_meta.persistence_authority_ledger" in runner
    assert "pg_catalog.to_regclass('public.hex_spatial_map') IS NULL" in runner
    assert "pg_catalog.to_regclass('babylon_state.campaign_foundation') IS NOT NULL" in runner
    smoke = runner.split("mise run qa:michigan-rollover-smoke", maxsplit=1)[0].rsplit(
        "timeout --signal=TERM --kill-after=30s 1800s", maxsplit=1
    )[1]
    assert 'BABYLON_RUNTIME_DSN="$BOOTSTRAP_DSN"' in smoke
    assert "BABYLON_PG_DSN=" not in smoke
    assert "BABYLON_TEST_PG_DSN=" not in smoke


def test_pr_runtime_contracts_clone_one_clean_activated_template() -> None:
    """Runtime tests amortize activation without weakening fresh activation proofs."""
    runner = (ROOT / "tools/run_rust_legacy_adopter_pg.sh").read_text(encoding="utf-8")
    runtime = (ROOT / "rust/crates/babylon-persistence/src/runtime.rs").read_text(encoding="utf-8")
    live_tests = runtime.split("mod live_tests {", maxsplit=1)[1].split(
        "\n#[cfg(test)]\nmod tests", maxsplit=1
    )[0]

    bootstrap = runner.index("mise run db:bootstrap")
    template = runner.index('create_runtime_template "$BOOTSTRAP_DSN"', bootstrap)
    smoke = runner.index("mise run qa:michigan-rollover-smoke", template)
    runtime_tests = runner.index(
        "cargo test -p babylon-persistence --lib \\\n    runtime::live_tests::live_",
        smoke,
    )
    template_check = runner.index("verify_runtime_template_and_clone_cleanup", runtime_tests)
    template_drop = runner.index("drop_runtime_template", template_check)
    template_check_body = runner.index("verify_runtime_template_and_clone_cleanup()")
    template_drop_body = runner.index("\ndrop_runtime_template()", template_check_body)

    assert bootstrap < template < smoke < runtime_tests < template_check < template_drop
    assert 'BABYLON_RUNTIME_TEMPLATE_DB="$RUNTIME_TEMPLATE"' in runner
    assert ':"template_database"' not in runner
    assert (
        "babylon_meta.persistence_authority_ledger"
        in runner[template_check_body:template_drop_body]
    )
    assert "babylon_meta.campaign" in runner[template_check_body:template_drop_body]
    assert "pg_catalog.pg_database" in runner[template_check_body:template_drop_body]

    assert 'const TEMPLATE_DB_ENV: &str = "BABYLON_RUNTIME_TEMPLATE_DB";' in live_tests
    # Five PER-281 runtime proofs plus five PER-22 declared-mapping proofs.
    assert live_tests.count("TestDatabase::create_from_template") == 10
    assert live_tests.count("activate_rust_persistence_v2(&config)") == 6
    assert "activate_rust_persistence_v1" not in live_tests
    frozen_activation = live_tests.split("fn verify_frozen_python_estate_activation", maxsplit=1)[
        1
    ].split("\n    #[test]", maxsplit=1)[0]
    assert frozen_activation.count("activate_rust_persistence_v2(&config)") == 2
    assert r"CREATE DATABASE \"{name}\" OWNER test TEMPLATE \"{template}\"" in live_tests


def test_runtime_template_helpers_return_psql_failures_to_checked_callers() -> None:
    """Template failures reach status capture, bounded logs, and checked cleanup."""
    runner = (ROOT / "tools/run_rust_legacy_adopter_pg.sh").read_text(encoding="utf-8")
    create = runner.split("create_runtime_template() {", maxsplit=1)[1].split(
        "\nverify_runtime_template_and_clone_cleanup()", maxsplit=1
    )[0]
    verify = runner.split("verify_runtime_template_and_clone_cleanup() {", maxsplit=1)[1].split(
        "\ndrop_runtime_template()", maxsplit=1
    )[0]
    drop = runner.split("drop_runtime_template() {", maxsplit=1)[1].split(
        "\n# shellcheck disable=SC2329", maxsplit=1
    )[0]

    assert create.count("|| return") == 3
    assert verify.count("|| return") == 4
    assert drop.count("|| return") == 1


def test_disposable_pg_runner_retains_bootstrap_logs_and_runtime_version() -> None:
    """Bootstrap refusals retain bounded server evidence and the pinned runtime identity."""
    runner = (ROOT / "tools/run_rust_legacy_adopter_pg.sh").read_text(encoding="utf-8")

    assert "die_with_runtime_logs()" in runner
    assert "emit_runtime_logs()" in runner
    assert 'docker logs --timestamps --tail 200 "$CONTAINER"' in runner
    assert (
        'die_with_runtime_logs "pinned PostgreSQL runtime was not ready within 90 seconds"'
        in runner
    )
    assert "current_setting('server_version_num')" in runner
    assert "current_setting('server_version')" in runner
    assert "PER-20 runtime PostgreSQL: major=%s version=%s" in runner
