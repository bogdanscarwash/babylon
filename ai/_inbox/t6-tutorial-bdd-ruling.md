# T6 ruling — the tutorial IS the BDD acceptance suite (BD, 2026-07-21)

> BD (verbatim intent): "in terms of e2e testing, the tutorial itself can be a kind of
> behavior-driven development final set of checks. 'Does the Babylon System behave? Does
> every option that gets clicked work as advertised?' And the BDD itself can be
> educational about how the game works for developers."

Binding on the T6 lane and the T8 DoD. Supersedes the plan's tutorial section where they
differ; everything else there (opening-arc overlay, authored vault pages, doctor
preflight) stands.

## The unification

ONE artifact — a data-driven tutorial step script — consumed three ways:

1. **Player**: the guided opening-arc overlay renders each step's Given/When/Then as
   instruction text over the live campaign (scripted first-session keyed to real state).
2. **CI**: the e2e harness executes the same steps headlessly (Textual Pilot idiom, the
   WO-50 `test_pilot_first_action.py` pattern) — drives the When, hard-asserts the Then.
3. **Developers**: the scenario suite reads as living documentation of how the game
   works. Scenario names are sentences; the suite is the game-loop's behavioral contract
   (the "rewrite test" for the whole client: a rebuilt TUI is correct iff the tutorial
   suite passes against it).

Because the strings the player reads ARE the scenario definitions CI runs,
advertisement and behavior cannot drift apart — a step that stops being true goes red in
CI before a player ever sees the lie. III.11 Loud Failure applied to UX.

## Design constraints (T6 lane spec)

- **Step script = frozen Pydantic data** (`TutorialStep`: id, given, when, then,
  anchor — the page/binding/verb it exercises, completion predicate). No prose
  duplication: overlay text and scenario name render from the same fields.
- **Coverage check, ADR090-style (declared and proved)**: enumerate every player-facing
  option the campaign shell surfaces (BINDINGS, verb plates, palette commands, lobby
  actions); a static check asserts each appears in ≥1 tutorial/BDD scenario or carries a
  cited exemption. An option with no scenario is a seam (∂L boundary node) — red.
- **Harness**: pytest + Textual Pilot (existing idiom). NO new BDD framework/Gherkin
  dependency — Given/When/Then live in the step model; educational value comes from the
  data and generated docs, not from cucumber tooling.
- **Tiering**: the suite runs headless-deterministic (narrator OFF, fixed seed, Wayne
  golden scenario for CI; nationwide steps tiered to the qa lane if runtime demands).
- **T8 DoD amendment**: "tutorial completes" → "tutorial BDD suite green headlessly AND
  completable interactively" + the option-coverage check green.

## Text is the assertion medium (BD addendum, 2026-07-21)

> BD: "because the output is a terminal it means its textual basis (with perhaps some
> spots for simple images like png or svg) and so it can be printed and examined as
> text, much better than visual testing."

The terminal grid IS a text buffer — so behavior asserts on STRINGS, not pixels.
Assertion tiers, strictly ordered:

1. **Semantic text (primary, behavioral)**: Then-clauses assert on rendered terminal
   text (Textual `export_text`-style capture) and on vault markdown — exact strings,
   greppable, printable, reviewable in a PR diff. The golden vault is already this
   (byte-diffed markdown, the ceremony gate).
2. **Structural (secondary)**: Pilot widget-tree queries — focus, screen mode, binding
   dispatch — where a string alone is ambiguous.
3. **Visual SVG snapshots (tertiary, AESTHETIC ONLY)**: the KSBC look. Never a
   behavioral gate; regenerate freely (standing rule: vault manifests are ceremony
   goldens, render SVGs are not).

Image spots (map-room kitty raster, PNG/SVG embeds) stay OUT of the behavioral
contract: ADR099's glyph floor (half-block text mode) is the assertable layer for the
map — every raster lane has a text floor beneath it, and the contract binds the floor.

**Playthrough transcript artifact**: the headless BDD run emits the full session as
text (every screen at every step) — a build artifact that is (a) diffable across
releases, (b) human-reviewable, (c) itself the developer-education document the
scenarios promise. Deterministic under narrator-OFF + fixed seed, so transcript drift
= behavior drift, caught as a text diff.

## Placement

- T6 lane builds: step-script model + the opening-arc overlay consumer + the Pilot
  executor + the option-coverage sentinel + the transcript artifact emitter + authored
  tutorial vault pages (unchanged).
- T4-core's C2 (bindings) and C3 (driver) are upstream anchors — the coverage
  enumeration reads THEIR registries; no T4 rework required.
