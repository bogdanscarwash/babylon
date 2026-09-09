# Amendment AH — Declared Event Schemas (`defevent`)

- **Date**: 2026-08-18
- **Status**: **DRAFT — DIRECTOR GATE.** Not ratified. This text is the R4.6
  deliverable of the BSL refactor program (`docs/superpowers/plans/
  2026-08-18-bsl-refactor-program.md` §14, worktree `wt-hygiene`): the
  Director approved the *amendment path* for `defevent` at the 2026-08-18
  popup sitting, over a deferred-to-WS1 alternative, and gated drafting on
  R4.1's event-schema registry landing (`dev @ 126fe9402`, this worktree's
  HEAD). Drafting is workforce work; **ratification is a future Director
  sitting**, not this document.
- **Version impact**: **MINOR** — a new additive `<top-form>` and its
  load-time checks; no principle redefined or removed, no verb/intrinsic/
  severity-rule/constructor-family minted (§3 below). Precedent: Amendment
  AG, also MINOR, for the same reason.
- **Letter**: **AH** — the next free letter after AG (ratified v3.2.0,
  2026-08-10; confirmed no `Amendment AH` text exists anywhere in the tree
  as of this draft). Letters **AH (this draft) and AI (the prior-tick
  draft) controller-reserved 2026-08-18**; re-verify at the ratifying
  sitting.
- **Amends**: NORTH_STAR §0's closed-formalism-surface sentence — checked
  directly (`NORTH_STAR.md:12-27`), that live parenthetical today names
  **only** Amendment AE/BSL; `Amendment AG` is not mentioned anywhere in
  `NORTH_STAR.md` despite `CONSTITUTION.md`'s own AG entry stating it
  "re-opens NORTH_STAR §0's formalism closure for exactly two constructs"
  — a pre-existing doc-drift gap between the constitutional text and the
  explanation document, not introduced by this draft. Against the text as
  it actually reads today, `defevent`/AH would be the **second** named
  construct in that parenthetical, not the third; landing AH's §0
  consequence is a reasonable moment to also backfill AG's still-missing
  mention, though that backfill is not this amendment's own obligation.
  Also amends: Article IX.2's amendment registry (new entry, inserted
  after Amendment AG).
- **Adds**: no new Article I–VIII principle text. `bsl-language.rst` gains
  one new normative section (a sibling of §2.13's `defenum`/`defvocabulary`)
  once ratified and specified; this draft does not write that section —
  see §7.
- **Supersedes**: nothing.
- **Authorizes**: the Rust/BSL implementation of `defevent` (grammar
  production, loader, the load-time refusal checks named in §5) as a
  future, separately-scoped, cargo-gated task — **not built by this draft
  and not built by R4.6**, which is documentation-only per its own brief.
- **Source**: R4.1's event-schema registry (`docs/reference/
  event-schema-registry.toml`, landed `dev @ 126fe9402efa28da604cda1d084c
  bd6c166009bc2`, this worktree); the BSL refactor program charter §6
  ("Phase 4 — event-schema registry as data") and §14 ("DIRECTOR RULINGS")
  (`docs/superpowers/plans/2026-08-18-bsl-refactor-program.md`, read from
  `wt-hygiene`); the D101 precedent for minting a declared construct
  (`docs/reference/bsl-language.rst` §2.13, `ADR195_enum_deffield_row.yaml`).
  **Recording ADR**: the next free number as of this draft is above
  ADR215 (the highest file in `ai/decisions/` today) — re-checked at
  ratification time per this repo's standing convention (ADR195's own
  practice); the ratifying commit assigns it.

---

## 0. The problem (IX.3.3 required element)

`emit` (BSL's event-effect verb, `bsl-language.rst` §2.8: `"(" "emit"
<enum-ref> <payload-item>* ")"`) checks its `EventType` operand against the
closed graph vocabulary today, and nothing else. Verified fresh against
`rust/crates/babylon-bsl/src/` on this branch:

- `grammar.rs:208` registers `("emit", 1, EnumKind::EventType)` — the
  `<enum-ref>` operand is checked to be a legal `EventType` member, full
  stop.
- `bound_checker.rs:577-591` reads `<payload-item>` forms only for fuel
  cost and to confirm each key `<symbol>` is static — it does not look at
  what the key names ARE or what shape the value expressions must take.
- `typecheck.rs` has **zero** lines that mention `emit`'s payload; the
  file's only "emit"-adjacent text is an unrelated docstring for
  `E-TYPE-016`/`E-TYPE-017`.

So an `emit` site's payload — which keys it carries, whether a key is
required, what kind of value belongs under it — is unchecked by the
grammar in every direction. `control-ratio.bsl`'s own `CONTROL_RATIO_CRISIS`
emits are the standing example the registry documents (`event-schema-
registry.toml` lines 123-143): the `enforcer-population > 0` branch emits 8
keys, the `= 0` branch emits 6 (both branches D-recorded, both legal,
neither a defect) — a real two-shape payload no consumer written against
one branch can see coming from the grammar.

R4.1 closed the DATA half of this gap: `docs/reference/
event-schema-registry.toml` now states, per `EventType` member, tiered by
evidence class, what a real emit site or a Python builder actually
carries. R4.2 (queued, not built by this draft) closes the ADVISORY half:
a `bsl-lint` check comparing an emit site's keys against the registry,
warning on drift — repo-relationship-shaped, per the Director's
2026-08-18 check-placement boundary ruling (charter, same session):
*"repo-relationship invariants (citations, numbering, cross-file
duplication) → `bsl-lint`."*

What neither R4.1 nor R4.2 can do, by that same boundary ruling, is make
an undeclared or malformed payload **refuse to load**. The ruling's other
half is explicit: *"semantic coherence of a content set → IN-LANGUAGE
load/type refusal (new E-LOAD/E-TYPE codes, amendment-free loader
hardening…)"* — but the charter's own Phase 4 section immediately notes
the boundary this clause does NOT clear: *"moving `emit` payload
enforcement INTO the grammar (a `defevent`-style declared construct)
crosses into amendment-class per the D101 precedent."* D101
(`bsl-language.rst` §2.13, ADR195) is the standing precedent for exactly
this shape of question — minting a NEW `<top-form>` declaration and a new
governing law for what it checks is not "amendment-free loader
hardening" of an existing refusal; it is a new declared construct, and
the Director ruled (§14, 2026-08-18) that this one specifically goes
through the amendment path rather than a workforce-decided D-row.

## 1. Proposed constitutional text (IX.3.3 required element)

*Registry entry, to be inserted in IX.2 after Amendment AG, in the house
voice:*

> **Amendment AH — Declared Event Schemas (`defevent`)** (ratified vX.Y.Z):
> re-opens NORTH_STAR §0's formalism closure for **exactly one** additive
> construct, `defevent` — a BSL declaration form attaching a payload
> schema to an existing, closed-vocabulary `EventType` member. Operative
> clauses: **(i) what it declares** — a `defevent` form names one
> `EventType` member (checked against the existing closed graph
> vocabulary, §3.6/D101; `defevent` mints no new `EventType` member and no
> new closed-vocabulary kind) and a set of payload keys, each flagged
> required or optional; it does not itself declare a value KIND per key
> (§2 below scopes this deliberately to what the R4.1 registry's own
> evidence actually carries — name and required-ness, not a per-key
> type). **(ii) what activates checking** — a `defevent` for `EventType/X`
> makes every `(emit EventType/X …)` site reachable from its declaration
> scope (a bare top-form or, joining `defenum`/`defvocabulary`/`defconst`/
> `deffield`, a shared D157 prelude) subject to load-time payload
> checking; an `EventType` with no `defevent` anywhere in scope is
> unchecked, exactly as today — the same backward-compatible, per-kind
> activation `defvocabulary` already established (D101). **(iii) what
> refuses** — a duplicate `defevent` for the same `EventType` (`E-LOAD-001`,
> the same code every duplicate declaration shares); an `emit` naming an
> `EventType` under the closed-world rollout obligation of clause (iv)
> with no matching `defevent` anywhere reachable; and a payload whose key
> set does not match its `defevent`'s declared required/optional keys
> (missing required, or a key the declaration does not name at all).
> **(iv) the rollout obligation** — ratifying this amendment changes no
> load behavior by itself; the implementing PR that turns on payload
> checking for a cohort of `EventType`s MUST land `defevent` declarations
> for that same cohort in the same change, so no existing content
> regresses. The event-schema registry's three evidence tiers
> (`docs/reference/event-schema-registry.toml`) are this obligation's
> named schedule: Tier 1 (12 `verified-bsl` members) are the first
> candidates, since real citations already back their key sets; Tier 2
> (68 `verified-python-builder` members) migrate when their emitters port
> from `EVENT_BUILDERS` to BSL at the engine-freeze cutover; Tier 3 (20
> `no-known-emitter` members) stay undeclared — the honesty ledger — until
> a first emitter exists for one. **(v) the closure re-seals** — this
> amendment adds no verb, no intrinsic, no severity rule, no constructor
> family, no adjunction, no level lattice; `defevent` expresses a payload
> shape for an effect the grammar already has (`emit`), exactly as
> `deffield` expresses a value shape for a field the grammar already has;
> everything else under NORTH_STAR §0 stays closed. Windows-impact note
> (AA duty): none — a content-declaration and loader change only. Source:
> Director ruling, BSL refactor program §14 (2026-08-18, popup sitting);
> D101 precedent (`bsl-language.rst` §2.13, ADR195); event-schema registry
> evidence base, R4.1 (ADR<TBD>). Director-ratified <DATE>.

## 2. Clause-by-clause rationale

**(i) Closed-kind discipline.** `defevent`'s `<enum-ref>` operand is
checked against the SAME closed `EventType` vocabulary `emit` already
checks against (`grammar.rs:208`, `EnumKind::EventType`) — it cannot name
a member outside it, and it cannot mint a new one. This mirrors
`defvocabulary`'s own restriction to the four existing structural kinds
(`bsl-language.rst` §2.13: *"needs no dedicated grammar production: the
restriction is a load-time check… not a lexical one"*) and answers the
"kinds closed" instruction directly: Amendment AG's clause (ii) already
states the house rule this amendment follows — content may declare
INSTANCES against an existing closed schema, never mint a new KIND.
`defevent` mints instances (per-EventType schemas); `EventType` the kind
stays exactly as closed as it is today.

**(ii) No per-key value kind in v1 — not for lack of evidence, but for
lack of a settled translation.** *Corrected in fix round 1 (2026-08-18):
this clause originally claimed no per-key kind evidence exists at all.
That was false, and is corrected below.* The R4.1 registry —
`RegistryKey` in `src/babylon/sentinels/event_schema_registry/
registry.py` — carries `name`, `required`, `source`, `note` per key, no
value type. But real per-field kind evidence DOES exist, for every one
of the 80 `EventType` members `EVENT_BUILDERS` covers (the 12 Tier 1 +
68 Tier 2 rows — every Tier 1 `EventType` also has an `EVENT_BUILDERS`
entry, confirmed directly by name for all twelve): each builder
constructs a typed Pydantic model, and those models declare real
per-field types plus constraints. Two spot-read in full: the
`CONTROL_RATIO_CRISIS` builder (`event_builders.py:226`) constructs
`ControlRatioCrisisEvent`, whose model
(`src/babylon/models/events/_legacy.py:387`, fields at `:416`, `:421`,
`:426`) declares `prisoner_population: int = Field(..., ge=0, …)`,
`enforcer_population: int = Field(..., ge=0, …)`,
`control_ratio: float = Field(..., ge=0.0, …)`; the `ENTITY_DEATH`
builder (`event_builders.py:536`) constructs `EntityDeathEvent`, whose
model (`src/babylon/models/events/spine_payloads.py:48`, fields at
`:56-61`) declares `entity_id: str`, `wealth: float`,
`consumption_needs: float`, `s_bio: float`, `s_class: float`,
`cause: str`. This is real, concrete evidence — not something a future
`defevent`-kind extension would have to invent, the way this clause
originally implied.

What it is NOT, yet, is BSL-kinded evidence, and that gap — not an
absence of evidence — is why `defevent` v1 still scopes to key NAME +
required/optional only. Two concrete obstacles a mechanical
Python-type-to-BSL-kind transcription would hit immediately: **(a)
unit-interval disambiguation** — a Python `float` field constrained
`ge=0.0`/`le=1.0` (the shape several of these Pydantic fields carry) is
consistent with any of BSL's THREE unit-interval kinds
(`probability`/`intensity`/`coefficient`, §3.1) and the Pydantic
constraint alone cannot say which; today that choice is made by the
*target field's own* `deffield :kind` declaration, a fact external to
the event payload itself. **(b) string fields are structurally
inadmissible** — `EntityDeathEvent.cause`/`entity_id` above are Python
`str`, and `bsl-language.rst` §2.8's own existing rule (`:2989`, "No
string payloads on `emit`" — `Str` has no operations, `<expr>` has no
string literal, one in a payload is `E-PARSE-010`) already refuses a
string value in ANY payload position; that same passage states the
required treatment is to "convert them to enum-refs or drop them," an
authoring judgment call per field, not a mechanical type copy. A
per-key `defevent` kind is therefore a real, evidence-backed FUTURE
extension — the 80 Pydantic models are exactly the source a follow-on
task would transcribe from, the same role R4.1 already played for key
*names* — but transcribing it needs the same kind of deliberate,
per-field authoring pass R4.1 did, not a script over `EVENT_BUILDERS`'
type annotations. This amendment therefore still scopes `defevent` to
key NAME + required/optional only in v1; per-key value-kind checking is
a genuinely separate, later extension (its own amendment, since it
would mint a NEW checked invariant class, not extend this one) — named
here, with its real evidence base now on record, so the boundary is not
silently redrawn by a future implementer and the deferral is not
mistaken for a claim that no such evidence exists.

**(iii) Per-EventType activation, prelude-eligible.** `defevent` follows
`defvocabulary`'s exact activation shape (D101): declaring nothing leaves
behavior unchanged, so an `EventType` this amendment never gets a
`defevent` for behaves exactly as it does today, forever, unless someone
declares one. Making it prelude-eligible (D157: `defenum`/`defvocabulary`/
`defconst`/`deffield`) lets the whole content estate share one canonical
schema set per `EventType` rather than fifteen content files each
re-declaring `CONTROL_RATIO_CRISIS`'s eight keys — the same motivation
D157 names for the four forms it already covers.

**(iv) The registry is the migration inventory, not just evidence.** This
is the clause that answers "how does `defevent` relate to the three
tiers" directly, and it is the one place this draft makes a **closed-
world** claim (an `EventType` a `defevent`-checking implementation deems
in-scope, but for which no schema exists anywhere reachable, is a load
refusal — "emit of an undeclared event"). A closed-world rule is only
honest if it is never turned on for a cohort of `EventType`s that has
no matching `defevent` yet, which is why clause (iv) makes landing the
declarations a same-change obligation, not a separate future cleanup.
The tiers give that obligation a concrete, evidence-backed order:

| Tier | Count | Registry file rows | What "first candidate" / "migrate at freeze" / "honesty ledger" means operationally |
|---|---|---|---|
| 1 — `verified-bsl` | 12 | `[[tier1]]`, lines 40-185 | Real `(emit …)` citations exist today (15 sites, `content/rules/*.bsl`); their key sets are already measured, D-recorded where two-shape (`CONTROL_RATIO_CRISIS`), and ready to transcribe into `defevent` forms with no invention. These are the first `defevent`s an implementing PR should write, since only they can turn on checking without landing new content first. |
| 2 — `verified-python-builder` | 68 | `[[tier2]]`, lines 211-815 | No BSL emit site exists yet — these events are still emitted from Python's `EVENT_BUILDERS` (`src/babylon/engine/event_builders.py`). The closed-world rule is VACUOUS for them today (nothing in BSL emits them to refuse), so declaring their `defevent`s is naturally sequenced with each event's own port to BSL at the Amendment AE engine-freeze cutover, transcribing the Tier 2 row (itself already flagged as inheriting `EVENT_BUILDERS`' own incompleteness — see the registry's own tier-2 note) rather than re-deriving a schema from nothing. |
| 3 — `no-known-emitter` | 20 | `[[tier3]]`, lines 825-883 | No builder, no BSL site — nothing anywhere emits these today. The closed-world rule stays vacuous for the same reason as Tier 2, but with no scheduled trigger: these stay undeclared, the honesty ledger `sentinels/fallback_coverage`'s `BUS_BOUNDARY_LEDGER` already ledgers independently (registry header comment, confirmed cross-check in the landed test suite), until a first emitter appears for one. |

`unminted_bsl_only`'s one row (`ORGANIZATION_SEEDED`, `organization.bsl:31`)
is deliberately outside this table — it is not an `EventType` member at
all (the registry's own header: *"folding it into Tier 1 would falsely
imply the three tiers exhaustively partition the 100-member Python
universe"*), so `defevent EventType/ORGANIZATION_SEEDED` is not
expressible: the `<enum-ref>` operand fails the same closed-vocabulary
check clause (i) states, the same way any other unregistered `EventType`
name would.

**(v) The closure re-seal.** Word-for-word the same discipline AE clause
(ii) and AG clause (iii) already state for their own additive constructs:
no new generator, constructor family, adjunction, level lattice, or
severity rule. `defevent` is schema-on-an-existing-effect, the same job
`deffield` already does for schema-on-an-existing-node/edge-attribute —
new DATA about the shape of something the grammar can already do, never
new mathematics.

## 3. What this does NOT mint

- **No new `EventType` member.** Clause (i); the closed vocabulary is
  unchanged.
- **No new mathematics.** Clause (v); no generator/constructor
  family/adjunction/level lattice/severity rule — the NORTH_STAR §0 test
  every additive re-opening since AE has had to clear.
- **No per-key value-kind checking.** Clause (ii), deliberately deferred
  — real per-field kind evidence DOES exist (the 80 `EVENT_BUILDERS`
  Pydantic models, Tier 1 + Tier 2), but it is Python-typed, not
  BSL-kinded, and translating it needs a real per-field authoring pass
  (unit-interval disambiguation; string fields converted to enum-refs or
  dropped per §2.8's existing rule) rather than a mechanical copy;
  building that translation would be a second, separately-amendable
  checked-invariant class layered on top of this one, not part of it.
- **No renaming of any BSL content payload key.** The registry this
  amendment's evidence base rests on already commits to this (port-AS-IS,
  ADR183, restated in the registry's own header comment); a `defevent`
  transcribing a Tier 1 row inherits that discipline — it transcribes
  content's own kebab-case spelling, never a builder's snake_case
  substitute (the registry's `normalize_key()` function documents the
  one narrow `_`/`-` comparison rule this never widens into a rename).

## 4. What refuses at load — the three classes, and the one open question

1. **Duplicate `defevent`.** Two `defevent` forms (bare, or one bare and
   one via a shared D157 prelude) naming the same `EventType`. Refuses
   `E-LOAD-001`, following `defvocabulary`'s own precedent exactly
   (`bsl-language.rst` §2.13: *"A duplicate `defenum` type name, or two
   `defvocabulary` forms naming the same `<enum-kind>`, is `E-LOAD-001`
   like any other duplicate declaration"*) — and, per D157, `defevent`
   does NOT get `defenum`'s identical-recognition special case: a
   re-declaration refuses even if byte-for-byte identical to the first,
   the same rule `deffield`/`defconst`/`defvocabulary` already carry.

2. **Payload-key mismatch.** An `emit EventType/X` site whose payload
   omits a key `defevent EventType/X` declares required, or supplies a
   key `defevent EventType/X` never declared at all. This is the SAME
   check R4.2's `bsl-lint` pass already performs advisory-only against
   the whole registry (Tier 1 and Tier 2 rows) — this amendment's
   consequence, once implemented, is that for an `EventType` with a
   ratified `defevent`, the identical comparison becomes a load refusal
   rather than a lint warning; `bsl-lint`'s check remains the advisory
   layer for every `EventType` that has not (yet, or ever) received a
   `defevent`.

3. **Emit of an undeclared event, under the closed-world rollout
   obligation (§2 clause iv).** This is the one place this draft is
   naming a design decision rather than transcribing an uncontested
   precedent, and it is flagged as such for the ratifying sitting:
   whether "no matching `defevent`" refuses **only** for the cohort an
   implementing PR explicitly brings under checking (this draft's
   proposed reading — §1 clause (ii)/(iv)), or refuses globally the
   moment `defevent` exists as a construct (which clause (iv)'s own
   Tier-2/Tier-3 vacuity argument shows would be harmless in practice,
   since neither tier has a BSL emit site to trip it today, but is a
   stricter, less reversible rule to ratify). The registry's own tiering
   was built exactly to let this decision be made with real numbers
   rather than a guess (12 sites needing schemas now, 68 needing them
   later, 20 needing none yet) — the ratifying sitting has that evidence
   in hand either way.

Exact `E-LOAD` numbers for classes 2 and 3 are **not** assigned by this
draft, following ADR195's own stated practice for D101's consequence
codes (*"grep-verified next-free at execution time"*) — they are owed by
the implementing PR, contiguous with whatever `E-LOAD` sequence is open
when that PR lands.

## 5. Principles affected, and how their text moves (IX.3.3 required element)

- **NORTH_STAR §0** — checked directly (`NORTH_STAR.md:12-27`): the live
  closed-formalism-surface parenthetical today names only BSL (AE); AG's
  own "exactly two constructs" reopening is not threaded into
  `NORTH_STAR.md` at all (pre-existing drift, not this draft's doing —
  see the header "Amends" bullet). Against the text as it stands,
  `defevent`/AH would be the **second** named additive construct in that
  parenthetical, not the third.
- **Article IX.2** — one new registry entry (§1 above), inserted after
  Amendment AG.
- **No Article I–VIII principle is redefined.** `bsl-language.rst` is a
  specification document, not constitutional text; its future normative
  section for `defevent` is a **consequence** of ratification (§7's
  Task R4.6 sibling, not yet scheduled), the same relationship D101's
  ADR195 has to §2.13's actual prose.

## 6. Draft invariance proof (IX.3.3 required element)

**The pre-existing `emit` grammar is unchanged and strictly more
constrained only where a `defevent` opts an `EventType` in.** Every
content set that declares zero `defevent` forms — every content set that
exists today — parses, typechecks, and evaluates identically to before
this amendment: `defevent` is a new `<top-form>` alternative no existing
file uses, and clause (ii)'s per-`EventType` activation means an
`EventType` with no `defevent` anywhere in scope is checked exactly as
today (nowhere). This is the same invariance argument D101 already
proved for `defvocabulary` (*"backward-compatible with every existing
content set (none of them declares one)"*), applied to a sibling
construct built the same way.

**The additive re-opening adds no expressive power to the closed
algebra.** §3 above states this directly: no generator, constructor
family, adjunction, level lattice, or severity rule. A `defevent` form
is data — a name, a set of key names, and a required/optional flag per
key — parsed and validated at load exactly like `deffield`'s existing
`<field-init>` shape; `emit`'s runtime semantics (§2.8, evaluating each
`<payload-item>`'s `<expr>` and posting the event) are unchanged by this
amendment in every case where a schema check passes.

## 7. What this draft does NOT do

- It does not write `bsl-language.rst`'s normative section for
  `defevent` — that is drafted once ratified, the same sequencing D101's
  own ADR195 followed (ADR195 covered "the LANGUAGE CHANGE… No Rust
  source is touched by this ADR").
- It does not implement any Rust grammar, loader, or checker code.
- It does not author any `defevent` content, including the Tier 1
  transcriptions §2 clause (iv) names as the natural first candidates.
- It does not assign `E-LOAD` numbers (§4).
- It does not resolve §4 class 3's open activation-scope question — that
  is this draft's one live decision for the ratifying sitting, named
  explicitly rather than silently picked.

All five are correctly out of scope for R4.6, whose own brief in the
charter (§14) names this document's job as "draft the constitutional
amendment… + its ADR" — drafting, not building — with ratification and
implementation both future, separately-scoped work.

## 8. DIRECTOR RULING REQUIRED

> **Ruling recorded (Director popup sequence, 2026-08-18 ~21:15 EDT):**
> item 2 is RULED — the undeclared-emit refusal activates **PER-COHORT**
> (this draft's own recommended reading: the refusal binds only content
> sets containing at least one `defevent`; adopters opt in; the three-tier
> registry is the drain plan; global enforcement arrives when the
> inventory empties). Items 1, 3, and 4 remain for the ratifying sitting
> proper (the ceremony that lands the §1 text in `CONSTITUTION.md`).

1. **Ratify or revise the proposed text in §1** (letter AH, MINOR version
   bump).
2. **Resolve §4 class 3's open question**: does "emit of an undeclared
   event" refuse only for the `EventType` cohort an implementing PR
   explicitly declares, or globally the instant `defevent` exists as a
   construct? This draft recommends the cohort-scoped reading (§2 clause
   iv, §4 class 3's first option) as the lower-risk, more reversible
   choice, consistent with `defvocabulary`'s own precedent.
3. **Confirm the registry's tier schedule (§2's table) as the intended
   rollout order** — Tier 1 first, Tier 2 at the engine freeze, Tier 3 on
   first emitter — or direct a different one.
4. **Assign the recording ADR number** at ratification time (§ header;
   next free above ADR215 as of this draft).

## 9. Drafting notes (not part of the proposed text)

- This draft follows the `ai/_inbox/amendment-*-draft.md` convention this
  repository already uses for a Director-gated, not-yet-ratified
  amendment text: `ai/_inbox/amendment-v3-refoundation-draft.md`
  (Amendment AE, P27 Phase 0 Task 16 — *"Create: `ai/_inbox/
  amendment-v3-refoundation-draft.md` (draft; the ratified text lands in
  `CONSTITUTION.md` via the Director's merge)"*), rather than
  `ai/decisions/`. Checked directly: no drafted-but-unratified amendment
  in this repo's history has an `ai/decisions/*.yaml` ADR filed for it
  before ratification — every located ADR whose status is `"accepted"`
  records a decision the Director had already made (ADR195's own D101
  ruling is a "Director ruling… approved twice" before the ADR was
  written; the AE precedent's own ADR172 is assigned "the ratifying
  commit"). Per this task's own instruction — skip the ADR stub if drafts
  don't get one pre-ratification — **no ADR stub accompanies this
  draft.** The recording ADR is created when this amendment ratifies, by
  the ratifying commit, exactly as ADR172 and ADR189 were.
- Every factual claim about current grammar/registry/precedent state in
  this draft was checked directly against source on this branch
  (`dev @ 126fe9402`, `wt-p4`) on 2026-08-18: `grammar.rs:208`,
  `bound_checker.rs:577-591`, `typecheck.rs` (zero `emit`-payload hits),
  `docs/reference/event-schema-registry.toml` (all three tier tables,
  the `unminted_bsl_only` row), `registry.py` (the `RegistryKey` shape),
  `bsl-language.rst` §2.13 (D101, D157) and §2.8 (`:2989`, the existing
  no-string-payload rule), `NORTH_STAR.md` §0 (`:12-27`, confirmed only
  AE is named there), `event_builders.py` (`:226`, `:536`, and the
  by-name presence check confirming all twelve Tier 1 `EventType`s also
  have an `EVENT_BUILDERS` entry), `src/babylon/models/events/
  _legacy.py` (`ControlRatioCrisisEvent`, `:387`, `:416`, `:421`, `:426`)
  and `spine_payloads.py` (`EntityDeathEvent`, `:48`, `:56-61`), and the
  charter document at `wt-hygiene` (§6, §14). No claim here is
  transcribed from the charter without independent verification against
  the file it describes, per this repo's Verifiability documentation
  standard.
- **Fix round 1 (2026-08-18)**: three verifiability defects (1 Critical,
  2 Important) found by an independent review
  (`.superpowers/sdd/2026-08-18-bsl-refactor-program/task-r46-review.md`)
  were corrected in place — clause (ii)'s false "no evidence"/"no
  per-field validation" claims (reworked with the real `EVENT_BUILDERS`
  Pydantic evidence and why it still doesn't ground a v1 per-key kind),
  the "unconstrained `<expr>`" overstatement of §2.8's existing
  no-string rule, and the two "NORTH_STAR §0 already names AG" claims
  (§0 names only AE; AH would be the second construct, not the third).
  The proposed text in §1 and the operative scope decisions (no per-key
  kind in v1, AH as the letter) were unaffected — every fix was to a
  factual claim in the surrounding rationale, independently re-verified
  against source before being written, not to the amendment itself.
