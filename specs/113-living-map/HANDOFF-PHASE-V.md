# Program 16 handoff — Phase V in flight (Fable → successor orchestrator)

Written 2026-07-11 ~15:50 EDT by Fable at Percy's request (Fable tokens running
out). You are picking up a *live, mostly-green* program. Read this whole file,
then `project/programs/16-living-map.md` (charter),
`specs/113-living-map/integration-ledger.md` (seam state), and
`project/research/16-living-map/DESIGN_BIBLE.md` §9b + §10 (binding law + exit
gates). The standing goal (Stop-hook): **"the complete, finished and polished
user interface for the Babylon game."**

## Where you are

- Waves 1–3 are COMMITTED (`577f62ec`/`0d7b269f`/`ae2c5fb4` → `fe747a02` →
  `04c3c93f`). The game is real: full-bleed TIGER county map, Φ default lens,
  InspectionStack→FormulaCard drill against live `/explain/`, Guix Installer
  chrome (bible §9b), narration scaffolding mounted. 742/742 vitest.
- Phase V (live verification) is MID-FLIGHT with an **uncommitted fix set on
  disk** (see below). e2e went 6→14-of-16 authenticated + most backend-free
  green through this afternoon's fixes.
- Task list: #4 Phase V in_progress; #2 Phase D pending.

## The uncommitted Phase-V fix set (verified: 742/742 vitest, tsc/eslint clean)

Commit this EARLY (one commit, `fix(frontend): Phase V live-run fixes —
spec-113`, Co-Authored-By trailer, `mise run commit` from worktree root):

- `src/index.css` — `@source not "../public/geo"` (Tailwind v4 scanner spun
  100% CPU forever on the single-line 2.4MB TopoJSON; killed every module
  transform. THE root cause of all "dev server hangs" today).
- `vite.config.ts` + `playwright.config.ts` + `.env.example` (new) — Percy
  ruling: ports/backend URL configurable via committed `.env.example` template
  → gitignored `.env`/`.env.local` → shell env wins. Repo gitignores all
  `.env*` except `.env.example` (post-token-leak hygiene) — do NOT commit .env.
- `playwright.config.ts` — `launchOptions.args: ["--use-angle=gl",
  "--enable-gpu"]`: headless Chromium gets the real RTX 3060; SwiftShader made
  deck.gl picking wedge the renderer for minutes (root cause of all
  canvas-click timeouts + the "Escape doesn't work" red herring). Falls back to
  SwiftShader on GPU-less boxes. Also: `event-popup.spec.ts` added to
  AUTHENTICATED_SPECS.
- `src/components/map/DeckGLMap.tsx` + `.test.tsx` — hex clicks now pass
  `h3_index ?? id` (backend contract is `GET /hex/:h3_index/`); found live as
  an unresolvable-403 card. TDD'd (2 new tests).
- `src/components/shell/AppShell.tsx` — chrome layer `z-20` (HUD must beat
  MapControls' in-map `z-10`; lens bar was swallowing TopBar clicks) + ActionDock
  wrapper `z-10` (BottomDrawer's full-width strip was swallowing composer
  Submit clicks).
- `src/components/map/MapControls.tsx` — legend `left-[250px] top-14`, lens
  cluster `right-[300px] top-14` (were buried under TopBar/outliner/tray).
- `src/components/chrome/FloatingPanel.tsx` — left/right anchors now `top-14`
  (outliner was burying the TopBar's left segment).
- `src/components/inspect/InspectionStack.tsx` — Escape listener subscribes
  once on stable `pop` (see open thread 1 — fix applied, e2e NOT yet green).
- `e2e/real-loop.spec.ts` — scenario picker is a listbox now
  (`scenario-option-wayne_county` testid), native `<select>` is gone.
- `e2e/end-turn-flow.spec.ts` — spacebar-pause budget 45s (one live resolve
  can be ~20-30s). STILL needs `test.setTimeout` (>30s default) — see thread 3.
- `e2e/event-popup.spec.ts` — `toBeAttached` (empty flex rail has zero box) +
  `test.setTimeout(MAX_STEPS*30s+60s)`.
- `e2e/inspection-stack.spec.ts` — bounded poll-click helper (WebGL has no DOM
  to auto-wait on) — keep even with GPU.
- `e2e/map-lens-cycling.spec.ts` — `test.slow()` on the 9-lens cycle.
- `specs/113-living-map/integration-ledger.md` — wave-3 sections + vision-doc
  audit (uncommitted edits).

## Open threads, in priority order

### 1. Escape-pop e2e still red (2 tests) — investigation nearly done
Root cause FOUND and fixed in InspectionStack.tsx: one physical Escape fired
`pop()` twice — pop mutates the stack, React 19 flushes the
`[stack.length, pop]`-keyed effect synchronously mid-dispatch (keydown is
discrete), the re-added document listener catches the SAME in-flight event
(Chrome live listener list; jsdom snapshots — units can't see it). Fix:
subscribe once on `[pop]`; pop() no-ops safely on empty/pinned.
**BUT** the post-fix e2e rerun still failed the same 2 tests
(`inspection-stack.spec.ts:170` Escape-pops, `:212` breadcrumb) — I ran out of
tokens before reading the post-fix error contexts. NEXT STEP (exact):
```
cd src/frontend && COCKPIT_E2E_PORT=5180 npx playwright test e2e/inspection-stack.spec.ts --workers=1 --reporter=line
# then read test-results/inspection-stack-*/error-context.md for the failed step
```
Possibilities: (a) Vite HMR hadn't picked up my edit when that run fired —
rerun first before diagnosing; (b) the double-pop also afflicts a SECOND
same-patterned listener; (c) test 212's breadcrumb `popTo` path has its own
issue. The probe methodology that cracked it is reusable: write a `.pwprobe.mjs`
in src/frontend (module resolution!), route-mock the endpoints (copy the
spec's fixtures), `addInitScript` monkeypatching
`document/window.addEventListener("keydown", ...)` to log registrations AND
firings. Delete probes after use.

### 2. Engine bug (OWNER ITEM, do NOT fix from this branch — engine untouched
is a charter constraint): `psycopg.errors.UniqueViolation
ux_simulation_event_session_tick_natural` — engine emitted 2 events at one
tick that both serialize `event_type=UNKNOWN`, empty entity/community → same
natural key → resolve 500s → session dies mid-play (frontend honestly shows
ERROR). Traceback: `postgres_runtime/_legacy.py:2344 _persist_events`.
Reproduced ~tick 17-18 of a wayne_county 2x play loop. ALSO: events surfacing
as UNKNOWN means the web event-type mapping has gaps → nothing ever
classifies urgent → **no toasts in 20 live ticks** (event-popup's red is the
spec honestly pinning this). Record both in ledger owner items; report to
Percy; leave the two specs red-as-designed OR test.fixme with a pointer —
Percy's call, ask via the report, default leave red with docstrings.

### 3. end-turn-flow spacebar test: add `test.setTimeout(90_000)` inside the
2x-speed test (my 45s expect exceeds the 30s default test budget), rerun.
It will STILL go red whenever thread-2's 500 fires mid-loop — same
disposition as thread 2 (real defect pinned; document in its docstring).

### 4. Remaining live-verification checklist
- Full suite green run: `COCKPIT_E2E_PORT=5180 npx playwright test --workers=2`
  (30 tests; expect green except the thread-2/3 engine-pinned reds).
- Screenshot review vs bible §10 exit gates: probe scripts can screenshot
  (`p.screenshot({path})`) — login, lobby, game shell with composer open,
  takeover framing, InspectionStack drill. Percy wants to SEE the Installer.
- `pytest tests/unit/web/ -q` from worktree root (467 tests) — untouched since
  Wave 1 but re-verify before PR.
- qa:regression NOT needed unless something under `src/babylon` changes (it
  hasn't — verify with `git diff dev...HEAD --stat -- src/babylon`).

## Environment (live right now; rebuild-able)

- Worktree `/home/user/projects/game/babylon/worktrees/living-map`, branch
  `feature/113-living-map` off dev @ `2d3b2547`. NEVER touch
  `ci/two-tier-refactor` (other CC session) or main-checkout state.
- Django :8001 = THIS worktree's web/ (started:
  `cd web && RUN_MAIN=true BABYLON_EXTRA_DEV_ORIGINS="http://localhost:5180,http://127.0.0.1:5180" nohup uv run python manage.py runserver 8001 --noreload`).
  Log: scratchpad `django-8001.log`. The :8000 Django is the MAIN checkout's —
  no `/explain/`, don't use, don't kill.
- Vite :5180 = `node node_modules/.bin/vite --port 5180 --strictPort --host
  127.0.0.1` with `COCKPIT_BACKEND_URL=http://localhost:8001` (MUST bind
  127.0.0.1: `localhost` binds ::1-only and headless Chromium won't connect).
  :5173 belongs to another worktree — never kill.
- Postgres :5432 up, web db migrated, superusers admin/admin, testuser, user.
  Seeded session `5ad0c6ae` exists; e2e create fresh wayne_county sessions.
- kill of a Vite pid can silently fail — verify with
  `ss -tlnp | rg 5180`, then `kill -9`.
- Machine load matters: Percy games on this box (RE9 at 250% CPU). Playwright
  runs fine on hardware GL even under load, but budget generously.

## Wave 4 + Phase D (Percy AUTHORIZED this fan-out — ultracode, Opus allowed)

Percy's mid-session ruling: dispatch remaining features as a parallel wave;
he explicitly offered Opus subagents. Plan I promised him:
- Lane PULSE (Opus): map-anchored critical-event cue — third channel of bible
  §5.2 (event→geo ref exists in classifier; render a deck.gl pulse at the
  linked county/hex on critical events; compositor-safe, no animated
  box-shadow on large surfaces).
- Lane STRIPE (Opus): contested-claim TRUE striping via
  `@deck.gl/extensions` FillStyleExtension (dash outline shipped Wave 2;
  pattern fill is the CK3 convention the bible names).
- Lane DELTA (Sonnet): verb predicted-delta arrows —
  `lib/verbs/types.ts.predictedEffect` exists unrendered ("Phase 5 renders
  this"); surface in VerbForm/VerbGrid pre-commit preview, honest-null when
  absent.
- Lane SHIMMER (Sonnet): wire `claim-shimmer` keyframe (index.css, shipped) to
  actual polity-claim redraws (Lane B/Carto's political.ts consumer), ≥600ms,
  cause-named per bible §2.2.
- Lane DS-SYNC (Sonnet): Phase D — add ALL new spec-113 components to
  `design-sync.entry.tsx` barrel + `.design-sync/config.json` componentSrcMap
  (currently ZERO of the new chrome is in the barrel), preview seeds, then
  sync to claude.ai/design project "Babylon Cockpit"
  (`9ccdf916-1447-4c12-92a3-dfc2a0939a4c` — NEVER the old "Babylon Design
  System"), grade renders. Recipe: `.design-sync/NOTES.md`.
Discipline (unchanged): one shared worktree, file-disjoint ownership grants,
agents NEVER commit, structured-output schemas, orchestrator verifies with
full gates and commits one atomic wave commit, integration seams go in the
ledger. Read the Wave-3 workflow script for the template:
`~/.claude/projects/-home-user-projects-game-babylon-worktrees-living-map/*/workflows/scripts/living-map-wave3-*.js`.

## Wrap-up (after Wave 4 + Phase V close)

- Update `ai/state.yaml`; write ADR (ADR066+) for Phase V findings; update
  charter + ledger; memory files (see MEMORY.md Program-16 pointers).
- Ledger's unchecked items → owner items. Report engine bugs (thread 2,
  `get_inspector_hex` stub returning `{}`, hex-ref/territory-id mapping for
  event deep-links) prominently.
- PR to dev only when Percy asks. `mise run check` must be green.

## Hard-won judgment (don't relearn these)

1. **Trust the live browser over deduction.** Every Phase-V bug (tailwind
   spin, IPv6 bind, SwiftShader picking, z-order interception, double-pop) was
   cracked by a probe observing reality, not by reading code harder. The
   error-context.md files + `--trace=on` screenshots + addInitScript
   instrumentation are your microscope.
2. **"Environment flakiness" usually isn't.** Three "flaky" clusters were
   real bugs with mechanical causes. Only mark environment-blocked with a
   root-caused mechanism in hand.
3. jsdom-green ≠ browser-green for anything involving dispatch timing, focus,
   layout, or WebGL. The unit suite cannot see listener-list semantics,
   pointer interception, or picking.
4. Playwright "element intercepts pointer events" names the offender —
   read the call log before theorizing.
5. Percy's style: full vision, no MVP scoping; verify before claiming;
   commit per unit of work with trailer; surface conflicts rather than
   simplify tests to green. When a spec pins a real defect, leave it red and
   say so loudly.
6. Sequential debugging threads: do NOT fan out agents on a single shared
   live environment (ports/CPU contention); fan out only file-disjoint
   feature lanes.
7. Watch shell cwd resets between Bash calls (worktree root vs src/frontend)
   — several "no such file" errors were just cwd drift. Use absolute paths.
