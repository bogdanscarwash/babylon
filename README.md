# Babylon: The Fall of America

Babylon is an entertainment-first emergent political-economy game. Babylon is
not a forecast and not a scientific reproduction. Theory constrains the causal
model but does not predetermine results.

Determinism proves computational identity, not scientific truth. Historical
cases test causal signatures and counterfactual behavior. The Bevy client
observes the campaign. Player interventions belong to Gate 5.

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

Babylon is a causal sandbox with a fixed weekly tick. Conditions, choices, and
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
- a frozen Python reference for retained reference tests

The planned decision cycle adds player and AI intent plus views limited by
player knowledge.

Read [`NORTH_STAR.md`](NORTH_STAR.md) for the full system model. Read
[`CONSTITUTION.md`](CONSTITUTION.md) v4.1.0 for the constitutional law.

## Live system

The Bevy window observes one durable Michigan campaign with 83 county QCEW
baselines. It has a county map, an inspector, and a production display with
3D cohort columns and a compact 2D option. The physical scenario contains
five Designed county-industry cohorts for 16 weeks. Two committed presets
compare the same productive configuration with different `sheet-transfer` durations.

The runtime commits weekly changes to Postgres. The window receives read
capabilities and controls pause, step, and speed through anonymous pipes.
Full observer and player-knowledge preview use different database roles.
The preview displays only granted facts. It has no material grants.

The live Rust path uses these crates:

- `babylon-kernel` for deterministic types
- `babylon-graph` for relations and world data
- `babylon-bsl` for the BSL language
- `babylon-tick` for weekly judgment
- `babylon-material-circuit` for physical production and routed freight
- `babylon-persistence` for the durable runtime and restricted readers
- `babylon-client` for the Bevy viewer

The Python engine is a frozen behavioral reference. Its tests, traces, and
goldens specify that behavior. Rust ports must keep it or record a replacement
decision. Python also prepares data and runs other periphery.

<!-- Vale: this paragraph preserves literal persistence and schema identifiers. -->
<!-- vale ste.UnapprovedWords = NO -->
<!-- vale ste.NounClusters = NO -->
Deterministic reference SQLite is a build artifact. The frozen Python reference
retains a separate mutable `RuntimeDatabase` SQLite store. Rust owns authoritative game-managed
Postgres and marker-last committed envelopes. Archive verification can lag
the durable week. The window shows that lag.

Python retains data tooling and
the frozen reference. It does not write the campaign shown in Bevy.
<!-- vale ste.NounClusters = YES -->
<!-- vale ste.UnapprovedWords = YES -->

The Django browser client in `web/` is legacy. Its failures do not gate v1.

## Install and check

The repository uses `mise.lock` to pin tool downloads and checksums. Start in a new clone:

Install the native Debian prerequisites and rustup described in
[`SETUP_GUIDE.md`](SETUP_GUIDE.md). Rust commands use the workspace toolchain.
uv installs Python dependencies from the committed lock.

```bash
mise trust
mise run setup
```

Run the frozen Python reference smoke test:

```bash
mise run reference:python-smoke
```

Run the repository check:

```bash
mise run check
```

Open or continue the native observer game:

```bash
mise run play
```

The launcher builds the runtime and client, reuses a reachable local database,
and starts at the campaign's durable week. New campaigns start at week zero.
Use the in-game menu to start a new campaign, reopen a saved campaign, or
compare two committed scenarios. Saved campaigns stay in the database.
See [`SETUP_GUIDE.md`](SETUP_GUIDE.md) for launch options and host requirements.

## Why Python tests continue

The Rust port does not make all Python tests obsolete. Python owns data builds,
periphery, and the frozen reference. Its behavioral tests can prove that a Rust
port kept the same external result.

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
- `src/babylon/` contains the frozen reference and Python periphery.
- `tests/` contains unit, integration, scenario, and contract tests.
- `data/` contains source artifacts and the reference data artifact.
- `ai/decisions/` contains architecture decision records.
- `docs/` contains the Sphinx manual.
- `project/` contains non-live context from earlier plans.
- `web/` contains the legacy browser, which uses Django.

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
