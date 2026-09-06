# The release version contract

<!-- Vale: this reference preserves literal Git, release, and semantic-versioning terms. -->
<!-- vale Vale.Spelling = NO -->
<!-- vale ste.UnapprovedWords = NO -->
<!-- vale ste.Ambiguity = NO -->
<!-- vale ste.NounClusters = NO -->
<!-- vale ste.Gerunds = NO -->

Owner-ratified 2026-07-20 (spec: docs/superpowers/specs/2026-07-20-versioning-rigor-design.md).
Commitizen enforces commit form. This document defines version meaning and the release process.

**Supersession note (2026-07-29, Amendment AE / ADR172):** ADR172 defines v1.0
as the **Rust engine's** release under Program 27. It replaces the earlier
Python "playable archive" release and resets save-compatibility semver. The
policy and ceremony below apply to the Rust engine release.

## The axis: player saves

- **MAJOR** — an existing campaign cannot load without migration. Examples
  include Ledger schema breaks and Archive embedding-column binding changes.
- **MINOR** — new features. Old campaigns load clean.
- **PATCH** — fixes. The behavior contract does not change.
- **Pre-1.0:** a 0.x MINOR cannot break saves.

`1.0.0` is a promise-keeping event, not a semantics change.

## Commit scopes (controlled vocabulary)

`cli`, `intelligence`, `engine`, `persistence`, `render`, `web`, `data`, `deps`, `ci`,
`docs`, `plans`, `ai`, `specs`, `tooling`, `hygiene`, `packaging`.
babylon-infra additionally: `tf`, `ansible`, `cloudflare`, `secrets`, `tasks`.
Add new scopes here before use.

## The release ceremony (owner-run)

1. Run `mise run release:bump` for the checked dry-run.
2. Run `mise run release:bump -- --yes` to create the untagged `dev` commit.
3. Push `dev` and open its release PR to `main`.
4. Retrieve both protected branches.
5. Run `git merge-base --is-ancestor origin/main origin/dev`.
6. Refuse the release when current `main` is absent from `dev`.
7. Run `gh workflow run main.yml --ref dev`.
8. Pin the green qualification run to the exact `dev` SHA.
9. The Director runs `mise run pr:merge -- N --director-main` after acceptance.
10. Create a sanctioned lane at exact `origin/main`.
11. Run `mise run release:prepare-dev-sync -- vX.Y.Z N`.
12. Commit its lineage record and open that lane's PR to `dev`.
13. Merge the lineage PR with the ordinary sanctioned command.
14. Update the local `main` checkout to exact `origin/main`.
15. Run `mise run release:tag -- --yes`.

The tag task refuses until the lineage PR is in `origin/dev`. The task creates
and pushes `vX.Y.Z` on the qualified main merge commit. The tag starts
`release.yml`.

The tag task proves that its target is exact protected `main`. It also proves
that the main commit returned to protected `dev`. Each publishing workflow
independently rejects a tag outside protected `main` history.

## Releases pin their environment

ADR252 makes native Debian, mise, rustup, and uv the development toolchain.
`.mise.toml`, `mise.lock`, `.python-version`, `rust/rust-toolchain.toml`, `uv.lock`, and
`rust/Cargo.lock` pin the language tools and dependencies at each tag.
`tools/check_release_pins.sh` checks their consistency offline.

Exact reference-data builds use the ordinary project interpreter. `mise.lock`
records its official standalone build URL and checksum for Linux x64. The
release check validates the locked tool versions, download sources, and hashes.
It also compares `PINNED_SQLITE_VERSION` with the `data-artifacts.yaml` product
block. The builder refuses a runtime SQLite version that differs from the pin.
The weekly rebuild verifies the resulting database's exact product SHA.

The ceremony and `release.yml` run the pin check. Before publication, the
workflow installs the locked Python environment and runs source smoke and
regression checks. Native source installation uses `mise run setup` and
`mise run play`. ADR252 retires the obsolete binary-cache installer and publisher.
Historical releases and their architecture records remain unchanged. The
babylon-infra repository versions its operations surface independently.

## Tag namespace

`v*` is only for releases. Historical and narrative tags live under `archive/`.
Owner-run cleanup 2026-07-20 deleted the abandoned `v1.0.0` tag.
`v0.2.3-rent-trinity` and `v0.3.7.1-george-jackson-validated` moved to `archive/`.

<!-- vale ste.Gerunds = YES -->
<!-- vale ste.NounClusters = YES -->
<!-- vale ste.Ambiguity = YES -->
<!-- vale ste.UnapprovedWords = YES -->
<!-- vale Vale.Spelling = YES -->
