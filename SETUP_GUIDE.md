# Install Babylon

These instructions install Babylon, run two checks, and open the native Bevy
observer game.

Read [`NORTH_STAR.md`](NORTH_STAR.md) for the game purpose, client status,
validation standard, and gate order. [`CONSTITUTION.md`](CONSTITUTION.md)
v4.1.0 is the authority.

## 1. Prepare the host

Babylon development targets Debian Linux. On Windows, use Debian in
[WSL](https://learn.microsoft.com/en-us/windows/wsl/install).

<!-- Vale: this paragraph preserves literal Docker product and command names. -->
<!-- vale ste.UnapprovedWords = NO -->
Install [Git](https://git-scm.com/downloads),
[`mise`](https://mise.jdx.dev/getting-started.html),
[rustup](https://rustup.rs/), and Docker Engine with the Docker Compose
plug-in from their official guides. Docker Compose is a host prerequisite
because `mise run setup` starts Postgres.
Check them:
<!-- vale ste.UnapprovedWords = YES -->

```bash
git --version
mise --version
rustup --version
docker compose version
```

The repository pins its language tools. It does not install Docker on the host.

Install these Debian packages before `mise run setup`:

```bash
sudo apt-get update
sudo apt-get install -y build-essential pkg-config git-lfs libssl-dev libpq-dev \
  libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev libx11-dev \
  libxcursor-dev libxi-dev libxrandr-dev libvulkan-dev mesa-vulkan-drivers
```

## 2. Get Babylon

Clone the repository and select `dev`:

```bash
git clone https://github.com/percy-raskova/babylon.git
cd babylon
git checkout dev
```

Trust the local `mise` settings:

```bash
mise trust
```

Read a script before you grant trust when you did not get the clone from the
official repository.

## 3. Install the tools

Run the repository installation task:

```bash
mise run setup
```

The task installs the pinned tools, project dependencies, and local hooks. The
first run can be slow.

Daily development and reference data use the same Python 3.12.14 environment.
Python in `mise.lock` includes SQLite 3.53.1. `uv.lock` pins dependencies.
For reference-data checks, see the
[reference-data instructions](docs/how-to/reference-data-pipeline.rst).

## 4. Open the observer game

<!-- Vale: this paragraph preserves literal Bevy and host package names. -->
<!-- vale ste.UnapprovedWords = NO -->
<!-- vale ste.NounClusters = NO -->
A Bevy build needs Rust and Cargo plus the host window,
input, and audio development libraries. `rustup` selects the pinned
Rust and Cargo from `rust/rust-toolchain.toml`. The Debian package set
includes `libasound2-dev`, `libudev-dev`, `libwayland-dev`, and
`libxkbcommon-dev`.
<!-- vale ste.NounClusters = YES -->
<!-- vale ste.UnapprovedWords = YES -->

From the repository root, open or continue a campaign:

```bash
mise run play
```

The launcher builds `babylon-runtime` and `babylon-client` with native Cargo
from `rust/`. It reuses a reachable local database. If the default
database is
unavailable, it starts the repository database with `db:up` and checks again.
By default, the launcher connects to `babylon_test` on `127.0.0.1:5433`.

The runtime initializes a new database, installs the observer read roles,
and connects to Bevy through anonymous pipes. The window receives read
credentials. New campaigns start at week zero. When you open a saved
campaign, the runtime reconciles its committed checkpoint before a new week.

The campaign menu has controls to continue, start, open, or compare campaigns.
It includes a scenario with longer `sheet-transfer` durations. The map shows 83
Michigan county baselines. The production display shows five Designed
county-industry cohorts, actual freight lots, and committed production for
16 weeks.

Observed QCEW jobs and wages are source records. Designed physical
quantities and labor-hours use declared scenario values. Player interventions
belong to Gate 5.

For a new campaign that keeps saved worlds:

```bash
mise run play -- --new
mise run play -- --new --preset delayed
```

Use `--campaign UUID` to open a specific saved campaign. The runtime recovers
its stored scenario. It refuses an explicit `--preset` that differs.
The launcher builds native binaries. The `--no-build` option uses the native
binaries on disk.
Use `mise run play` to connect the client to its runtime.

The continuation pointer is a plain UUID in
`$XDG_STATE_HOME/babylon/observer-campaign`, or
`~/.local/state/babylon/observer-campaign` by default.
The pointer is a personal preference. Campaign data stays in Postgres.

For a dedicated local database, set `BABYLON_RUNTIME_DSN` to an explicit
local host, port, database, user, and password before launch. Create that
database first. The launcher refuses an unavailable custom target. It does
not start a different database. It reuses a reachable target and leaves
the shared container unchanged.

## 5. Run the repository check

Run the standard local gate:

```bash
mise run check
```

The gate checks format, lint, types, and Python unit contracts. Rust changes
also use the Rust gate:

```bash
mise run rust:check-no-docs
```

Python tests have value during the Rust port. They test the frozen reference,
data tools, and language-neutral behavior contracts.

## 6. Run the optional frozen-reference smoke test

Run the frozen Python reference for one small tick:

```bash
mise run reference:python-smoke
```

The command must show an initial tick, one simulation step, and a completed
tick. The command does not prove that the Bevy client has player actions.


## Make a contribution

Read [`CONTRIBUTORS.md`](CONTRIBUTORS.md) before a change. Create a lane from
`dev`, add a failing test first, and open a PR against `dev`.

Use this command for a commit:

```bash
mise run commit -- "type(scope): description"
```

Do not commit directly to `dev` or `main`.

## Fault help

If `mise` is not available after installation, open a new terminal. Then run
`mise --version` again.

If `mise` does not load the repository settings, run `mise trust` from the
repository root.

If dependency installation stops, run `mise run setup` again and keep the first error.
Open an [issue](https://github.com/percy-raskova/babylon/issues) with the command,
host system, and full error text.
