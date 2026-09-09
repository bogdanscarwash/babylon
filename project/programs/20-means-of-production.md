# Program 20 — Means of Production

**Status: ACTIVE** — charter created 2026-07-15 (ADR071) · owner-approved design
"Approved as presented" · branch `worktree-infra-iac`

**Spec:** [`docs/superpowers/specs/2026-07-15-infra-iac-ai-modernization-design.md`](../../docs/superpowers/specs/2026-07-15-infra-iac-ai-modernization-design.md)

**One sentence:** extract the real, live `deploy/` Terraform+Ansible stack into a local-only
`babylon-infra` subrepo mounted at `infra/` (ADR069 mechanism), standardize the build environment
with a Nix flake devshell (system layer only), grow the Cloudflare Terraform an AI Gateway +
Workers AI surface, and modernize the AI narrator stack behind its existing flag — consolidation
and extension, not greenfield.

## Origin

Owner directive (2026-07-14): focus on infrastructure as code — Ansible for the server, Terraform
for Cloudflare + Cloudflare Workers AI, Nix or Guix to standardize the build environment — housed
in a top-level subrepository, independent and local for now. Since the Cloudflare Terraform grows
an AI surface, catch the AI subsystem up to the codebase: the templates, the chat engine, and
modernize what exists. Evidence base: 13-agent read-only recon (workflow `wf_fc8272a4-33a`,
2026-07-14) over repo, host, Cloudflare footprint, and governance docs.

## The two tracks

- **Track A — the `babylon-infra` subrepo.** `git subtree split --prefix=deploy` preserves
  `deploy/`'s full history into a new local repo, mounted at top-level `infra/`, gitignored in the
  parent until it has a remote, then converted to a real git submodule at the same path (the
  ADR069 / babylon-cockpit precedent). Adds a Nix flake devshell (system layer only — Python,
  Poetry, Node, GEOS/GDAL/PROJ, OpenBLAS, Rust, git-lfs, Playwright browsers), grows the Cloudflare
  Terraform with a `cloudflare_ai_gateway` + Workers AI surface, keeps VPS Ansible roles as a
  validated mockup, and adds a workstation playbook (the real, owner-run apply target) codifying
  the 2026-07-13 mount-displacement incident learnings. Parent CI's `infra-validate` and
  `infra-live.yml` retire; their checks are reborn as the infra repo's own pre-commit + mise
  validation battery. Accepted gap: infra validation is local-only until the repo has a remote.
- **Track B — AI modernization (main repo).** Catches the AI narrator stack up to the codebase
  behind the existing `BABYLON_LLM_NARRATOR` flag (Mock Doctrine; flag flip stays an owner
  ruling, D3): a `WorkersAIClient` implementing the existing `LLMProvider` protocol, prompt
  templates promoted from Python string literals to versioned, schema-validated data artifacts,
  RIOT-style AI-fillable abstract event templates (per the emergent-endgames ruling), the real
  `GET /api/games/{id}/narration/` endpoint the cockpit client already expects, and
  `NarrativeResult` persistence replacing the ephemeral in-process dict. Narration stays
  out-of-tick (fire-and-forget post-tick); the tick hash and `qa:regression` are untouched by
  design.

## Owner decisions (this session, 2026-07-15)

Copied verbatim from spec §2:

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

## Governance

ADR071 charters the program: subrepo mechanism + `deploy/` absorption; Article X scoping
clarification (X.1's "no Docker, no Nix" binds the production host, not dev/build tooling —
recorded, with an optional one-line Amendment T if the owner wants it constitutional); notes the
X.4 Woodpecker-vs-GitHub-Actions drift without fixing it. Constitution IX.1 lists
infrastructure/deployment change as an explicit ADR trigger — this satisfies it.

## Phases

See spec §6 for the full implementation-plan skeleton (0: ADR + program doc, 1: subtree split +
parent detach, 2: Nix flake devshell, 3: Terraform AI surface, 4: Ansible cleanup + workstation
playbook, 5: Track B, 6: docs/close-out). This document (Task A0) closes Phase 0.
