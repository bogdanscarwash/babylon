# Wiki Content Architecture — Namespaces, Templates, and the Fact→Flavor Contract

**Date:** 2026-07-27 · **Status: DRAFT FOR DIRECTOR REVIEW — NOTHING HERE IS RATIFIED.**
**The content/ideological line is the Director's alone (Amendment AD, Constitution §IX.5); this
document proposes structure, never doctrine.** Every claim below is either (a) grounded in an
existing file/line, marked **[EXISTS]**, or (b) a proposal with no code behind it yet, marked
**[PROPOSAL]**. Nothing is claimed that isn't one or the other.

**Scope:** Diataxis explanation quadrant — a mental model for "what kind of page is this, who
authored it, and what may touch it," answering the Director's brief: *"some templates or something
to work from in order to generate a wiki structure... look into semantic wiki and Wikipedia...
Babylon Glossary that's a thing we write which no AI touches, but actual game-state files could be
generated with creative flavor while maintaining facts — the flavor is derived from the facts."*

## 1. The headline finding: two-thirds of this already exists

The vault estate (`src/babylon/projection/vault/`) already implements two of the three namespaces
the Director described, under different names. This document proposes naming the third and
formalizing the seam between them — it is much less new work than "generate a wiki structure"
sounds.

| Director's ask | Existing Babylon construct | Status |
|---|---|---|
| "Babylon Glossary... no AI touches" | Concept cards (`vault/concept_cards.py`, `vault/templates/concept.md.j2`) | **[EXISTS]**, unnamed as a namespace |
| "game state files... generated" | County/state/national/org/... dossiers (`vault/render*.py`, `vault/templates/*.md.j2`, the two tick bakers) | **[EXISTS]** |
| "creative flavor... derived from facts" | The narrator-cache skeleton (`vault/narrator_cache.py`) + provider seam (`intelligence/providers.py`) | **[EXISTS as skeleton]** — cache/attribution/async plumbing is built; the fact-derivation *validator* is **[PROPOSAL]** |

## 2. Namespace model (proposal, naming what exists + one new tier)

### 2.1 `Glossary` — hand-written, AI-forbidden, static

**[EXISTS], unnamed.** `vault/concept_cards.py` already ships exactly this: four pages
(`fundamental-theorem`, `survival-calculus`, `metabolic-rift`, `nine-verbs`) as a closed
`CONCEPT_CARDS` registry (lines 259–267), each a frozen `ConceptCard` with `statement`, `formula`,
`terms`, `implementation` (dotted code refs), `citation`, `see_also`. Two properties make it the
Glossary prototype:

- **No `verified_tick`.** The module docstring (lines 34–43) is explicit: "a mathematical
  definition does not go stale the way a committed-tick observation does" — it is compiled once at
  import time, not projected from live state.
- **No narrator fence.** `templates/concept.md.j2` carries only `{statblock}`; there is no
  `{narrative}` block on a concept card, structurally — nothing here is ever AI-authored.

**Enforcement gap — currently NONE, and this is the one piece of actual net-new engineering the
Director's ask requires: [PROPOSAL].** Today "no AI touches the Glossary" holds only because
nothing calls a narrator over `concept.md.j2` output — there is no sentinel that would catch a
future PR wiring one in. A path-based guard (any bake targeting `concept/*.md` may only originate
from `render_concept_card()`, never from `NarratorCache.narrate()`) would be the cheapest version,
mirroring the existing `mise run check:vocabulary` sentinel-per-error-class pattern
(`src/babylon/sentinels/vocabulary/`, cited in this repo's `CLAUDE.md` Gotchas section) rather than
inventing a new enforcement idiom.

### 2.2 `State` — engine-templated facts, deterministic, tick-hash-adjacent

**[EXISTS].** The bulk of the vault: county/state/national/organization/institution/sovereign/
industry/social_class/economy/field_state/community dossiers. Grounding:

- `render.py`'s `_build_environment()` (lines 234–254) is an `ImmutableSandboxedEnvironment` with
  `StrictUndefined`, no custom filters/globals (ADR099: "environment construction is code, never
  data"), loading only from this package's own `templates/` via `PackageLoader` — no
  filesystem-escape surface.
- Every optional field is resolved through a `statblock_rows`/`absent_fields` precompute
  (`render.py` lines 73–134) rather than the template touching an `Optional` field directly —
  Jinja's `StrictUndefined` only fires on a genuinely *absent* name, so this is how the templates
  stay honest about `None` vs a present zero (III.11).
- Determinism is structural, not a convention: `ArchiveTickBaker` (`tick_baker.py`) re-bakes every
  kind every tick (the correctness baseline); `IncrementalArchiveTickBaker`
  (`incremental_baker.py`) re-bakes only dirty entities, but every enumeration and budget clamp
  iterates a *sorted* sequence (module docstring lines 62–64) so the two bakers are byte-identical
  on the same inputs.
- One more sub-case worth naming explicitly: **`epilogues.py`** is *dev-authored conditional
  prose* (the six terminal-outcome texts, `EPILOGUES` dict lines 73–146) that is nonetheless
  `State`, not `Chronicle` — `templates/epilogue.md.j2` says so directly (lines 5–9): "the headline
  and body below are deterministic, dev-authored copy, not narrator-generated prose... none of this
  text is AI-attributed." This is a real, already-shipped instance of "hand-written flavor
  triggered by a fact pattern" that is distinct from both a stat-scaffold page and an AI-narrated
  one — a third texture worth keeping in mind when the Director rules on Chronicle's voice (§6.1).

### 2.3 `Chronicle` (a.k.a. Flavor) — AI-narrated prose, derived from facts, outside the tick hash

**[EXISTS as skeleton — the cache/attribution/async plumbing]. [PROPOSAL — the fact-derivation
validator itself, which does not exist yet].**

`vault/narrator_cache.py` is the closest thing in this repo to the Director's "flavor is derived
from the facts" sentence, already built to the following contract:

- **Path-disjoint from `State`.** Narrative pages live under `narrative/<entity>/` (line 71), never
  under `county/`, `concept/`, etc. — "the byte-identity gates never see it" (line 20). This is
  *exactly* the namespace-as-path enforcement the Glossary needs and doesn't have yet (§2.1); the
  pattern is proven, just not applied to `concept/`.
- **Keyed `(entity, tick, model_pin)`** per Constitution III.6 (`CONSTITUTION.md` line 438:
  "Every persisted AI artifact... MUST carry its model pin... Replayability across model
  deprecation is a constitutional requirement"). A provider swap writes a *new* page; the old pin's
  page is untouched (`narrator_cache.py` lines 9–10).
- **Silence is honest, degradation is loud.** An empty narration writes nothing (`narrate()` line
  313: `if result.text == "": return None`) — the deterministic dossier is fully informative with no
  narrator (III.11/R4). A transport failure writes a *visible* `{absence}` fence naming the error
  (lines 154–157), never a silently-dropped generation.
- **LLM text never meets Jinja** (module docstring lines 22–27): narrative pages are assembled by
  plain string building (`_render_narrative_page`, lines 135–161), with the fence outrun-length
  computed from the prose itself (`_fence_for`, lines 119–132) so hostile model output containing
  backtick runs cannot close the page's own block early. This is the concrete answer to "how does
  untrusted generated text stay safely embedded in a trusted Markdown page."
- **Off the tick path, single-flight.** `NarratorSideProcess` (lines 339–393) is a
  one-worker-thread fire-and-forget scheduler — deliberately one worker so narrative commits to
  the shared dulwich repo serialize with each other and with the tick baker's own commits
  (`git_backend.py`, referenced but not separately cited here).

What is **missing** — the actual net-new work the Director's ask requires beyond naming things:

1. **No fact-derivation validator exists.** `narrate()` (`intelligence/providers.py` lines 406–429)
   takes opaque `(system, prompt)` strings and returns opaque `text`; nothing checks that a number
   or named entity in `text` traces back to the `CountyView`/whichever view-model that prompted it.
   `narrator_cache.py`'s own docstring flags the adjacent gap directly (line 42): "Doctrine-
   conditioning of prompts is DEFERRED past v1... callers supply `(system, prompt)` and this module
   treats them as opaque." **[PROPOSAL]:** a validator function,
   `verify_grounded(view: BaseModel, prose: str) -> GroundingReport`, run *before* `NarratorCache._write`
   commits a healthy entry — walking the view's populated fields (the same walk `render.py`'s
   `_statblock_rows` already does per-kind) and confirming every number/proper-noun surfaced in
   `prose` appears (verbatim or in a declared paraphrase set — e.g. "a third" for 0.33) among the
   view's own field values. A prose block that fails the check is **not** silently dropped —
   consistent with III.11, it should write a *visible* `{absence}`-style flag ("narration rejected —
   ungrounded claim: `<span>`") rather than either fabricating trust or discarding the failure.
   This is the one piece of this whole document that has zero prior art in the codebase; it needs
   the Director's or an ADR's sign-off before anyone builds it (§6.3).
2. **`Persona` is built but not wired here.** `intelligence/ai/persona.py`'s `Persona`/`VoiceConfig`
   (voice/tone/obsessions/directives/restrictions, `render_system_prompt()` lines 150–175) is a
   complete "who is narrating and in what register" model, consumed today by
   `intelligence/ai/prompt_builder.py`/`director.py` — but `narrator_cache.narrate()`'s `system`
   parameter is just a `str`; nothing currently threads a `Persona` into a vault narration call.
   Wiring this is small (a `Persona.render_system_prompt()` call at the narration call site) but is
   a real decision point, not a mechanical one — see §6.1.

## 3. Template layer

**Proposal: extend the existing `.j2` vault estate, do not invent a second templating system.**

The vault already has the primitives Wikipedia/SMW templates provide:

- **Infobox partial [EXISTS, implicit]:** every dossier's `{statblock}` fence (e.g.
  `templates/county.md.j2` lines 25–29) IS the infobox — a fixed-order key/value panel resolved
  from typed fields. It is currently duplicated per template (`county.md.j2`, `concept.md.j2`, and
  every other `templates/*.md.j2` each write their own `{statblock}` loop) rather than factored into
  a shared Jinja macro/partial. **[PROPOSAL]:** a single `_statblock.md.j2` include, parameterized
  on `(rows,)`, would remove that duplication without changing rendered output — pure factoring,
  no behavior change, safe to do without Director sign-off.
- **Lead-paragraph convention [PROPOSAL, no prior art]:** Wikipedia's lead-section rule ("a summary
  of the article's most important contents... before the first heading," WP:LEAD) has no analog
  today — every dossier template goes straight from the H1 to the `{statblock}` fence
  (`county.md.j2` lines 23–25). Adding one deterministic summary line (e.g. "County {fips}:
  population N, sovereign {sovereign_id or 'unclaimed'}, bifurcation {axis}") ahead of the
  statblock is a `State`-namespace, non-AI change — cheap, and arguably improves the page even
  without any Chronicle content. Flagging as a proposal because it changes every baked page's bytes
  (a baseline-ceremony-triggering change per this repo's `CLAUDE.md` §6.5), not because it's
  risky in kind.
- **Categories/footer partial [PROPOSAL]:** nothing today renders a "category" or "see also beyond
  `concept.md.j2`'s own `see_also` loop" footer on non-concept pages. `facets_by_type()`
  (`tui/shell/backlinks.py` lines 27–33) already computes exactly this grouping (page slugs grouped
  by their `type/` prefix) for the Wiki view's use, but it is not rendered *into* the baked page
  itself — it is a client-side facet, not a page-embedded category list. Whether categories belong
  baked into pages (Wikipedia-style, in the git history) or computed live by the client (current
  approach) is an open question (§6.2), not a foregone one.
- **What-links-here [EXISTS]:** `build_backlink_index()` (`tui/shell/backlinks.py` lines 18–24)
  inverts every page's outbound `[[wikilinks]]` (matched via
  `tui/wikilinks.py::WIKILINK_RE`, line 51) into a target→sources map. The ratatui design doc
  explicitly homes this as a real M1 work item on the Rust side too: "`backlinks_json`:
  `GameSession` has no counterpart today — the M1 host task **builds** the vault backlink-index
  read path" (`docs/superpowers/specs/2026-07-26-ratatui-client-design.md` lines 225–226).

## 4. The semantic layer — Semantic MediaWiki and Wikipedia mapped onto what exists

External research grounding (WebSearch, this session): SMW's typed-property model
(`Has type` annotations on a property; semantic-mediawiki.org/wiki/Help:Properties_and_types),
`#ask` query syntax (properties prefixed `?`, semantic-mediawiki.org/wiki/Help:Inline_queries), and
`Special:Browse` ("displays all semantic properties of a page as well as all semantic links that
point to that page," semantic-mediawiki.org/wiki/Help:Browsing_interfaces). Wikipedia grounding:
lead-section convention (`Wikipedia:Manual_of_Style/Lead_section`), infobox as a
"panel that summarizes key facts... top-right... next to the lead" (`Wikipedia:Manual_of_Style/
Infoboxes`), categories as a reader/editor navigation aid
(`Wikipedia:Manual_of_Style/Category_pages`), the namespace system ("30 current namespaces: 14
subject, 14 talk, 2 virtual"; Template namespace = transclusion; Portal = reader/editor bridge;
`Help:Namespaces`), `What links here` (`Template:What_links_here`), protection tiers (full /
semi- / template-protection, `Wikipedia:Protection_policy`), and WP:V ("material must be
attributable to a reliable published source... whether or not it is cited,"
`Wikipedia:Verifiability`).

| SMW / Wikipedia concept | Babylon mapping | Grounding |
|---|---|---|
| Typed property (`Has type: Number`) | A `ProjectionRecord` field (e.g. `CountyView.imperial_rent_phi: SignedLaborHours \| None`) | `view_models.py` lines 141–201; the whole point of the module docstring (lines 1–19): "records... composes fields drawn from multiple subsystems... every field a fog/veil gate can withhold is `Optional` with honest `None` semantics" |
| `#ask` query / `Special:Ask` | **[PROPOSAL, no prior art]** — nothing today lets a page or client ask "every county with `bifurcation_score < -0.5`" | `DeclaredView.fts_columns` (`registry.py` lines 62–64, 80) is the closest existing surface — a declared *subset* of columns exposed to full-text search — but it is lexical search over SQL views, not a typed-property query language |
| `Special:Browse` (all properties + all inbound links for one page) | `peek(view, depth=3)` (full transclusion, every field) + `build_backlink_index()` | `tui/peek.py` docstring lines 17–36 (depth table) + `backlinks.py` lines 18–24 — the two together ARE Special:Browse, just not unified behind one page/command yet |
| Lead section (summary-first) | Not yet present | §3, `[PROPOSAL]` |
| Infobox | `{statblock}` fence + `peek()` at any depth | `render.py` `_statblock_rows`/`sovereign_statblock_rows`; `peek.py` `_FIELD_CAP_BY_DEPTH` (line 96) is literally Wikipedia's "infobox vs. hover-preview vs. full-page" idea made explicit as a size table |
| Categories | `facets_by_type()` (client-side only) | `backlinks.py` lines 27–33; see §3's open question on baked-in vs. computed |
| Template namespace (transclusion) | The `.j2` templates themselves, plus `peek(depth=3)` as literal transclusion of a whole dossier inline | `peek.py` line 35: "page transclusion (embeds the *whole* dossier inline)" |
| Portal namespace | No analog; **[PROPOSAL, not yet scoped]** — a possible home for a hand-curated "start here" page distinct from both Glossary (definitions) and a dossier (one entity) | none |
| Protected pages | The Glossary's *intended* AI-forbidden status | §2.1 — currently a convention, not an enforced protection tier; **[PROPOSAL]** to make it one |
| What-links-here | `build_backlink_index()` | as above |
| WP:V ("traceable to a source, whether or not cited") | Constitution III.8, the Aleksandrov Test: "every formal construct... MUST trace a chain of abstractions back to a material relation" (`CONSTITUTION.md` line 442) + III.11 Loud Failure (line 456) | Babylon's version is *stricter* than WP:V — WP:V permits an uncited-but-verifiable claim; Babylon's honest-`None` discipline (§2.2) means an untraceable field renders as an explicit `{absence}` block, never an omitted-but-true claim |

## 5. Determinism boundaries

The load-bearing rule, stated once so every section above can rely on it rather than re-derive it:
**`Glossary` and `State` pages are pure functions of committed engine state and are part of the
byte-identity gate surface (`qa:vault-regression-ci`, cited in this repo's `CLAUDE.md` under
"Definition of done"); `Chronicle` pages are not, by construction, and must never be made to be.**

This is the same shape as the project's existing fog/epistemic ruling (project memory:
"Fog = EPISTEMIC, engine = MATERIAL — player knowledge never in tick hash; ask who WRITES the
store"): the question to ask of any new wiki page kind is not "does this look like game content"
but "who writes this store, and does a second identical run at the same tick produce the same
bytes." `narrator_cache.py` already answers this correctly in its own docstring (lines 28–32): "the
byte-identity gates run narrator-OFF and never see this subtree." The Glossary needs the same
answer made *structural* rather than incidental (§2.1's enforcement gap) — today it is
byte-identical only because nothing writes to `concept/*.md` except `render_concept_card()`, not
because anything would stop a future caller from doing otherwise.

## 6. Open questions for the Director

### 6.1 Chronicle's voice — is `Persona` the right instrument, and whose call is the register?

`Persona`/`VoiceConfig` (`intelligence/ai/persona.py`) already models tone/obsessions/directives/
restrictions as data, not code — matching this repo's Paradox Pattern
(`/home/user/projects/game/CLAUDE.md` §5: "game logic... defined in TOML/data, not hardcoded"). But
*which* persona narrates the Chronicle, and what its `directives`/`restrictions` say about
ideological framing, is squarely "touches the ideological line" (Constitution §IX.5) — this
document does not propose a persona, only that the wiring point (`narrator_cache.narrate()`'s
opaque `system` string, §2.3 item 2) exists and is small once a persona is chosen.

### 6.2 Do categories/see-also get baked into `State` pages, or stay a client-side facet?

`facets_by_type()` already computes the grouping (§3). Baking it into every page's footer makes the
git-history vault itself browsable outside any client (closer to Wikipedia, where categories are
page content); leaving it client-side keeps `State` pages minimal and avoids a whole-vault
re-bake whenever the faceting logic changes. Both are legitimate; this is a taste/cost call, not a
correctness one.

### 6.3 Is a fact-grounding validator (§2.3 item 1) worth building pre-1.0, and what's its failure mode?

This is the one genuinely new engineering surface in this whole document. Three sub-questions the
Director's ruling should settle before anyone writes code: (a) does an ungrounded Chronicle block
get rejected outright (never committed) or committed-but-flagged (III.11-style visible `{absence}`
annotation alongside the prose); (b) is the check exact-string-match against view-model field
values, or does it need a declared paraphrase vocabulary (so "roughly a third" passes against
`0.33`) — the former is buildable today, the latter needs its own small spec; (c) does this gate
every Chronicle write or only the ones a player will actually see (narration is currently
fire-and-forget and off the tick path — gating it synchronously would reintroduce latency the
`NarratorSideProcess` design (§2.3) exists specifically to avoid).

### 6.4 Does the Glossary ever need a *second* hand-written tier — a Portal-style "start here"?

Flagged in §4's mapping table as unscoped. Not urgent; noted so it isn't lost.

---

**Everything in §§1–5 not explicitly marked `[PROPOSAL]` is a description of code that exists
today, cited to file and line, at the time this document was written (2026-07-27, worktree
`political-superstructure`).** Nothing here authorizes building the `[PROPOSAL]` items; they are
scoped for discussion, not for autonomous implementation, per Amendment AD's escalation rule for
anything touching the ideological line or adding a primitive.
