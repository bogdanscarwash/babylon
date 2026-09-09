# ADR095 Packaging Train Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development
> (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** Land the packaging train's opening moves — the `babylon` CLI entry point with its six
subcommands, the dependency extras split plus deletion of the stdlib-shadowing `uuid` relic, and
the single-lock migration from Poetry to uv (ruling U) — so `install.sh`, the flake wrapper, and
the downstream 094/096/097 plans have a stable binary name, a slim default closure, and one lock.

**Architecture:** A new `src/babylon/cli/` Typer package exposes `app` (entry point
`babylon = "babylon.cli:app"`); `play` is the default subcommand and reuses the existing
`babylon.__main__` demo path, while `doctor`/`login`/`telemetry`/`self-update`/`uninstall` reuse
the landed §A8 provider seam and config-dir conventions. `pyproject.toml` moves to a PEP-621
dependency layout (main deps + a `server` optional-dependencies extra + PEP 735 dependency-groups),
keeping the `poetry-core` build backend so uv builds via PEP 517; the `.mise.toml`, pre-commit, and
CI toolchains swap `poetry` for `uv` against one `uv.lock`.

**Tech Stack:** Python `>=3.12,<4.0`, Typer + Rich (already deps), Pydantic v2, Poetry-core build
backend, uv package/lock tool, mise task runner, pytest (with `typer.testing.CliRunner`), GitHub
Actions.

## Global Constraints

- Base branch: `dev` @ 8ee8707f. Execute in a fresh git worktree off `dev`
  (superpowers:using-git-worktrees); the owner's live checkout must never be touched.
  Pushes are owner-run; commits use conventional format with the Co-Authored-By trailer.
- Python `>=3.12,<4.0` (pyproject); mise `[tools]` pins `python = "3.12"`, `poetry = "2.2.1"`
  (`.mise.toml:17-19`) until this plan's uv swap replaces the poetry pin with a uv pin.
- Strict typing (mypy strict, function signatures fully annotated); Pydantic models for data
  objects; explicit exception types, no bare except; all loops bounded.
- Amendment V: narrator-only AI — no LLM output may enter the simulation input path.
  Amendment X.6: no LLM framework dependencies (no langchain etc.).
- Cloudflare: Workers Free plan everywhere; nothing that can bill.
- Verification battery before each commit: `poetry run pytest <touched tests>` (Tasks 1–6, still on
  Poetry) or `uv run pytest <touched tests>` (Tasks 7–9, after the uv swap), plus
  `poetry|uv run ruff check src tests` and `poetry|uv run mypy src` — scaled to what the task
  touched. The final task (Task 9) runs the full gate.
- NEVER read, print, or commit secrets: `.env`, `terraform.tfvars`, `*.tfstate*`, `vault.yml`,
  age keys, or a populated `~/.config/babylon/credentials`. `login` tests write to a `tmp_path`
  config dir only.

### Plan-specific constraints

- **Upstream dependencies:** none — this is the packet's critical-path opener (execution order
  095 → 094 → 096 → 097). 094 consumes this plan's `uv.lock` + `[project.scripts]` entry point;
  096/097 consume the `src/babylon/cli/` skeleton. Do not break those contracts.
- **Fixed interface names (other plans rely on them):** package `src/babylon/cli/` with
  `__init__.py` defining `app = typer.Typer(...)`; entry point
  `[project.scripts] babylon = "babylon.cli:app"`; subcommand modules `play.py`, `doctor.py`,
  `login.py`, `telemetry.py`, `self_update.py`, `uninstall.py`; `babylon` with no subcommand runs
  `play`. Tests under `tests/unit/cli/`. The `server` extra is named exactly `server`; dependency
  groups stay `dev` and `docs`.
- **Reuse, do not reinvent:** the config-dir precedence (`$BABYLON_CONFIG_DIR` >
  `$XDG_CONFIG_HOME/babylon` > `~/.config/babylon`) and the 0600 credentials convention already
  live in `src/babylon/intelligence/providers.py` (`_config_dir`, `_load_credentials`,
  `load_settings`, `resolve_provider`). `login`/`doctor` reuse them.
- **K-fallback (abort path, RECORDED — not executed):** ruling U adopts uv; K (keep Poetry, two
  locks) is the fallback. If `uv lock` (Task 7) fails to resolve ANY dependency, the executor
  **stops and reports** — it does NOT force the migration or hand-edit the lock. Tasks 1–6 (CLI,
  uuid deletion, PEP-621 conversion) stand on their own under Poetry even if D3 is aborted.
- **qa:regression gate:** `mise run qa:regression` (Amendment Q closure-regression discipline) and
  `mise run test:unit` require the local isolated Postgres up — run `mise run db:up` first. These
  are dev-box gates; if Postgres/Docker is unavailable the executor records the skip honestly
  rather than claiming a pass.
- **Django is behind the `server` extra after D2.** Because `[tool.pytest.ini_options]` sets
  `DJANGO_SETTINGS_MODULE` for the whole session and `django-stubs`' mypy plugin loads Django
  settings at startup, every dev/CI path that runs pytest or mypy needs the `server` extra
  installed. Developer `mise run install` and the gdal CI jobs therefore install `--extra server`;
  only the shipped Nix player closure gets the slim default. (Judgment call — see Task 6/8 notes.)

---

## Task 1: CLI skeleton — Typer app, `play` default, `--version`, entry point

**Files:**

- Create: `src/babylon/cli/__init__.py`
- Create: `src/babylon/cli/play.py`
- Create: `tests/unit/cli/__init__.py`
- Create: `tests/unit/cli/test_app.py`
- Modify: `pyproject.toml` (add `[project.scripts]` after `[project.urls]`, lines 32–36)

**Interfaces:**

- Produces: `babylon.cli.app` (`typer.Typer`); `babylon.cli.play.run() -> None`;
  `babylon.cli.play.play() -> None`; console entry point `babylon = "babylon.cli:app"`.
- Consumes: `babylon.__version__` (str); `babylon.__main__.main() -> None`.

Steps:

- [ ] Write the failing test `tests/unit/cli/__init__.py` (empty) and `tests/unit/cli/test_app.py`:

  ```python
  """CLI app contract: entry-point shape, --version, and play-default (ADR095 D1)."""

  from __future__ import annotations

  import babylon.__main__ as demo_main
  from babylon import __version__
  from babylon.cli import app
  from typer.testing import CliRunner

  runner = CliRunner()


  def test_help_lists_all_six_subcommands() -> None:
      result = runner.invoke(app, ["--help"])
      assert result.exit_code == 0
      for name in ("play", "doctor", "login", "telemetry", "self-update", "uninstall"):
          assert name in result.stdout


  def test_version_flag_prints_version() -> None:
      result = runner.invoke(app, ["--version"])
      assert result.exit_code == 0
      assert __version__ in result.stdout


  def test_no_subcommand_runs_play(monkeypatch) -> None:  # type: ignore[no-untyped-def]
      calls: list[str] = []
      monkeypatch.setattr(demo_main, "main", lambda: calls.append("ran"))
      result = runner.invoke(app, [])
      assert result.exit_code == 0
      assert calls == ["ran"]


  def test_play_subcommand_runs_demo(monkeypatch) -> None:  # type: ignore[no-untyped-def]
      calls: list[str] = []
      monkeypatch.setattr(demo_main, "main", lambda: calls.append("ran"))
      result = runner.invoke(app, ["play"])
      assert result.exit_code == 0
      assert calls == ["ran"]
  ```

- [ ] Run the test, expect FAIL (no `babylon.cli` package yet):

  ```bash
  poetry run pytest tests/unit/cli/test_app.py -q
  ```

  Expected: `ModuleNotFoundError: No module named 'babylon.cli'` (collection error, exit code 2).

- [ ] Create `src/babylon/cli/play.py`:

  ```python
  """`babylon play` — launch the game (currently the bundled two-node demo).

  Delegates to the existing ``babylon.__main__`` entry logic rather than
  duplicating it (DRY); the TUI client replaces this body in a later plan.
  """

  from __future__ import annotations


  def run() -> None:
      """Run the bundled demo simulation. Imported lazily so importing the CLI
      package never triggers ``babylon.__main__``'s import-time logging setup."""
      from babylon.__main__ import main as run_demo

      run_demo()


  def play() -> None:
      """Play Babylon (currently the bundled two-node demo scenario)."""
      run()
  ```

- [ ] Create `src/babylon/cli/__init__.py`:

  ```python
  """Babylon command-line interface (ADR095 D1).

  Entry point: ``[project.scripts] babylon = "babylon.cli:app"``. ``babylon``
  with no subcommand runs ``play``. The subcommand modules reuse the landed
  §A8 provider seam and config-dir conventions — nothing here reinvents them.
  """

  from __future__ import annotations

  import typer

  from babylon import __version__
  from babylon.cli import play as play_cmd

  app = typer.Typer(
      name="babylon",
      help="Babylon — The Fall of America. A Marxist simulation engine.",
      add_completion=False,
      no_args_is_help=False,
  )


  def _register() -> None:
      """Register subcommands. Lazy imports keep the root ``--help`` fast and
      avoid import cycles between subcommand modules and the seam."""
      from babylon.cli import doctor as doctor_cmd
      from babylon.cli import login as login_cmd
      from babylon.cli import self_update as self_update_cmd
      from babylon.cli import telemetry as telemetry_cmd
      from babylon.cli import uninstall as uninstall_cmd

      app.command(name="play")(play_cmd.play)
      app.command(name="doctor")(doctor_cmd.doctor)
      app.command(name="login")(login_cmd.login)
      app.command(name="telemetry")(telemetry_cmd.telemetry)
      app.command(name="self-update")(self_update_cmd.self_update)
      app.command(name="uninstall")(uninstall_cmd.uninstall)


  def _version_callback(value: bool) -> None:
      if value:
          typer.echo(__version__)
          raise typer.Exit()


  @app.callback(invoke_without_command=True)
  def main(
      ctx: typer.Context,
      version: bool = typer.Option(
          False,
          "--version",
          callback=_version_callback,
          is_eager=True,
          help="Show the Babylon version and exit.",
      ),
  ) -> None:
      """Babylon CLI root. With no subcommand, launches the game (play)."""
      if ctx.invoked_subcommand is None:
          play_cmd.run()


  _register()
  ```

  > NOTE: Tasks 2–4 create `doctor.py`, `login.py`, `telemetry.py`, `self_update.py`,
  > `uninstall.py`. To keep Task 1 independently runnable, create thin one-line stub bodies for
  > those five modules NOW (each a `def <name>() -> None: raise typer.Exit()` placeholder) and
  > replace them with real logic in Tasks 2–4. Create the stubs in this same step so `_register()`
  > imports resolve.

- [ ] Create the five stub modules so imports resolve (each replaced in Tasks 2–4):

  ```python
  # src/babylon/cli/doctor.py
  """`babylon doctor` — placeholder (real body: Task 2)."""

  from __future__ import annotations


  def doctor() -> None:
      """Diagnose the local Babylon install."""
      raise NotImplementedError
  ```

  Repeat the identical shape for `login.py` (`def login() -> None:`), `telemetry.py`
  (`def telemetry() -> None:`), `self_update.py` (`def self_update() -> None:`), and
  `uninstall.py` (`def uninstall() -> None:`). Each is one function raising `NotImplementedError`,
  fully annotated.

- [ ] Run the test, expect PASS:

  ```bash
  poetry run pytest tests/unit/cli/test_app.py -q
  ```

  Expected: `4 passed`.

- [ ] Add the entry point to `pyproject.toml` immediately after the `[project.urls]` block
  (after line 36):

  ```toml
  [project.scripts]
  babylon = "babylon.cli:app"
  ```

- [ ] Verify the console script installs and runs under Poetry hybrid mode (works before the uv
  swap):

  ```bash
  poetry install && poetry run babylon --help
  ```

  Expected: usage text beginning `Usage: babylon [OPTIONS] COMMAND [ARGS]...` listing `play`,
  `doctor`, `login`, `telemetry`, `self-update`, `uninstall`. Also run
  `poetry run babylon --version` → prints `0.3.0`.

- [ ] Lint + typecheck the new package:

  ```bash
  poetry run ruff check src/babylon/cli tests/unit/cli && poetry run mypy src/babylon/cli
  ```

  Expected: `All checks passed!` and `Success: no issues found`.

- [ ] Commit:

  ```bash
  git add src/babylon/cli tests/unit/cli pyproject.toml
  git commit -m "feat(cli): babylon typer entry point — play default, --version, subcommand skeleton (ADR095 D1)" -m "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
  ```

---

## Task 2: `doctor` — config + provider health + database reachability

**Files:**

- Modify: `src/babylon/cli/doctor.py` (replace the Task 1 stub)
- Create: `tests/unit/cli/test_doctor.py`

**Interfaces:**

- Consumes: `babylon.intelligence.providers.load_settings`, `.resolve_provider`, `._config_dir`,
  `.ProviderHealth`; `babylon.config.base.BaseConfig`.
- Produces: `babylon.cli.doctor.doctor() -> None`; helper
  `babylon.cli.doctor.check_database(dsn: str | None) -> tuple[bool, str]`.

Steps:

- [ ] Write the failing test `tests/unit/cli/test_doctor.py`:

  ```python
  """`babylon doctor` reports config, provider lane, and DB reachability (ADR095 D1)."""

  from __future__ import annotations

  import babylon.cli.doctor as doctor_mod
  from babylon.cli import app
  from babylon.intelligence.providers import MuteProvider
  from typer.testing import CliRunner

  runner = CliRunner()


  def test_doctor_reports_config_dir_and_lane(monkeypatch, tmp_path) -> None:  # type: ignore[no-untyped-def]
      monkeypatch.setenv("BABYLON_CONFIG_DIR", str(tmp_path))
      monkeypatch.setattr(doctor_mod, "resolve_provider", lambda: MuteProvider())
      monkeypatch.setattr(doctor_mod, "check_database", lambda dsn: (False, "no DSN configured"))
      result = runner.invoke(app, ["doctor"])
      assert result.exit_code == 0
      assert str(tmp_path) in result.stdout
      assert "mute" in result.stdout
      assert "config.toml" in result.stdout


  def test_check_database_handles_missing_dsn() -> None:
      ok, detail = doctor_mod.check_database(None)
      assert ok is False
      assert "DSN" in detail or "dsn" in detail
  ```

- [ ] Run, expect FAIL:

  ```bash
  poetry run pytest tests/unit/cli/test_doctor.py -q
  ```

  Expected: `AttributeError`/`NotImplementedError` (stub `doctor` raises) — exit code 1.

- [ ] Replace `src/babylon/cli/doctor.py`:

  ```python
  """`babylon doctor` — diagnose the local install: config, provider lane, DB.

  Report-only skeleton (ADR095 D1). Extended by 096 (``--provision``) and 097
  (render-capability probe writing ``[render] tier`` into config.toml). Reuses
  the §A8 seam's config-dir precedence and provider resolution rather than
  reinventing them.
  """

  from __future__ import annotations

  import os

  import typer
  from rich.console import Console

  from babylon.intelligence.providers import (
      ProviderError,
      _config_dir,
      load_settings,
      resolve_provider,
  )

  console = Console()


  def check_database(dsn: str | None) -> tuple[bool, str]:
      """Best-effort reachability probe. Never raises: an unreachable DB is a
      reported condition, not a doctor crash (III.11 loud-but-degrade)."""
      if not dsn:
          return (False, "no DSN configured (set BABYLON_DATABASE_URL)")
      try:
          import psycopg

          with psycopg.connect(dsn, connect_timeout=2) as conn:
              conn.execute("SELECT 1")
      except ImportError as exc:
          return (False, f"psycopg not installed: {exc}")
      except psycopg.Error as exc:
          return (False, f"unreachable: {exc}")
      return (True, "reachable")


  def doctor() -> None:
      """Diagnose the local Babylon install (config, provider lane, database)."""
      cfg_dir = _config_dir(os.environ)
      config_toml = cfg_dir / "config.toml"
      credentials = cfg_dir / "credentials"

      console.print(f"[bold]config dir:[/bold] {cfg_dir}")
      console.print(
          f"  config.toml: {'present' if config_toml.exists() else 'absent (defaults apply)'}"
      )
      console.print(
          f"  credentials: {'present' if credentials.exists() else 'absent (mute/local only)'}"
      )

      try:
          settings = load_settings()
          console.print(f"[bold]intelligence mode:[/bold] {settings.mode}")
      except ProviderError as exc:
          console.print(f"[bold red]config error:[/bold red] {exc}")

      provider = resolve_provider()
      health = provider.health()
      lane = provider.endpoint.kind.value
      mark = "ok" if health.ok else "degraded"
      console.print(f"[bold]provider lane:[/bold] {lane} ({mark}: {health.detail})")

      db_ok, db_detail = check_database(os.environ.get("BABYLON_DATABASE_URL"))
      console.print(f"[bold]database:[/bold] {'ok' if db_ok else 'unavailable'} — {db_detail}")

      raise typer.Exit(code=0)
  ```

- [ ] Run, expect PASS:

  ```bash
  poetry run pytest tests/unit/cli/test_doctor.py -q
  ```

  Expected: `2 passed`.

- [ ] Lint + typecheck:

  ```bash
  poetry run ruff check src/babylon/cli/doctor.py tests/unit/cli/test_doctor.py && poetry run mypy src/babylon/cli/doctor.py
  ```

  Expected: clean.

- [ ] Commit:

  ```bash
  git add src/babylon/cli/doctor.py tests/unit/cli/test_doctor.py
  git commit -m "feat(cli): babylon doctor — config, provider health, DB reachability skeleton (ADR095 D1)" -m "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
  ```

---

## Task 3: `login` — write the Cloudflare credentials file at mode 0600

**Files:**

- Modify: `src/babylon/cli/login.py` (replace the Task 1 stub)
- Create: `tests/unit/cli/test_login.py`

**Interfaces:**

- Consumes: `babylon.intelligence.providers._config_dir`.
- Produces: `babylon.cli.login.login(api_key: str) -> None` writing
  `<config_dir>/credentials` with a `[cloudflare] api_key = "..."` table at mode 0600.

Steps:

- [ ] Write the failing test `tests/unit/cli/test_login.py`:

  ```python
  """`babylon login` writes the credentials file at mode 0600 (ADR095 D1)."""

  from __future__ import annotations

  import stat
  import tomllib

  from babylon.cli import app
  from typer.testing import CliRunner

  runner = CliRunner()


  def test_login_writes_0600_credentials(monkeypatch, tmp_path) -> None:  # type: ignore[no-untyped-def]
      monkeypatch.setenv("BABYLON_CONFIG_DIR", str(tmp_path))
      result = runner.invoke(app, ["login", "--api-key", "bk_test_123"])
      assert result.exit_code == 0
      creds = tmp_path / "credentials"
      assert creds.exists()
      mode = stat.S_IMODE(creds.stat().st_mode)
      assert mode == 0o600, f"expected 0600, got {mode:o}"
      data = tomllib.loads(creds.read_text())
      assert data["cloudflare"]["api_key"] == "bk_test_123"


  def test_login_creates_missing_config_dir(monkeypatch, tmp_path) -> None:  # type: ignore[no-untyped-def]
      nested = tmp_path / "deep" / "babylon"
      monkeypatch.setenv("BABYLON_CONFIG_DIR", str(nested))
      result = runner.invoke(app, ["login", "--api-key", "bk_x"])
      assert result.exit_code == 0
      assert (nested / "credentials").exists()
  ```

- [ ] Run, expect FAIL:

  ```bash
  poetry run pytest tests/unit/cli/test_login.py -q
  ```

  Expected: stub `login` raises `NotImplementedError` — exit code 1.

- [ ] Replace `src/babylon/cli/login.py`:

  ```python
  """`babylon login` — write the Cloudflare credentials file (mode 0600).

  The credentials file is the §A3 secret jurisdiction, read only by the
  intelligence lane; the engine never sees it. Written 0600 from creation so
  there is never a window where it is group/world-readable.
  """

  from __future__ import annotations

  import os

  import typer

  from babylon.intelligence.providers import _config_dir


  def login(
      api_key: str = typer.Option(
          ...,
          "--api-key",
          prompt="Cloudflare Babylon beta key",
          hide_input=True,
          help="Beta key for the babylon-api Worker (issued by seed-beta-key.sh).",
      ),
  ) -> None:
      """Write ~/.config/babylon/credentials with the Cloudflare beta key (0600)."""
      cfg_dir = _config_dir(os.environ)
      cfg_dir.mkdir(parents=True, exist_ok=True)
      creds = cfg_dir / "credentials"
      content = f'[cloudflare]\napi_key = "{api_key}"\n'

      # O_CREAT with 0600, then chmod to defeat any inherited umask widening.
      fd = os.open(creds, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
      with os.fdopen(fd, "w", encoding="utf-8") as fh:
          fh.write(content)
      os.chmod(creds, 0o600)
      typer.echo(f"Wrote {creds} (mode 0600).")
  ```

- [ ] Run, expect PASS:

  ```bash
  poetry run pytest tests/unit/cli/test_login.py -q
  ```

  Expected: `2 passed`.

- [ ] Lint + typecheck:

  ```bash
  poetry run ruff check src/babylon/cli/login.py tests/unit/cli/test_login.py && poetry run mypy src/babylon/cli/login.py
  ```

  Expected: clean.

- [ ] Commit:

  ```bash
  git add src/babylon/cli/login.py tests/unit/cli/test_login.py
  git commit -m "feat(cli): babylon login — write credentials file at mode 0600 (ADR095 D1)" -m "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
  ```

---

## Task 4: `telemetry`, `self-update`, `uninstall` — honest status and teardown

**Files:**

- Modify: `src/babylon/cli/telemetry.py`, `src/babylon/cli/self_update.py`,
  `src/babylon/cli/uninstall.py` (replace the Task 1 stubs)
- Create: `tests/unit/cli/test_ops.py`

**Interfaces:**

- Consumes: `babylon.intelligence.providers._config_dir`; stdlib `shutil`, `subprocess`, `tomllib`.
- Produces: `babylon.cli.telemetry.telemetry() -> None`;
  `babylon.cli.self_update.self_update() -> None`; `babylon.cli.uninstall.uninstall() -> None`.

Steps:

- [ ] Write the failing test `tests/unit/cli/test_ops.py`:

  ```python
  """telemetry / self-update / uninstall honest-status behavior (ADR095 D1)."""

  from __future__ import annotations

  import babylon.cli.self_update as su
  from babylon.cli import app
  from typer.testing import CliRunner

  runner = CliRunner()


  def test_telemetry_prints_local_only_status(monkeypatch, tmp_path) -> None:  # type: ignore[no-untyped-def]
      monkeypatch.setenv("BABYLON_CONFIG_DIR", str(tmp_path))
      result = runner.invoke(app, ["telemetry"])
      assert result.exit_code == 0
      assert "local" in result.stdout.lower()
      assert "unratified" in result.stdout.lower()


  def test_self_update_no_nix_is_graceful(monkeypatch) -> None:  # type: ignore[no-untyped-def]
      monkeypatch.setattr(su.shutil, "which", lambda _name: None)
      result = runner.invoke(app, ["self-update"])
      assert result.exit_code == 0
      assert "not installed via" in result.stdout.lower() or "nix not found" in result.stdout.lower()


  def test_self_update_invokes_nix_when_present(monkeypatch) -> None:  # type: ignore[no-untyped-def]
      calls: list[list[str]] = []
      monkeypatch.setattr(su.shutil, "which", lambda _name: "/usr/bin/nix")
      monkeypatch.setattr(su.subprocess, "run", lambda cmd, check: calls.append(cmd))
      result = runner.invoke(app, ["self-update"])
      assert result.exit_code == 0
      assert calls == [["nix", "profile", "upgrade", "babylon"]]


  def test_uninstall_prints_steps_deletes_nothing() -> None:
      result = runner.invoke(app, ["uninstall"])
      assert result.exit_code == 0
      assert "nix profile remove babylon" in result.stdout
      assert ".config/babylon" in result.stdout
  ```

- [ ] Run, expect FAIL:

  ```bash
  poetry run pytest tests/unit/cli/test_ops.py -q
  ```

  Expected: stubs raise `NotImplementedError` — failures.

- [ ] Replace `src/babylon/cli/telemetry.py`:

  ```python
  """`babylon telemetry` — honest local status (ingest lane unratified).

  The metrics collector (``babylon.metrics.collector``) is in-process only; no
  upload code exists and the telemetry-ingest lane is unratified (no ADR).
  This command reports that truth and echoes any ``[telemetry]`` config the
  player has set — it never uploads anything.
  """

  from __future__ import annotations

  import os
  import tomllib

  import typer

  from babylon.intelligence.providers import _config_dir


  def telemetry() -> None:
      """Show telemetry status (local, in-process only; ingest is unratified)."""
      typer.echo("Telemetry: local, in-process only (babylon.metrics.collector).")
      typer.echo("Upload lane: NOT configured — telemetry ingest is unratified (no ADR yet).")

      config_toml = _config_dir(os.environ) / "config.toml"
      if config_toml.exists():
          try:
              data = tomllib.loads(config_toml.read_text(encoding="utf-8"))
          except (OSError, tomllib.TOMLDecodeError) as exc:
              typer.echo(f"config.toml unreadable: {exc}")
              return
          section = data.get("telemetry")
          if isinstance(section, dict):
              typer.echo(f"[telemetry] config: {section}")
          else:
              typer.echo("[telemetry] config: (none set)")
      else:
          typer.echo("[telemetry] config: (no config.toml)")
  ```

- [ ] Replace `src/babylon/cli/self_update.py`:

  ```python
  """`babylon self-update` — wrap ``nix profile upgrade babylon``.

  Graceful no-op with an honest message when the install did not come through
  the Nix player channel (no ``nix`` on PATH). ADR095 D1: the flake wrapper
  and install.sh own the real upgrade path; this is the in-game shortcut.
  """

  from __future__ import annotations

  import shutil
  import subprocess

  import typer


  def self_update() -> None:
      """Upgrade Babylon via `nix profile upgrade babylon` (no-op if not Nix-installed)."""
      if shutil.which("nix") is None:
          typer.echo(
              "nix not found — Babylon was not installed via the Nix player channel; "
              "nothing to update."
          )
          raise typer.Exit(code=0)
      try:
          subprocess.run(["nix", "profile", "upgrade", "babylon"], check=True)
      except subprocess.CalledProcessError as exc:
          typer.echo(f"`nix profile upgrade babylon` failed (exit {exc.returncode}).")
          raise typer.Exit(code=1) from exc
      typer.echo("Babylon upgraded via nix profile.")
  ```

- [ ] Replace `src/babylon/cli/uninstall.py`:

  ```python
  """`babylon uninstall` — print the honest teardown (deletes nothing)."""

  from __future__ import annotations

  import typer


  def uninstall() -> None:
      """Print the manual teardown steps. This command removes NOTHING itself."""
      typer.echo("Babylon uninstall — manual teardown (this command deletes nothing):")
      typer.echo("  1. nix profile remove babylon     # if installed via the Nix player channel")
      typer.echo("  2. rm -rf ~/.local/share/babylon  # models + game data ($XDG_DATA_HOME/babylon)")
      typer.echo("  3. rm -rf ~/.config/babylon       # config.toml + credentials")
  ```

- [ ] Run, expect PASS:

  ```bash
  poetry run pytest tests/unit/cli/test_ops.py -q
  ```

  Expected: `4 passed`.

- [ ] Lint + typecheck the whole CLI package and its tests:

  ```bash
  poetry run ruff check src/babylon/cli tests/unit/cli && poetry run mypy src/babylon/cli
  ```

  Expected: clean.

- [ ] Commit:

  ```bash
  git add src/babylon/cli/telemetry.py src/babylon/cli/self_update.py src/babylon/cli/uninstall.py tests/unit/cli/test_ops.py
  git commit -m "feat(cli): babylon telemetry/self-update/uninstall — honest status + teardown (ADR095 D1)" -m "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
  ```

---

## Task 5: Delete the stdlib-shadowing `uuid` relic (D2)

**Ordering decision (per brief):** the `server`-extra split is folded into the PEP-621 conversion
in Task 6 to avoid touching the dependency tables twice; the `uuid` deletion is done here as its
own trivial, independently revertable commit. All 150+ `import uuid` sites repo-wide use the
stdlib API (`uuid.uuid4`, `uuid.UUID`, …); the PyPI `uuid ^1.30` package is a 2006-era relic that
shadows the stdlib module — deleting it is safe.

**Files:**

- Modify: `pyproject.toml` (remove line 80 `uuid = "^1.30"`)
- Modify: `poetry.lock` (regenerated by `poetry lock`)
- Create: `tests/unit/cli/test_uuid_relic_gone.py`

**Interfaces:** none (dependency-manifest change).

Steps:

- [ ] Write the failing test `tests/unit/cli/test_uuid_relic_gone.py`:

  ```python
  """The PyPI `uuid` relic is gone from pyproject; stdlib uuid still works (ADR095 D2)."""

  from __future__ import annotations

  import tomllib
  import uuid
  from pathlib import Path


  def test_pyproject_does_not_declare_uuid_dependency() -> None:
      data = tomllib.loads(Path("pyproject.toml").read_text(encoding="utf-8"))
      poetry_deps = data.get("tool", {}).get("poetry", {}).get("dependencies", {})
      project_deps = data.get("project", {}).get("dependencies", [])
      names = set(poetry_deps) | {d.split(">")[0].split("<")[0].split("=")[0].strip() for d in project_deps}
      assert "uuid" not in names, "PyPI uuid relic still declared — it shadows the stdlib module"


  def test_stdlib_uuid_still_functions() -> None:
      value = uuid.uuid4()
      assert isinstance(value, uuid.UUID)
  ```

  > NOTE: this test reads both `[tool.poetry.dependencies]` (its home in Task 5) and
  > `[project.dependencies]` (its home after Task 6), so it stays green across the conversion.

- [ ] Run, expect FAIL (relic still declared at `pyproject.toml:80`):

  ```bash
  poetry run pytest tests/unit/cli/test_uuid_relic_gone.py -q
  ```

  Expected: `test_pyproject_does_not_declare_uuid_dependency` fails with the assertion message.

- [ ] Remove the relic line from `pyproject.toml` (currently line 80):

  ```toml
  uuid = "^1.30"
  ```

  Delete that single line from `[tool.poetry.dependencies]`.

- [ ] Regenerate the lock and confirm consistency (needs package-index access; if offline the
  executor records the block rather than faking the lock):

  ```bash
  poetry lock && poetry check --lock
  ```

  Expected: `poetry check --lock` prints `All set!` (pyproject/lock consistent). `git diff --stat
  poetry.lock` shows `uuid` removed.

- [ ] Run, expect PASS:

  ```bash
  poetry run pytest tests/unit/cli/test_uuid_relic_gone.py -q
  ```

  Expected: `2 passed`.

- [ ] Commit:

  ```bash
  git add pyproject.toml poetry.lock tests/unit/cli/test_uuid_relic_gone.py
  git commit -m "fix(deps): delete stdlib-shadowing uuid ^1.30 relic (ADR095 D2)" -m "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
  ```

---

## Task 6: PEP-621 dependency layout + `server` extra split (D2/D3a)

Convert the main dependency table to PEP-621 `[project.dependencies]` and lift the eight legacy
web-stack packages into a `[project.optional-dependencies].server` extra, keeping the `poetry-core`
build backend and the `[tool.poetry.group.dev/docs.dependencies]` groups untouched (those become
PEP 735 groups in Task 7, at the uv switch). Caret constraints are translated to explicit
`>=,<` ranges (Poetry caret semantics: `^X.Y.Z`→`>=X.Y.Z,<(X+1).0.0` for `X≥1`; `^0.N`→
`>=0.N.0,<0.(N+1).0`). Still on Poetry, so verified with `poetry check` + `poetry lock`.

**Judgment call (recorded):** moving Django/DRF/gunicorn into the `server` extra means the default
install no longer satisfies `DJANGO_SETTINGS_MODULE`/`django-stubs`. To keep dev + CI gates green
while slimming the *player* closure, developer install (Task 7 `mise install`) and the gdal CI jobs
(Task 8) pull `--extra server`. Only the shipped Nix player closure (plan 094) gets the slim
default.

**Files:**

- Modify: `pyproject.toml` — replace `[tool.poetry.dependencies]` (lines 38–86) with
  `[project.dependencies]` + `[project.optional-dependencies]`; edit `dynamic` (line 20) to drop
  `"dependencies"`; keep `[tool.poetry] packages` and both `[tool.poetry.group.*]` tables.
- Modify: `poetry.lock` (regenerated).
- Create: `tests/unit/cli/test_pep621_layout.py`

**Interfaces:** none (manifest change); produces the `server` extra name consumed by Task 8.

Steps:

- [ ] Write the failing test `tests/unit/cli/test_pep621_layout.py`:

  ```python
  """PEP-621 dependency layout + server extra (ADR095 D3a/D2)."""

  from __future__ import annotations

  import tomllib
  from pathlib import Path

  DATA = tomllib.loads(Path("pyproject.toml").read_text(encoding="utf-8"))


  def _names(specs: list[str]) -> set[str]:
      out: set[str] = set()
      for spec in specs:
          head = spec.split(";")[0].strip()
          for sep in (">", "<", "=", "!", "~", "["):
              head = head.split(sep)[0]
          out.add(head.strip())
      return out


  def test_project_dependencies_is_static_list() -> None:
      assert isinstance(DATA["project"]["dependencies"], list)
      assert "dependencies" not in DATA["project"].get("dynamic", [])
      # classifiers stay poetry-derived
      assert "classifiers" in DATA["project"].get("dynamic", [])


  def test_no_legacy_poetry_dependencies_table() -> None:
      assert "dependencies" not in DATA["tool"]["poetry"], "legacy [tool.poetry.dependencies] remains"


  def test_server_extra_absorbs_legacy_web_stack() -> None:
      server = _names(DATA["project"]["optional-dependencies"]["server"])
      expected = {
          "ansible-dev-tools",
          "rstcheck",
          "doc8",
          "boto3",
          "django",
          "djangorestframework",
          "django-cors-headers",
          "gunicorn",
      }
      assert expected <= server


  def test_core_stays_in_default_deps() -> None:
      core = _names(DATA["project"]["dependencies"])
      for pkg in ("pydantic", "typer", "rich", "rustworkx", "openai"):
          assert pkg in core
      # server packages are NOT in the default set
      assert "django" not in core
      assert "gunicorn" not in core


  def test_build_backend_stays_poetry_core() -> None:
      assert DATA["build-system"]["build-backend"] == "poetry.core.masonry.api"
  ```

- [ ] Run, expect FAIL:

  ```bash
  poetry run pytest tests/unit/cli/test_pep621_layout.py -q
  ```

  Expected: `test_no_legacy_poetry_dependencies_table` and the `[project.dependencies]` assertions
  fail (deps still under `[tool.poetry.dependencies]`).

- [ ] Edit `pyproject.toml` line 20 to drop `"dependencies"` from `dynamic`:

  ```toml
  dynamic = ["classifiers"]
  ```

- [ ] Replace the entire `[tool.poetry.dependencies]` block (lines 38–86, with `uuid` already gone
  from Task 5) with a PEP-621 `[project.dependencies]` array placed immediately after `dynamic`
  (inside `[project]`, before `[tool.poetry]`). Preserve the two design comments verbatim as TOML
  comments above the moved lines:

  ```toml
  dependencies = [
      "pydantic>=2.10.3",
      "numpy>=1.26.4",
      "rustworkx>=0.18.0,<0.19.0",
      "sqlalchemy>=2.0.36",
      "pandas>=2.2.3",
      "jsonschema>=4.25.1,<5.0.0",
      "referencing>=0.37.0,<0.38.0",
      "openpyxl>=3.1.5,<4.0.0",
      "openai>=2.9.0,<3.0.0",
      "tokenizers>=0.19.1",
      "aiohttp>=3.13.3,<4.0.0",
      "backoff>=2.2.1",
      "ratelimit>=2.2.1",
      "typer>=0.15.1",
      "rich>=13.9.4",
      "coloredlogs>=15.0.1",
      "h3>=4.2.0,<5.0.0",
      "geopandas>=1.0.0",
      "pyproj>=3.6.0",
      "shapely>=2.0.0",
      "pyyaml>=6.0.2",
      "python-dotenv>=1.0.1",
      "requests>=2.32.3",
      "certifi>=2025.11.12",
      "filelock>=3.20.3",
      "scipy>=1.11.0",
      "xgi>=0.10.0,<0.11.0",
      "polars>=1.38.1,<2.0.0",
      "psycopg>=3.3.3,<4.0.0",
      "psycopg-pool>=3.3.0,<4.0.0",
      "pyarrow>=25.0.0,<26.0.0",
      "duckdb>=1.4.4,<2.0.0",
      "pgvector>=0.5.0,<0.6.0",
      "sentence-transformers>=5.6.0,<6.0.0",
      "tqdm>=4.66.0,<5.0.0",
  ]

  [project.optional-dependencies]
  # server: the legacy web-client stack (Django + DRF + gunicorn) plus the
  # ansible / rst / boto dev-ops tools. X.8 marks this stack legacy — it is
  # opt-in via `--extra server`; the default install slims toward the player
  # closure. Dev + CI install it (see plan Task 7/8); the Nix player closure
  # does not.
  server = [
      "ansible-dev-tools>=26.2.0,<27.0.0",
      "rstcheck>=6.2.5,<7.0.0",
      "doc8>=2.0.0,<3.0.0",
      "boto3>=1.42.59,<2.0.0",
      "django>=5.0,<6.0",
      "djangorestframework>=3.15",
      "django-cors-headers>=4.0",
      "gunicorn>=21.0",
  ]
  ```

  Then DELETE the now-empty `[tool.poetry.dependencies]` header and its former body. Keep the
  `# PyQt6 ...` and `# babylon_data ...` comments where they still apply (move the PyQt6 comment
  block into the `dependencies` array as an inline comment near `coloredlogs`, or drop it —
  executor's discretion; it is a historical note, not load-bearing).

- [ ] Regenerate the lock (Poetry 2.2.1 reads `[project.dependencies]` +
  `[project.optional-dependencies]` natively) and verify:

  ```bash
  poetry lock && poetry check --lock
  ```

  Expected: `All set!`. If `poetry lock` cannot resolve, STOP — this is the D3 resolution risk;
  report and consider the K-fallback (this task's PEP-621 layout is still valid; only D3b/uv would
  be affected).

- [ ] Confirm the env still installs with the extra (mypy hook needs Django):

  ```bash
  poetry install --extras server && poetry run mypy src/babylon/cli
  ```

  Expected: install succeeds; mypy `Success: no issues found`.

- [ ] Run the layout test, expect PASS:

  ```bash
  poetry run pytest tests/unit/cli/test_pep621_layout.py tests/unit/cli/test_uuid_relic_gone.py -q
  ```

  Expected: `7 passed`.

- [ ] Commit:

  ```bash
  git add pyproject.toml poetry.lock tests/unit/cli/test_pep621_layout.py
  git commit -m "refactor(deps): PEP-621 dependency layout + server extra split (ADR095 D2/D3a)" -m "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
  ```

---

## Task 7: Adopt uv — groups, lock, mise + pre-commit sweep (D3b/D3c/D3d)

Switch the toolchain to uv against one `uv.lock`: convert the dev/docs groups to PEP 735
`[dependency-groups]`, pin uv in `.mise.toml [tools]`, generate `uv.lock`, sweep `poetry run`→
`uv run` and `poetry install`→`uv sync` across `.mise.toml`, swap the pre-commit lock hook and
runner entries, then delete `poetry.lock` and the poetry `[tools]` pin. **K-fallback gate:** if
`uv lock` fails to resolve any dependency, STOP and report — do not force it.

**Files:**

- Modify: `pyproject.toml` — replace `[tool.poetry.group.dev.dependencies]` (88–127) and
  `[tool.poetry.group.docs.dependencies]` (129–135) with `[dependency-groups]`.
- Modify: `.mise.toml` — `[tools]` pin (17–19); task bodies (`poetry run`→`uv run`,
  `poetry install`→`uv sync`, lines 55, 62, 901, 1464; comment 29; description 50).
- Modify: `.pre-commit-config.yaml` — mypy (27), pytest-fast (39), lint-imports (52), radon-mi
  (61) runner entries; `poetry-lock-consistency` hook (166–171).
- Create: `uv.lock` (generated, committed).
- Delete: `poetry.lock`.
- Create: `tests/unit/cli/test_uv_migration.py`

**Interfaces:** produces `uv.lock` (consumed by plan 094) and the `uv`-based `mise` front door.

Steps:

- [ ] Write the failing test `tests/unit/cli/test_uv_migration.py`:

  ```python
  """uv single-lock migration invariants (ADR095 D3)."""

  from __future__ import annotations

  import tomllib
  from pathlib import Path

  ROOT = Path(".")
  PYPROJECT = tomllib.loads((ROOT / "pyproject.toml").read_text(encoding="utf-8"))
  MISE = (ROOT / ".mise.toml").read_text(encoding="utf-8")
  PRECOMMIT = (ROOT / ".pre-commit-config.yaml").read_text(encoding="utf-8")


  def test_dependency_groups_are_pep735() -> None:
      groups = PYPROJECT["dependency-groups"]
      assert "dev" in groups and "docs" in groups
      assert "group" not in PYPROJECT["tool"]["poetry"], "legacy [tool.poetry.group.*] remains"


  def test_uv_lock_committed_poetry_lock_gone() -> None:
      assert (ROOT / "uv.lock").exists(), "uv.lock must be generated + committed"
      assert not (ROOT / "poetry.lock").exists(), "poetry.lock must be deleted at the single-lock fork"


  def test_mise_has_no_poetry_invocations() -> None:
      assert "poetry run" not in MISE
      assert "poetry install" not in MISE
      # [tools] pin swapped poetry -> uv
      assert "\npoetry = " not in MISE
      assert "uv = " in MISE


  def test_precommit_lock_hook_is_uv() -> None:
      assert "uv lock --check" in PRECOMMIT
      assert "poetry check --lock" not in PRECOMMIT
      assert "poetry run" not in PRECOMMIT
  ```

- [ ] Run, expect FAIL:

  ```bash
  poetry run pytest tests/unit/cli/test_uv_migration.py -q
  ```

  Expected: all four fail (still Poetry groups/lock/tools).

- [ ] Convert the two Poetry groups to PEP 735. Replace `[tool.poetry.group.dev.dependencies]`
  (88–127) and `[tool.poetry.group.docs.dependencies]` (129–135) with a single
  `[dependency-groups]` table (place it after `[project.optional-dependencies]`):

  ```toml
  [dependency-groups]
  dev = [
      "pytest>=9.0.3,<10.0.0",
      "pytest-mock>=3.14.0,<4.0.0",
      "pytest-asyncio>=1.0.0,<2.0.0",
      "pytest-cov>=7.0.0,<8.0.0",
      "mypy>=1.13.0,<2.0.0",
      "ruff>=0.15.12,<0.16.0",
      "pre-commit>=4.0.0,<5.0.0",
      "commitizen>=4.1.0,<5.0.0",
      "types-jsonschema>=4.25.1.20251009,<5.0.0",
      "types-pyyaml>=6.0.12.20250915,<7.0.0",
      "types-ratelimit>=2.2.0.20250501,<3.0.0",
      "midiutil>=1.2.1,<2.0.0",
      "markdownify>=1.2.2,<2.0.0",
      "yamllint>=1.37.1,<2.0.0",
      "mido>=1.3.3,<2.0.0",
      "linkify-it-py>=2.0.3,<3.0.0",
      "pip-audit>=2.7.3,<3.0.0",
      "pytest-playwright>=0.8.0,<0.9.0",
      "radon>=6.0.1,<7.0.0",
      "xenon>=0.9.3,<0.10.0",
      "pytest-randomly>=4.0.1,<5.0.0",
      "mutmut>=3.4.0,<4.0.0",
      "deptry>=0.25.1,<0.26.0",
      "salib>=1.5.2,<2.0.0",
      "optuna>=4.9.0,<5.0.0",
      "optuna-dashboard>=0.20.0,<0.21.0",
      "types-docutils>=0.22.3.20251115,<0.23.0",
      "types-tqdm>=4.67.0.20250809,<5.0.0",
      "vulture>=2.14.0,<3.0.0",
      "hypothesis>=6.149.0,<7.0.0",
      "pytest-django>=4.8.0,<5.0.0",
      "django-stubs[compatible-mypy]>=5.0",
      "djangorestframework-stubs[compatible-mypy]>=3.15",
      "testcontainers[postgres]>=4.14.2,<5.0.0",
      "pytest-json-report>=1.5.0,<2.0.0",
      "pytest-html>=4.1.0,<5.0.0",
      "pytest-timeout>=2.4.0,<3.0.0",
      "import-linter>=2.13.0,<3.0.0",
  ]
  docs = [
      "sphinx>=9,<10",
      "sphinx-rtd-theme>=3.0.2,<4.0.0",
      "sphinx-autodoc-typehints<3.13",
      "myst-parser>=5.0.0,<6.0.0",
      "sphinx-autobuild>=2025.8,<2026.0.0",
      "sphinxcontrib-mermaid>=2.0.0,<3.0.0",
  ]
  ```

  Then DELETE both `[tool.poetry.group.*.dependencies]` headers and bodies. `[tool.poetry]`
  `packages` and `[build-system]` stay.

- [ ] Pin uv in `.mise.toml [tools]`. Capture the installed uv version and write it — do not
  invent a number:

  ```bash
  UV_VER="$(uv --version | awk '{print $2}')"
  echo "pinning uv = ${UV_VER}"
  ```

  Edit `.mise.toml` lines 17–19: remove the `poetry = "2.2.1" ...` line and add
  `uv = "<UV_VER>"` under `[tools]` (keep `python = "3.12"`):

  ```toml
  [tools]
  python = "3.12"
  uv = "<UV_VER>"  # single-lock fork (ADR095 D3, ruling U); uv2nix consumes this same lock
  ```

- [ ] Generate the uv lock (K-fallback gate):

  ```bash
  uv lock
  ```

  Expected: `Resolved N packages` and `uv.lock` created. If resolution FAILS on any dependency,
  STOP and report (do not hand-edit `uv.lock`; the K-fallback keeps Poetry two-locks).

- [ ] Verify uv installs the full dev+server env and the CLI runs:

  ```bash
  uv sync --extra server && uv run babylon --version
  ```

  Expected: sync resolves; `0.3.0`.

- [ ] Sweep `.mise.toml` deterministically:

  ```bash
  python - <<'PY'
  from pathlib import Path

  p = Path(".mise.toml")
  s = p.read_text(encoding="utf-8")
  s = s.replace("poetry run ", "uv run ")
  s = s.replace("poetry install --with dev", "uv sync --extra server")
  s = s.replace("(cd web && poetry install)", "(cd web && uv sync --extra server)")
  s = s.replace("poetry show --outdated | head -30", "uv pip list --outdated | head -30")
  p.write_text(s, encoding="utf-8")
  print("swept .mise.toml")
  PY
  ```

- [ ] Fix the two residual comment/description mentions the sweep does not catch. Edit `.mise.toml`
  line 29 (`raw \`poetry run\`` → `raw \`uv run\``) and confirm the description on line 50 became
  `uv run python tools/build_reference_db.py`. Then verify ZERO poetry references remain:

  ```bash
  rg -n 'poetry' .mise.toml || echo "OK: no poetry references remain"
  ```

  Expected: `OK: no poetry references remain`.

- [ ] Swap the pre-commit runner entries and lock hook. Edit `.pre-commit-config.yaml`:
  - lines 2–3 (header comment) `poetry run pre-commit install ...` → `uv run pre-commit install ...`
    and `poetry run pre-commit run --all-files` → `uv run pre-commit run --all-files`
  - line 27 `entry: poetry run mypy src` → `entry: uv run mypy src`
  - line 39 `entry: poetry run pytest ...` → `entry: uv run pytest ...` (same args)
  - line 52 `entry: poetry run lint-imports` → `entry: uv run lint-imports`
  - line 61 `poetry run radon mi ...` → `uv run radon mi ...` (inside the bash `-c`)
  - lines 166–171 (`poetry-lock-consistency`): rename to `uv-lock-consistency`,
    `name: uv lock --check`, `entry: uv lock --check`, `files: '^(pyproject\.toml|uv\.lock)$'`.

  Verify:

  ```bash
  rg -n 'poetry' .pre-commit-config.yaml || echo "OK: pre-commit clean"
  ```

  Expected: `OK: pre-commit clean`.

- [ ] Delete the old lock:

  ```bash
  git rm poetry.lock
  ```

- [ ] Smoke the migrated toolchain (needs Postgres for the full unit gate — bring it up first;
  record the skip honestly if Docker/Postgres is unavailable):

  ```bash
  mise run db:up
  uv run pytest tests/unit/cli -q
  uv run pytest tests/unit -x -q -m 'not red_phase and not slow and not requires_ollama and not requires_reference_db'
  ```

  Expected: CLI suite green; the broader unit smoke passes (or the executor records exactly which
  leg was skipped for lack of Postgres — never a fabricated pass).

- [ ] Run the migration invariants, expect PASS:

  ```bash
  uv run pytest tests/unit/cli/test_uv_migration.py -q
  ```

  Expected: `4 passed`.

- [ ] Commit:

  ```bash
  git add pyproject.toml uv.lock .mise.toml .pre-commit-config.yaml tests/unit/cli/test_uv_migration.py
  git rm --cached poetry.lock 2>/dev/null || true
  git commit -m "build(deps): adopt uv single lock — PEP 735 groups, mise + pre-commit sweep, drop poetry.lock (ADR095 D3)" -m "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
  ```

---

## Task 8: CI — rewrite the Python bootstrap and workflows for uv (D3e)

Rewrite the shared bootstrap composite and every workflow that provisioned Poetry so CI runs on uv
against `uv.lock`. The gdal jobs (mypy + web/Django tests) additionally install `--extra server`
because Django is now optional (Task 6).

**Files:**

- Modify: `.github/actions/bootstrap-python/action.yml` (full rewrite)
- Modify: `.github/workflows/ci.yml` (lines 51, 53, 77; add `server` input to gdal jobs)
- Modify: `.github/workflows/main.yml` (lines 48, 50, 74; `poetry run`→`uv run` sweep; server input)
- Modify: `.github/workflows/nightly.yml` (3.13 leg 199–216; `poetry run`→`uv run` sweep)
- Modify: `.github/workflows/docs.yml` (lines 31–60 — includes the `Set up Python` step at 31)
- Modify: `.github/workflows/copilot-setup-steps.yml` (lines 28–73 — includes `Set up Python 3.12` at 28)
- Modify: `.github/workflows/release.yml` (line 53 comment: `poetry.lock`→`uv.lock`)

**Interfaces:** consumes `uv.lock` + the `server` extra from Tasks 6/7.

> This task is pure CI config; it is verified with `actionlint` + residue greps rather than TDD.

Steps:

- [ ] Rewrite `.github/actions/bootstrap-python/action.yml` for uv (mise still installs the pinned
  toolchain, now including uv):

  ```yaml
  # Shared Python bootstrap (Program 15). ONE provisioner: mise installs the
  # exact [tools] pins from .mise.toml (python 3.12 + uv), so CI and local dev
  # run the identical toolchain against the single uv.lock. The venv cache key
  # hashes uv.lock AND .mise.toml (the uv pin lives there); bump -vN to evict.
  name: Bootstrap Python
  description: mise toolchain + uv-managed venv cache + uv sync

  inputs:
    gdal:
      description: >-
        Install GeoDjango system libraries (GEOS/GDAL/PROJ) — needed by anything
        importing Django settings, including mypy (django-stubs plugin loads
        settings at startup)
      required: false
      default: "false"
    docs:
      description: Install the docs dependency group
      required: false
      default: "false"
    server:
      description: >-
        Install the `server` optional-dependencies extra (Django/DRF/gunicorn +
        ansible/rst/boto). Required by any job that runs pytest or mypy, since
        DJANGO_SETTINGS_MODULE + django-stubs load Django at import.
      required: false
      default: "false"

  runs:
    using: composite
    steps:
      - name: Set up tools with mise
        uses: jdx/mise-action@v4
        with:
          cache: true

      - name: Load cached venv
        id: venv-cache
        uses: actions/cache@v6
        with:
          path: .venv
          key: venv-${{ runner.os }}-py3.12-uv-v1-${{ inputs.docs }}-${{ inputs.server }}-${{ hashFiles('uv.lock', '.mise.toml') }}

      - name: Sync dependencies (uv)
        shell: bash
        run: |
          EXTRA=""
          GROUP=""
          if [ "${{ inputs.server }}" = "true" ]; then EXTRA="--extra server"; fi
          if [ "${{ inputs.docs }}" = "true" ]; then GROUP="--group docs"; fi
          uv sync --frozen $EXTRA $GROUP

      # GeoDjango: settings/base.py uses the postgis backend, which loads
      # GEOS/GDAL/PROJ at import — pytest touching Django settings AND mypy
      # (django-stubs plugin) die with ImproperlyConfigured without these.
      - name: Install GeoDjango system libraries
        if: inputs.gdal == 'true'
        shell: bash
        run: sudo apt-get update && sudo apt-get install -y --no-install-recommends binutils libproj-dev gdal-bin
  ```

- [ ] Add `server: "true"` to every `bootstrap-python` invocation that passes `gdal: "true"`.
  Find them and edit each:

  ```bash
  rg -n 'gdal: "true"' .github/workflows/ci.yml .github/workflows/main.yml
  ```

  For each match, the `with:` block gains `server: "true"` alongside `gdal: "true"` (the gdal jobs
  are exactly the Django/mypy jobs that need the extra).

- [ ] In `.github/workflows/ci.yml`: change the mypy cache-key `hashFiles('poetry.lock')` →
  `hashFiles('uv.lock')` on lines 51 and 53; change line 77 `run: poetry check --lock` →
  `run: uv lock --check`.

- [ ] In `.github/workflows/main.yml`: change `hashFiles('poetry.lock')` → `hashFiles('uv.lock')`
  on lines 48 and 50; change line 74 `run: poetry check --lock` → `run: uv lock --check`. Then
  sweep the `poetry run` invocation lines:

  ```bash
  sed -i 's/poetry run /uv run /g' .github/workflows/main.yml
  rg -n 'poetry' .github/workflows/main.yml || echo "OK: main.yml clean"
  ```

  Expected: `OK: main.yml clean`.

- [ ] Rewrite the nightly 3.13 forward-compat leg. Replace `.github/workflows/nightly.yml` lines
  199–216 with a uv-based setup (uv manages the 3.13 interpreter directly; the leg deliberately
  runs a different python than the mise pin):

  ```yaml
      # Documented exception to the mise-pinned toolchain: this leg exists
      # precisely to run a DIFFERENT python (3.13) than .mise.toml pins.
      - name: Install uv
        uses: astral-sh/setup-uv@v6
      - name: Install dependencies on 3.13
        run: uv sync --frozen --extra server --python 3.13
      - name: Install GeoDjango system libraries
        run: sudo apt-get update && sudo apt-get install -y --no-install-recommends binutils libproj-dev gdal-bin
      # Max verbosity by owner ruling (2026-07-11): CI logs are agent-read.
      - name: Full suite on 3.13
        run: >-
          uv run --python 3.13 pytest tests --ignore=tests/unit/ai --ignore=tests/integration/web
          -o addopts="" -vv -ra -l --tb=long --durations=0
          -m "not red_phase and not requires_reference_db and not pacing_gate"
  ```

  Then sweep the remaining `poetry run` lines in nightly.yml:

  ```bash
  sed -i 's/poetry run /uv run /g' .github/workflows/nightly.yml
  rg -n 'poetry' .github/workflows/nightly.yml || echo "OK: nightly.yml clean"
  ```

  Expected: `OK: nightly.yml clean`.

- [ ] Rewrite `.github/workflows/docs.yml` lines 31–60 for uv (this range includes the
  `Set up Python` step at line 31, which the uv rewrite removes):

  ```yaml
        - name: Install uv
          uses: astral-sh/setup-uv@v6

        - name: Install dependencies (docs group)
          run: uv sync --frozen --group docs

        - name: Build documentation
          run: |
            cd docs
            uv run sphinx-build -b html . _build/html
  ```

  Remove the now-dead `Set up Python` / `Install Poetry` / `Load cached venv` / `Install project`
  steps (uv manages python + venv + caching). Verify:

  ```bash
  rg -n 'poetry' .github/workflows/docs.yml || echo "OK: docs.yml clean"
  ```

- [ ] Rewrite `.github/workflows/copilot-setup-steps.yml` lines 28–73 for uv (this range
  includes the `Set up Python 3.12` step at line 28, which the uv rewrite removes):

  ```yaml
        - name: Install uv
          uses: astral-sh/setup-uv@v6

        - name: Sync dependencies (dev + server)
          run: uv sync --frozen --extra server

        - name: Install mise (task runner)
          uses: jdx/mise-action@v4
          with:
            install: true

        - name: Install pre-commit hooks
          run: uv run pre-commit install --install-hooks

        - name: Verify environment
          run: |
            echo "=== Python version ==="
            python --version
            echo "=== uv version ==="
            uv --version
            echo "=== Installed packages ==="
            uv tree | head -50
            echo "=== Available mise tasks ==="
            mise tasks 2>/dev/null || echo "mise tasks not available"
            echo "=== Quick sanity check ==="
            uv run python -c "import babylon; print(f'babylon version: {babylon.__version__}')"
  ```

  Verify:

  ```bash
  rg -n 'poetry' .github/workflows/copilot-setup-steps.yml || echo "OK: copilot clean"
  ```

- [ ] Update the stale comment in `.github/workflows/release.yml` line 53
  (`the version poetry.lock resolves` → `the version uv.lock resolves`).

- [ ] Lint all workflows and confirm the only remaining `poetry` mentions repo-wide in CI are none:

  ```bash
  uv run --with pre-commit pre-commit run actionlint --all-files || actionlint .github/workflows/*.yml
  rg -n 'poetry' .github/workflows .github/actions || echo "OK: CI is poetry-free"
  ```

  Expected: actionlint reports no errors; `OK: CI is poetry-free`.

- [ ] Commit:

  ```bash
  git add .github/actions/bootstrap-python/action.yml .github/workflows
  git commit -m "ci: migrate bootstrap + workflows from poetry to uv single lock (ADR095 D3e)" -m "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
  ```

---

## Task 9: Final gate — battery, residue sweep, docs, CHANGELOG

Run the full local gate on uv, confirm no executable poetry references remain outside historical
docs, and update the two in-scope command surfaces (`README.md`, `CLAUDE.md`). The wider ~docs
debt is recorded as an honest follow-up, not fixed here (accuracy over comprehensiveness).

**Files:**

- Modify: `README.md` (poetry command blocks — lines 189, 222–224 and any siblings)
- Modify: `CLAUDE.md` (the poetry command references at lines 109, 161, 224)
- Modify: `CHANGELOG.md` (add an Unreleased note)

**Interfaces:** none.

Steps:

- [ ] Update the in-scope command blocks. In `README.md`, replace `poetry install` → `uv sync
  --extra server`, `poetry run pytest ...` → `uv run pytest ...`, and `poetry run pre-commit
  install` → `uv run pre-commit install` in the quickstart/testing blocks. In `CLAUDE.md`, replace
  the three `poetry run ...` references with `uv run ...` and update the "raw-poetry exceptions"
  note (line 161) to name the uv leg. Do NOT rewrite the wider `docs/` tree in this task.

- [ ] Record the docs-debt follow-up honestly — capture the real count rather than asserting one:

  ```bash
  echo "poetry references still in docs/ (out-of-scope follow-up): $(rg -l 'poetry run|poetry install' docs | wc -l) files"
  ```

  Note that number in the CHANGELOG entry below (do not fix them here).

- [ ] Add a CHANGELOG note under `## Unreleased` → `### Feat` (or a new `### Build`):

  ```markdown
  - **packaging**: babylon CLI entry point (play/doctor/login/telemetry/self-update/uninstall),
    server extra split + uuid relic deletion, and the uv single-lock migration (ADR095). Follow-up
    debt: docs/ still references poetry in N files (tracked separately).
  ```

  Replace `N` with the count captured above.

- [ ] Confirm no executable poetry references remain in the toolchain surfaces (docs are the only
  allowed residue):

  ```bash
  rg -n 'poetry (run|install|lock|check)' .mise.toml .pre-commit-config.yaml .github || echo "OK: toolchain is poetry-free"
  ```

  Expected: `OK: toolchain is poetry-free`.

- [ ] Run the full local gate on uv (bring Postgres up first for the unit + regression legs):

  ```bash
  mise run db:up
  mise run lint
  mise run typecheck
  mise run test:unit
  mise run qa:regression
  ```

  Expected: ruff clean; mypy `Success: no issues found`; `test:unit` all green; `qa:regression`
  reports baseline match. If Postgres/Docker is unavailable, the executor records exactly which
  leg was skipped — never a fabricated pass (Amendment Q honesty).

- [ ] Verify the shipped console entry point one more time end-to-end:

  ```bash
  uv run babylon --help && uv run babylon --version
  ```

  Expected: full subcommand list; `0.3.0`.

- [ ] Commit:

  ```bash
  git add README.md CLAUDE.md CHANGELOG.md
  git commit -m "docs(packaging): switch README/CLAUDE command blocks to uv; changelog note (ADR095)" -m "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
  ```

---

## ADR-clause coverage

| ADR095 clause | What it requires | Task(s) |
| --- | --- | --- |
| D1 — entry point | `[project.scripts] babylon = "babylon.cli:app"` | Task 1 |
| D1 — `play` (default) | default subcommand reusing `__main__` demo | Task 1 |
| D1 — `doctor` | provision/diagnose skeleton (config + provider + DB) | Task 2 |
| D1 — `login` | credentials file at 0600 | Task 3 |
| D1 — `telemetry` | honest status stub | Task 4 |
| D1 — `self-update` | wraps `nix profile upgrade babylon` | Task 4 |
| D1 — `uninstall` | prints honest teardown | Task 4 |
| D2 — `uuid` relic deletion | remove stdlib-shadowing `uuid ^1.30` | Task 5 |
| D2 — `server` extra split | 8 legacy web-stack pkgs → `[server]` extra | Task 6 |
| D3a — PEP-621 dep tables | `[project.dependencies]`, drop `dynamic` deps | Task 6 |
| D3b — uv lock (ruling U) | PEP 735 groups, `uv lock`, delete `poetry.lock` | Task 7 |
| D3c — mise `poetry run`→`uv run` | 126-site sweep + install→sync | Task 7 |
| D3d — pre-commit swap | lock hook + runner entries → uv | Task 7 |
| D3e — CI migration | bootstrap composite + all workflows → uv | Task 8 |
| D3 — safety net | closure/regression gate + full battery | Tasks 7, 9 |
| D3f — docs sweep (in-scope) | README + CLAUDE command blocks only | Task 9 |

## Recorded gates and honest-status notes

- **K-fallback (not executed):** if `uv lock` (Task 7) or `poetry lock` (Task 6) fails to resolve
  any dependency, STOP and report. Tasks 1–6 stand under Poetry even if D3 is aborted.
- **Postgres-dependent gates:** `mise run test:unit` and `mise run qa:regression` require the
  isolated local Postgres (`mise run db:up`, port 5433). On a box without Docker/Postgres the
  executor records the skip; it never claims a pass it did not observe.
- **Django-behind-extra consequence:** dev + gdal-CI installs pull `--extra server` so existing
  gates stay green; only the Nix player closure (plan 094) gets the slim default. Judgment call —
  the ADR mandates the split but not the dev/CI opt-in mechanism.
- **Out-of-scope docs debt:** the wider `docs/` tree still references poetry (count captured in
  Task 9). Left as a tracked follow-up per the demand-driven / accuracy-over-comprehensiveness
  documentation principle.
- **`babylon_data` symlink:** `[tool.poetry] packages` includes `babylon_data` (a committed
  symlink to an external drive). `uv sync`'s editable build of the project may warn if the symlink
  is dangling on the executor's box; this is pre-existing and not introduced by this plan.
