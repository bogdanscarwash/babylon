# The World-View Ternary — Unifying the Ideology Estate onto the Ruled Simplex

**Status:** Director-approved design (2026-08-14, live brainstorm — twelve rulings, ledger in §1).
**Tracking:** this design addresses register rows 14 and 19 on issue #564 (see §7); the
2026-08-12 comment rulings discharged rows 9/12; the remaining async rows stand open.
**Provenance:** a five-way archaeology sweep executed 2026-08-14 — the `ai/` corpus (all prior
formulations F1–F16, the binding-ruling register, and contradictions C1–C7), the `docs/`+`specs/`
corpus (sixteen landed formulations, nineteen recorded rejections), the as-built code map (every
quantity, classification, consumer, and sign convention), the MIA + MIM theory mirrors (Marx/Engels,
Lukács, Gramsci, Althusser, Lenin, MIM — extracted with local path citations), and the prior Gramsci
deep dive (`reports/gramsci-althusser-institution-memo-2026-08-11.md`). Anchors below cite the
primary sources, not the digests.
**This is a spec, not a plan.** The implementation plan follows the Director's written-spec review,
per the standing brainstorm→spec→plan workflow.

The Director's framing, verbatim [sic]: *"lets really pause and brainstorm about fascist/liberal/
revolutionary because those are fundamentally the political simplex we're working in so we need to
represent those in an ideologically principled way"* — and, ruling the pivotal fork: *"Ideology is
fundamentally a world view this i why [sic] I had you read Gramsci and Engels and Marx and MIM and all
that."*

---

## 1. The ruling ledger (normative)

The Director made every ruling below on 2026-08-14. "(R)" = accepted the workforce
recommendation; "(D)" = the Director's own direction beyond or against it.

| # | Ruling |
|---|---|
| W1 (R) | **Unify onto the ruled ternary.** Spec 034's (r, l, f) simplex is the one canonical representation; classes, orgs, and factions migrate onto it. The ruled theory stays as-is — nothing in spec 034's frame is re-opened. |
| W2 (D) | **Ideology is a world view, not a meter.** The poles are content, not coordinates; the ternary measures *which world view holds hegemonic leadership over which population*. This ruling redirected the design away from a storage-shape question (measurement vs. memory) to the content-first frame. |
| W3 (R) | **The poles are asymmetric by nature.** Only the revolutionary pole has an articulated content structure (the doctrine tree — the line IS its world view). Liberalism is hegemonic common sense: no tree; the design computes its content per conjuncture from the ruling bloc. Fascism is the capture/degeneration terminus: no tree; coarse parasitism-defense affect plus a demagogy flag. Three poles, three *kinds* of thing. |
| W4 (R) | **Approach A — the World-View Registry.** Declared world-view content + measured alignment + a computed gap readout; all current surfaces migrate to projections/readings of the one representation. (Rejected: B projection-only — leaves the world views without a first-class home and keeps the dual-storage rot; C full gap-machine — kills the shipped accumulator and risks AE(ii) new-formalism.) |
| W5 (R) | **The practice rule: mass work counts.** EDUCATE/PROPAGANDIZE/PROVIDE_SERVICE move alignment because organized mass work alters the conditions of intercourse (ADR087's ruled pattern); narrative with no organization behind it moves nothing. The German Ideology's "fighting phrases with phrases" polemic stands as a hard rule: no mechanic may move minds without moving practice/conditions. |
| W6 (R) | **Orgs carry lines, not points.** An organization never holds a ternary position; it carries a line (doctrine (Major, Minor) + measured practice) and moves populations through mass work/apparatus. Populations have alignments; orgs have lines. (This disposes CCL OQ-3 in its cleanest horn.) |
| W7 (R) | **Charter the per-county seeding data spec.** ADR043's named replacement: ACS correlates + 2010–2020 election results, logit model; the uniform placeholder (0.05, 0.50, 0.45) stays explicit and uniform until that spec lands. |
| W8 (R) | **War of position stands as a forward constraint.** This design records that rupture readiness reads hegemonic-organization depth (Q17 §28's three preconditions); the mechanics land with the doctrine/organization trains, not here. |
| W9 (R) | **One declared faction classification.** The token lists (four homes) and free-text labels collapse to a single content-declared closed enum on faction content, minted by `defenum` ceremony; every predicate (capturability, spoiler poles, fascist vehicle) computes from it. The FAC_DECOLONIAL substring-bug class dies with the lists. |
| W10 (D — scope expansion) | **Deliverable = design + mint + port.** Beyond the workforce recommendation (design + ceremony charters): this train mints the `WorldView` defenum into the Rust closed vocabulary, and the class-surface migration port begins. Recorded as a Director directive; it supersedes, for this workstream only, the Program-29 charter's "no Wave ports beyond design docs" boundary. |
| W11 (R) | **Strike the legacy scalar.** The recording ADR removes the `Ideology` [-1,1] sort from THE_FORMALISM's ⟦LAW⟧ table (contradiction C1's reconciliation); sign conventions live in the flow field (ADR051), never on the simplex state. |
| W12 (D) | **Ideology is a gravitational pull that commands labor vis-à-vis hegemony.** A world view couples to the flow of value *through* hegemony — the pull is what makes a world view causal rather than a label. The three poles are three modes of pull: liberal is the hegemonic pull proper (the ruling bloc's world view commands unorganized labor through common sense — the *why* beneath A-001's unorganized-is-liberal default); revolutionary is the counter-hegemonic pull (W8's war of position reads as accumulating the counter-pull's organizational mass until it commands labor away from capital); fascist is the capture pull keyed to the rent gradient (W3 — labor rallied to defend the extraction position). Recorded as a forward constraint on the port: the pull cashes out as couplings to the *existing* value-flow estate (WAGES/EXPLOITATION/SOLIDARITY edges, the wealth axis, the reserve army) — chartered port-stage design work, minting no new formalism (AE ii). |

## 2. The theory grounding (what any principled representation must honor)

Extracted 2026-08-14 from the local mirrors; every claim carries its text.

1. **Ideology is downstream of material relation, never autonomous.** *The German Ideology*
   (`/media/user/data/old-hdd/old-hdd/www.marxists.org/archive/marx/works/1845/german-ideology/ch01a.htm`):
   consciousness is a social product from the start; the camera obscura — "life is not
   determined by consciousness, but consciousness by life"; the division of material and mental
   labor as the birth of ideology ("the first form of ideologists, priests"). Althusser's gloss
   ("Lenin and Philosophy", the one mirrored Althusser text): philosophy "has no history… everything
   which seems to happen in it really happens outside it." **Design consequence:** the engine
   computes consciousness variables from material state — with lag and inertia, but never free
   drift.
2. **The gap is the object.** Lukács (*History and Class Consciousness*, `archive/lukacs/works/
   history/hcc07_3.htm`): any attempt to give class consciousness "an immediate form of existence"
   lapses "into mythology"; consciousness is a *process* toward totality, never a stationary level.
   Lenin (*What Is To Be Done?*, `archive/lenin/works/1901/witbd/ch02.htm`): spontaneous development
   yields only trade-union consciousness; the revolutionary world view arrives from without.
   **Design consequence:** the engine's ideological object is the *distance between a class's
   material position and its organized awareness*, closing through practice — a computed readout,
   not a stored scalar.
3. **Class ceilings differ structurally.** Lukács: for the bourgeoisie, full consciousness "would
   be tantamount to suicide" (a hard cap, not a probability); the proletariat's position drives
   toward totality. MIM: the labor aristocracy's ideology tracks the imperial-rent gradient —
   pro-liberal in the oppressed nations, "inevitably to fascism" in the declining imperialist ones
   (`/media/user/data/mim/etext/wim/cong/fascismcong2005.html`, passed unanimously 2005).
4. **The triangle is not symmetric.** Lenin: "no third ideology… there can never be a non-class or
   an above-class ideology." MIM (2002 Congress, `wim/cong/fascismdef.html`, glossing Dimitrov):
   bourgeois democracy and fascism are "forms of the class dictatorship of finance or comprador
   capital" — a state-form dial, not coequal alternatives to revolution. Gramsci 1921
   (`archive/gramsci/1921/07/arditi_del_popolo.htm`): fascism is "a spontaneous swarm of reactionary
   energies that coalesce, dissolve and then reassemble" — crisis-organic, not a party badge.
   **Design consequence:** the simplex's base (l–f) is one ruling-class continuum; height above the
   base is the only axis of qualitative change — exactly spec 034's ruled geometry, now with the
   full textual apparatus behind it.
5. **Consciousness changes through altered practice or not at all.** The GI polemic against the
   Young Hegelians ("fighting phrases with phrases… the staunchest conservatives"); Gramsci's war of
   position; Lukács's process. **Design consequence (W5):** propaganda-only mechanics are the
   precise error the tradition names conservative.
6. **MIM's carve of the First World:** no revolutionary mass pole inside the exploiter camp — its
   liberal and fascist wings differ *tactically* over defending parasitism ("both… strains of
   militant parasitism"); revolution lives in the proletarian camp. And: fascist consciousness is
   "not politically conscious in a detailed way" (`wim/cong/thesesonfascism02.html`) — coarse
   parasitism-defense affect plus hypocritical appropriation of Marxist critique as demagogy.
   **Design consequence (W3):** fascism gets no doctrine tree; the design declares its "content" as
   capture plus appropriation, keyed to the rent gradient.

Coverage caveats (so nothing silently substitutes): the German Ideology mirror lacks the verbatim
"ruling ideas" chapter file (the surrounding mechanism passages are present and grounded the
extraction); Lukács's standalone "Class Consciousness" essay is not mirrored (imputation enters
through its operational content); the Prison Notebooks chapters are TOC stubs here — the
load-bearing Gramsci content comes from the prior deep dive (`reports/gramsci-althusser-
institution-memo-2026-08-11.md`, twelve passages read in the Italian); "On Contradiction" is not
mirrored. Any design element needing those exact texts must source them externally with Director
approval.

## 3. The current estate and its diseases

Five coexisting representations, one ruled:

1. **Communities** — the native ternary `(r, l, f)` (spec 034, ruled, current):
   `src/babylon/models/entities/consciousness.py:51-222`, recomputed per tick from org
   contributions + declared substrate floors (`engine/systems/community.py:399-462`).
2. **Classes** — the legacy two-axis `IdeologicalProfile` (cc/ni/agitation), with the bridge
   mapping `r = cc·(1−ni)`, `f = ni·(1−cc)`, `l = max(0, 1−r−f)` explicitly *not* the native
   computation (`src/babylon/models/entities/social_class.py:61-152`;
   `src/babylon/projection/aggregation.py:50-98`).
3. **Orgs** — a single `ConsciousnessTendency` enum, LIBERAL-defaulted
   (`src/babylon/models/entities/organization.py:172-175`).
4. **Factions** — free-text strings + duplicated token lists
   (`engine/systems/reactionary.py:71`, `engine/systems/allegiance.py:77`,
   `engine/systems/electoral.py:139-144,159-164`, `domain/organizations/migration.py:32-37`).
5. **Doctrine** — the org line as (Major, Minor) opposition (ruled 2026-07-29, chartered for the
   Rust port).

The disease register (each entry verified against the tree 2026-08-14; "killed by" refers to §4):

- **The absent-data 0.5 defaults** — `national_identity: 0.5` in ≥10 sites
  (`engine/systems/ideology.py:74,82,89`; `engine/systems/struggle.py:100,107,128,137,139,164,171`;
  `engine/systems/solidarity.py:66,73`; model default `social_class.py:99`; placeholder
  `BASELINE_IDEOLOGY` at `engine/headless_runner/bridge.py:93-100`). A class with no ideology
  record reads as half-fascist. **Killed by:** UNPOSITIONED (absence over fabrication, ADR070's
  L-ABS law applied to the unified surface).
- **The duplicated token classifications** — four homes, substring-matched; the FAC_DECOLONIAL bug
  class (`"settler" ⊂ "anti-settler abolitionism"`, `seed_factions.json:33`; `min()` id-ordering
  then *selects* the decolonial front as the fascist faction). **Killed by:** W9's one declared
  enum — row 9's 2026-08-12 ruling ("repair at the port, declared property") lands here.
- **The sign tangle** — the manifesto's Ψ (−1 fascist / +1 communist) sign-flipped against every
  code convention; the `national` opposition balance carrying + = internationalism (opposite
  valence to the legacy scalar); `to_legacy_ideology = 1 − 2·cc` reading an unconscious
  internationalist class as *reactionary* (`social_class.py:142-152`). **Killed by:** no signs on
  the simplex (ADR051: the asymmetry lives in the directed flow field) + W11's LAW-table strike.
- **The scattered liberal tie-breaks** — argmax ties → LIBERAL (`consciousness.py:183-192`),
  simplex rest → liberal, bridge corner (cc=1 ∧ ni=1) → liberal
  (`projection/aggregation.py:69-73`). Deliberate theory (spec 034 A-001: unorganized = liberal
  hegemony) but smeared across five sites. **Killed by:** one declared hegemonic-default rule in
  one home.
- **C5's stored contestation entropy** — H/log3 ruled diagnostic-only (ADR051/ADR054) yet stored on
  community hyperedges. **Handled by:** the limitation recorded in the reference doc; the field is
  not ported.
- **C1's deprecated-but-law scalar** — `Ideology` [-1,1] replaced in every data spec since Sprint
  3.4.3, still listed in THE_FORMALISM's ⟦LAW⟧ table (`ai/THE_FORMALISM.md:213`) and both Haskell
  drafts. **Killed by:** W11.

## 4. The design

### 4.1 The `WorldView` content kind

A content-declared closed enum, minted into the Rust closed vocabulary by `defenum` ceremony
(the second consumer of ADR195's enum deffield row, after ADR196's OrgKind):

```
REVOLUTIONARY / LIBERAL / FASCIST
```

The three members are the simplex's vertices as *declared content*, with asymmetric payloads
(W2/W3):

- **REVOLUTIONARY** — the articulated pole. Its content home is the doctrine tree (landed:
  `src/babylon/data/game/doctrine_tree_mvp.json`; graph-native per Q17's 2026-08-11 ruling). The
  line IS the world view; org lines are the ruled (Major, Minor) pairs. The patsoc→Strasser
  degeneration pipeline stays where it already lives: the tree's trap paths — fascism is where
  degeneration *arrives*, not a tree of its own.
- **LIBERAL** — hegemonic common sense. No tree. The design computes its content per conjuncture
  from the ruling bloc (the GI universalization mechanic: whoever rules presents their interest as
  the general interest). Declared instead: its carrier apparatus kinds (school, press, NGO, church,
  party machinery — the Gramsci/ISA estate, per the four-channel institutionality ruling of
  2026-08-11) and its marker behaviors as observer/narrative content (MIM's markers:
  moral-universalist both-sidesism, NGO/UN faith, lifestyle fixes — `faq/howtotellliberal.html`).
- **FASCIST** — the capture pathology. No tree. Declared as a degeneration *terminus*:
  parasitism-defense affect keyed to the imperial-rent gradient (mass base: the declining labor
  aristocracy — MIM 2005), plus a demagogy flag (appropriated Marxist critique — content-free by
  design). The simplex geometry itself expresses the state-form relationship to liberalism (one
  class dictatorship, two management programs): they share the base; only revolution is off it.

Every declared value carries provenance (ADR043 discipline) and is Director-reserved content (§5).

### 4.2 The measured ternary

The ternary `(r, l, f)`, `r+l+f=1`, is the *measured alignment* of a population to the three world
views — the pattern the community surface already implements, extended to classes at migration:

- **Computed, never assigned** (R-MEASURED, ADR070): per-tick from the organizational/solidarity
  landscape + material anchors + declared floors where ruled (ADR171's national incidence stays
  DECLARED from history — the one sanctioned authored input).
- **UNPOSITIONED on absence** (L-ABS): no data, no reading — never a default. This kills the
  entire 0.5-default disease class by law, not by vigilance.
- **Shares only, never signs** (ADR202 R7 consonant): the ternary carries magnitudes; *direction*
  lives in the flow field and the solidarity topology (ADR016 untouched).
- **The gap readout** — first-class but computed, never stored: per class, the distance between
  the position-side measure (the E/P/S partition on the ADR194-R1 quantile carrier, ADR202 R7) and
  the current alignment. The gap closes only through altered practice (W5). This makes Lukács's
  imputed-vs-empirical readable without minting a stored gap-dynamic (which is what killed
  approach C).

### 4.3 Dynamics — preserved laws and the practice rule

Preserved verbatim from the ruled estate: the **f→r ε-gate** (proletarianization ∧ adjacent-r ∧
solidarity; detailed balance broken deliberately, spec 071 FR-017); the **r→f capacity transfer**;
**unorganized = liberal hegemonic default** (spec 034 A-001 — now one declared rule in one home,
§3); the **hegemonic-community semantic inversion** (on SETTLER/PATRIARCHAL communities `r` is the
conscious defense of the extraction position — same math, inverted reading; this spec discharges
the missing doc page, §8). Memory lives in material stores (repression, conditions), never in the
ideology state itself. The design records the rupture gate as a forward constraint (W8).

It likewise records the **gravitational coupling** as a second forward constraint (W12): a world
view commands labor through hegemony, so the measured shares must read as pulls on the flow of
value — liberal hegemonic (the default orbit of unorganized labor), revolutionary
counter-hegemonic (the war-of-position mass, W8), fascist capture (rent-keyed, W3). The coupling
targets the *existing* value-flow estate — WAGES/EXPLOITATION/SOLIDARITY edges, the wealth axis,
the reserve army — and the port stage charters the coupling's mechanics as design work. This spec mints no
pull-dynamic formalism (AE ii); it rules only that the ternary is causally coupled to value flow,
never an inert label.

### 4.4 Orgs and factions

- **Orgs carry lines, not points** (W6). `ConsciousnessTendency` on orgs dies as state; the org's
  doctrine line + measured practice carry its ideological character. Its effect on populations
  flows through mass work and apparatus — the community surface's OrgContribution channel already
  implements the read side.
- **Factions** get the one declared classification (W9): a content-declared closed enum field on
  faction content; capturability (row 9's repair), spoiler poles, and the fascist-vehicle predicate
  all compute from it. Seed strings map onto it exactly once, in the ceremony's content diff.
- **Stance tables** (`_STANCE_TO_POLICY`, `_STANCE_CHAUVINISM_SCORE`) stay unchanged here —
  register rows 11/15 ride their async memos.

### 4.5 The Rust landing (per W10's mint + port scope)

1. **The mint:** `WorldView` defenum via the ADR195/196 ceremony machinery — one vocabulary
   ceremony PR with its own ADR, the mint-and-retire record per ADR176 (34)/ADR187 OQ-7. The
   faction-classification enum (W9) joins the same ceremony as a second defenum if the Director
   rules its member list content-complete; otherwise it charters separately. The ceremony needs no new deffield
   row — the enum row exists (ADR195); alignment shares ride the existing probability lane.
2. **The port begins:** the class-surface migration — measured-ternary read path + UNPOSITIONED
   first (ConsciousnessSystem's port target), practice-coupled updates after. The frozen Python
   engine stays reference-only; every behavioral divergence from the frozen engine earns a
   D-record; goldens move only by declared §6.5 ceremony.
3. **The C1 strike (W11):** the recording ADR's train strikes `Ideology` [-1,1] from
   THE_FORMALISM's ⟦LAW⟧ table; the Haskell drafts stay as historical artifacts (immutable
   history).

### 4.6 Seeding (W7)

A chartered follow-on spec: per-county (r, l, f) seeding from ACS correlates + 2010–2020 election
results via a logit model — ADR043's named replacement, pending since 2026-07. The uniform
placeholder stays explicit and uniform until it lands (ADR043's own discipline). The seeding spec
also houses the SYNTHETIC community-defaults calibration (spec 034 SC-007).

## 5. The reserved ledger

**Director-only (never workforce):** the pole names and content text; floor values; taxonomy
membership of every declared enum; ADR171 OQ8 (SETTLER as an on-screen name); the five canonical
outcomes; the RED_OGV two-expansions contradiction (parked for the endgame train — two name
expansions and two mechanics currently ride one enum value:
`openwiki/glossary/core-concepts.md:122-129` vs `specs/070-balkanization/spec.md:143`);
any ternary→scalar collapse (R-TERNARY-LINE).

**Workforce-delegable:** storage mechanics, ceremony mechanics, migration sequencing, test design,
the seeding spec's statistical machinery (measured constants set scale, never shape — R-SHAPE).

## 6. Staging

1. **This spec** + the session's rulings ADR (ADR204) — one docs PR.
2. **The vocabulary ceremony PR** — the `WorldView` mint (+ faction enum if content-complete).
3. **The class-surface migration port** — staged per §4.5.2, with its own train charter.
4. **The seeding spec** — chartered per §4.6.

Each stage is its own PR, its own CI, its own Copilot harvest, `mise run pr:merge` only.

## 7. Register-row dispositions

- **Row 14** (token lists): **answered by W9** — one declared classification, one home.
- **Row 19** (`national_identity: 0.5` defaults): **answered by §4.2's UNPOSITIONED law** — the
  value question the memo reserved ("0.5 = halfway to the fascist pole") is moot when absence has
  no reading at all.
- **Row 22** (Slice-4 residue): untouched — rides T3's landing.
- **Rows 10, 11, 13, 15, 16, 17, 18, 20, 21, 23, 24:** unaffected; their async memos await the
  Director in `reports/register-memos/`.

## 8. Open questions this design leaves

- The seeding spec's calibration targets (W7's follow-on).
- The hegemonic-community semantic inversion doc page — spec 034 ordered it documented; this
  design's reference-docs stage discharges it.
- CCL's remaining open questions (OQ-1, OQ-2, OQ-4, OQ-5, OQ-6): this spec records them as
  **superseded by non-adoption** — the CCL never reached ratification and W4's approach-A ruling
  holds; its storage discipline (P + Q) stays on file should a future amendment revisit stored
  ideological memory.
- Whether `assimilation_ratio`/`ideological_contestation` (community-side derived diagnostics)
  survive the port as observer-only reads — decided at the port packet, not here.
