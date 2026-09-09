# Babylon: The Fall of America

Babylon is an entertainment-first emergent political-economy game. Babylon is
not a forecast and not a scientific reproduction. Theory constrains the causal
model but does not predetermine results.

Determinism proves computational identity, not scientific truth. Historical
cases test causal signatures and counterfactual behavior. The Bevy client
observes the campaign. Player interventions belong to Gate 5.

## Download the native preview

The [0.4.0 preview](https://github.com/percy-raskova/babylon/releases/tag/v0.4.0)
lets you observe the Michigan economy, advance four-week periods, inspect
production and staffing, and compare saved campaigns. Player actions are not
implemented yet.

### Prerequisites

<!-- Vale: these prerequisites name the actual graphics API and C library. -->
<!-- vale Vale.Spelling = NO -->
Use an Ubuntu 24.04 x86_64 desktop (glibc 2.39+) and Python 3.12 or newer.
You also need a Vulkan graphics driver and local
[Docker Engine with the Docker Compose plugin](https://docs.docker.com/engine/install/ubuntu/).
We qualify this preview on Ubuntu 24.04 x86_64 only.
<!-- vale Vale.Spelling = YES -->
Install any missing desktop libraries and the download tool:

```sh
sudo apt-get install curl python3 libpq5 libasound2t64 libudev1 libwayland-client0 \
  libxkbcommon0 libxkbcommon-x11-0 libx11-6 libxi6 libxrandr2 libxcursor1 \
  libvulkan1 mesa-vulkan-drivers
```

Docker must be running and usable by your account. Check it before continuing:

```sh
docker info
docker compose version
```

### Download, check, and start

Download the archive and checksum file into the directory where you want to keep
the preview:

```sh
curl -fLO https://github.com/percy-raskova/babylon/releases/download/v0.4.0/babylon-0.4.0-linux-x86_64.tar.gz
curl -fLO https://github.com/percy-raskova/babylon/releases/download/v0.4.0/babylon-0.4.0-linux-x86_64.tar.gz.sha256
sha256sum --check babylon-0.4.0-linux-x86_64.tar.gz.sha256
```

After the archive check reports `OK`, extract it and check the contents:

```sh
tar -xzf babylon-0.4.0-linux-x86_64.tar.gz
cd babylon-0.4.0-linux-x86_64
sha256sum --check SHA256SUMS
```

After all files pass, run the smoke check and open the observer:

```sh
./babylon --smoke && ./babylon
```

The smoke check creates a campaign, commits one period, restarts the runtime,
and verifies the saved state without a window. It does not test your GPU.
`./babylon` opens the observer.

The first launch downloads and builds the packaged Postgres image through Docker.
It needs internet access and free disk space for the image and database.
Later launches reuse the build cache and database. The runtime and window run as
native programs. Only the database runs in Docker.

The archive includes the native binaries, assets, and Python launcher dependencies.
You do not need a checkout or Rust compiler. Campaigns persist in a Docker volume
isolated by your user and preview version. The database listens on the local
computer only.

When you quit the observer, the database remains available.
`./babylon --stop-database` stops it and retains saves.
Keep the volume and old archive to revisit that version's campaigns.
Saves cannot move between preview versions.

Read the [download guide](tools/release/DOWNLOAD.md) for controls, save locations,
and notices. The release's
[`release-provenance.json`](https://github.com/percy-raskova/babylon/releases/download/v0.4.0/release-provenance.json)
identifies the exact source and qualification run for the download.

## Development milestones

The four executable gates are:

<!-- Vale: each protected item is a governed gate name. -->
<!-- vale Vale.Terms = NO -->
<!-- vale ste.UnapprovedWords = NO -->
<!-- vale ste.NounClusters = NO -->
1. **PostgreSQL/H3/Archive decision-loop slice**
1. **Productive & distributive circuit**
1. **Player agency**
1. **COVID emergence benchmark**
<!-- vale Vale.Terms = YES -->
<!-- vale ste.UnapprovedWords = YES -->
<!-- vale ste.NounClusters = YES -->

[![Project license](https://img.shields.io/badge/code-AGPL--3.0--or--later-blue.svg)](LICENSE)
[![Asset license](https://img.shields.io/badge/assets-CC0--1.0-lightgrey.svg)](LICENSE-ASSETS)

## What Babylon is

Babylon is a causal sandbox with a fixed four-week tick. Conditions, choices, and
feedback change a shared world. The engine applies rules and produces a stable
tick report.

Rust owns game judgment and world hashes. BSL has live rules, but no executable
shock vocabulary or shock content. Planned shocks must add pressure while the
engine derives downstream results.

The political-economy model gives the sandbox its game domain. At a higher
level, the live engine has:

- typed world data
- ordered causal rules
- committed Rust tick reports and checkpoints
- reproducible reference-data artifacts

The planned decision cycle adds player and AI intent plus views limited by
player knowledge.

Read [`NORTH_STAR.md`](NORTH_STAR.md) for the full system model. Read
[`CONSTITUTION.md`](CONSTITUTION.md) v4.2.0 for the constitutional law.

## Live system

The Bevy window observes one durable Michigan campaign with 83 county QCEW
baselines. It has a county map, an inspector, and a production display with
3D cohort columns and a compact 2D option. The physical scenario contains
five Designed county-industry cohorts. The supplied parameters cover 16 four-week
periods (64 weeks); each campaign can select a shorter horizon. Standard and
delayed delivery use the route durations in that campaign's authored parameters.
The comparison shows the same committed period in two saved campaigns. Each
campaign retains its own parameters, so their differences can extend beyond
route duration.

The runtime commits four-week changes to Postgres. The window receives read
capabilities and controls pause, step, and speed through anonymous pipes.
Full observer and player-knowledge preview use different database roles.
The preview displays only granted facts. It has no material grants.

The live Rust path uses these crates:

- `babylon-kernel` for deterministic types
- `babylon-graph` for relations and world data
- `babylon-bsl` for the BSL language
- `babylon-tick` for four-week judgment
- `babylon-material-circuit` for physical production and routed freight
- `babylon-persistence` for the durable runtime and restricted readers
- `babylon-client` for the Bevy viewer

Rust owns mechanics and their executable contracts. The Python engine is
retired; retained source datasets, language-neutral vectors, and Git history
preserve its evidence. Python prepares reference data and runs operator tools.

<!-- Vale: this paragraph preserves literal persistence and schema identifiers. -->
<!-- vale ste.UnapprovedWords = NO -->
<!-- vale ste.NounClusters = NO -->
Deterministic reference SQLite is a build artifact. Rust owns authoritative game-managed
Postgres and marker-last committed envelopes. Archive verification can lag
the durable period. The window shows that lag.

Python tooling does not write the campaign shown in Bevy.
<!-- vale ste.NounClusters = YES -->
<!-- vale ste.UnapprovedWords = YES -->

## Develop from source

The repository uses `mise.lock` to pin tool downloads and checksums. Start in a new clone:

Install the native Debian prerequisites and rustup described in
[`SETUP_GUIDE.md`](SETUP_GUIDE.md). Rust commands use the workspace toolchain.
uv installs Python dependencies from the committed lock.

```bash
mise trust
mise run setup
```

Run the repository check:

```bash
mise run check
```

Open or continue the native observer:

```bash
mise run play
```

The launcher builds the runtime and client, reuses a reachable local database,
and starts at the campaign's durable period. New campaigns start at period zero.
Use the in-game menu to start a new campaign, reopen a saved campaign, or
compare two committed scenarios. Saved campaigns stay in the database.
See [`SETUP_GUIDE.md`](SETUP_GUIDE.md) for launch options and host requirements.

## Why Python tests continue

Python tests protect the retained data builders, repository commands, provider
integrations, and operator tools. Rust tests own mechanics, persistence, and
replay. Tests of the retired Python engine have been removed.

Use the smallest applicable test first. Then run the full gate for the changed
area:

```bash
mise run test:q -- tests/unit/path/to/test_file.py
mise run rust:check-no-docs
mise run check
```

`pytest` checks Python behavior and language-neutral contracts. Cargo checks
the Rust engine. A port can retire an engine-specific Python test after a
durable replacement contract exists.

## Repository map

- `rust/crates/` contains the shipping engine and Bevy client.
- `src/babylon/` contains data and operator tooling.
- `tests/` contains unit, integration, scenario, and contract tests.
- `data/` contains source artifacts and the reference data artifact.
- `ai/decisions/` contains architecture decision records.
- `docs/` contains the Sphinx manual.
- `project/` contains non-live context from earlier plans.

<!-- Vale: the next sentence preserves exact control-surface terminology. -->
<!-- vale ste.UnapprovedWords = NO -->
Linear alone owns current status and work. The contributor guide links its
control surface.
<!-- vale ste.UnapprovedWords = YES -->

## Contributor path

Read [`CONTRIBUTORS.md`](CONTRIBUTORS.md) before you make a change. Create a lane
from `dev`, use TDD, and run the gates that `CLAUDE.md` assigns to the changed
area.

Do not report a planned system as complete. Check the source and an executable test
before you update a live status claim.

## License

The source uses `AGPL-3.0-or-later`. Shipped game assets use `CC0-1.0`. See
[`LICENSING.md`](LICENSING.md) for the directory inventory and legacy asset
notes.
