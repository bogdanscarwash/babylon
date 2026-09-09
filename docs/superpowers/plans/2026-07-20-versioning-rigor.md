# Versioning Rigor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development
> (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn commitizen-enforced commits into rigorous releases: written save-compat semver
policy, clean `v*` tag space, owner-run `release:bump` ceremonies in both repos, and a
machine-checked "releases pin released infra" invariant.

**Architecture:** One policy doc + one POSIX check script shared by the mise ceremony, CI, and
the red/green verification; commitizen's own `cz bump` does all version math. Infra gains a
`.cz.toml` (version lives in the config) and mirrors the ceremony.

**Tech Stack:** commitizen (already enforced both repos), mise tasks, POSIX sh + shellcheck,
GitHub Actions.

## Global Constraints

- Parent work in the worktree `WTV` =
  `/tmp/nix-shell.FbZjat/claude-1000/-home-user-projects-game-babylon-infra/e67c8341-b7a0-41b2-a7e5-4d183c55d496/scratchpad/wt-version`
  (branch `feat/versioning-rigor` off dev @ e3ec4179). Infra work (Task 5 only) in `WTI` =
  `/tmp/nix-shell.FbZjat/claude-1000/-home-user-projects-game-babylon-infra/e67c8341-b7a0-41b2-a7e5-4d183c55d496/scratchpad/wt-infra-main`
  (babylon-infra, branch `main` — session convention, direct commits). NEVER touch
  `/home/user/projects/game/babylon` or `/home/user/projects/game/babylon-infra` working trees.
- Git TAGS are repo-global (shared across worktrees): local tag create/delete affects the whole
  repo — intended here. NEVER push tags or delete origin tags: every `git push` (including
  `git push origin :refs/tags/...`) is OWNER-RUN and only ever RECORDED in docs, not executed.
- Pre-commit hooks must pass; never `--no-verify`. NEVER read/print secrets. Conventional
  commits ending with a second `-m "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"`.
- Parent verification env quirks (PYTHONPATH=src + nix-store GDAL/GEOS/LD_LIBRARY_PATH exports
  for pytest): see the 095 task-9 report if a Python command fails on imports; the tasks below
  avoid needing them (sh + yaml + cz only).
- shellcheck must pass on any `*.sh` (repo pre-commit runs it).

---

## Task 1: Parent tag hygiene (D2)

**Files:**

- No tracked files change; tag refs only. Verification via `git tag` listings.

**Interfaces:**

- Produces: a clean `v*` namespace (releases only) that Task 3's ceremony can trust.

Steps:

- [ ] Record the before-state and confirm the three targets exist:

  ```bash
  cd "$WTV" && git tag -l 'v*' | sort
  ```

  Expected to include: `v0.2.3-rent-trinity`, `v0.3.0`, `v0.3.7.1-george-jackson-validated`,
  `v1.0.0` (others may exist; only these three change).

- [ ] Archive the narrative tags (annotated, preserving what they pointed at), then delete the
  stray bump tag and the originals:

  ```bash
  git tag archive/v0.2.3-rent-trinity v0.2.3-rent-trinity
  git tag archive/v0.3.7.1-george-jackson-validated v0.3.7.1-george-jackson-validated
  git tag -d v1.0.0 v0.2.3-rent-trinity v0.3.7.1-george-jackson-validated
  ```

- [ ] Verify — expect NO output from the first command, two lines from the second:

  ```bash
  git tag -l 'v1.0.0' -l 'v0.2.3-rent-trinity' -l 'v0.3.7.1-george-jackson-validated'
  git tag -l 'archive/*'
  ```

- [ ] No commit (ref-only task). Append the owner-run origin commands to the task report so
  they reach the final summary verbatim:

  ```text
  OWNER-RUN (origin half): git push origin :refs/tags/v1.0.0 \
    :refs/tags/v0.2.3-rent-trinity :refs/tags/v0.3.7.1-george-jackson-validated \
    archive/v0.2.3-rent-trinity archive/v0.3.7.1-george-jackson-validated
  ```

---

## Task 2: `docs/versioning.md` + CLAUDE.md pointer (D1)

**Files:**

- Create: `docs/versioning.md`
- Modify: `CLAUDE.md` (one pointer line in the existing docs-pointer region)

**Interfaces:**

- Produces: the policy doc Tasks 3–5 cite (`docs/versioning.md`).

Steps:

- [ ] Write `docs/versioning.md`:

  ```markdown
  # Versioning — the save-compatibility contract

  Owner-ratified 2026-07-20 (spec: docs/superpowers/specs/2026-07-20-versioning-rigor-design.md).
  Commitizen enforces commit form; this doc defines what versions MEAN and how releases happen.

  ## The axis: player saves

  - **MAJOR** — an existing save/campaign cannot load without migration. Examples: Ledger
    schema breaks, Archive embedding-column (`vector(N)`) binding changes, defines-format
    breaks that invalidate campaign state.
  - **MINOR** — new features; old saves load clean.
  - **PATCH** — fixes; no behavior-contract change.
  - **Pre-1.0:** the strict reading applies anyway — a 0.x MINOR may not break saves.
    `1.0.0` is a promise-keeping event, not a semantics change.

  ## Commit scopes (controlled vocabulary)

  `cli`, `intelligence`, `engine`, `persistence`, `render`, `web`, `data`, `deps`, `ci`,
  `nix`, `flake`, `docs`, `plans`, `ai`, `specs`, `tooling`, `hygiene`, `packaging`.
  babylon-infra additionally: `tf`, `ansible`, `cloudflare`, `secrets`, `tasks`.
  New scopes are added here first, then used.

  ## The release ceremony (owner-run)

  1. `mise run release:bump` — refuses on a dirty tree or off-`dev`; runs the
     releases-pin-released-infra check (below); shows `cz bump --dry-run`.
  2. `mise run release:bump -- --yes` — the real bump commit + `vX.Y.Z` tag (local).
  3. Owner pushes `dev`, then the tag. **Pushing the tag IS the release**: it fires
     `release.yml` (GitHub release) and `nix-release.yml` (signed closure → player cache).

  ## Releases pin released infra

  A release may only ship an `infra/` gitlink pointing at a commit that carries an infra
  `v*` tag (`tools/check_release_pins.sh`, run by the ceremony and re-checked in
  `release.yml`). Day-to-day dev may pin any sha. When the infra contract surface (flake
  devshells, deploy workflows, terraform interface) changed since the last infra tag, tag
  infra first (`mise run release:bump` in babylon-infra), bump the gitlink, then release.

  ## Tag namespace

  `v*` is for releases, only. Historical/narrative tags live under `archive/`.
  Owner-run cleanup 2026-07-20: `v1.0.0` (abandoned 2025-12 bump) deleted;
  `v0.2.3-rent-trinity` and `v0.3.7.1-george-jackson-validated` moved to `archive/`.
  ```

- [ ] Add one line to `CLAUDE.md` — locate the existing pointer/reference section
  (`rg -n "docs/" CLAUDE.md | head` to find the idiom) and append, matching its style:

  ```text
  - `docs/versioning.md` — save-compat semver policy + the owner-run release ceremony.
  ```

- [ ] Verify markdownlint:

  ```bash
  npx --yes markdownlint-cli -c .markdownlint.yaml docs/versioning.md && echo MD-OK
  ```

  Expected: `MD-OK`.

- [ ] Commit:

  ```bash
  git add docs/versioning.md CLAUDE.md
  git commit -m "docs(versioning): save-compat semver policy + release ceremony (D1)" \
    -m "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
  ```

---

## Task 3: `tools/check_release_pins.sh` + `release:bump` task (D3/D5)

**Files:**

- Create: `tools/check_release_pins.sh` (mode 755)
- Modify: `.mise.toml` (add `[tasks."release:bump"]` near the other release-adjacent tasks)

**Interfaces:**

- Produces: `tools/check_release_pins.sh` (exit 0 = pin is a tagged infra commit; exit 1 =
  untagged; exit 2 = structural error). Consumed by Task 4's CI step and the ceremony.

Steps:

- [ ] Write `tools/check_release_pins.sh`:

  ```sh
  #!/bin/sh
  # Releases pin released infra (D5, docs/versioning.md): the infra/ gitlink must
  # point at a commit carrying an infra v* tag. Exit 0 tagged / 1 untagged / 2 error.
  # Works initialized (offline, asks the submodule) or uninitialized (CI: ls-remote).
  set -eu

  sha=$(git ls-tree HEAD infra | awk '{print $3}')
  [ -n "$sha" ] || { echo "check_release_pins: FATAL no infra gitlink in HEAD" >&2; exit 2; }

  if [ -e infra/.git ]; then
    tags=$( (cd infra && git tag --points-at "$sha") | grep '^v' || true)
  else
    url=$(git config -f .gitmodules submodule.infra.url)
    [ -n "$url" ] || { echo "check_release_pins: FATAL no submodule.infra.url" >&2; exit 2; }
    # ls-remote lists both refs/tags/vX and peeled refs/tags/vX^{}; match either at our sha.
    tags=$(git ls-remote --tags "$url" \
      | awk -v s="$sha" '$1==s {print $2}' \
      | sed -e 's|^refs/tags/||' -e 's|\^{}$||' | grep '^v' | sort -u || true)
  fi

  if [ -n "$tags" ]; then
    printf 'check_release_pins: OK — infra %s carries tag(s): %s\n' "$sha" "$tags"
    exit 0
  fi
  printf 'check_release_pins: REFUSE — infra gitlink %s carries no v* tag.\n' "$sha" >&2
  printf 'Tag infra first (babylon-infra: mise run release:bump), bump the gitlink, retry.\n' >&2
  exit 1
  ```

  ```bash
  chmod 755 tools/check_release_pins.sh && shellcheck -s sh tools/check_release_pins.sh && echo SC-OK
  ```

  Expected: `SC-OK`.

- [ ] RED — the current pin is an untagged infra sha, so the check must refuse:

  ```bash
  ./tools/check_release_pins.sh; echo "exit=$?"
  ```

  Expected: the REFUSE message and `exit=1`. (GREEN comes in Task 5 after infra is tagged and
  the gitlink bumped — the cross-task red/green is the point.)

- [ ] Add to `.mise.toml` (after the `[tasks."release"]`-adjacent or `qa:*` block — match file
  style):

  ```toml
  [tasks."release:bump"]
  description = "OWNER-RUN release ceremony: pin check + cz bump dry-run; pass -- --yes for the real bump (docs/versioning.md)"
  run = """
  #!/usr/bin/env bash
  set -euo pipefail
  [ -z "$(git status --porcelain)" ] || { echo "release:bump: refuse — dirty tree" >&2; exit 1; }
  [ "$(git branch --show-current)" = "dev" ] || { echo "release:bump: refuse — not on dev" >&2; exit 1; }
  tools/check_release_pins.sh
  uv run cz bump --dry-run
  if [ "${1:-}" = "--yes" ]; then
    uv run cz bump
    echo "release:bump: bumped. OWNER: push dev, then push the new tag — the tag push IS the release."
  else
    echo "release:bump: dry-run only. Execute with: mise run release:bump -- --yes"
  fi
  """
  ```

- [ ] Verify the task guards fire here (this worktree is on a feature branch — the refusal IS
  the pass):

  ```bash
  mise run release:bump 2>&1 | tail -1; echo "exit=$?"
  ```

  Expected: `release:bump: refuse — not on dev` (or the dirty-tree refusal if tree is dirty)
  and non-zero exit.

- [ ] Commit:

  ```bash
  git add tools/check_release_pins.sh .mise.toml
  git commit -m "feat(tooling): release:bump ceremony + releases-pin-released-infra check (D3/D5)" \
    -m "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
  ```

---

## Task 4: `release.yml` pin re-check (D5 defense in depth)

**Files:**

- Modify: `.github/workflows/release.yml` (add one step to the existing job, after checkout,
  before the release-creation steps)

**Interfaces:**

- Consumes: `tools/check_release_pins.sh` (Task 3).

Steps:

- [ ] Read the current job layout (`sed -n '20,60p' .github/workflows/release.yml`), then
  insert after the checkout step (indentation matched to siblings):

  ```yaml
        - name: Releases pin released infra (docs/versioning.md D5)
          run: ./tools/check_release_pins.sh
  ```

- [ ] Verify parse + step presence:

  ```bash
  python3 -c "import yaml; d=yaml.safe_load(open('.github/workflows/release.yml')); \
  steps=[s.get('name','') for j in d['jobs'].values() for s in j['steps']]; \
  assert any('pin released infra' in s for s in steps); print('release.yml OK')"
  ```

  Expected: `release.yml OK`. (PyYAML quirk: if the file's `on:` key is needed use `d[True]`.)

- [ ] Commit:

  ```bash
  git add .github/workflows/release.yml
  git commit -m "ci(release): re-check releases-pin-released-infra at tag time (D5)" \
    -m "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
  ```

---

## Task 5: Infra `.cz.toml` + ceremony + first tag; parent GREEN (D4 + D5 green)

**Files (in `WTI` — babylon-infra):**

- Create: `.cz.toml`
- Modify: `.mise.toml` (add `[tasks."release:bump"]`)
- Modify: `ai/backlog.md` (owner-run note: push `main` + tag `v0.1.0`)

**Files (in `WTV` — parent):**

- Modify: `infra` gitlink (bump to the newly tagged infra sha)

**Interfaces:**

- Consumes: Task 3's check script (parent). Produces: infra tag `v0.1.0` (local), the green
  half of the D5 red/green.

Steps:

- [ ] In `WTI`, write `.cz.toml`:

  ```toml
  [tool.commitizen]
  name = "cz_conventional_commits"
  version = "0.1.0"
  tag_format = "v$version"
  update_changelog_on_bump = true
  # Version lives HERE (cz bump rewrites this file). Bump when the contract surface
  # changes: flake devshells, deploy workflows, terraform module interface
  # (docs/versioning.md in the parent repo, D4).
  ```

- [ ] In `WTI` `.mise.toml`, add (mirroring the check task's devshell-sentinel style):

  ```toml
  [tasks."release:bump"]
  description = "OWNER-RUN infra release ceremony: cz bump dry-run; pass -- --yes for the real bump (parent docs/versioning.md D4)"
  run = """
  #!/usr/bin/env bash
  set -euo pipefail
  if [ -z "${BABYLON_INFRA_DEVSHELL:-}" ]; then
    exec nix develop "${MISE_PROJECT_ROOT:-$PWD}#ci" --command mise run "release:bump" -- "${1:-}"
  fi
  [ -z "$(git status --porcelain)" ] || { echo "release:bump: refuse — dirty tree" >&2; exit 1; }
  [ "$(git branch --show-current)" = "main" ] || { echo "release:bump: refuse — not on main" >&2; exit 1; }
  cz bump --dry-run
  if [ "${1:-}" = "--yes" ]; then
    cz bump
    echo "release:bump: bumped. OWNER: push main, then the new tag."
  else
    echo "release:bump: dry-run only. Execute with: mise run release:bump -- --yes"
  fi
  """
  ```

- [ ] In `WTI`: backlog note under **Owner-run only**: push `main` and tag `v0.1.0` when
  pushing (one line). Run `mise run check`, then commit all three files:

  ```bash
  git add .cz.toml .mise.toml ai/backlog.md
  git commit -m "feat(tooling): infra versioning — .cz.toml, release:bump ceremony (D4)" \
    -m "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
  ```

- [ ] In `WTI`: tag the commit just made (local; push owner-run):

  ```bash
  git tag -a v0.1.0 -m "infra v0.1.0 — first contract-surface tag (D4)"
  git tag --points-at HEAD
  ```

  Expected: `v0.1.0`.

- [ ] In `WTV` (parent): bump the gitlink to the tagged sha and confirm GREEN:

  ```bash
  INFRA_SHA=$(cd "$WTI" && git rev-parse HEAD)
  cd "$WTV" && git update-index --cacheinfo 160000,"$INFRA_SHA",infra
  git commit -m "build(deps): bump infra gitlink to v0.1.0 — first tagged pin (D5)" \
    -m "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
  ./tools/check_release_pins.sh; echo "exit=$?"
  ```

  Expected: `check_release_pins: OK — infra <sha> carries tag(s): v0.1.0` and `exit=0`.
  NOTE: the check's initialized-path needs `infra/.git`; if `infra` is not initialized in
  `WTV`, the ls-remote path would query origin (which does NOT yet have the tag — owner-run
  push pending). In that case run the check with the submodule pointed at the local infra
  repo: `git -c protocol.file.allow=always submodule update --init infra` is NOT available
  (origin URL). Instead verify the initialized path by temporarily symlinking:
  `ln -s "$WTI/.git" infra/.git` is forbidden (dirty hack) — the honest fallback is:
  `(cd "$WTI" && git tag --points-at "$INFRA_SHA")` must print `v0.1.0`, and record in the
  report that the parent-side check goes green once the owner pushes the infra tag. Prefer
  whichever of these paths reality allows and record which ran.

- [ ] Final verification sweep (both repos): `git log --oneline -3` each; parent
  `git tag -l 'v*'` shows only real releases; report the owner-run list (origin tag pushes
  from Task 1, infra main+tag push, parent dev push).

---

## Coverage

| Spec clause | Task |
| --- | --- |
| D1 policy doc + scopes | Task 2 |
| D2 tag hygiene | Task 1 (+ owner-run origin half recorded) |
| D3 parent ceremony | Task 3 |
| D4 infra versioning + ceremony + first tag | Task 5 |
| D5 check script + CI re-check + red/green | Tasks 3 (red), 4 (CI), 5 (green) |
| D6 out-of-scope | (none — recorded in spec) |
