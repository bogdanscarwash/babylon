# Versioning Rigor — Design

Date: 2026-07-20. Status: owner-approved in-session ("Approve as presented").
Owner rulings: save-compatibility semver; delete the stray `v1.0.0` tag.

## Goal

Make commitizen-enforced commits carry through to rigorous, mechanical versioning: a
written semver policy, an owner-run release ceremony, clean `v*` tag space, infra
versioning, and a machine-checked compatibility invariant between the game repo and
babylon-infra.

## Context (verified 2026-07-20)

- Parent `[tool.commitizen]` is configured (version `0.3.0`, `tag_format = "v$version"`,
  `version_files`, `update_changelog_on_bump`); `cz check` gates commits in both repos
  (pre-commit + CI). `release.yml` fires on `v*` tags and documents the owner-run
  `cz bump` flow (push-triggered auto-bump deliberately retired). As of ADR094,
  `nix-release.yml` ALSO fires on `v*` — a tag push is now the player-channel release act.
- Tag-space anomalies: `v1.0.0` (2025-12-10, abandoned bump — two days later `0.2.0 → 0.3.0`
  was tagged; pyproject agrees on 0.3.0), plus narrative tags `v0.2.3-rent-trinity` and
  `v0.3.7.1-george-jackson-validated` inside the `v*` glob.
- babylon-infra: commitizen enforced on messages, but no version, no tags, no changelog.
- Cross-repo machine truth: the parent's `infra/` submodule gitlink.

## Decisions

### D1 — Save-compatibility semver (parent policy, `docs/versioning.md`)

- MAJOR: an existing save/campaign cannot load without migration (Ledger schema breaks,
  Archive `vector(N)` binding changes, defines-format breaks).
- MINOR: new features; old saves load clean.
- PATCH: fixes; no behavior contract change.
- Pre-1.0: the strict reading applies anyway (a 0.x minor may not break saves) — `1.0.0`
  is a promise-keeping event, not a semantics change.
- The policy doc also fixes the commit-scope vocabulary (controlled list per repo) and
  points to the ceremony. Referenced from CLAUDE.md (one line).

### D2 — Tag hygiene (parent)

- Delete `v1.0.0` (local now; `git push origin :refs/tags/v1.0.0` is owner-run).
- Retag `v0.2.3-rent-trinity` → `archive/v0.2.3-rent-trinity` and
  `v0.3.7.1-george-jackson-validated` → `archive/v0.3.7.1-george-jackson-validated`;
  delete the `v*` originals (origin half owner-run). After this, `v*` = releases, only.
- `0.3.0` remains current; the next release is `0.4.0`.

### D3 — Release ceremony (parent, `mise run release:bump`)

Owner-run task wrapping commitizen: refuses on a dirty tree or off-`dev`; always shows
`uv run cz bump --dry-run` output first; performs the real bump only with an explicit
`--yes` argument. The bump commit + tag stay local; pushing the tag is the owner's
release act (fires `release.yml` + `nix-release.yml`). Ceremony steps documented in
`docs/versioning.md`, not duplicated elsewhere.

### D4 — Infra versioning (babylon-infra, `.cz.toml`)

`.cz.toml` with `cz_conventional_commits`, version stored in the file itself
(`version = "0.1.0"` initial), `tag_format = "v$version"`, `update_changelog_on_bump`.
Infra bumps when the contract surface changes (flake devshells, deploy workflows,
terraform module interface) — not per docs commit. First tag `v0.1.0` minted at
execution time (local; push owner-run). A `release:bump` mise task mirrors the parent's
ceremony (guard: clean tree, `main`).

### D5 — The cross-repo invariant: releases pin released infra

- Day-to-day dev: the parent gitlink may pin any infra sha (status quo).
- A parent RELEASE may only ship a gitlink that points at an infra commit carrying an
  infra `v*` tag. Enforced two ways:
  1. `release:bump` (D3) checks it before bumping (loud refusal naming the pinned sha).
  2. A `release.yml` job step re-checks it in CI at tag time (defense in depth).
- Check mechanics: read the gitlink sha (`git ls-tree HEAD infra`), then in the infra
  repo ask `git tag --points-at <sha>` for a `v*` match. In CI the infra repo is
  fetched via the submodule URL (read-only, public).

### D6 — Out of scope (YAGNI, recorded)

No auto-bump automation; no version-range compatibility matrices; no infra semver-axis
policy beyond "tag on contract-surface change"; no backfilled changelog history.

## Testing

- D5 check: red/green — a pin at an untagged sha must fail the check; the tagged pin
  passes. The check is a standalone script (`tools/check_release_pins.sh`) so the mise
  task, CI, and tests all run the same code.
- D3/D4 ceremonies: `--dry-run` paths executed in tests; real bumps are owner-run.
- D2: verified by `git tag` listings before/after.

## Consequences

A `v*` tag becomes a complete, trustworthy release signal: policy-classified version,
changelog, GitHub release, signed Nix closure to the player cache, and a provably
tagged infra toolchain underneath it. Cost: one extra owner step per release (tag infra
first when the contract surface moved).
