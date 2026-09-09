# Program 20 Track A — babylon-infra Subrepo Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development
> (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract `deploy/` (with history) into a local top-level `infra/` subrepo with a Nix
flake devshell, a Cloudflare Workers-AI/AI-Gateway Terraform surface, a sops+age secrets story,
and a workstation Ansible playbook — leaving the parent repo detached and green.

**Architecture:** `git subtree split` preserves history; the new repo mounts at `infra/`
(gitignored in the parent until it gets a remote, then submodule per ADR069 precedent). The flake
provides both the Babylon build env (system layer) and the infra toolchain. VPS roles stay
validated-only ("mockup" per owner); the workstation playbook is the real apply target.

**Tech Stack:** git subtree, Nix flakes (nixpkgs 25.11), Terraform ≥1.9 + cloudflare provider ~>5,
Ansible (ansible-core 2.20 + ansible-lint), sops + age, mise, pre-commit.

## Global Constraints

- ALL subagents run on **Sonnet 5** (`model: 'sonnet'`) — owner directive 2026-07-14.
- **Never run `terraform apply`, `terraform plan` against live accounts, or any Ansible play
  against production/`babylon-1`.** Validation is offline: `fmt`, `validate`, `tflint`,
  `--syntax-check`, `ansible-lint`, `--check` on localhost only when a task explicitly says so.
- Heavy test/build runs stay single-flight (machine-safety, CLAUDE.md); never fan out pytest.
- Conventional commits, each ending with `Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>`.
- Parent-repo work happens in the worktree
  `/home/user/projects/game/babylon/.claude/worktrees/infra-iac` (branch `worktree-infra-iac`).
  NEVER touch `/home/user/projects/game/babylon` (owner's live checkout) except read-only.
- Constitution: III.11 loud failure (no silent skips in playbooks/tasks); X.1–X.6 division of
  labor; the flake is dev/build tooling, NOT deployment (ADR records the Article X scoping).
- `terraform.tfvars`, `terraform.tfstate*`, `vault.yml`, age private keys: NEVER committed,
  NEVER printed to logs, NEVER read into agent context. Work around them by filename only.

---

### Task A0: ADR + program doc

**Files:**
- Create: `ai/decisions/ADR0NN_program20_means_of_production.yaml` (NN = next free number —
  determine with `ls ai/decisions/ | sort | tail -3`; expected ADR071)
- Modify: `ai/decisions/index.yaml` (append entry, mirror the format of the last entry)
- Create: `project/programs/20-means-of-production.md`

**Interfaces:**
- Produces: the program charter later tasks cite in commit messages (`program-20`).

- [ ] **Step 1: Determine next ADR number**

Run: `ls ai/decisions/ | sort | tail -3`
Expected: highest existing is `ADR070_*.yaml` → use `ADR071`. If higher exists, use next free and
substitute below.

- [ ] **Step 2: Write the ADR** (full content; adjust `NN` only)

```yaml
id: ADR071
title: "Program 20: Means of Production — infra-as-code subrepo + AI modernization"
date: 2026-07-15
status: accepted
owners: [persephone-raskova]
context: |
  Owner directive 2026-07-14/15 (session 327c9e90): consolidate infrastructure-as-code into a
  top-level subrepository (local-only for now), standardize the build environment, grow the
  Cloudflare Terraform an AI surface (Workers AI per Program 07 Decision 3), and modernize the
  AI narrator stack. Spec: docs/superpowers/specs/2026-07-15-infra-iac-ai-modernization-design.md
  (owner-approved in-session).
decision: |
  1. deploy/ is extracted with history (git subtree split) into a new local repo `babylon-infra`,
     mounted at top-level infra/, gitignored in the parent until it has a remote, then converted
     to a git submodule at the same path (ADR069 mechanism).
  2. Build environment standardization is a Nix flake devshell living in the infra repo
     (system layer only; Poetry and mise keep their jobs). Article X.1's "no Docker, no Nix"
     is SCOPED: it binds the production host (every X clause speaks in production vocabulary);
     dev/build tooling is outside its jurisdiction. This ADR records that interpretation;
     a one-line clarifying amendment is offered to the owner but not required.
  3. Terraform grows a Cloudflare AI Gateway + Workers AI surface (Program 07 Decision 3,
     Constitution X.5 already assigns Workers AI to Cloudflare). No LoRA this program.
  4. Ansible: VPS roles retained as validated mockup (owner: "mock up the VPS ... fine just
     with the devbox right now"); a workstation playbook codifying the 2026-07-13
     mount-displacement learnings is the real apply target (owner-run).
  5. Parent CI infra-validate/infra-live retire; their checks move into the infra repo's
     pre-commit battery. Accepted gap: infra validation is local-only until a remote exists.
consequences: |
  - Parent repo loses deploy/ and .tflint.hcl (moved); hygiene allowlist updated.
  - X.4 names Woodpecker among supervised processes (X.5 assigns CI/CD to Hetzner); reality is GitHub Actions — drift NOTED, not fixed here.
  - AI narration stays flag-off per D3; Track B builds behind the flag (Mock Doctrine).
supersedes: []
related: [ADR063, ADR064, ADR069]
```

- [ ] **Step 3: Write `project/programs/20-means-of-production.md`** — one page: link the spec,
the two tracks, owner rulings table (copy §2 of the spec verbatim), status ACTIVE.

- [ ] **Step 4: Append to `ai/decisions/index.yaml`** following its existing entry format
(read the last 2 entries first; keep field order identical).

- [ ] **Step 5: Validate + commit**

Run: `poetry run yamllint ai/decisions/ADR071_program20_means_of_production.yaml && git add ai/decisions project/programs docs/superpowers && git commit -m "docs(adr): ADR071 program-20 charter — infra subrepo + AI modernization"`
Expected: yamllint clean; commit hooks pass.

---

### Task A1: Parent pre-detach guards

**Files:**
- Modify: `.gitignore` (repo root, worktree)
- Verify only: `deploy/` divergence between origin default and the spine branch

**Interfaces:**
- Produces: `/infra/` ignore rule that Task A2 relies on before creating the directory.

- [ ] **Step 1: Check whether deploy/ differs between our base and the owner's live branch**

Run: `git fetch origin && git diff origin/main...origin/feature/epochs-wave1-spine --stat -- deploy/ 2>/dev/null || git diff origin/main...feature/epochs-wave1-spine --stat -- deploy/`
Expected: empty (no divergence). **If non-empty:** rebase the worktree branch onto the spine
branch first (`git rebase feature/epochs-wave1-spine`) so the split captures the newest deploy/.

- [ ] **Step 2: Add ignore rule**

Append to `.gitignore` under the existing top-level ignores:

```gitignore
# program-20: local infra subrepo mount (becomes a submodule when it gets a remote)
/infra/
```

- [ ] **Step 3: Commit**

Run: `git add .gitignore && git commit -m "chore(infra): ignore /infra/ subrepo mount (program-20)"`

---

### Task A2: Subtree split + seed babylon-infra

**Files:**
- Create: `infra/` (new git repo seeded from split branch)

**Interfaces:**
- Produces: `infra/` repo on branch `main` containing the full history of `deploy/` with
  `terraform/`, `ansible/`, docs at its root (paths shifted up one level by the split).

- [ ] **Step 1: Split**

Run (from the worktree root): `git subtree split --prefix=deploy -b infra-split`
Expected: prints a commit SHA; `git log --oneline infra-split | tail -3` shows deploy/'s earliest
commits.

- [ ] **Step 2: Seed the new repo**

```bash
mkdir infra && cd infra && git init -b main
git fetch .. infra-split && git merge --ff-only FETCH_HEAD
git log --oneline | head -5 && ls
cd ..
git branch -D infra-split
```

Expected: `ls` shows `terraform ansible README.md HOW-TO-DEPLOY-HETZNER.md ansible-setup-guide.md
requirements.yml .gitignore` (deploy/'s former contents at root).

- [ ] **Step 3: Verify parent is untouched**

Run: `git status --short`
Expected: empty (infra/ is ignored).

---

### Task A3: Infra repo scaffolding (README, CLAUDE.md, .gitignore, mise, pre-commit)

**Files (all inside `infra/`):**
- Rewrite: `README.md` (replace vendored tgunawan content)
- Create: `CLAUDE.md`, `.mise.toml`, `.pre-commit-config.yaml`
- Modify: `.gitignore` (extend the one inherited from deploy/)
- Move: `docs/` — `git mv HOW-TO-DEPLOY-HETZNER.md docs/` ; `git mv ansible-setup-guide.md docs/archive/` (historical research, labeled as such)
- Copy in: `.tflint.hcl` from parent root (`cp ../.tflint.hcl .tflint.hcl` — parent removal
  happens in Task A7)

**Interfaces:**
- Produces: `mise run check` inside infra/ = fmt+validate+lint battery consumed by every later task.

- [ ] **Step 1: Write `README.md`** — sections: What this repo is (Babylon's infrastructure:
Terraform=provision, Ansible=configure, Nix flake=dev/build env — Constitution Article X);
Layout; Quickstart (`nix develop`, `mise run check`); Apply gates (owner-run only); pointer to
`docs/HOW-TO-DEPLOY-HETZNER.md` as the canonical deploy guide; provenance note (extracted from
babylon `deploy/` with history, program-20).

- [ ] **Step 2: Write `CLAUDE.md`**

```markdown
# CLAUDE.md — babylon-infra

Infrastructure-as-code for Babylon (program-20, ADR071). Terraform provisions cloud resources,
Ansible configures hosts, the Nix flake provides the dev/build environment. Constitution
Article X governs: bare-metal production, systemd sole supervisor, Cloudflare edge / Hetzner
compute, solo-dev maintainability filter.

## Hard rules for agents

- NEVER `terraform apply`/`destroy`, and never `terraform plan` against live credentials,
  unless the owner explicitly says so in the current session. Offline `fmt`/`validate`/tflint
  are always fine.
- NEVER run playbooks against production hosts. `workstation.yml` may be run with `--check`
  when asked; real applies are owner-run.
- NEVER read, print, or commit: `terraform.tfvars`, `terraform.tfstate*`, `vault.yml`,
  age private keys. sops-encrypted files are committed only by the owner.
- Validate everything: `mise run check` before every commit.
- VPS roles are a validated mockup until the owner rules otherwise (2026-07-15).
```

- [ ] **Step 3: Extend `.gitignore`** (append)

```gitignore
# nix
result
result-*
# sops/age — plaintext side of encrypted files
secrets/*.dec.*
```

- [ ] **Step 4: Write `.mise.toml`**

```toml
[tasks."fmt"]
description = "terraform fmt (write)"
run = "terraform -chdir=terraform fmt -recursive"

[tasks."validate"]
description = "terraform validate (offline)"
run = """
terraform -chdir=terraform init -backend=false -input=false >/dev/null
terraform -chdir=terraform validate
"""

[tasks."lint:tf"]
description = "tflint"
run = "tflint --chdir=terraform --config=$PWD/.tflint.hcl"

[tasks."lint:ansible"]
description = "ansible syntax + lint (blocking profile min)"
run = """
ansible-playbook --syntax-check -i ansible/inventory.yml ansible/site.yml
ansible-playbook --syntax-check -i ansible/inventory.yml ansible/playbooks/bootstrap.yml
ansible-playbook --syntax-check -i ansible/inventory.yml ansible/playbooks/backup-smoke.yml
ansible-playbook --syntax-check -i localhost, ansible/workstation.yml
ansible-lint --profile min ansible/
"""

[tasks."lint:sh"]
description = "shellcheck all shell scripts"
run = "git ls-files '*.sh' | xargs -r shellcheck"

[tasks."check"]
description = "full validation battery"
depends = ["fmt", "validate", "lint:tf", "lint:ansible", "lint:sh"]
```

(Note: `workstation.yml` and the pruned `hetzner.yml` — Task A6 — keep this list in sync when
they land; syntax-check lines for files that do not exist yet are ADDED in the task that
creates them, not before.)

- [ ] **Step 5: Write `.pre-commit-config.yaml`**

```yaml
repos:
  - repo: local
    hooks:
      - id: terraform-fmt
        name: terraform fmt -check
        entry: terraform -chdir=terraform fmt -check -recursive
        language: system
        pass_filenames: false
        files: ^terraform/
      - id: terraform-validate
        name: terraform validate
        entry: bash -c 'terraform -chdir=terraform init -backend=false -input=false >/dev/null && terraform -chdir=terraform validate'
        language: system
        pass_filenames: false
        files: ^terraform/
      - id: tflint
        name: tflint
        entry: bash -c 'tflint --chdir=terraform --config=$PWD/.tflint.hcl'
        language: system
        pass_filenames: false
        files: ^terraform/
      - id: ansible-lint
        name: ansible-lint (profile min, blocking)
        entry: ansible-lint --profile min ansible/
        language: system
        pass_filenames: false
        files: ^ansible/
  - repo: https://github.com/shellcheck-py/shellcheck-py
    rev: v0.11.0.1
    hooks:
      - id: shellcheck
  - repo: https://github.com/adrienverge/yamllint
    rev: v1.37.1
    hooks:
      - id: yamllint
        args: [--strict]
        exclude: ^(ansible/roles/.*/templates/|terraform/)
```

- [ ] **Step 6: Install hooks + first validation + commit**

```bash
cd infra
pre-commit install
mise run check          # expect: green (nothing changed yet functionally)
git add -A && git commit -m "chore(scaffold): babylon-infra identity — README, CLAUDE.md, mise tasks, pre-commit battery"
```

If any tool is missing on the host PATH, note it and continue — Task A4's flake provides the
full toolchain; re-run `mise run check` inside `nix develop` after A4.

---

### Task A4: Nix flake devshell

**Files (inside `infra/`):**
- Create: `flake.nix`, commit generated `flake.lock`

**Interfaces:**
- Produces: `nix develop` shell used by all later validation; `PLAYWRIGHT_BROWSERS_PATH`,
  `LD_LIBRARY_PATH` (libpq), `*_NUM_THREADS=1` exported by shellHook.

- [ ] **Step 1: Write `flake.nix`**

```nix
{
  description = "Babylon — standardized dev/build environment + infra toolchain (program-20)";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfreePredicate = pkg:
            builtins.elem (nixpkgs.lib.getName pkg) [ "terraform" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            # --- Babylon build env (system layer; Poetry/mise keep their jobs) ---
            python312
            poetry
            nodejs_22
            git-lfs
            postgresql_16.lib   # libpq for pure-python psycopg
            gdal                # GeoDjango runtime
            geos
            proj
            openblas
            rustc
            cargo
            fluidsynth
            playwright-driver.browsers
            # --- infra toolchain ---
            terraform
            tflint
            ansible
            ansible-lint
            sops
            age
            shellcheck
            yamllint
            pre-commit
          ];

          shellHook = ''
            # Determinism + 2026-07-12 freeze fix (mirrors ../.mise.toml [env])
            export OMP_NUM_THREADS=1 OPENBLAS_NUM_THREADS=1 MKL_NUM_THREADS=1
            export NUMEXPR_NUM_THREADS=1 RAYON_NUM_THREADS=1
            # libpq discoverable for pure-python psycopg
            export LD_LIBRARY_PATH=${pkgs.postgresql_16.lib}/lib''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}
            # Pinned Playwright browsers (no npx playwright install)
            export PLAYWRIGHT_BROWSERS_PATH=${pkgs.playwright-driver.browsers}
            export PLAYWRIGHT_SKIP_VALIDATE_HOST_REQUIREMENTS=true
            echo "babylon devshell: python=$(python3 --version) node=$(node --version) terraform=$(terraform version | head -1)"
          '';
        };
      });
}
```

- [ ] **Step 2: Evaluate + lock**

Run: `cd infra && nix flake check 2>&1 | tail -5 && nix develop --command bash -c 'python3 --version && node --version && terraform version | head -1 && ansible --version | head -1'`
Expected: flake check passes; Python 3.12.x, v22.x, Terraform 1.x, ansible-core 2.x.
If `playwright-driver.browsers` or version mismatches fail on nixos-25.11, note the exact error
and bump the input to `nixos-unstable` for that package via an overlay — record the reason in
the commit message.

- [ ] **Step 3: Re-run the battery inside the shell**

Run: `nix develop --command mise run check`
Expected: green.

- [ ] **Step 4: Commit**

`git add flake.nix flake.lock && git commit -m "feat(nix): devshell — babylon build env + infra toolchain, thread pins, pinned playwright browsers"`

- [ ] **Step 5: Parent `.envrc` wiring (worktree, ONE line, guarded)**

Append to the parent worktree's `.envrc` (do NOT reorder or touch existing lines — it contains
key material):

```bash
# program-20: opt-in nix devshell (no-op when infra/ absent or nix missing)
if [ -d infra ] && command -v nix >/dev/null 2>&1; then use flake ./infra; fi
```

Commit in parent: `git add .envrc && git commit -m "chore(env): guarded use-flake for infra devshell (program-20)"`
(If `.envrc` is gitignored in the parent, skip the commit and instead document the line in
`infra/README.md` Quickstart — verify with `git check-ignore .envrc` first.)

---

### Task A5: Terraform — AI Gateway + Workers AI surface

**Files (inside `infra/`):**
- Create: `terraform/ai.tf`
- Modify: `terraform/variables.tf` (append), `terraform/outputs.tf` (append),
  `terraform/terraform.tfvars.example` (append + token-scope comment update)

**Interfaces:**
- Produces: TF outputs `ai_gateway_id`, `workers_ai_chat_base_url` — Track B's
  `LLMConfig.WORKERS_AI_*` defaults mirror these values.

- [ ] **Step 1: Write `terraform/ai.tf`**

```hcl
# Program 07 Decision 3 (owner, 2026-07-03): Workers AI narrator.
# Constitution X.5: Workers AI is Cloudflare's side of the division of labor.
# The gateway fronts every narrator call: logging, caching, rate limiting.
resource "cloudflare_ai_gateway" "narrator" {
  count      = var.manage_cloudflare ? 1 : 0
  account_id = var.cloudflare_account_id
  id         = var.ai_gateway_name

  collect_logs                = true
  cache_ttl                   = 0     # narration is per-tick unique; caching off by default
  cache_invalidate_on_update  = true
  rate_limiting_interval      = 60
  rate_limiting_limit         = var.ai_gateway_rpm_limit
  rate_limiting_technique     = "sliding"
}
```

- [ ] **Step 2: Append to `terraform/variables.tf`**

```hcl
variable "ai_gateway_name" {
  description = "AI Gateway id/slug fronting the Workers AI narrator (program-20)."
  type        = string
  default     = "babylon-narrator"
}

variable "ai_gateway_rpm_limit" {
  description = "Requests/min through the narrator gateway (one narrate_tick per tick + retries)."
  type        = number
  default     = 30
}
```

- [ ] **Step 3: Append to `terraform/outputs.tf`**

```hcl
output "ai_gateway_id" {
  description = "AI Gateway slug for cf-aig-gateway-id headers."
  value       = var.manage_cloudflare ? cloudflare_ai_gateway.narrator[0].id : null
}

output "workers_ai_chat_base_url" {
  description = "OpenAI-compatible chat base URL (Django WORKERS_AI settings consume this)."
  value       = "https://api.cloudflare.com/client/v4/accounts/${var.cloudflare_account_id}/ai/v1"
}
```

- [ ] **Step 4: Update `terraform/terraform.tfvars.example`** — extend the token-scope comment:

```hcl
# Token scopes: Zone DNS Edit, Zone SSL and Certificates Edit, Account R2 Edit,
#               Workers AI Read + Edit, AI Gateway Read + Edit   (program-20)
```

- [ ] **Step 5: Validate**

Run: `nix develop --command bash -c 'mise run validate && mise run lint:tf'`
Expected: PASS. **If `cloudflare_ai_gateway` attribute names are rejected** (provider v5 schema
drift): run `terraform -chdir=terraform providers schema -json | jq '.provider_schemas[].resource_schemas.cloudflare_ai_gateway.block.attributes | keys'`
and rename fields to match; do not guess twice.

- [ ] **Step 6: Commit**

`git add terraform && git commit -m "feat(terraform): AI Gateway + Workers AI surface — narrator gateway, scopes, outputs (program-07 D3)"`

---

### Task A6: sops+age + Ansible cleanup

**Files (inside `infra/`):**
- Create: `.sops.yaml`, `docs/secrets.md`
- Modify: `ansible/playbooks/bootstrap.yml` (fix dangling `roles/common` template ref)
- Delete: `ansible/playbooks/hetzner.yml` (vendored generic content: Jakarta timezone, Dokploy)
- Modify: `ansible/README.md` (playbook inventory update)

- [ ] **Step 1: age key + `.sops.yaml`**

```bash
mkdir -p ~/.config/sops/age
[ -f ~/.config/sops/age/keys.txt ] || age-keygen -o ~/.config/sops/age/keys.txt
age-keygen -y ~/.config/sops/age/keys.txt   # prints the public recipient
```

Write `.sops.yaml` with the REAL recipient from above:

```yaml
creation_rules:
  - path_regex: secrets/.*\.enc\.(json|yaml|tfvars)$
    age: "<the age1... public key printed above>"
```

- [ ] **Step 2: Write `docs/secrets.md`** — the owner-run flow (agents never do this):
`sops --encrypt terraform/terraform.tfvars > secrets/terraform.tfvars.enc.json` to bring secrets
into the repo encrypted; decrypt with `sops --decrypt`. Note: private key lives ONLY in
`~/.config/sops/age/keys.txt`; committing encrypted secrets is the OWNER's call, default is
local-plaintext-gitignored exactly as today.

- [ ] **Step 3: Fix `bootstrap.yml`** — read it; it references
`roles/common/templates/sshd_config.j2` but no `common` role exists. Fix = point at the base
role instead: create `ansible/roles/base/templates/sshd_config.j2` containing the hardened
config the playbook expects (PermitRootLogin no, PasswordAuthentication no, PubkeyAuthentication
yes, the custom port variable) — copy the template content from
`docs/HOW-TO-DEPLOY-HETZNER.md`'s hardening section if present there, else write those four
directives plus `Port {{ ssh_port | default(22) }}`. Update the playbook's `template:` src path
to `../roles/base/templates/sshd_config.j2`. Verify with syntax-check.

- [ ] **Step 4: Delete `ansible/playbooks/hetzner.yml`**; update `ansible/README.md`'s live-
playbook list (6 → 5) and remove its syntax-check line if one was added to `.mise.toml`.

- [ ] **Step 5: Validate + commit**

`nix develop --command mise run lint:ansible` → PASS.
`git add -A && git commit -m "fix(ansible): repair bootstrap sshd template ref, prune vendored hetzner.yml, sops+age scaffold"`

---

### Task A7: Workstation playbook (the real target)

**Files (inside `infra/`):**
- Create: `ansible/workstation.yml`, `ansible/roles/workstation/tasks/main.yml`,
  `ansible/roles/workstation/defaults/main.yml`
- Modify: `.mise.toml` (add workstation syntax-check line to `lint:ansible`)

**Interfaces:**
- Consumes: parent-repo scripts referenced read-only at
  `{{ babylon_repo_root }}/tools/data_doctor.sh`.

- [ ] **Step 1: `ansible/roles/workstation/defaults/main.yml`**

```yaml
babylon_repo_root: /home/user/projects/game/babylon
data_mount_point: /media/user/data
data_luks_device: /dev/mapper/luks-1b2ddee8-005d-4efd-9f82-482168bde87c
# The exact fstab line the 2026-07-13 incident heal added — preserved verbatim (nofail + timeout)
data_fstab_line: "{{ data_luks_device }} {{ data_mount_point }} ext4 defaults,nofail,x-systemd.device-timeout=10 0 2"
workstation_services_enabled:
  - docker
  - earlyoom
  - ollama
  - cron
workstation_packages:
  - fluidsynth
  - fluid-soundfont-gm   # SOUNDFONT=/usr/share/sounds/sf2/FluidR3_GM.sf2 in .mise.toml [env]
  - earlyoom
  - age
```

- [ ] **Step 2: `ansible/roles/workstation/tasks/main.yml`**

```yaml
---
# Codifies the 2026-07-13 data-mount-displacement incident learnings
# (reports/incident-2026-07-13-data-mount-displacement.md). Constitution III.11:
# every guard FAILS LOUD; no silent skip.

- name: fstab guard — the LUKS data mount entry must exist with nofail + device-timeout
  ansible.builtin.lineinfile:
    path: /etc/fstab
    line: "{{ data_fstab_line }}"
    state: present
  become: true

- name: data drive must actually be mounted at the mount point
  ansible.builtin.command: findmnt --target {{ data_mount_point }} --source {{ data_luks_device }}
  changed_when: false

- name: workstation packages present (audio + safety tooling)
  ansible.builtin.apt:
    name: "{{ workstation_packages }}"
    state: present
  become: true

- name: required services enabled + running
  ansible.builtin.systemd:
    name: "{{ item }}"
    enabled: true
    state: started
  become: true
  loop: "{{ workstation_services_enabled }}"

- name: system postgres (5432) is the apt cluster, project postgres (5433) is the container
  ansible.builtin.shell: |
    set -euo pipefail
    ss -tln | grep -q ':5432 '
    docker ps --filter name=babylon-pg-isolated --filter status=running --format '{{ "{{" }}.Names{{ "}}" }}' | grep -q babylon-pg-isolated
  changed_when: false

- name: dormant apt postgresql-17 cluster must NOT be running (it squats port 5433)
  ansible.builtin.shell: "! pg_lsclusters --no-header | awk '$1==17 {print $4}' | grep -q online"
  changed_when: false

- name: trove health — delegate to the repo's own doctor (loud on failure)
  ansible.builtin.command: "{{ babylon_repo_root }}/tools/data_doctor.sh"
  changed_when: false
```

- [ ] **Step 3: `ansible/workstation.yml`**

```yaml
---
- name: Babylon workstation — dev-box state codified (program-20)
  hosts: localhost
  connection: local
  gather_facts: true
  roles:
    - workstation
```

- [ ] **Step 4: Add to `.mise.toml` `lint:ansible`** the line
`ansible-playbook --syntax-check -i localhost, ansible/workstation.yml` (if not already added
in A3).

- [ ] **Step 5: Validate (syntax + lint + CHECK MODE ONLY)**

```bash
nix develop --command mise run lint:ansible
nix develop --command ansible-playbook -i localhost, --check --diff ansible/workstation.yml
```

Expected: syntax/lint green. Check mode will show the fstab line as ok (already present) and may
FAIL on `become: true` sudo prompts — that is acceptable and expected for an agent run; record
the output. The real converge is owner-run: `ansible-playbook -i localhost, -K ansible/workstation.yml`.

- [ ] **Step 6: Commit**

`git add -A && git commit -m "feat(ansible): workstation playbook — mount guard, service state, 5432/5433 split, doctor delegation"`

---

### Task A8: Parent detach (destructive — LAST)

**Files (parent worktree):**
- Delete: `deploy/` (tracked), `.github/workflows/infra-live.yml`, `.tflint.hcl`
- Modify: `.github/workflows/main.yml` (remove `infra-validate` job), `tools/check_repo_hygiene.py`
  (remove `"deploy"` from ALLOWED_TOP_LEVEL_DIRS and `".tflint.hcl"` from ALLOWED_TOP_LEVEL_FILES),
  `.pre-commit-config.yaml` (hadolint hook: verify it targets only `docker/` — leave if so)

Pre-condition: Tasks A2–A7 all green (`nix develop --command mise run check` inside infra/).

- [ ] **Step 1: Reference sweep BEFORE deleting**

Run: `rg -l 'deploy/' --glob '!deploy/**' --glob '!.claude/**' --glob '!infra/**' . | sort`
For each hit decide: update path to `infra/` (docs) or delete the reference (CI). List the hits
in the commit message.

- [ ] **Step 2: Remove**

```bash
git rm -r deploy
git rm .github/workflows/infra-live.yml .tflint.hcl
```

- [ ] **Step 3: Edit `main.yml`** — delete the entire `infra-validate:` job block (find with
`rg -n 'infra-validate' .github/workflows/main.yml`); also remove it from any `needs:` lists.
Verify: `pre-commit run actionlint --files .github/workflows/main.yml` → PASS.

- [ ] **Step 4: Edit `tools/check_repo_hygiene.py`** — remove the `"deploy",` line from
`ALLOWED_TOP_LEVEL_DIRS` and the `".tflint.hcl",` line from `ALLOWED_TOP_LEVEL_FILES`.
Verify: `PYTHONPATH="$PWD/src" poetry run python tools/check_repo_hygiene.py` → exit 0.

- [ ] **Step 5: Parent gate + commit**

```bash
mise run check:quick
git add -A && git commit -m "refactor(infra)!: retire deploy/ — extracted to local babylon-infra subrepo at infra/ (program-20, ADR071)"
```

Expected: check:quick green. BREAKING marker justified: repo layout change.

---

### Task A9: Close-out

- [ ] Update `ai/state.yaml` (program-20 Track A status), commit.
- [ ] `nix develop --command mise run check` (infra) + parent `mise run check:quick` one last time;
  paste both outputs into the task report.
- [ ] Write memory: infra subrepo location/mechanics, flake usage, apply gates.
