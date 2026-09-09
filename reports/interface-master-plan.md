# The Interface Master Plan — making Babylon feel like a genuine game

**Charter:** Director directive 2026-07-27, after the first clean-screen
playtest of the Rust client: *"there's still no HUD with keybinds … I don't
see the ability to really navigate with mouse and click, it feels crowded —
we really need to plan the whole interface out to feel like a genuine
game."* This is that plan: the binding canon, the precise as-built defect
map, the adopted design (each pattern with a named precedent), a wave-
ordered build-out, and the rulings only the Director can make.

**Sources:** three-scout recon `wf_807422cf-4e4` (canon / as-built /
genre), the DESIGN_BIBLE (`project/research/16-living-map/DESIGN_BIBLE.md`
§9b/§11/§7), ADR097/ADR099/ADR150, Constitution II.8 + Article V + Article
VII, `reports/epochs-vision-gap-audit.md` (whose ledger+waves structure
this document deliberately reuses), and the M1–M4 seam contracts.

---

## 1. What is already ruled (build WITHIN this, never re-litigate)

- **Palette:** ksbc/"THE INSTALLER" (crimson `#dc143c` / gold `#ffd700` /
  near-black field `#1a0000` / bone `#e8e8e8`), ratified 2026-07-11,
  drift-guarded in code on both sides of the FFI. Article VII binds the
  *principles* (color-as-data, no chartjunk, no mood-over-meaning); the
  concrete tokens may evolve without amendment.
  **`ai/design-system.yaml`'s "Bunker Constructivism" palette is dead** —
  it still self-labeled source-of-truth; the Director ruled it DELETED
  outright (ruling 3, 2026-07-27; git history preserves it). Three
  `ai/mantras.yaml` entries (`red_is_pain`-family, `purple_is_life_green_
  is_data`, `screen_is_physical`) still encode that palette's purple/green
  and CRT-overlay aesthetics — their `reference:` pointers are redirected
  to history, but the mantra TEXT is aesthetic line and awaits the
  Director's own revision.
- **Dialog anatomy ("newt idiom"):** menu surfaces = flat near-black field,
  one centered plate, hard offset shadow, title-tab-breaks-border,
  inverse-video gold selection bar, chunky keyboard-hint buttons. In-game
  floating chrome reuses the anatomy at reduced opacity.
- **Render tiers (ADR097/ADR099):** Tier 0 glyph canon is the DESIGN
  TARGET — everything legible in character cells; pixel plates carry zero
  unique information; probe-once, never silently re-probe.
- **Client contract (II.8/Amendment V):** the client is a disposable
  viewport over `observe()`; it emits intents through the verb registry
  and never runs sim logic. This plan designs chrome, not new seams.
- **Article V:** exactly 9 player verbs; any new verb (strike…) is
  amendment-gated (task #36), NOT a UI decision.
- **Weather grammar (§11):** extensive = stuff, intensive = color,
  qualitative change = hard cut, ONE motion budget per tick.
- **Voice (§7):** newspaper register for wire/HUD, theory register for
  drill-down; the adopted/rejected lexicon stands.
- **Mock Doctrine:** any staged surface for a not-yet-built mechanic is
  visibly MOCK-badged (the M4 honest-absence lines are this doctrine
  working). **Emergent endgames:** no scripted win/lose screens, ever.
- **Patches** the golden snub-nosed monkey is settled tutorial content.

## 2. The defect map (as built, cited by the recon; the "why it doesn't
   feel like a game" facts)

| # | Defect | Evidence |
|---|--------|----------|
| D1 | **No keybind bar.** F1–F9 are visible only because the verb plate bakes them in; Tab, `/`, P, K, `[`/`]`, t/r/a, 1–4, Esc have zero on-screen representation. Textual's Footer gave this for free; the Rust shell never got an equivalent. | app.rs/views: no footer surface |
| D2 | **No help overlay.** Neither client has a one-keypress help screen (Textual's was a 2-hop palette search). | recon as-built (5) |
| D3 | **Mouse covers 2 of ~9 regions.** Only wikilinks and verb rows are clickable; rails, HUD, tutorial strip, lobby rows, pane switching are mouse-dead, and a click on a rail doesn't even focus it. Wheel scroll is silently dropped. | handle_mouse catch-all |
| D4 | **Focus is nearly invisible + Tab-only.** No border-color focus state (center pane has none at all); no Shift-Tab reverse cycle. | recon as-built |
| D5 | **Verb plate clips at 80×24 with the tutorial up** — F5/F6/F9 silently truncate off-screen for exactly the audience the tutorial serves (new players). Reproducible ratatui-solver math, not a vibe. | recon as-built (3) |
| D6 | **Two fixed 24-col rails eat 60% of an 80-col terminal** (center pane: 30 usable columns at 80 wide vs 70 at 120). The single largest "crowded" contributor. | recon as-built (4) |
| D7 | **No game-feel texture:** no confirmation feedback on verb/tick beyond text, SFX estate (39 sounds, ADR152) fully unwired, no title/menu takeover in the newt idiom. | recon genre (6,7) |

## 3. The design (adopt / skip, each with a precedent)

**ADOPT — Wave 1 (discoverability + input honesty):**
1. **Persistent bottom keybar**, htop/Midnight-Commander style, clickable
   (mouse machinery exists), **context-aware per focus/pane/mode** — the
   topology pane shows its camera keys, the wiki its jumplist, the lobby
   its mint/load keys. This alone answers D1 and half of D7.
2. **`?` help overlay** — a full, mode-scoped binding list as a view-stack
   entry (lazygit/yazi/NetHack), title naming the active surface.
3. **Mouse parity sweep**: click-to-focus AND click-to-open on both rails,
   clickable pane titles, clickable keybar, wheel scroll in wiki +
   topology glyph floor + rails. Doctrine: **keyboard-primary, mouse as
   full parity** (Cogmind), never mouse-first (the Dwarf-Fortress-Steam
   anti-pattern); a Qud-style "adventure mouse" layer only if later
   demanded.
4. **Focus indicators**: focused region gets the crimson heavy border
   (Textual's own `:focus` rule ported); Shift-Tab reverse cycle.
5. **Verb-plate floor fix** (D5): a `Constraint::Min` so the solver can
   never cannibalize it below its 11-line requirement — or paginate with
   a visible "+N more" indicator. Release-blocking bug, not polish.

**ADOPT — Wave 2 (density):**
6. **Responsive rails**: percentage widths below a threshold (D6). With
   the ruled 100×30 floor, the fixed 24-col rails leave 52 center
   columns at exactly 100 wide — still tight; below the floor the
   graceful too-small notice takes over (ruling 1), so no 80-col
   collapse mode is needed.
7. **Pane zoom** (lazygit `+`/`_`): default → half → full-screen for the
   center pane — the escape hatch the 3D topology + field surfaces need.
8. **Cogmind-style verb menu** (one key, wide layout, letter shortcuts,
   ~150 ms delay so experts never see it) — discoverability for the 9
   verbs without touching Article V semantics.

**ADOPT — Wave 3 (game feel):**
9. **SFX wiring**: a hand-authored verb/event→category→cue table over the
   ADR152 estate (Cogmind's "category not instance" rule); tick-advance,
   verb-issue, autopause, endgame-axis movement get cues. Table authored
   in Wave 3's charter, Director reviews it as the aesthetic line
   (ruling 6, RULED).
10. **Confirmation feedback**: restrained glyph/color flash on verb-issue
    and tick-commit (never screen-shake — rejected as arcade-adjacent and
    against the ksbc register).
11. **Newt-idiom takeovers**: the lobby/briefing/login surfaces restyled
    to the ratified dialog anatomy (centered plate, hard shadow, gold
    selection bar) — the "title screen energy" of a genuine game.
12. **Hover status line** (Brogue/CK3): `Moved` events are already
    plumbed; render a one-line context hint for whatever the cursor is
    over. Near render-only cost.
13. **The FULL event-system doctrine** (ruling 5, RULED: build it, not
    just bless the rail): urgent + ambient streams, three severity tiers
    with tier coloring, critical events on THREE channels (chronicle
    rail + HUD flash + SFX cue), and a recoverable dismissal tray, all
    terminal-native chrome. Lands as ONE unit in Wave 3 — the standing
    no-MVP-split rule forbids phasing it, and the third critical channel
    needs item 9's cue table anyway. The chronicle rail remains the
    ambient stream's home; the urgent stream and tray are new chrome.

**SKIP (with reasons, so they stay skipped):** external keybind remapping
(no demand; input is not `defines.yaml`'s moddable surface), which-key
chord popups (no chords exist), CK3 nested tooltips (Peek IS the
terminal-native equivalent), screen-shake/juice (register violation), rail
row-compaction (demand-driven; wait for evidence).

## 4. Waves and homes

- **Wave 1** = its own train (`feature/interface-wave1`), START
  IMMEDIATELY after M4 ships (ruling 7, RULED) — it is the Director's
  reported pain, and every later milestone benefits. Independent of
  M5/M6 content.
- **Wave 2** rides alongside M5 (map/choropleth) — zoom and responsive
  rails are what make the map pane livable.
- **Wave 3** is the polish train before the v1.0 cut (post-M6, pre-M7),
  where the SFX estate and takeover styling land as a unit.
- The epochs-gap backlog (Doctrine-Tree UI, Epistemic-Horizon fog
  surfaces, Hegemony lenses…) is CONTENT, not chrome — per ruling 4
  (RULED: fresh-prioritize) it re-enters through new terminal-native
  charters, never a wholesale re-target of the web-era ledger.

## 5. Director rulings — ALL SEVEN RULED (2026-07-27/28)

1. **Minimum terminal size: 100×30 declared floor** (with a graceful
   too-small notice below it). *Amended 2026-07-28 (Director field
   report — a fullscreen 151×27 laptop terminal was locked out): the
   GUARD floor is 100×24; density stays designed to 100×30, and the
   verb-plate invariant holds at every admitted height via the tutorial
   strip clamp (Wave 1 contract §7.7).* Density is designed to 100×30;
   the D5/D6 math is recomputed against it — the verb-plate Min-constraint fix
   stays release-blocking regardless (resize mid-session can still
   squeeze the solver).
2. **Terminal grammar is the legitimate successor** to the DESIGN_BIBLE's
   web-era interaction grammar (left-click=inspect / right-click=act /
   Q/E lenses / space=pause). The t/r/a, F1–F9, Tab, pane scheme is
   canon; the web grammar is history, not a porting target.
3. **`ai/design-system.yaml`: DELETE outright.** Git history preserves
   it; no annotated husk. Live pointers (`ai/README.md` index row,
   three `ai/mantras.yaml` `reference:` fields) are cleaned in the same
   commit; the mantra TEXT (purple/green, CRT overlays) is flagged in §1
   for the Director's own revision.
4. **Epochs roadmap: fresh-prioritize per terminal constraints** — the
   web-era Wave 1–6 ledger is not re-targeted wholesale; content items
   re-enter through new terminal-native charters.
5. **Event-system doctrine: BUILD THE FULL DOCTRINE** in the terminal
   client — urgent+ambient streams, three severity tiers, triple-channel
   criticals, recoverable dismissal tray (§3 item 13; one unit, Wave 3).
   The chronicle rail is the ambient stream, not the whole system.
6. **SFX pairing table: authored in Wave 3's charter**, reviewed by the
   Director as aesthetic line before wiring.
7. **Wave 1 pre-empts M5 kickoff** — `feature/interface-wave1` starts
   immediately after M4 merges.

*The binding token table stays in code (`babylon/render/tiers.py`) and
its parity guards; DESIGN_BIBLE §9b + this plan are the design canon of
record now that `ai/design-system.yaml` is deleted.*
