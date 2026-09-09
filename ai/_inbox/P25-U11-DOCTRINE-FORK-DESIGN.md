# P25 U11 — The Doctrine Fork (§3): derived verdict + five stances (ADR137)

**Status:** DESIGN SETTLED (post-recon `wf_fd6b613d-7be`, 6 VERIFIED scouts). Lane marker for
the political-superstructure worktree. Implement TDD, commit-by-commit, byte-safe each.

**Predecessors LANDED on this branch:** U8 AllegianceSystem @17.42 (ADR134), U9 PolicySystem
@17.47 (ADR135), U10 ElectoralSystem @17.45 (ADR136 — `political_form` PROMOTED shadow→canonical).
The ambient machine (§2) is complete. U11 opens the doctrine fork (§3).

---

## 0. Scope fence (the charter file list IS the fence)

Charter U11(b) names: `data/game/doctrine_tree_mvp.json`, `models/{enums,entities}/doctrine.py`,
`domain/doctrine/{mechanics,congress}.py`, `engine/systems/doctrine.py`,
`engine/actions/campaign.py` (+ mobilize/negotiate per §5.2/(a)), `engine/trap_detection.py`,
`ai/decisions/ADR137_*.yaml`. **Deliberately ABSENT:** `engine/systems/electoral.py`,
`engine/systems/allegiance.py`, `ooda/state_ai/`, `ooda.py`, tui/cli/play. Design additions the
charter (b) omits but (a)/§2.6/§3.1 require: `config/defines/politics.py` (new coeffs),
`domain/dialectics/instances/catalog.py` (political_form `pole_measure`),
`engine/systems/contradiction.py` (thread new GraphInputs fields — the ONE assembly site),
`models/events/*` builder wiring for `LINE_STRUGGLE_SPLIT` (publisher), `sentinels/superstructure/`
(new register owners). **U11 is doctrine-side + verb-resolver-side + trap_detection→political_form
only.** Spoiler arithmetic, host derecognition, the SYRIZA governance ceiling, popular-front
conjunctures are U12 — NOT here.

**DoctrineSystem @14.7 is EXTENDED, not added** → no system-order pins move (33 stays 33). The
byte-safety guard is the org-less loop `for node in graph.query_nodes(node_type=ORGANIZATION)`;
EVERY new computation stays inside it (or inside `step_organization`/its callees). qa six carry
`org_count=0` → no writes, no draws → byte-identical.

---

## 1. The nine deliverables → concrete design

### D1 — Typed measured-practice vocabulary (do NOT fake pseudo-tags)
New `PracticeVariable(StrEnum)` in `models/enums/doctrine.py`, a namespace DISTINCT from
`DoctrineTag` (charter: "do NOT fake pseudo-tags"). Members (I-FRESH quantities, never accumulated
into `doctrine_tags`):
- `SOLIDARITY_MASS = "solidarity_mass"` — weighted SOLIDARITY out-edge density of the org.
- `CO_OPTIVE_SHARE = "co_optive_share"` — CO_OPTIVE in-edge weight ÷ total incident tie weight.
- `OFFICE_TENURE = "office_tenure"` — accumulated tenure-ticks in office (normalized).
- `DELIVERY_DEPENDENCE = "delivery_dependence"` — reliance on ceiling-delivered promises (from the
  PolicySystem delivery register, one-tick-stale — I-ORD compliant).
- `PETTY_BOURGEOIS_DRIFT = "petty_bourgeois_drift"` — class_character distance toward PETTY_BOURGEOIS.

Type alias `DoctrineVariable = DoctrineTag | PracticeVariable`. These are RESOLVED by the DSL and
READ from a per-org environment assembled fresh each tick; they are NOT model fields on the tag
accumulator.

### D2 — DSL generalization (`domain/doctrine/mechanics.py`)
Grammar today: `comparison := TAG OP INT`, `evaluate_trap_condition(condition, tags: Mapping[DoctrineTag,float])`.
Generalize:
- `_resolve_variable(token) -> DoctrineVariable`: try `DoctrineTag[token]`, then `PracticeVariable[token]`,
  else `DoctrineExpressionError`. (Tokens stay UPPERCASE enum NAMES, matching current JSON.)
- New RHS operand: `operand := INT | COEFF` where `COEFF` is a `@name` sigil token
  (`@[a-z_][a-z0-9_]*`) resolved against a supplied coefficient namespace `Mapping[str, float]`.
  Retires magic-threshold literals from the data too (III.1 / ruling 8): practice thresholds are
  defines, referenced by name — `CO_OPTIVE_SHARE >= @co_optive_liquidation_threshold`.
- `evaluate_trap_condition(condition, env: Mapping[DoctrineVariable, float], coeffs: Mapping[str,float] = {})`.
  Absent variable → 0 (unchanged honest-null). Unknown `@coeff` → `DoctrineExpressionError`.
- **Backward compatible:** pure-tag conditions with INT RHS (adventurism `MASS_LINK <= 0`) evaluate
  identically with `env=tags, coeffs={}`. This keeps the DSL change itself byte-inert for the
  existing insurrectionist trap; only the tree-content change (D4) and system-wiring (D5) move the
  golden chain.
- Full DSL-vocabulary test suite here (charter (c)).

### D3 — Capability rewires (the re-founding; `models/entities/doctrine.py`)
New frozen `DoctrineCapability(BaseModel)` field on `DoctrineNode`:
```
verb_modes: tuple[str, ...] = ()      # "campaign:election:boycott", "mobilize:canvass", "negotiate:coalition"
edge_types: tuple[str, ...] = ()      # EdgeMode value strings this stance's mass work can build
cadre_valve_decouple: bool = False    # Boycott: cadre conversion decouples from the H valve
```
A doctrine node no longer earns its meaning from punitive `tag_deltas`; it grants CAPABILITIES.
`tag_deltas` stays as a field (scientific/insurrectionist trunks keep theirs; congress purge math
reuses it) but the reformist stances carry small/zero deltas and rich capabilities. Tag drift for
the reformist line comes from PRACTICE (D6), not acquisition deltas.

### D4 — Tree content: five stances + liquidationism absorbing state (`doctrine_tree_mvp.json`)
Replace the reformist trunk (`electoral_socialism → coalition_politics → liquidationism` as
purchasable) with the five-stance fork under `trade_unionism` (keeps its tier-1 seat):
- `abstention_boycott` (tier 2, trunk reformist) — caps `campaign:election:boycott`,
  `cadre_valve_decouple=true`.
- `class_struggle_elections` (tier 2, "Debs") — `campaign:election:run`, builds SOLIDARITY edges.
- `entryism` (tier 2, "DSA") — `campaign:election:run` + `negotiate:coalition`; host-discipline
  costs realized via existing `host_threat_threshold` (state-side derecognition is U12/state_ai —
  NOT built here; the capability is granted, the counter-play deferred).
- `independent_ballot_line` (tier 3, parents [class_struggle_elections]) — own party line.
- `governance_road` (tier 4, parents [entryism, independent_ballot_line] — the deep multi-parent
  node) — grants nothing new mechanically in U11 (the §2.4 levers already exist via PolicySystem);
  its ceiling/SYRIZA fork is U12. It is a real reachable node so U12 can hang the fork on it.
- `liquidationism` becomes an **absorbing-state trap** (is_trap=true, cost_tl=0, parents = the
  electoral stances so it is reachable on the reformist fork), trap_condition over measured practice:
  `SOLIDARITY_MASS <= @solidarity_liquidation_floor AND CO_OPTIVE_SHARE >= @co_optive_liquidation_threshold AND PETTY_BOURGEOIS_DRIFT >= @petty_bourgeois_liquidation_threshold`.
  You are not told you liquidated; you measurably did.
- Keep scientific (`democratic_centralism → mass_line → united_front` goal) and insurrectionist
  (`armed_vanguard → urban_guerrilla → adventurism` trap) trunks UNCHANGED.
- `validation.py`: traps still require trap_condition (holds); update `_check_trap_and_goal_flags`
  only if a multi-parent goal/trap edge case needs it (governance_road is neither trap nor goal).
- **GOLDEN_CHAIN moves (structure).** Regenerate + document in the commit (ADR073 obligation).

### D5 — DoctrineSystem wiring: practice env + officeholder capture + CLASS_ANALYSIS decay-when-vetoed
All inside the org-loop (byte-safe):
- **Practice env producer:** per org compute `PracticeVariable`→float from graph edges
  (SOLIDARITY out / CO_OPTIVE in), `class_character` (PETTY_BOURGEOIS_DRIFT), the electoral
  governments register (OFFICE_TENURE accrual), the PolicySystem delivery register
  (DELIVERY_DEPENDENCE, one-tick-stale). Thread `env = {**tag_env, **practice_env}` and
  `coeffs = politics defines subset` into `evaluate_trap_condition` for trap firing.
- **Officeholder capture:** `office_tenure` and `institutional_pull` become NEW Organization
  fields (frozen, default 0.0; declared → round-trip + check:vocabulary safe). While the org holds
  office (in `electoral_governments`), `office_tenure += 1`; `institutional_pull` drifts by
  `office_capture_rate` (existing), resisted by `cadre_level` and `cohesion` (Michels as a rate).
- **CLASS_ANALYSIS decay-when-vetoed** (Unit-6b extended, symmetric to `theory_bonus_per_class_analysis`):
  when the org's line predicted a delivery the ceiling then vetoed (delivery register shows a
  vetoed/struck item attributable to the org's governing stance), CLASS_ANALYSIS decays by
  `class_analysis_veto_decay` — theory rots when practice stops testing it.
- **GOLDEN_CHAIN moves (behavior).** Regenerate + document (ADR073).
- **Current stance is DERIVED**, not a new field: `acquired_doctrine_ids ∩ {five stance ids}`
  (DRY — reuse the accumulator). Needed by D7.

### D6 — Practice→tag feedback (the re-founding's other half)
Reformist-stance tag drift comes from practice, not deltas: e.g. sustained CO_OPTIVE dependence
pulls MASS_LINK down; SOLIDARITY-building practice (Debs) pushes it up. Implemented as a small
per-tick drift term in `step_organization` keyed off the practice env. (Bundled with D5.)

### D7 — Line-changes-as-splits (`congress.py` + `doctrine.py`)
Switching stances = a congress motion resolved by the EXISTING DT-5 purge machinery (`run_congress`,
`purge_probability`). On a successful stance-change split: publish `LINE_STRUGGLE_SPLIT` (payload +
builder + severity ALL already exist — U2 landed them; U11 is the first PUBLISHER), shed
branch-specific assets by `split_asset_retention` (existing define). `old_stance`/`new_stance`
derived from D5. Byte-safe: org-less → no congress → no publish.

### D8 — trap_detection.py absorption → political_form (ruling 8, III.1)
`detect_liberal_trap`'s four factors (liberal verb ratio, budget-cadre imbalance, stagnant
consciousness, low cohesion) are reborn as a per-org REPRESSION-pole signal feeding `political_form`:
- DoctrineSystem org-loop writes a `political_form_org_positions` register (org_id → signed σ in
  [-1,1], + toward representation) computed from the absorbed factors, thresholds now DEFINES.
- `contradiction.py` (the ONE GraphInputs assembly site, @18.0) threads a new
  `political_form_positions: tuple[tuple[str,float],...] | None` field via
  `graph.get_graph_attr("political_form_org_positions", None)` — exactly the
  `political_labor_share` precedent.
- `catalog.py`: add `political_form`'s **`pole_measure`** (it currently has none) — reads the new
  field → per-org `PoleSample`s. The aggregate balance stays driven by `political_labor_share`
  (U8). Ultra-left/rightist detectors: retire the hardcoded thresholds → defines (or leave the
  detectors as a parallel diagnostic per ruling-8's "or keep" clause — DECISION: absorb liberal,
  retire liberal's magic constants; the ultra-left/rightist detectors' magic constants also move to
  defines so the file is III.1-clean, but only the LIBERAL factors feed political_form). The sole
  live caller is `web/game/engine_bridge.py` (legacy, module-skipped) — rewire/retire freely.
- Byte-safe: org-less → register absent → None → no pole readings → byte-identical.

### D9 — Verb parameter growth (`engine/actions/{campaign,mobilize,negotiate}.py`)
`Action.params` is the established channel (educate.py already does `params.get('doctrine_node_id')`).
- `campaign.py`: `params.get('mode')` ∈ {RUN, BOYCOTT}. BOYCOTT → convert ambient H to agitation at
  `boycott_conversion`; if base H high while boycotting, MASS_LINK decays by `sect_isolation_rate`.
  RUN (Debs) → build SOLIDARITY edges at `debs_solidarity_efficiency` (η_cse, NEW define).
  **Capability-gated:** the mode is authorized only if the acting org acquired a stance node whose
  `capabilities.verb_modes` grants it; else loud `ActionResult(success=False, failure_reason=...)`.
- `mobilize.py`: `params.get('sub_mode') == CANVASS` variant.
- `negotiate.py`: `params.get('mode') == COALITION` variant.
- **Verb sentinel row cites the ActionSpec registry (`game/actions/registry.py`) as blocking
  dependency** — U11 does NOT add registry rows / ooda.py / tui/cli/play (interface train's seam).
- Byte-safe: org-less → no OODA org actions → resolvers never invoked → byte-identical.

---

## 2. New defines (Θ_feel; one metadata `defines_hash` ceremony)
Already present (reuse): `office_capture_rate`, `split_asset_retention`, `sect_isolation_rate`,
`boycott_conversion`, `host_threat_threshold`. **ADD** to `PoliticsDefines`:
- `solidarity_liquidation_floor` (Θ_feel) — SOLIDARITY_MASS floor for the liquidation absorbing state.
- `co_optive_liquidation_threshold` (Θ_feel) — CO_OPTIVE_SHARE trigger.
- `petty_bourgeois_liquidation_threshold` (Θ_feel) — class_character drift trigger.
- `debs_solidarity_efficiency` (Θ_feel, η_cse) — Campaign(Election,RUN) SOLIDARITY-edge efficiency.
- `class_analysis_veto_decay` (Θ_feel) — CLASS_ANALYSIS decay-when-vetoed rate.
- `co_optive_dependence_drift` (Θ_feel) — practice→MASS_LINK drift magnitude (D6).
Regenerate `defines.yaml`; metadata-only baseline ceremony `blessed(p25-u11-doctrine-defines-hash)`
(defines_hash tooth re-hashes by design; every checkpoint byte-identical).

---

## 3. Commit plan (byte-safe each; qa:regression byte-identical except A's defines_hash metadata)
- **A** — defines additions + regenerate yaml + defines_hash ceremony.
- **B** — `PracticeVariable` enum + `DoctrineVariable` alias (pure; nothing reads it yet).
- **C** — DSL generalization (mechanics.py) + full DSL-vocabulary test suite (backward-compatible;
  byte-inert for adventurism trap).
- **D** — tree content (five stances + liquidationism absorbing state) + `DoctrineCapability`
  model field + validation; regenerate GOLDEN_CHAIN (structure), document ADR073.
- **E** — DoctrineSystem: practice-env producer + trap-eval wiring + officeholder capture
  (`office_tenure`/`institutional_pull` fields) + CLASS_ANALYSIS decay-when-vetoed + practice→tag
  drift; regenerate GOLDEN_CHAIN (behavior), document ADR073.
- **F** — line-changes-as-splits: LINE_STRUGGLE_SPLIT publisher via congress DT-5 + split_asset_retention.
- **G** — verb parameter growth (campaign/mobilize/negotiate resolvers, capability-gated).
- **H** — trap_detection.py absorption → political_form pole_measure + GraphInputs thread
  (contradiction.py) + retire hardcoded thresholds → defines.
- **I** — superstructure sentinel (new register owners) + check:vocabulary + wiring-doctrine rows.
- **J** — governance: ADR137 + index + memory + CLAUDE.md (systems count stays 33; note the
  doctrine-fork §3 landing).

## 4. Estate blast radius (recon-verified)
- **GOLDEN_CHAIN** (`test_doctrine_system.py:380`) moves twice (D structure, E behavior) —
  regenerate-and-document each (ADR073), NOT a §6.5 baseline ceremony (it's a unit-test constant).
- **New DoctrineTag member?** NO — measured practice is `PracticeVariable`, a separate enum, so the
  `test_doctrine_tags.py` exact-dict pins, `_STARTING_TAG_VALUES`, and the "3 tags only" docstring
  DO NOT move. (This is the payoff of "do NOT fake pseudo-tags".)
- **check:vocabulary:** new Organization fields (`office_tenure`, `institutional_pull`) must be
  declared Pydantic fields (they are) → `MODEL_FIELDS_BY_NODE_TYPE[ORGANIZATION]` covers them.
  New graph-attr REGISTERS (`political_form_org_positions`, any electoral/delivery reads) → owners
  in the superstructure sentinel.
- **System-order pins:** UNMOVED (DoctrineSystem extended, not added; 33 stays 33).
- **qa:regression:** byte-identical throughout (org-less no-op preserved); ONLY commit A touches
  baselines (defines_hash metadata).
- Doctrine docstring "five scenarios" → fix to "six" while touching doctrine.py (ADR090 stale count).

## 5. Wiring-doctrine (ADR109) rows
- **W-𝔇** — political_form gains its per-node `pole_measure` (absorbed liberal-trap factors); the
  couplings (wage feeds / constrains atomization / imperial transforms) stay OPEN → U12.
- **W-C** — practice-env + officeholder-capture producers → the doctrine registers + trap evaluator;
  political_form_org_positions producer → GraphInputs consumer (contradiction.py).
- **W-P** — the five stances' verb-mode capabilities → resolver gating; **Verb row CITES the
  ActionSpec registry as blocking dependency** (surfacing to player/CPU = interface train).
- **W-A4** — verb resolvers carry `creates_value=False` (no value minted by a campaign/mobilize).

## 6. Open risks flagged for the design critique
1. DSL `@coeff` RHS sigil — is a define-reference in a data file the right III.1 move, or should
   practice thresholds be code-side? (Design: `@coeff`, keeps condition in data + threshold in
   defines — most faithful to ruling 8 "retire hardcoded thresholds".)
2. Two GOLDEN_CHAIN regenerations (D, E) vs one combined commit — reviewability vs a throwaway
   intermediate constant.
3. `governance_road` is mechanically inert in U11 (levers pre-exist; ceiling is U12) — is a
   reachable-but-inert node acceptable, or does it need a sentinel-inert citation?
4. trap_detection ultra-left/rightist detectors: absorb-liberal-only + move-all-magic-to-defines
   vs delete the unabsorbed detectors — the recon shows web-bridge is the only (legacy) caller.
5. PETTY_BOURGEOIS_DRIFT metric: class_character is an enum (discrete), so "drift toward
   PETTY_BOURGEOIS" needs a defined distance — is a discrete indicator (1.0 if already
   PETTY_BOURGEOIS else a graded proxy) faithful, or is a continuous class_character needed (scope
   creep into the class model — likely NO for U11)?

---

## REV 2 — Adversarial critique incorporated (`wf_467e3556-434`, 4 Opus agents)
Verdicts: DSL=RISKY, byte-safety=SOUND_WITH_FIXES, faithfulness=SOUND_WITH_FIXES,
wiring/estate=SOUND_WITH_FIXES. 3 BLOCKERs + 6 MAJORs resolved below. These decisions OVERRIDE
the rev-1 text above where they conflict.

**R1 (BLOCKER, DSL). No crashing intermediate.** At Commit D the tree must NOT reference `@coeffs`
the call site can't supply (doctrine.py:215 still 2-arg → `DoctrineExpressionError` when a reachable
trap fires). → **Commit D ships liquidationism with a TAG-ONLY placeholder** trap_condition
(`CLASS_ANALYSIS <= 0 AND MILITANCY <= 0`, INT-only, coeffs untouched). **Commit E** threads
`env`+`coeffs` into the call site AND swaps in the practice condition. D and E each coherent + non-crashing.

**R2 (BLOCKER, byte-safety). CLASS_ANALYSIS decay keys off the DELIVERY GAP, not "struck".** STRUCK
resolutions are never written to `policy_delivery`. → decay ∝ the governing org's delivery gap
(promised − delivered, which IS in the register). The veto's material trace is the gap. Same
register inversion feeds DELIVERY_DEPENDENCE.

**R3 (BLOCKER→resolved, estate). Commit D also updates 3 live pinned tests** (§4 rev-1 was WRONG):
`tests/unit/models/test_doctrine.py` (EXPECTED_NODE_COUNT 11→14), `tests/unit/domain/doctrine/test_doctrine_tags.py`
(reformist-path scenario on deleted electoral_socialism/coalition_politics — retarget to surviving/new ids),
`tests/unit/web/test_doctrine_tree_endpoint.py` (len==11 + liquidationism condition string; `mark.unit`,
NOT module-skipped). Verify exact pins at implementation.

**R4 (MAJOR, faithfulness). PETTY_BOURGEOIS_DRIFT = continuous material proxy, NEVER the discrete
`class_character` enum label** (tautology / Aleksandrov fail). → the petty-bourgeois share of the
org's MEMBERSHIP-weighted class base (or a co_optive+office+delivery composite). Verify the
membership-base class data exists when implementing E; reserve continuous class_character for a
later class-model unit (out of U11 scope).

**R5 (MAJOR, faithfulness). No free-lunch stance.** entryism + independent_ballot_line are strictly
dominant in a U11-only build if their costs are silently deferred. → (a) **ADR137 carries an explicit
DEFERRAL LEDGER** re-homing host-discipline projection, entryism host-derecognition, and independent-
ballot spoiler arithmetic to U12 (declared ruling, not erosion — NORTH_STAR "scope moves by ruling");
**amend U12(a) charter line** to name them. (b) **U11 ships the within-fence costs**: entryism accrues
CO_OPTIVE dependence (raising co_optive_share → toward the liquidationism absorbing state) + paper-
membership (low-weight MEMBERSHIP, so the P(S|R) numerator doesn't rise with headcount); abstention →
sect_isolation MASS_LINK decay (already designed); Debs → SOLIDARITY at η_cse below mass-work base
(already designed). No interval ships a free lunch.

**R6 (MAJOR, faithfulness). Charter U11(a) "host-discipline projection" is an enactment-time clamp in
fenced files** (PolicySystem/electoral/state_ai). → ADR137 formally rules it moves U11→U12, quoting
the fence. The capability is GRANTED in U11; the clamp lands in U12.

**R7 (MAJOR, wiring). `institutional_pull` must have a reader** (else dead write, III.10). →
office_tenure producer → `institutional_pull` drift (office_capture_rate, resisted by cadre/cohesion)
→ **reads INTO the practice→tag drift term** (Michels as theory-rot: high institutional_pull erodes
CLASS_ANALYSIS). W-C row names this consumer.

**R8 (MAJOR, estate). Public-import-surface pin.** Commit B adds `PracticeVariable` to
`EXPECTED_ENUMS_PUBLIC` (`tests/unit/test_public_import_surface.py`) — same as U9's `+PolicyAxis`.
Export `PracticeVariable`; keep `DoctrineVariable` union internal to mechanics.py (not exported).

**R9 (MAJOR, estate). GOLDEN_CHAIN fixture has ZERO coverage of new machinery** (empty
relationships=[] graph → practice env all-zero, office/institutional fields unhashed). → Commit E
EXTENDS the determinism fixture: seed SOLIDARITY + CO_OPTIVE edges, an electoral_governments +
policy_delivery register, an office-holding org; add office_tenure/institutional_pull to the hashed
payload. Regenerate GOLDEN_CHAIN with REAL coverage.

**R10 (MINORs).** (a) `evaluate_trap_condition(condition, env, coeffs=None)` — None sentinel, not
`{}` (B006). (b) liquidationism: **single parent `[trade_unionism]`, tier 2**, is_trap (multi-parent
AND-semantics would make the absorbing state unreachable; the practice condition is the real gate,
reachable by any trade-unionist org, fired only by measured co-optation). trunk="reformist" as a
display label. (c) GraphInputs `political_form_positions: tuple[tuple[str,float],...] | None` — the
`wage_value_id_pairs` shape; contradiction.py converts the register dict → deterministically-SORTED
tuple (III.7). (d) **superstructure owner row for `political_form_org_positions` lands in Commit H**
(where the write is), not I. (e) DoctrineSystem reads of electoral_governments/policy_delivery need
NO owner row (I-ORD sentinel gates WRITES only). (f) W-A4 (creates_value=False) folds into the same
ActionSpec-registry BLOCKED citation as W-P (no creates_value sentinel exists). (g) Tag/Practice
**disjointness sentinel test** (names AND values). (h) office_tenure = persistent accumulator field
(cross-term Michels drift, formed_tick gives only current term); OFFICE_TENURE variable = fresh
normalization read off it — fix the "never accumulated" rev-1 wording. (i) **ZERO** acquisition
tag_deltas on all five reformist-fork stance nodes (a "small" delta reintroduces punitive statics —
all reformist tag movement flows through the practice drift term). (j) ADR137 ratifies the U11(b)
file-list expansion (contradiction.py + catalog.py + politics.py + sentinels sanctioned); update the
REFORMIST trunk docstring when D lands.

**REV-2 COMMIT PLAN (supersedes §3):** A defines+ceremony · B PracticeVariable+surface-pin+disjointness
· C DSL generalize (backward-compat, byte-inert) · **D** tree(5 stances, ZERO deltas)+capability
model+validation+placeholder liquidationism+3 pinned-test updates+golden(structure) · **E** practice-env
producer+coeffs threading+practice liquidationism+officeholder capture(office_tenure/institutional_pull,
pull→tag-drift)+CLASS_ANALYSIS decay(delivery gap)+practice→tag drift+EXTEND fixture+golden(behavior) ·
F line-splits(LINE_STRUGGLE_SPLIT via DT-5)+prove-inert-or-golden · G verb params(capability-gated;
entryism CO_OPTIVE cost; Debs η_cse; boycott+sect_isolation) · H trap_detection→political_form
pole_measure+GraphInputs(dict→sorted-tuple)+owner row+retire thresholds→defines · I sentinel/vocab/wiring
finalize · J ADR137(deferral ledger+charter amend+(b)-expansion)+index+memory+CLAUDE.md.
