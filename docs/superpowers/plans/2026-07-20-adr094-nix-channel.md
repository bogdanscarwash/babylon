# Nix Player Channel Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development
> (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the parent-repo deployment mechanics for ADR094 — a `flake.nix` exposing
`packages.babylon` (a uv2nix workspace virtualenv), a player-facing `install.sh`, an
un-templated `nix-release.yml` with a closure regression gate, and a monthly regression-gated
nixpkgs pin-cadence workflow — plus honest gate records for the owner-run keygen (D1) and the
blocked hypergraph-rs input adoption (D6).

**Architecture:** The game repo becomes its own Nix flake. `uv2nix` ingests the committed
`uv.lock` (produced by ADR095) into a `pyproject-nix` package set, and `mkVirtualEnv` yields a
closure exposing the `babylon` console script. Players install that closure from the R2-backed
binary cache fronted by `cache.babylon.percypedia.biz` (serving lane already live, D2), trusting
narinfo signatures made with a key whose secret half lives only in CI (D1).

**Tech Stack:** Nix flakes (`nixpkgs` nixos-25.11, `flake-utils`, the uv2nix trio: `uv2nix` +
`pyproject-nix` + `pyproject-build-systems`), Python 3.12, POSIX `sh` (install.sh), GitHub
Actions (`actions/checkout@v7`, `cachix/install-nix-action@v31`, `DeterminateSystems/update-flake-lock`),
`shellcheck`, `markdownlint`.

## Global Constraints

- Base branch: `dev` @ 8ee8707f. Execute in a fresh git worktree off `dev`
  (superpowers:using-git-worktrees); the owner's live checkout must never be touched.
  Pushes are owner-run; commits use conventional format with the Co-Authored-By trailer.
- Python `>=3.12,<4.0` (pyproject). PRECONDITION (hard gate, run before Task 1): this plan
  runs AFTER the ADR095 packaging-train plan — the tree must be on uv (PEP-621 dep tables,
  committed `uv.lock` for uv2nix, `babylon` entry point). Gate:
  `test -f uv.lock && uv run babylon --help >/dev/null && echo "precondition OK"` —
  expected `precondition OK`; if it fails, STOP and execute the ADR095 plan first.
- Strict typing (mypy strict, function signatures fully annotated); Pydantic models for
  data objects; explicit exception types, no bare except; all loops bounded.
- Amendment V: narrator-only AI — no LLM output may enter the simulation input path.
  Amendment X.6: no LLM framework dependencies (no langchain etc.).
- Cloudflare: Workers Free plan everywhere; nothing that can bill.
- Verification battery before each commit: `uv run pytest <touched tests>`,
  `uv run ruff check src tests`, `uv run mypy src` — scale to what the task touched; the
  plan's final task runs the full gate (`mise run lint typecheck test:unit` equivalents).
- **Markdown hygiene:** prose lines ≤ 120 chars; blank lines around fenced blocks and lists
  (repo `.markdownlint.yaml`).
- **NEVER read, print, or commit secrets:** `terraform.tfvars`, `*.tfstate*`, `vault.yml`,
  age/ed25519 private keys, `.env`. The signing secret key never appears in this repo — only
  the public half is baked into `install.sh`.

## Upstream dependencies (execution order 095 → 094)

- **Requires ADR095 executed first.** This plan consumes: (a) a committed `uv.lock` at repo
  root (uv2nix's sole input — verified ABSENT on dev @ 8ee8707f, so `nix build` fails until
  095 lands it), (b) `[project.scripts] babylon = "babylon.cli:app"` (the console script
  `mkVirtualEnv` exposes as `bin/babylon`), (c) a PEP-621 `[project]` table with a wheel-
  building `[build-system]`. Verified on dev: `[project] name = "babylon"`,
  `requires-python = ">=3.12,<4.0"`, build backend still `poetry.core.masonry.api`, and
  `[project.scripts]` ABSENT — all of which 095 changes.
- If `uv.lock` is still absent when this plan runs, STOP: 095 has not landed. Do not
  fabricate a lockfile.

---

## Verified repo facts (checked in the worktree, 2026-07-20)

- No `flake.nix` at parent root; `infra/` is an uninitialized submodule (the only flake in
  the program lives in babylon-infra, not here).
- `.github/workflows/nix-release.yml` EXISTS as a template (header: "the flake itself is
  decision-queue work"); triggers `push tags v*` + `workflow_dispatch`; `permissions:
  contents: read`; `timeout-minutes: 60`; pins `actions/checkout@v7`,
  `cachix/install-nix-action@v31`.
- `install.sh` does not exist anywhere.
- `src/babylon_data` is a committed SYMLINK → `/home/user/projects/game/babylon-data/...`
  (dead in a Nix sandbox). Its ONLY importer under `src/babylon` is a **docstring doctest**
  (`src/babylon/domain/economics/shadow_labor.py:358`, a `>>>` line) — NOT load-bearing at
  import time. The flake source filter must still exclude the symlink so the store copy
  isn't a dangling link.
- `qa:regression` = `uv run python tools/regression_test.py compare` post-095
  (`poetry run ...` at `.mise.toml:849` on raw `dev`; the 095 sweep converts it).
  The CI job `qa-regression` (`main.yml:95-103`) runs it with `bootstrap-python` ONLY —
  **no Postgres service** (it is the pure determinism gate vs committed baselines under
  `tests/baselines/`). The closure regression gate therefore needs no database.
- `/hypergraph-rs/` is gitignored (`.gitignore:184`), has its own git repo and no flake;
  PR #210 was docs-only; no port-parity test suite exists → D6 is a documented gate only.
- No flake-lock update automation exists in `.github/workflows/`.

---

## Task 1: Game flake with `packages.babylon`

**Files:**

- Create: `flake.nix` (repo root)
- Test: `nix flake check` (drives the `checks.smoke` output defined in the flake)

**Interfaces:**

- Consumes (from ADR095): `uv.lock` at repo root; `[project.scripts] babylon` entry point;
  PEP-621 `[project]` with wheel build backend.
- Produces: `packages.<system>.babylon` and `packages.<system>.default` — a virtualenv
  derivation exposing `bin/babylon` and `bin/python`; `checks.<system>.smoke`.

Steps:

- [ ] Verify the upstream lock exists (fail loud if 095 has not landed):

  ```sh
  test -f uv.lock && echo "uv.lock present" || { echo "MISSING uv.lock — run ADR095 first"; exit 1; }
  grep -q '\[project.scripts\]' pyproject.toml && echo "entry point present" \
    || { echo "MISSING [project.scripts] babylon — run ADR095 first"; exit 1; }
  ```

  Expected output:

  ```text
  uv.lock present
  entry point present
  ```

- [ ] Confirm `babylon_data` is not a runtime import (should print only the docstring hit):

  ```sh
  rg -n "^[^#>]*\b(import babylon_data|from babylon_data)" src/babylon
  ```

  Expected output: no matches (exit 1). The only occurrence is a `>>>` doctest line, which
  this regex excludes. If real import statements appear, STOP and record the finding — do not
  fake a build that would crash at runtime.

- [ ] Write `flake.nix` at the repo root with this exact content:

  ```nix
  {
    description = "Babylon — MLM-TW geopolitical simulation engine (Nix Player Channel, ADR094)";

    inputs = {
      # Match infra's channel (babylon-infra flake pins nixos-25.11).
      nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
      flake-utils.url = "github:numtide/flake-utils";

      pyproject-nix = {
        url = "github:pyproject-nix/pyproject.nix";
        inputs.nixpkgs.follows = "nixpkgs";
      };
      uv2nix = {
        url = "github:pyproject-nix/uv2nix";
        inputs.pyproject-nix.follows = "pyproject-nix";
        inputs.nixpkgs.follows = "nixpkgs";
      };
      pyproject-build-systems = {
        url = "github:pyproject-nix/build-system-pkgs";
        inputs.pyproject-nix.follows = "pyproject-nix";
        inputs.uv2nix.follows = "uv2nix";
        inputs.nixpkgs.follows = "nixpkgs";
      };
    };

    outputs =
      { nixpkgs, flake-utils, pyproject-nix, uv2nix, pyproject-build-systems, ... }:
      flake-utils.lib.eachDefaultSystem (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          lib = pkgs.lib;
          python = pkgs.python312;

          # Source filter. The committed src/babylon_data symlink points outside
          # the sandbox and is DEAD in any Nix build; a fileset difference drops it
          # (plus heavy non-package trees) so uv2nix builds babylon from a clean copy.
          projectSrc = lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.difference ./. (
              lib.fileset.unions [
                (lib.fileset.maybeMissing ./src/babylon_data)
                (lib.fileset.maybeMissing ./web)
                (lib.fileset.maybeMissing ./node_modules)
                (lib.fileset.maybeMissing ./ai)
                (lib.fileset.maybeMissing ./hypergraph-rs)
              ]
            );
          };

          workspace = uv2nix.lib.workspace.loadWorkspace { workspaceRoot = ./.; };

          # Overlay generated from uv.lock; wheels preferred for hermetic, fast builds.
          lockOverlay = workspace.mkPyprojectOverlay { sourcePreference = "wheel"; };

          # Point the local `babylon` package at the filtered tree so the dead
          # symlink never enters its build. `babylon` is the normalized project
          # name (pyproject `[project] name = "babylon"`, verified).
          srcOverlay = _final: prev: {
            babylon = prev.babylon.overrideAttrs (_old: { src = projectSrc; });
          };

          pythonSet =
            (pkgs.callPackage pyproject-nix.build.packages { inherit python; }).overrideScope
              (lib.composeManyExtensions [
                pyproject-build-systems.overlays.wheel
                lockOverlay
                srcOverlay
              ]);

          babylonEnv = pythonSet.mkVirtualEnv "babylon-env" workspace.deps.default;
        in
        {
          packages = {
            babylon = babylonEnv;
            default = babylonEnv;
          };

          # `nix flake check` runs this. It is a SMOKE gate only — importable
          # package + working console script. The determinism/regression gate needs
          # tools/ + committed baselines and runs as a CI step (Task 3), NOT here.
          checks.smoke = pkgs.runCommand "babylon-smoke" { } ''
            ${babylonEnv}/bin/python -c 'import babylon; print("babylon import OK")'
            ${babylonEnv}/bin/babylon --help > /dev/null
            echo "babylon --help OK"
            touch $out
          '';

          apps.default = {
            type = "app";
            program = "${babylonEnv}/bin/babylon";
          };
        }
      );
  }
  ```

- [ ] Stage `flake.nix` (flakes ignore untracked files during evaluation), then show the
  outputs to confirm the attribute names resolved:

  ```sh
  git add flake.nix
  nix flake show 2>&1 | grep -E "babylon|default|smoke"
  ```

  Expected output includes `babylon:` and `default:` under `packages`, and `smoke:` under
  `checks`. FIX-FORWARD: if `nix flake show` errors that `prev.babylon` is undefined, the
  local package attr is not literally `babylon` — resolve the real name with
  `nix eval .#packages.$(nix eval --raw --impure --expr builtins.currentSystem) --apply builtins.attrNames`
  is not available; instead inspect the overlay set via
  `nix eval --impure --expr 'builtins.attrNames (builtins.getFlake (toString ./.)).packages.x86_64-linux'`
  and update `srcOverlay` to the reported name. Given pyproject `name = "babylon"`, `babylon`
  is expected.

- [ ] Build the closure players receive (first run downloads flake inputs; needs network):

  ```sh
  nix build .#babylon --print-build-logs
  ls -la result/bin/babylon result/bin/python
  ```

  Expected: build succeeds; both `result/bin/babylon` and `result/bin/python` exist
  (symlinks into the venv store path).

- [ ] Run the flake's own check (drives `checks.smoke`):

  ```sh
  nix flake check --print-build-logs 2>&1 | grep -E "babylon import OK|babylon --help OK"
  ```

  Expected output:

  ```text
  babylon import OK
  babylon --help OK
  ```

- [ ] Commit:

  ```sh
  git add flake.nix
  git commit -m "feat(nix): game flake with packages.babylon via uv2nix (ADR094)" \
    -m "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
  ```

---

## Task 2: Player `install.sh` with cache-key guard

**Files:**

- Create: `install.sh` (repo root)
- Create: `tests/install/test_install_sh.sh` (POSIX sh harness: shellcheck + stubbed dry-run)

**Interfaces:**

- Produces: `install.sh` — POSIX `sh`, shellcheck-clean; substituter
  `https://cache.babylon.percypedia.biz` (D2 serving lane, live); flake ref
  `github:percy-raskova/babylon#babylon`; `CACHE_KEY` placeholder that HARD-REFUSES to run
  until the owner replaces it (D1 keygen pending).
- Consumes: nothing at build time; at runtime, an already-installed `nix` (never installs Nix
  itself).

Steps:

- [ ] Create the test directory (it does not exist on `dev`), then write the failing test at
  `tests/install/test_install_sh.sh`:

  ```sh
  mkdir -p tests/install
  ```

  ```sh
  #!/bin/sh
  # Dry-run harness for install.sh: shellcheck lint + placeholder-guard + stubbed nix call.
  # No network, no real nix. Exits non-zero on the first failed assertion.
  set -eu

  here=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
  script="$here/install.sh"

  fail() { printf 'FAIL: %s\n' "$1" >&2; exit 1; }

  # 1. Lint clean (POSIX sh dialect).
  shellcheck -s sh "$script" || fail "shellcheck reported issues"

  # 2. Unmodified script must REFUSE (placeholder public key still present).
  if sh "$script" >/dev/null 2>&1; then
    fail "install.sh ran with the placeholder CACHE_KEY still in place"
  fi
  sh "$script" 2>&1 | grep -q "placeholder" || fail "guard did not mention the placeholder"

  # 3. With the key replaced and a stubbed nix, it must call 'nix profile install'
  #    carrying the substituter + trusted key flags. Work on a scratch copy.
  tmp=$(mktemp -d)
  trap 'rm -rf "$tmp"' EXIT
  sed 's/babylon-cache-1:REPLACE_WITH_PUBLIC_KEY/babylon-cache-1:AAAATESTKEY000/' \
    "$script" > "$tmp/install.sh"

  mkdir -p "$tmp/bin"
  cat > "$tmp/bin/nix" <<'STUB'
  #!/bin/sh
  printf '%s\n' "nix $*" >> "$NIX_CALL_LOG"
  STUB
  chmod +x "$tmp/bin/nix"

  NIX_CALL_LOG="$tmp/calls.log"
  export NIX_CALL_LOG
  : > "$NIX_CALL_LOG"

  PATH="$tmp/bin:$PATH" sh "$tmp/install.sh" >/dev/null 2>&1 || fail "install.sh errored with stub nix"

  grep -q "nix profile install github:percy-raskova/babylon#babylon" "$NIX_CALL_LOG" \
    || fail "did not invoke 'nix profile install' with the flake ref"
  grep -q -- "--extra-substituters https://cache.babylon.percypedia.biz" "$NIX_CALL_LOG" \
    || fail "missing --extra-substituters flag"
  grep -q -- "--extra-trusted-public-keys babylon-cache-1:AAAATESTKEY000" "$NIX_CALL_LOG" \
    || fail "missing --extra-trusted-public-keys flag"

  echo "PASS: install.sh dry-run"
  ```

- [ ] Make it executable and run it — expect FAILURE (install.sh does not exist yet):

  ```sh
  chmod +x tests/install/test_install_sh.sh
  sh tests/install/test_install_sh.sh; echo "exit=$?"
  ```

  Expected: a `shellcheck` "cannot open" / file-not-found error and `exit=1` (the script
  under test is absent).

- [ ] Write `install.sh` at the repo root with this exact content:

  ```sh
  #!/bin/sh
  # Babylon Player Channel installer (ADR094 D1/D2).
  # Installs the `babylon` package from the R2-backed Nix binary cache. It does
  # NOT install Nix itself — if Nix is absent it points at the official installer.
  set -eu

  # ── Public cache-signing key (ADR094 D1) ──────────────────────────────────
  # The SECRET half lives ONLY in CI (nix copy --sign-key at release). This is
  # the PUBLIC half, baked in so players can verify narinfo signatures. The owner
  # replaces the placeholder after the owner-run keygen
  # (babylon-infra cloudflare/scripts/generate-cache-key.sh). Until then this
  # script refuses to run rather than install unverified binaries.
  CACHE_KEY="babylon-cache-1:REPLACE_WITH_PUBLIC_KEY"

  SUBSTITUTER="https://cache.babylon.percypedia.biz"
  FLAKE_REF="github:percy-raskova/babylon#babylon"

  die() {
      printf 'babylon-install: %s\n' "$1" >&2
      exit 1
  }

  # Refuse until the owner has replaced the placeholder public key (keygen pending).
  case "$CACHE_KEY" in
      *REPLACE_WITH_PUBLIC_KEY*)
          die "CACHE_KEY is still the placeholder — the cache-signing keypair has not been minted (owner-run keygen pending, ADR094 D1). Refusing to install unverified binaries."
          ;;
  esac

  # Nix must already be present; we never install it for the user.
  if ! command -v nix >/dev/null 2>&1; then
      die "Nix is not installed. Install it first (https://nixos.org/download or https://install.determinate.systems), then re-run this script."
  fi

  printf 'babylon-install: installing %s from %s\n' "$FLAKE_REF" "$SUBSTITUTER"
  nix profile install "$FLAKE_REF" \
      --extra-substituters "$SUBSTITUTER" \
      --extra-trusted-public-keys "$CACHE_KEY"

  printf 'babylon-install: done. Run `babylon doctor` to verify your setup.\n'
  ```

- [ ] Make it executable and run the test — expect PASS:

  ```sh
  chmod +x install.sh
  sh tests/install/test_install_sh.sh
  ```

  Expected output:

  ```text
  PASS: install.sh dry-run
  ```

- [ ] Commit:

  ```sh
  git add install.sh tests/install/test_install_sh.sh
  git commit -m "feat(nix): player install.sh with refuse-until-keyed cache guard (ADR094 D1/D2)" \
    -m "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
  ```

---

## Task 3: Un-template `nix-release.yml` + closure regression gate

**Files:**

- Modify: `.github/workflows/nix-release.yml` (rewrite header comment lines 1-18; add a
  `regression` job; wire `nix-release` to depend on it — keep the sign/push mechanics at
  lines 49-60 byte-for-byte)

**Interfaces:**

- Consumes: `packages.babylon` (Task 1); `tools/regression_test.py compare` + committed
  `tests/baselines/` (verified — no Postgres needed).
- Produces: a release workflow whose push job runs only after the built closure passes the
  determinism gate.

Steps:

- [ ] Read the current file end-to-end so the sign/push block is preserved verbatim:

  ```sh
  sed -n '1,69p' .github/workflows/nix-release.yml
  ```

  Confirm the sign/push `run:` block (`umask 077` … `rm -f /tmp/cache-key`) matches Task 1's
  cache facts; it is copied UNCHANGED below.

- [ ] Replace the header comment (lines 1-18) and insert the `regression` job. Overwrite the
  file with this exact content (the `Sign and push` step body is unchanged from the template):

  ```yaml
  # Nix release channel — the player-channel §7 job, now LIVE: the flake exists
  # (flake.nix at repo root, ADR094 Task 1), so this workflow is no longer
  # aspirational. It builds the closure players receive, gates it on the closure
  # regression suite, then signs and pushes it to the R2 binary cache.
  #
  # Secrets required (game repo):
  #   R2_ACCESS_KEY_ID / R2_SECRET_ACCESS_KEY
  #       R2 API token scoped "Object Read & Write" to the babylon-cache bucket
  #       ONLY (dash → R2 → Manage API tokens). Not an account-wide token.
  #   CF_ACCOUNT_ID
  #       same account id as the workers; forms the S3 endpoint below.
  #   NIX_CACHE_SIGNING_KEY
  #       secret half from babylon-infra cloudflare/scripts/generate-cache-key.sh.
  #       OWNER-RUN keygen is still pending — until the secret exists this workflow
  #       builds and gates fine but the push step has no key to sign with.
  #
  # Free-tier ledger for this job: R2 storage 10 GB free (a ~1.2 GB closure per
  # release → ~8 releases resident; `nix store` paths dedupe across releases, and
  # stale releases can be pruned from the bucket). Class A ops 1M/month free —
  # one `nix copy` is one PUT per store path (~hundreds), noise. Egress: free.
  name: nix-release

  on:
    push:
      tags: ["v*"]
    workflow_dispatch:

  permissions:
    contents: read

  jobs:
    regression:
      # Determinism gate ON the built closure (Amendment Q / Amendment-L discipline).
      # Runs tools/regression_test.py with the venv's python — the same 5-scenario
      # baseline compare as main.yml's qa-regression, which needs NO Postgres.
      runs-on: ubuntu-latest
      timeout-minutes: 30
      steps:
        - uses: actions/checkout@v7

        - uses: cachix/install-nix-action@v31
          with:
            extra_nix_config: |
              experimental-features = nix-command flakes

        - name: Build the closure
          run: nix build .#babylon --print-build-logs

        - name: Regression gate on the closure (5 scenarios vs committed baselines)
          run: ./result/bin/python tools/regression_test.py compare

    nix-release:
      needs: [regression]
      runs-on: ubuntu-latest
      # House rule (Constitution III.11): every job carries timeout-minutes —
      # a hang must die loudly, not run to the 360min default.
      timeout-minutes: 60
      steps:
        - uses: actions/checkout@v7

        - uses: cachix/install-nix-action@v31
          with:
            extra_nix_config: |
              experimental-features = nix-command flakes

        - name: Build the closure players receive
          run: nix build .#babylon

        - name: Regression gate ON the closure (Amendment Q extended)
          run: nix flake check

        - name: Sign and push to the babylon-cache R2 bucket
          env:
            AWS_ACCESS_KEY_ID: ${{ secrets.R2_ACCESS_KEY_ID }}
            AWS_SECRET_ACCESS_KEY: ${{ secrets.R2_SECRET_ACCESS_KEY }}
          run: |
            umask 077
            printf '%s' "${{ secrets.NIX_CACHE_SIGNING_KEY }}" > /tmp/cache-key
            ENDPOINT="${{ secrets.CF_ACCOUNT_ID }}.r2.cloudflarestorage.com"
            STORE_URI="s3://babylon-cache?scheme=https&endpoint=${ENDPOINT}"
            STORE_URI="${STORE_URI}&region=auto&secret-key=/tmp/cache-key&compression=zstd"
            nix copy --to "$STORE_URI" .#babylon
            rm -f /tmp/cache-key

        # Players then substitute from the Worker front-door, no compiling:
        #   extra-substituters = https://cache.babylon.percypedia.biz
        #   extra-trusted-public-keys = babylon-cache-1:<PUBKEY>
        - name: AppImage demo artifact (zero-install path)
          run: nix bundle --bundler github:ralismark/nix-appimage .#babylon || true
  ```

- [ ] Validate the workflow YAML parses (offline; no runner needed):

  ```sh
  python -c "import yaml,sys; d=yaml.safe_load(open('.github/workflows/nix-release.yml')); \
  assert d['jobs']['nix-release']['needs']==['regression']; \
  assert any('nix copy --to' in (s.get('run') or '') for s in d['jobs']['nix-release']['steps']); \
  print('nix-release.yml OK')"
  ```

  Expected output:

  ```text
  nix-release.yml OK
  ```

- [ ] Commit:

  ```sh
  git add .github/workflows/nix-release.yml
  git commit -m "ci(nix): un-template nix-release, gate release on closure regression (ADR094)" \
    -m "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
  ```

---

## Task 4: D5 monthly regression-gated pin cadence

**Files:**

- Create: `.github/workflows/flake-update.yml`

**Interfaces:**

- Produces: a scheduled workflow that opens a `nix flake update` PR monthly; the PR's CI
  (the existing test suite plus, on release, the closure regression gate) decides merge.
- Consumes: `DeterminateSystems/update-flake-lock` (version resolved at implement time —
  never invented).

Steps:

- [ ] Resolve the latest release tag of the pin action (do NOT hand-write a version):

- [ ] Write the workflow using a quoted heredoc (so `${{ }}` GitHub expressions are NOT shell-
  expanded), resolving the action tag and injecting it in the SAME shell invocation (shell
  variables do not survive across separate steps — everything below is one block). If `gh` is
  unauthenticated, resolve `TAG` instead via
  `git ls-remote --tags https://github.com/DeterminateSystems/update-flake-lock` (highest `vN`):

  ```sh
  TAG=$(gh api repos/DeterminateSystems/update-flake-lock/releases/latest --jq .tag_name)
  test -n "$TAG" || { echo "FATAL: could not resolve update-flake-lock tag" >&2; exit 1; }
  echo "resolved update-flake-lock tag: $TAG"
  cat > .github/workflows/flake-update.yml <<'EOF'
  # Monthly nixpkgs / flake-input pin cadence (ADR094 D5, Amendment-L discipline).
  # Opens a `nix flake update` PR on the 1st of each month. The PR does NOT
  # self-merge: the repo's normal CI (unit + the determinism gate) plus, at
  # release, nix-release.yml's closure regression job decide whether the bump is
  # safe. A red regression gate blocks the merge.
  #
  # Escape hatch (sanctioned narrow deviation): a single package that needs a
  # version the pinned nixpkgs lacks gets its OWN extra input with
  # `inputs.nixpkgs.follows`, bumped independently, each with a stated reason —
  # mirroring babylon-infra's nixpkgs-sec (trivy) and nixpkgs-data (sqlite)
  # lockstep inputs. Add such inputs to flake.nix by hand, not via this cron.
  name: flake-update

  on:
    schedule:
      - cron: "0 4 1 * *"
    workflow_dispatch:

  permissions:
    contents: write
    pull-requests: write

  jobs:
    flake-update:
      runs-on: ubuntu-latest
      timeout-minutes: 20
      steps:
        - uses: actions/checkout@v7

        - uses: cachix/install-nix-action@v31
          with:
            extra_nix_config: |
              experimental-features = nix-command flakes

        - uses: DeterminateSystems/update-flake-lock@__TAG__
          with:
            pr-title: "chore(nix): monthly flake.lock update (ADR094 D5)"
            pr-labels: |
              dependencies
              nix
            commit-msg: "chore(nix): monthly flake.lock update (ADR094 D5)"
  EOF
  sed -i "s/@__TAG__/@${TAG}/" .github/workflows/flake-update.yml
  ```

- [ ] Verify the placeholder was replaced and the YAML parses:

  ```sh
  rg -q "update-flake-lock@v[0-9]" .github/workflows/flake-update.yml && echo "tag injected"
  python -c "import yaml; d=yaml.safe_load(open('.github/workflows/flake-update.yml')); \
  assert d['on']['schedule'][0]['cron']=='0 4 1 * *'; \
  assert d['permissions']['pull-requests']=='write'; print('flake-update.yml OK')"
  ```

  Expected output:

  ```text
  tag injected
  flake-update.yml OK
  ```

- [ ] Commit:

  ```sh
  git add .github/workflows/flake-update.yml
  git commit -m "ci(nix): monthly regression-gated flake pin cadence (ADR094 D5)" \
    -m "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
  ```

---

## Task 5: Gate records — D1 owner-run keygen, D6 hypergraph-rs (docs only)

**Files:**

- Create: `docs/adr094-gates.md`

**Interfaces:**

- Produces: an honest record of the two obligations this plan does NOT execute (D1 keygen is
  owner-run; D6 hypergraph-rs adoption is blocked on a nonexistent parity suite). No code,
  no fabricated execution.

Steps:

- [ ] Write `docs/adr094-gates.md` with this exact content:

  ````markdown
  # ADR094 open gates — owner-run keygen (D1) and hypergraph-rs adoption (D6)

  Two ADR094 obligations are deliberately NOT executed by the implementation plan. They are
  recorded here so their status is explicit and nobody fakes them.

  ## D1 — cache-signing keypair (OWNER-RUN, pending)

  The ed25519 keypair that signs narinfo files is the integrity boundary (TLS is transport,
  not trust). The SECRET half must exist only in CI; the PUBLIC half is baked into
  `install.sh` as `CACHE_KEY`. Minting it is owner-run — the plan ships `install.sh` with a
  placeholder that HARD-REFUSES to run until replaced.

  Owner command (in babylon-infra, refuses to overwrite an existing key; writes the secret to
  `$XDG_DATA_HOME/babylon-infra/keys/`; prints the public key and the `gh secret set`
  next-step):

  ```sh
  cloudflare/scripts/generate-cache-key.sh
  ```

  After running it, the owner:

  1. Replaces `CACHE_KEY="babylon-cache-1:REPLACE_WITH_PUBLIC_KEY"` in `install.sh` with the
     printed public key.
  2. Registers the secret half in the game repo:
     `gh secret set NIX_CACHE_SIGNING_KEY` (consumed by `nix-release.yml`).

  Until both steps are done, `nix-release.yml` builds and passes the regression gate but has
  no key to sign with, and `install.sh` refuses to install.

  ## D6 — hypergraph-rs as a pinned flake input (BLOCKED on port parity)

  `hypergraph-rs` is a gitignored local subrepo with its own git history and no flake of its
  own — invisible to flake builds as a subtree of `self`. Adoption checklist (implement none
  of this until the gate opens):

  - Enter the game flake as a pinned INPUT with `inputs.nixpkgs.follows = "nixpkgs"` — never a
    subtree of `self`.
  - A version bump is an input-hash change that rides the SAME monthly closure regression gate
    as nixpkgs (Task 4 / D5).
  - **Adoption is gated on the PR #210 port-parity test suite passing.** As of 2026-07-20 that
    suite does not exist (PR #210 was docs-only). Do not add the input until the parity suite
    exists and is green.
  ````

- [ ] Lint the new docs (repo markdownlint config):

  ```sh
  npx --yes markdownlint-cli --config .markdownlint.yaml docs/adr094-gates.md && echo "docs OK"
  ```

  Expected output: `docs OK` (no rule violations). If `markdownlint-cli` is unavailable
  offline, verify manually that prose lines are ≤ 120 chars and fenced blocks/lists have blank
  lines around them.

- [ ] Commit:

  ```sh
  git add docs/adr094-gates.md
  git commit -m "docs(nix): record ADR094 D1 keygen and D6 hypergraph-rs gates" \
    -m "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
  ```

---

## Task 6: Final verification gate

**Files:**

- Test: whole worktree (no new files; runs the full battery)

**Interfaces:**

- Consumes: everything built in Tasks 1-5.
- Produces: a green end-to-end verification proving the flake builds, checks, install dry-run,
  and repo lint all pass on the branch as executed.

Steps:

- [ ] Build + check the flake:

  ```sh
  nix build .#babylon --print-build-logs
  nix flake check --print-build-logs 2>&1 | grep -E "babylon import OK|babylon --help OK"
  ```

  Expected: build succeeds; both smoke lines print.

- [ ] Re-run the install dry-run:

  ```sh
  sh tests/install/test_install_sh.sh
  ```

  Expected output: `PASS: install.sh dry-run`.

- [ ] Validate both workflow files parse and the release job still gates on regression:

  ```sh
  python -c "import yaml; \
  r=yaml.safe_load(open('.github/workflows/nix-release.yml')); \
  u=yaml.safe_load(open('.github/workflows/flake-update.yml')); \
  assert r['jobs']['nix-release']['needs']==['regression']; \
  assert '__TAG__' not in open('.github/workflows/flake-update.yml').read(); \
  print('workflows OK')"
  ```

  Expected output: `workflows OK`.

- [ ] Run the repo's own gate on the parts this plan touched (uv after 095; falls back to
  poetry if uv migration is incomplete). The flake adds no Python source, so this confirms the
  branch is still green:

  ```sh
  mise run qa:regression
  ```

  Expected: the 5-scenario determinism compare passes (byte-identical to committed baselines).

- [ ] Confirm the working tree is clean (all work committed):

  ```sh
  git status --porcelain
  ```

  Expected output: empty (no uncommitted changes).

- [ ] No commit needed (verification only). If any check failed, STOP and fix the owning task
  rather than papering over it.

---

## Honest gates / not executed here (per ADR094)

- **D1 keygen — OWNER-RUN, pending.** `install.sh` refuses to run and `nix-release.yml` cannot
  sign until the owner mints the keypair and sets `NIX_CACHE_SIGNING_KEY`. Recorded in
  `docs/adr094-gates.md` (Task 5). The first real `nix copy` release cannot happen until the
  keygen + CI secrets exist.
- **D2 serving lane — DONE (2026-07-20, post-ADR).** The ADR's context section describes the
  pre-deployment state ("NEITHER drop Worker is deployed"); deployment happened later the same
  day. Evidence lives in babylon-infra, not this repo: `ai/backlog.md` "Done 2026-07-20"
  (Workers deployed on `cache./api.babylon.percypedia.biz`, smoke 12/12, `workers_dev` off,
  commit `fe48877`). `install.sh` targets the live substituter; nothing to build here.
- **D3 Terraform import — DONE in babylon-infra (2026-07-20, post-ADR).** All six live
  resources imported, drift closed (same babylon-infra backlog record). Out of scope for the
  parent repo.
- **D4 wrangler pin — DONE in babylon-infra.** `wrangler-action@v4` + `wranglerVersion "4"`
  landed at the drop merge (babylon-infra commit `88b1606`). Out of scope here.
- **D6 hypergraph-rs — BLOCKED.** No PR #210 parity suite exists; the plan documents the
  adoption checklist (Task 5) but implements nothing.
- **Pushes are owner-run.** This plan commits locally on a worktree branch; the owner pushes.

## ADR094 clause coverage

| ADR094 clause | Covered by | Status |
| --- | --- | --- |
| X.8 flake channel (the packaging vehicle) | Task 1 (`flake.nix`, `packages.babylon`, `checks.smoke`) | Implemented |
| D1 — cache crypto / signing key | Task 2 (`install.sh` `CACHE_KEY` + refuse guard) + Task 5 (keygen gate) | Guarded; keygen owner-run |
| D2 — serving lane (`cache.babylon.percypedia.biz`) | Task 2 (substituter URL) | Consumed (already live) |
| D3 — Terraform import boundary | — (babylon-infra) | Done upstream; recorded |
| D4 — wrangler deploy precondition | — (babylon-infra) | Done upstream; recorded |
| D5 — monthly regression-gated pin cadence | Task 4 (`flake-update.yml`) + Task 3 (closure regression gate) | Implemented |
| D6 — hypergraph-rs pinned input, parity-gated | Task 5 (`docs/adr094-gates.md` checklist) | Documented gate (blocked) |
| nix-release un-template + closure gate | Task 3 (`nix-release.yml`) | Implemented |
| End-to-end verification | Task 6 | Gate |
