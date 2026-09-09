# Register memos — rows 9–12 (director-gate register, issue #564)

Per ADR202 R11 (2026-08-14): register rows 9–24 proceed **memos-first, async**; no second
synchronous session is chartered. This memo distills rows 9–12. Verbatim row text from
`reports/port-estate-survey-2026-08-12.md` §5b (the register's canonical source). Every
`file:line` anchor re-verified against the working tree on branch `docs/adr202-t4-rulings`
(2026-08-14); stale anchors are marked and corrected.

**Register-integrity finding up front:** rows 9 and 12 are CHECKED and both checks are
legitimate — each carries an explicit Director ruling, delivered live 2026-08-12 and recorded
as a comment on #564 (percy-raskova, 2026-08-12T18:44:06Z, "Director rulings, 2026-08-12 (live
session, AskUserQuestion)"). Two record-keeping gaps, neither a discharge failure: (i) neither
ruling is recorded in any ADR — a sweep of ADR190–ADR202 finds the rulings only in that issue
comment; (ii) ADR202 R11's phrase "the sixteen remaining director-gate rows" (rows 9–24) reads
past the two already-ruled rows. Recommended hygiene: the FascistFaction and Class-D port-train
ADRs cite the #564 comment verbatim.

---

## Row 9 — CHECKED. Discharged by Director ruling; discharge claim verified against code

> **FascistFaction FAC_DECOLONIAL mis-selection** — token-match of 'settler' inside
> 'anti-settler abolitionism' + min() ordering picks the decolonial faction for fascist capture;
> LIVE on all five balkanization goldens. Adjudicator: Director escalation, NOT a port-as-is row

**The ruling (issue #564 comment, 2026-08-12, verbatim):** "Row 9 — RULED: repair at the port,
declared property. Fascist-capturability becomes an explicit declared faction property
(content-declared; no token matching). A conformance vector pins FAC_DECOLONIAL as never
capturable. D-record filed at the FascistFaction port train; golden divergence on the five
balkanization vectors is declared and attributed to the repair (ADR183 discipline: the frozen
engine is a structure contract, not a correctness oracle)."

**Verification of the row's factual claims — all confirmed:**

- `_FASCIST_IDEOLOGY_TOKENS = ("fascist", "reaction", "revanch", "settler")` —
  `src/babylon/engine/systems/reactionary.py:71`.
- Substring token match: `ideology = str(attrs.get("ideology", "")).lower()` then
  `any(tok in ideology ...)` — `reactionary.py:223-224`. `"settler" in "anti-settler
  abolitionism"` is True.
- `min(candidates)` selection — `reactionary.py:227` (function `_find_fascist_faction`,
  :215-227).
- `src/babylon/data/game/balkanization/seed_factions.json:31-35`: FAC_DECOLONIAL has
  `ideology: "anti-settler abolitionism"`, `colonial_stance: "abolish"`,
  `is_settler_formation: false` — admitted ONLY via the token bug (the `settler_uphold`
  predicate at :220-222 correctly excludes it). FAC_RESTORATIONIST (:5-9) is admitted both ways.
  `min` returns FAC_DECOLONIAL (`"D" < "R"`). The anti-colonial front is what the engine calls
  "the fascist faction" — confirmed end to end.
- "LIVE on all five balkanization goldens" — confirmed as the five electoral goldens
  `weimar`/`mitterrand`/`syriza`/`debs`/`bernie_valve` (`tools/regression_scenarios.py:77-118`),
  each factory calling `apply_political_terrain` (`electoral_goldens.py:226,270,334,428,504`),
  which calls `apply_balkanization_seed(state)` unconditionally at `electoral_fixture.py:204`,
  minting all four factions. One nuance from the inventory's own adjudication: capture is
  *structurally* live on `mitterrand`/`syriza` (LA class `C004` co-seeded); whether
  `fascist_alignment` reaches threshold in-horizon is UNVERIFIED, and the effect
  (`aligned_faction_id`, a declared SocialClass field) is byte-gated the moment capture fires
  (`reports/port-inventories/fascist-faction-port-phase1-inventory-2026-08-12.md`, corrections
  1–2, confirmation 12).
- **Repair status today:** nothing in `rust/` references FAC_DECOLONIAL or capturability — the
  declared-property repair and its conformance vector do not exist yet. Consistent with the
  ruling: the repair lands *at* the FascistFaction port train, which is unscheduled (Wave C/D,
  behind T2/T3). The ruling is prospective, not yet executed — the check mark is warranted
  because the *disposition* is ruled.

**Verdict:** discharged; no options owed. Follow-up: cite the comment in the port train's ADR.

---

## Row 12 — CHECKED. Discharged by Director ruling; discharge claim verified against code

> **ControlRatio revolution-vs-genocide branch** — Program 19 rules it LAST; does the joint
> Class-D train respect that or split?

**The ruling (issue #564 comment, 2026-08-12, verbatim):** "Row 12 — RULED: the joint train
stands, port-as-is. Transcription is not the cutover: #566 ports the frozen
revolution-vs-genocide branch verbatim with a D-record marking it P19-cutover-pending; Program
19's LAST-ordering continues to govern the actual partition cutover in Rust. #566 is unblocked
for chartering."

**The prior ruling it reconciles:** ADR070 (2026-07-14, Program 19), cutover roadmap —
"ControlRatio (revolution-vs-genocide branch) LAST, no exception, only after low flip-count
evidence, with a dedicated high-effort review" (`ai/decisions/ADR070_emergent_class_partition.yaml`,
decision text; ADR070 binds the *cutover* order, not the transcription).

**Verification — all confirmed:**

- The Class-D coupling is exactly one key wide, from both sides:
  `src/babylon/engine/systems/decomposition.py:223` writes
  `persistent["_class_decomposition_tick"] = tick`; `control_ratio.py:128` reads it (early
  return at :129-130 while absent).
- The branch: `control_ratio.py:219-232` — `avg_organization >= revolution_threshold` →
  `outcome = "revolution"`, else `"genocide"`, emitted as `TERMINAL_DECISION` (:234-247).
  Threshold from `services.defines.carceral.revolution_threshold` (:219).
- #566 ("Wave A: Decomposition @11.0 + ControlRatio @12.0 joint port train") is OPEN and its
  body names row 12 as the charter gate — the ruling discharges exactly that gate.

**Verdict:** discharged; no options owed. The ruling's logic is sound under ADR070: porting the
frozen branch verbatim does not execute the emergent-partition cutover, so the joint train does
not jump the LAST-ordering — provided the D-record marking it P19-cutover-pending is actually
filed (make that a #566 acceptance criterion).

---

## Row 10 — UNCHECKED. Doctrine: is transcribing the 14-node tree into `.bscn` defconsts a Director-gated act?

> Doctrine — is transcribing the 14-node tree into .bscn defconsts a Director-gated act?
> (adjudicator: yes)

**Surface.** DoctrineSystem @14.7 (`src/babylon/engine/systems/doctrine.py`) over content
`src/babylon/data/game/doctrine_tree_mvp.json`. The adjudication
(`reports/port-inventories/doctrine-port-phase1-inventory-2026-08-12.md:529-548,647-653`)
rules the system "is THE reserved surface" and the inventory's missing reserved-line section a
labelling failure: "transcribing any of it into BSL content is a Director-gated act, not a
mechanical transcription."

**Code evidence (verified):**

- The tree: `doctrine_tree_mvp.json` — `root_id: class_consciousness`, exactly **14 nodes**:
  reformist trunk ×5 (`abstention_boycott`, `class_struggle_elections`, `entryism`,
  `independent_ballot_line`, `governance_road` — **all five `tag_deltas: {}`**, the
  zero-acquisition-delta discipline), scientific ×3 (`democratic_centralism`, `mass_line`,
  `united_front`), insurrectionist ×3 (`armed_vanguard`, `urban_guerrilla`, `adventurism`),
  plus untrunked `class_consciousness`, `trade_unionism`, `liquidationism`.
- Two traps: `liquidationism` and `adventurism`, both `is_trap: true`. Liquidationism's
  `trap_condition`: `SOLIDARITY_MASS <= @solidarity_liquidation_floor AND CO_OPTIVE_SHARE >=
  @co_optive_liquidation_threshold AND PETTY_BOURGEOIS_DRIFT >= @petty_bourgeois_liquidation_threshold`.
- The three thresholds: `src/babylon/config/defines/politics.py:445-479` — defaults **0.05 /
  0.6 / 0.6**; supplied to the trap DSL at `doctrine.py:659-661`.
- Port-relevant mechanics: greedy acquisition `min(candidates, key=lambda nid: (cost_tl, nid))`
  in `_cheapest_acquirable` (`engine/systems/doctrine.py:104-113` — the inventory's
  `mechanics.py:110-113` anchor is STALE, corrected here) and the `_reachable_traps` loop
  (`doctrine.py:347-358`) iterate a content-derived set — no landed BSL form serves that.
- **Live edge:** three trap conditions are ALREADY transcribed as Rust conformance vectors —
  `rust/crates/babylon-bsl/tests/conformance/doctrine_adventurism.bsl`,
  `doctrine_liquidationism.bsl`, `doctrine_liquidation_absorbing.bsl` (the last references all
  three thresholds as `:const doctrine/*`), wired into `conformance_corpus.rs:263-318`. A
  sliver of the reserved content already sits in the test estate, transcribed from the pinned
  Python tests, before any general ruling on tree transcription.

**Binding rulings.** Constitution §IX.5 / Amendment AD (the Director holds the ideological
line; doctrine content is its core case); ADR073 + **ADR137** (the five-stance fork, the
zero-`tag_delta` discipline, the absorbing state and the `@coeff` DSL are *already ruled
content* — the question is re-expression, not authorship); ADR195/196 (content enters the
closed vocabulary by ceremony; `defconst`/`defenum` declarations are **hash-bearing** — the
transcription fixes the canonical byte form of the line); ADR198 (Wave E: "Doctrine
reserved-line heavy"); ADR200 (the batch's model discipline — the reserved Amin/Wallerstein
routing was *transcribed exactly*, not re-decided); ADR202 R11 (this lane).

**Options.**

- **A. Gate the act (dossier-then-ruling).** No tree content enters `.bscn` until the Director
  rules the whole tree in one pass — the ADR198-R5/T4 curves pattern. Maximal line control;
  cheap because Doctrine is unscheduled (Wave E). Leaves the three landed vectors in limbo.
- **B. Port-as-is under ADR137 cover (ADR200 model).** The content is already ruled; the
  transcription is mechanical re-expression with D-records for deviations; the Director reviews
  the conformance diff at train close. No new ceremony — but the adjudicator ruled precisely
  this insufficient for THE reserved surface, and silence risk is highest here.
- **C. Split ruling.** Structure (ids, parents, tiers, `cost_tl`) transcribes mechanically
  under ADR137 cover; the semantic payload (trunk taxonomy, every `tag_deltas` value, both trap
  conditions + three thresholds, names/warnings/narratives) gets ONE explicit async
  confirmation pass; plus an explicit disposition for the three landed vectors.
- **D. Defer entirely.** Rule nothing now; the port is Wave E behind Slice 2, D35/D65 storage,
  RNG binding, and the content-set-iteration lane. Leaves the row open and the vectors
  unratified.

**Workforce recommendation: C.** The DAG skeleton is mechanics; the ideology lives in the
semantic payload — exactly what §IX.5 reserves. One async confirmation over that payload is
cheaper than a session (A) and honest where (B) is silent. Recommend the vector disposition be:
ratify as sanctioned test fixtures (they mirror pinned Python tests verbatim, carry no tree
structure). If the Director prefers maximal control, A is the safe ruling and costs little at
Wave E timing.

**Reserved-line flags — ONLY the Director may decide:** the 14-node taxonomy and trunk
assignments; all `tag_deltas` values; both trap conditions and the three liquidation
thresholds; the five stances' zero-delta discipline; node names/warnings/narratives; whether
doctrine content is player-moddable content at all. The workforce may not unilaterally decide
which parts are "merely structural" — the split in option C is itself hers to confirm.

---

## Row 11 — UNCHECKED. Sovereignty `_STANCE_TO_POLICY` + metabolic impacts — hash-bearing declaration order on a reserved axis

> Sovereignty _STANCE_TO_POLICY + metabolic impacts — hash-bearing declaration order on a
> reserved axis

Register detail (survey §5b row 11): "`_STANCE_TO_POLICY` (`formulas/balkanization.py:24-28`) +
`metabolic_impact_intensify` −0.02 / `_continue` −0.005 / `_cease` **+0.01**. The int-ordinal
`extraction_policy` workaround fixes a **hash-bearing declaration order on a reserved axis**
(ADR195/196). Director disposition, not a port-time D-record. Inventory files no check."

**Code evidence (verified; the survey's anchor is exact, not stale):**

- `_STANCE_TO_POLICY` — `src/babylon/formulas/balkanization.py:24-28`: UPHOLD→INTENSIFY,
  IGNORE→CONTINUE, ABOLISH→CEASE.
- `calculate_metabolic_impact` — `balkanization.py:31-74`, a zero-arithmetic three-way dispatch
  returning the defines values.
- The coefficients: `src/babylon/config/defines/balkanization.py:46-57` —
  **−0.02 / −0.005 / +0.01**; mirrored at `src/babylon/data/defines.yaml:318-320`.
- The reserved axis: `src/babylon/models/enums/balkanization.py:33-52` — `ColonialStance`'s own
  docstring: "The principal contradiction in MLM-TW analysis (Constitution I.1)"; declaration
  order UPHOLD/IGNORE/ABOLISH. `ExtractionPolicy` at :54-70, declaration order
  INTENSIFY/CONTINUE/CEASE.
- Read site: `sovereignty.py:100` (`_coerce_policy(sov_node.attributes.get("extraction_policy"))`);
  sole downstream consumer `metabolism.py:80-86` (one-tick lag, position 13.0 < 17.5).
- **Ownership nuance (load-bearing for scoping):** `SovereigntySystem` never calls
  `_STANCE_TO_POLICY` — the derivation belongs to FactionInfluence/CollapseTransition
  (sovereignty inventory, module table line 30; zero imports grep-confirmed). Sovereignty's
  only formula is `calculate_metabolic_impact`; the policy value arrives seeded
  (`seed_sovereigns.json`: `extraction_policy: "intensify"`).

**Binding rulings.** Constitution I.1/I.4 (the axis itself); **ADR171** (the National Question
rulings — national/colonial dispositions are Director-reserved; OQ8 reserves even the on-screen
SETTLER name); **ADR195** (enum storage law: "declaration order is normative — it IS the
ordinal," stored in the hash-bearing binary64 lane; `defenum` declarations are hashed content);
D102 (`field-of` refused on `:enum-type` fields at load — the int-ordinal and `defenum` routes
are mutually exclusive per the survey); ADR196 (the vocabulary-ceremony model for minting
content); ADR198 R1–R3 (the system's actual blocker — CLAIMS `control_level` edge-attribute
storage, train T3); ADR202 (curve-estate analog: signed theoretical magnitudes get explicit
rulings, e.g. C7 "full antagonism must be displayable").

**The question, sharpened.** Under ADR195 the int-ordinal encoding makes the enum's declaration
order part of the canonical hash. On a reserved axis, an unruled D-record would fix the byte
form of the line (which stance is ordinal 0) as a side effect of engineering. Separately, the
three signed coefficients encode a theoretical claim — settler extraction degrades the land,
abolition heals it — they are not calibrations.

**Options.**

- **A. Ratify-then-transcribe (async, one ruling).** Director explicitly ratifies (i) the
  stance→policy correspondence verbatim, (ii) the declaration order (INTENSIFY/CONTINUE/CEASE
  mirroring UPHOLD/IGNORE/ABOLISH) as normative, (iii) the three coefficients as-is. The port
  then files the int-ordinal D-record under that ratification.
- **B. `defenum` route.** Wait for the ADR195 machinery (Org-foundation Tasks 3-10, unlanded)
  and declare both enums as content `defenum`s with ratified member order — the survey's
  preferred route, mutually exclusive with int-ordinal. Sovereignty is T3-blocked anyway, so
  the wait may be free; the cost is coupling this ruling to an unlanded Rust lane.
- **C. Split by ownership.** Rule the three metabolic coefficients now (Sovereignty's actual
  surface); defer the stance→policy map and the enum encoding to the train that owns the
  derivation (FactionInfluence @14.5 / CollapseTransition @20.5), so the map is ruled once, at
  its producer.
- **D. Port-as-is D-record with post-hoc review.** Not recommended — it is exactly the "silent
  hash-bearing order on a reserved axis" the adjudicator flagged.

**Workforce recommendation: A+C combined.** Ask for one async ruling covering the three
ratifications in A, scoped per C — the correspondence and order ruled at the producer train,
the coefficients ruled now. Once order and coefficients are ratified, the storage mechanism
(int-ordinal vs `defenum`, i.e. A vs B) is engineering and should be delegated to the
workforce: the mechanism is not the line; the order is. This matches ADR191 R7's delegation
pattern (edge/hyperedge field storage delegated on rigor-plus-entertainment).

**Reserved-line flags — ONLY the Director may decide:** the stance→policy correspondence
itself; the declaration ORDER on the reserved axis (which member is ordinal 0); the three
signed coefficients in sign AND magnitude; the `ColonialStance`/`ExtractionPolicy` taxonomies;
any renaming (ADR171 OQ8). Not reserved: the int-vs-`defenum` storage mechanism, once the order
is ratified.
