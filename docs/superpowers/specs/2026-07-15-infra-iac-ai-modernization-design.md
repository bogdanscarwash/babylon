# Program 20 — "Means of Production": Infra-as-Code Subrepo + AI Modernization

- **Date**: 2026-07-15
- **Status**: APPROVED by owner (Persephone Raskova), this session, "Approved as presented"
- **Branch**: `worktree-infra-iac` (worktree; implementation branch naming settled in the plan)
- **Evidence base**: 13-agent read-only recon (workflow `wf_fc8272a4-33a`, 2026-07-14) over repo,
  host, Cloudflare footprint, and governance docs. Key sources cited inline.

## 1. Why

Owner directive (2026-07-14): focus on infrastructure as code — Ansible for the server, Terraform
for Cloudflare + Cloudflare Workers AI, Nix or Guix to standardize the build environment — housed
in a top-level subrepository, independent and local for now. And since the Cloudflare Terraform
grows an AI surface, catch the AI subsystem up to the codebase: the templates, the chat engine,
and modernize what exists.

Recon reframe: this is **consolidation + extension, not greenfield**. A real, live Terraform +
Ansible stack exists in `deploy/` (Hetzner VPS `babylon-1`, `babylon.percypedia.biz` behind
Cloudflare, R2 buckets, nftables locked to Cloudflare CIDRs, CI validation + weekly ephemeral
provision/destroy). The AI narrator pipeline (NarrativeDirector → DeepSeek, RAG over pgvector) is
real, wired, and flag-off per owner ruling D3. The cockpit already ships a fully-typed client for
`GET /api/games/{id}/narration/` that 404s because the backend route was never built.

## 2. Owner decisions (this session, 2026-07-15)

| # | Question | Ruling |
|---|----------|--------|
| 1 | Build-env tool | **Nix flake devshell** (system layer only; Poetry + mise keep their jobs) |
| 2 | Subrepo shape | **Absorb `deploy/` now** via `git subtree split` (history preserved) |
| 3 | AI scope | **Narrator stack + endpoint** (no M2 prose→stance parser this program) |
| 4 | Ansible scope | **Dev box is the real apply target; VPS roles stay as validated mockup** — owner mid-turn: "you can mock up the VPS but I'm fine just with the devbox right now" |
| 5 | Sequencing | **Approach A**: one program, two tracks, infra-first phases |

Standing rulings that bind this program: Decision 3 of Program 07 ("We are using Workers-AI/LoRA
narrator", 2026-07-03); D3 ("AI narration deferred and mocked", Program 17 — flag flip stays an
owner ruling); the Mock Doctrine (2026-07-14); emergent-endgames ruling (2026-07-14); CI/tests
never touch the babylon-data drive (2026-07-14); Constitution Article X; III.6/III.7/III.11/III.12.

## 3. Track A — the `babylon-infra` subrepo

### 3.1 Repo mechanics

1. In the main repo: `git subtree split --prefix=deploy -b infra-split` — preserves the full
   history of `deploy/` in the split branch.
2. New repo **babylon-infra** seeded from that branch, mounted at top-level `infra/` as a nested
   git repo. Parent adds `/infra/` to `.gitignore` and to the allowlist in
   `tools/check_repo_hygiene.py` (comment: program-20 subrepo mount; submodule pointer to follow).
3. Local-only for now (owner directive: no GitHub yet). When it gets a remote, it converts to a
   real submodule at the same mount path per the ADR069 precedent (babylon-cockpit pattern).
4. Parent repo removes `deploy/` and updates references (docs, CI). Parent CI's `infra-validate`
   job and `infra-live.yml` retire; their checks are reborn inside the infra repo (see 3.5).
   **Accepted gap**: infra validation is local-only until the repo has a remote.
5. The infra repo gets its own lean `CLAUDE.md`: apply gates (agents never `terraform apply` /
   never run playbooks against production), secrets rules, Article X pointers.

### 3.2 Layout

```text
infra/
├── CLAUDE.md               # agent operating rules for this repo
├── README.md               # canonical doc (replaces vendored tgunawan README)
├── flake.nix / flake.lock  # devshell (see 3.3)
├── .mise.toml              # validate/plan/lint tasks
├── .pre-commit-config.yaml # the validation battery (see 3.5)
├── .sops.yaml              # sops+age rules for terraform secrets
├── terraform/              # moved from deploy/terraform + Workers AI additions (see 3.4)
├── ansible/                # moved from deploy/ansible + workstation playbook (see 3.6)
└── docs/                   # HOW-TO-DEPLOY-HETZNER.md moved; stale guides pruned or archived
```

`docker/` and `docker-compose.yml` stay in the **main** repo: they are local dev/test tooling for
the code (spec-087 isolated Postgres), not deployment infrastructure.

### 3.3 Nix flake devshell

- `flake.nix` at the infra repo root; main repo `.envrc` gains a guarded `use flake ./infra`.
- Provides the **system layer only**: Python 3.12, Poetry 2.2.1, Node 22, libpq, GEOS/GDAL/PROJ,
  OpenBLAS/LAPACK, Rust toolchain, git-lfs, pinned Playwright browsers
  (`PLAYWRIGHT_BROWSERS_PATH`), fluidsynth + GM soundfont (MIDI tasks), shellHook exporting the
  `*_NUM_THREADS=1` pins (mirrors `.mise.toml` — correctness per Constitution III.7, not comfort).
- Poetry remains the Python package manager; mise remains the task runner and stays usable
  standalone (documented precedence: inside the shell, nix-provided toolchain leads).
- Known drift fixed alongside: live `.venv` is on Python 3.13.9 vs the 3.12 pin everywhere —
  rebuild coordinated with the owner (their running environment).
- Explicitly **not** hermetic poetry2nix — that is a possible later hardening step.
- Guix: respected as UI/aesthetic canon (DESIGN_BIBLE §9b); not a build tool here (owner ruling
  Q1; X.6 solo-dev filter).

### 3.4 Terraform — Cloudflare AI surface

- Existing resources move unchanged (Hetzner VPS/firewall, DNS, zone settings, rulesets, 3 R2
  buckets + lifecycle).
- New: `cloudflare_ai_gateway` for the narrator (caching, rate-limiting, logging — the M8 "AI
  Gateway" item); variables/outputs for Workers AI (`workers_ai_token` sensitive var, account id
  reuse, gateway endpoint URL output consumed by Django settings); documented token-scope
  addition (Workers AI Read/Edit — current token scopes are DNS/SSL/R2 only, per
  `terraform.tfvars.example`).
- Exact provider-v5 resource names verified against current Cloudflare docs at implementation
  time (cloudflare skill); an early spike verifies GPT-OSS-20B availability + tool-calling on
  Workers AI (fallback: DeepSeek path unchanged).
- Secrets: sops+age encrypts `terraform.tfvars` (ansible-vault stays for Ansible secrets;
  HashiCorp Vault remains constitutionally rejected, X.6).
- **No `terraform apply` by agents, ever.** Validate/plan only; applies are owner-run.
- Out of scope: Turnstile, Tunnel (M8 Wave-6 items), LoRA fine-tuning (in scope for the M8/X1
  narrator spec per Program 07 Decision 3, but a separate ML workstream — deferred there).

### 3.5 Validation battery (replaces parent CI infra jobs)

Pre-commit + mise tasks in the infra repo, mirroring the retired `infra-validate` job:
`terraform fmt -check` / `terraform validate` / tflint (existing `.tflint.hcl`), ansible
`--syntax-check` over live playbooks, `ansible-lint --profile min` (blocking) + full report
(advisory), shellcheck, yamllint, `nix flake check`. The ephemeral Hetzner live-test becomes a
documented owner-run mise task (was: `infra-live.yml` weekly CI), dormant until the repo has a
remote.

### 3.6 Ansible

- **Prod roles (mockup status)**: cleaned and fully validated, never applied this program.
  Cleanups: canonical README; fix the dangling `roles/common/templates/sshd_config.j2` reference
  in `playbooks/bootstrap.yml`; prune or rewrite the vendored `playbooks/hetzner.yml`
  (Jakarta timezone, Dokploy — inherited template content).
- **Workstation playbook (the real target)**: `workstation.yml` against localhost, codifying the
  2026-07-13 incident learnings: LUKS data-drive fstab guard (`nofail`,
  `x-systemd.device-timeout=10` preserved exactly), docker + compose plugin, ollama, earlyoom,
  the 5432/5433 split guard (system postgresql@18 on 5432; container `babylon-pg-isolated` on
  5433; never resurrect the dormant apt PG17 cluster), verification tasks that invoke
  `tools/data_doctor.sh` (subsume behavior, don't discard it).
- Owner-run apply; agent runs are check-mode/lint only unless explicitly permissioned.

## 4. Track B — AI modernization (main repo)

All work ships behind `BABYLON_LLM_NARRATOR` + MOCK badges (Mock Doctrine). Flag flip remains an
owner ruling (D3). Narration stays out-of-tick (fire-and-forget post-tick) — the tick hash and
`qa:regression` are untouched by design.

1. **WorkersAIClient** implementing the existing `LLMProvider` protocol
   (`src/babylon/intelligence/ai/llm_provider.py`): OpenAI-compatible endpoint routed through the
   AI Gateway; `narrate_tick` tool-call returning `{headline, body, strategic_implication, tone}`
   per X1; structured `LLMGenerationError` codes; template fallback intact. Provider selection via
   `LLM_PROVIDER` (deepseek | workers_ai | mock); `WORKERS_AI_*` settings per the existing
   spec-prompt sketch (`ai/spec-prompts/infrastructure/cloudflare_edge_services.md` §7).
2. **Templates as versioned data artifacts**: CORPORATE/LIBERATED/system prompts extracted from
   Python string literals into schema-validated data files (the `persona.json` +
   `persona.schema.json` pattern); `PROMPT_VERSION` auto-derived from a content hash, with a
   drift test (closes the manual-bump gap). III.6 pins extended as far as the API honestly
   allows: model id + model version + prompt hash; tokenizer version recorded where exposed,
   documented as unavailable where not (no fake pins).
3. **RIOT-style abstract event templates**: schema for AI-fillable event archetypes (structure
   without scripting, per the emergent-endgames ruling) + first archetypes, consumed by both the
   LLM path and the deterministic fallback. Full 45/79 EventType coverage remediation stays with
   the seam punch-list, not this program.
4. **`GET /api/games/{id}/narration/`**: implemented to the contract the cockpit client already
   defines (`src/frontend/src/lib/narration/client.ts`, honest-offline semantics), registered in
   `endpoints.ts` (fixing the registry bypass), riding the sentinel/seam rails.
5. **Persistence**: `NarrativeResult` stored in a Django-side table keyed (session, tick) with its
   III.6 pins, replacing the ephemeral in-process dict (survives restart; replayable narration
   history).
6. **Stale cleanup**: `ai/architecture.yaml` ai_narrative flow block (still describes ChromaDB);
   `CHROMADB_PERSIST_DIR` in `.env.example`; reconcile the `CANONICAL_EMBEDDING_DIM=768` /
   mpnet-name pin vs the actual Ollama `embeddinggemma` default in
   `src/babylon/config/llm_config.py` + `persistence/pgvector_store.py`.

Out of scope for Track B: M2 prose→stance parser (owner ruling Q3); enabling the narrator by
default (D3); LoRA fine-tuning (deferred to the M8/X1 narrator spec); WebSocket/SSE push (polling
stays).

## 5. Governance

One ADR charters the program: subrepo mechanism + `deploy/` absorption; Article X scoping
clarification (X.1's "no Docker, no Nix" binds the **production host**, not dev/build tooling —
recorded, with an optional one-line Amendment T if the owner wants it constitutional); notes the
X.4 Woodpecker-vs-GitHub-Actions drift without fixing it. Constitution IX.1 lists
infrastructure/deployment change as an explicit ADR trigger — this satisfies it. Program doc
registered under `project/programs/`; `ai/state.yaml` updated at close-out.

## 6. Phases (implementation-plan skeleton)

| Phase | Deliverable | Verify |
|-------|-------------|--------|
| 0 | ADR + program doc | ADR in `ai/decisions/` + index; `mise run check` green |
| 1 | Subtree split; `infra/` repo; parent detach (deploy/ removed, CI adjusted, hygiene allowlist) | split history intact; parent `mise run check` green; hygiene gate passes |
| 2 | Nix flake devshell + `.envrc` wiring + venv 3.12 rebuild (coordinated) | `nix flake check`; `poetry install` + scoped `mise run test:q` inside shell |
| 3 | Terraform AI surface + sops | `terraform validate` + tflint; spike note on GPT-OSS-20B/tool-calling |
| 4 | Ansible cleanup + workstation playbook | ansible-lint + syntax; owner-run `--check` on localhost |
| 5 | Track B: provider, templates, endpoint, persistence, cleanup (TDD) | unit + contract tests green; flag-off parity byte-identical; `qa:regression` 5/5 |
| 6 | Docs, `ai/state.yaml`, memory, close-out | `mise run check` green |

Each phase is a conventional commit (or several); heavy test runs stay single-flight per
machine-safety rules.

## 7. Risks

- **GPT-OSS-20B on Workers AI** (availability, tool-calling fidelity): early Phase-3 spike;
  DeepSeek fallback unchanged either way.
- **Parent CI coverage gap** for infra until the subrepo has a remote: accepted; mitigated by the
  pre-commit battery.
- **venv rebuild** disturbs the owner's live environment: coordinated timing, not sprung.
- **Hygiene/tooling assumptions about `deploy/`**: sweep for path references before detach.
- **Worktree venv shadow**: `PYTHONPATH="$PWD/src"` pattern for any pytest run from the worktree.

## 8. Success criteria

1. `infra/` is a self-contained repo with `deploy/`'s history, a green validation battery, and a
   devshell that reproduces the build env on a clean machine.
2. Parent repo has no `deploy/`, no broken references, green `mise run check`, `qa:regression`
   5/5 byte-identical.
3. `terraform validate`/plan describes the AI Gateway + Workers AI surface without any apply
   having occurred.
4. Workstation playbook converges the dev box (owner-run) and encodes the mount-incident guard.
5. Cockpit's narration slot receives real (flag-on, owner-run) or honestly-mocked (flag-off)
   responses from a built `/narration/` endpoint — no more 404 contract.
6. All prompts are versioned data artifacts with hash-derived versions and a drift test.
