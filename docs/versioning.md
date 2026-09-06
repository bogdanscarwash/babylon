# Versions and native releases

<!-- Vale: this reference preserves literal Git, release, and semantic-versioning terms. -->
<!-- vale Vale.Spelling = NO -->
<!-- vale ste.UnapprovedWords = NO -->
<!-- vale ste.Ambiguity = NO -->
<!-- vale ste.NounClusters = NO -->
<!-- vale ste.Gerunds = NO -->

Director decision, 2026-09-06: `dev` is the fast integration branch. Full
qualification makes `main` the source for public downloads. Commitizen validates Conventional Commits and
prepares the changelog and release version. Publication remains a separate,
Director-authorized action from qualified `main` history.

## Version meaning

The native preview series uses `0.x.y`:

- `x` increases by one for each deliberate public release. Each release resets `y` to zero.
- Between releases, `y` counts newly reachable commits since the nearest canonical release tag.
- Development builds append `+g<12-character-SHA>` to distinguish branches with equal commit counts.
- Releases include a version, tag, source SHA, download assets, and SHA-256 checksums.

A merge includes the incoming commits and the merge commit itself. This avoids a
generated version-bump commit for every edit. Dirty builds also append a hash of
their changed bytes.

These are semantic-version-shaped development identifiers, not a promise that
pre-1.0 saves remain compatible. The runtime supports current-format save/reopen.
Developers can retire obsolete development formats. Declaring the native product stable
is a future Director decision. The historical `v1.0.0` release and its tag already
exist and must not be overwritten or republished.

`[project].version` in `pyproject.toml` is the sole planned release version.
Commitizen's `uv` provider updates it and the matching package row in `uv.lock`
together, without upgrading dependencies. Cargo's internal crate versions do not
identify a public Babylon build.

`mise run release:version` prints the development identity. Use
`python3 tools/release_version.py --json` for its source SHA, anchor, commit count,
and dirty status. `--release-version` prints the planned public version.
`--release-tag vX.Y.Z` additionally refuses a mismatched version, wrong commit,
shallow history, or dirty checkout. A separate guard checks main ancestry.

The native cutover starts from commit
`947699e40fe75a65e603e3302d425643f416e917` at development version `0.3.0`.
Only tags descended from that native cutover can reset the counter. Returning
historical main ancestry cannot revive a pre-native release. That bootstrap
appears in `[tool.babylon.versioning]` because the older canonical tags are on
disconnected history. It applies only until the first reachable canonical
release tag.

Preparing `0.4.0` does not relabel earlier
builds. Development continues as `0.3.y`. After `v0.4.0`, the next commit is
`0.4.1+g<sha>`. Missing anchors and shallow clones fail loudly.

## Commit and hook policy

Use Conventional Commits, for example `feat(runtime): add campaign comparison` or
`fix(persistence): reject an incomplete save`. Scopes describe the changed part
of the code. Commitizen also accepts its standard merge and revert prefixes.

Run `mise run hooks` after setup. It installs the configured `pre-commit`,
`commit-msg`, and `pre-push` hooks. `uv.lock` pins `pre-commit`.
The Commitizen hook uses the same pinned version as the project CLI.

Normal commits
run applicable formatting, source, lock, and contract checks. The message hook
runs Commitizen. The pre-push gate uses the reduced Rust development selection.

CI independently validates the exact commit range with
`uv run --frozen python tools/release_version.py --check-commits BASE_SHA HEAD_SHA`.
Both endpoints must be full commit SHAs and the base must be an ancestor of the
head. For the first promotion of accumulated historical work, the checker starts
at the explicit native cutover commit. The checker validates later ranges in full. This
preserves history while enforcing the policy on every new commit.

## Release procedure

Source now declares the `0.4.0` native preview. The bump task updates files
without creating a commit or tag. It refuses direct work on `dev` or `main`.
For later releases:

1. Create an ordinary lane from current `dev` after retrieving full history and tags.
2. Run `mise run release:bump` to preview the next explicit MINOR increment.
3. Run `mise run release:bump -- --yes` to update the version, lock, and changelog.
4. Review those changes, stage them, and commit with `mise run commit`.
5. Merge the lane's qualified PR to `dev` through `mise run pr:merge -- N`.

For publication:

1. Retrieve both protected branches. Prove
   `git merge-base --is-ancestor origin/main origin/dev`.
2. Open the `dev` to `main` release PR. Pin its complete successful qualification
   manifest to the exact source SHA.
3. After review and qualification, run the Director-authorized merge:
   `mise run pr:merge -- N --director-main`.
4. Create a sanctioned lane at exact `origin/main` and run
   `mise run release:prepare-dev-sync -- vX.Y.Z N`.
5. Commit its lineage record, open its PR to `dev`, and merge with the sanctioned
   command. This returns exact `main` ancestry to protected `dev`.
6. Update a clean local `main` checkout to exact `origin/main` and run
   `mise run release:tag -- --yes`.

The main PR runs full CI and `main.yml`, including native package validation.
That PR supplies the authoritative qualification evidence. The optional
`gh workflow run main.yml --ref dev` command helps diagnose release-only checks.

The tag task refuses an existing tag and checks the canonical version and
lineage before publication. The tag starts `release.yml`, which independently
verifies the main-reachable tag, version, and returned lineage. A manual retry
must identify the same tag. No dev push publishes a release.

The main PR builds and exercises the unpacked native archive once. Publication
promotes that exact archive after checking the merged PR, identical source tree,
successful qualification run, GitHub artifact digest, and package checksum.
It refuses expired or mismatched artifacts and does not rebuild during tagging.
`release-provenance.json` links the qualified source commit to the main release
commit. A partial draft upload requires explicit recovery; retries cannot
overwrite an existing release.

## Downloads and qualification

GitHub Actions stores temporary qualification artifacts. Persistent public
download assets live in GitHub Releases. Versions `0.x.0` use GitHub's prerelease
label. Link directly to the preview tag because the latest-stable link can still
select the historical `v1.0.0` release.

The first native preview targets
Ubuntu 24.04 x86_64 desktops. The archive includes both Rust binaries, runtime
assets, editable `defines.toml`, and the launcher. Unpack it and run `./babylon`.

Install Python 3.12+, local Docker Engine with Compose, libpq, and the desktop
graphics libraries. The first launch needs internet to build its pinned
Postgres image. See `tools/release/DOWNLOAD.md` for exact prerequisites and
controls. This preview is an administrative simulation viewer with no player
commands.

`.mise.toml`, `mise.lock`, `.python-version`, `rust/rust-toolchain.toml`, `uv.lock`,
and `rust/Cargo.lock` pin each release environment. The release pin check and
full `main.yml` qualification preserve the deeper validation beyond the fast
dev gate. Download manifests and checksums identify the qualified build.
Historical releases and architecture records remain unchanged.

<!-- vale ste.Gerunds = YES -->
<!-- vale ste.NounClusters = YES -->
<!-- vale ste.Ambiguity = YES -->
<!-- vale ste.UnapprovedWords = YES -->
<!-- vale Vale.Spelling = YES -->
