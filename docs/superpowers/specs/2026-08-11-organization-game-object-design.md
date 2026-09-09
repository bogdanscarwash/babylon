# Organization as Game Object — the Base Contract Content Instantiates

**Status:** Director-approved design (2026-08-11, live brainstorm — all twenty §5 questions of the
design-inputs dossier ruled, plus the Gramsci/Althusser Q3 study and its adoption rulings).
**Tracking:** issue #513. **Provenance:** `reports/organization-design-inputs-2026-08-11.md`
(the 18-agent dossier, PR #511), `reports/gramsci-althusser-institution-memo-2026-08-11.md`
(the commissioned theory study), and the rulings ledger recorded during the session.
**This is a spec, not a plan.** The implementation plan follows the Director's written-spec
review, per the standing brainstorm→spec→plan workflow.

The Director's framing, verbatim: *"we really need to develop out what an Organization is as a
game object and abstract base class that can be instantiated."* In this codebase that translates
to a **schema + capability contract** — a kernel node type, declared fields, doctrine data,
kernel capacity mechanics, and BSL rules as behavior — instantiated by content packs, shared by
the player org and AI orgs alike.

---

## 1. The ruling ledger (normative)

Every ruling below was made by the Director on 2026-08-11. "(R)" = accepted the workforce
recommendation; "(D)" = the Director's own direction beyond or against it.

| # | Ruling |
|---|---|
| Q1 (R) | **Content-declared kinds.** One kernel `NodeType/ORGANIZATION`; the four kinds (state / capital / party / civil-society) are a content-declared enum field; the eligibility matrix moves to BSL content per R8. |
| Q2 (R) | **Node + coalition hyperedge.** The org is a node; the coalition/united-front construct is an Amendment AG attributed-membership hyperedge; per-member payload defers to first consumer. |
| Q3 (D→R) | **Institution-ness is a relation, not a kind** — ruled (d) as translated after the commissioned Gramsci/Althusser study (§3 below). |
| Q3-adj (D) | **Three-strata composition adopted in full** — social base / mass membership / cadre-staff, hollowing-out pathology, bound-vs-free mandate variable (§4). |
| Q4 (R) | **`is_player` retired.** The `player_org_id` graph singleton is the only player marker; the test estate and both reference docs migrate. |
| Q5 (D) | **The player org's nature is its doctrine identity** — one of the twelve (Major, Minor) trajectories from the four-trunk tree, not the OrgType ladder. Follow-up ruled: ADR176 (6) stands — Major chosen at founding, Minor **earned** from measured practice; the twelve identities are discovered, never menu-picked. |
| Q6 (R) | **One player organization.** Line-splits mint **NPC rival orgs** carrying the shed line. Coalition play arrives via the Q2 hyperedge: influence, never control. |
| Q7 (R) | **Two-channel progression.** Doctrine gates what an org may *attempt*; material state gates whether the attempt *lands*. |
| Q8 (R) | **Founding is campaign setup** (county, name, Major). Mid-run node minting has exactly two other paths: splits and Institution SpawningBlueprints. |
| Q9 (R) | **Real, rule-driven death; player collapse is game over.** Never threshold-driven: no capacity + no members + no edges *is* dead by arithmetic. The existing liquidation rule is finished (decapitation path). Absorbing states remain the partial deaths. |
| Q10 (R) | **States may name themselves.** Trap/absorbing-state text and split announcements are the world speaking, lawful mid-run; scripted verdicts stay in the epilogue. |
| Q11 (R) | **State verb estate: contract now, phased landing.** The differentiated state menu is part of the contract; the REPRESS collapse ports first as a recorded defect; differentiation is a chartered train. |
| Q12 (R) | **Mint the `enum` deffield row** — a seventh type: declared value set, validated symbol, canonical hashing. Language change, chartered; first consumer is Q1's kind field. |
| Q13 (R) | **Budget graph-visible; Capacity allocates.** Budget total + source composition as declared Currency fields (typed-Currency storage's second consumer); `Capacity` remains the kernel spend-side allocator. |
| Q14 (R) | **Strain targets derive from the trunk's own tag_deltas** — never an authored profile matrix (lawful under ADR172 r5 by construction). Fingerprint gates live in the test estate. |
| Q15 (R, sharpened) | **Founding edge vocabulary:** MEMBERSHIP, PRESENCE, COMMAND, TRANSACTIONAL, SOLIDARISTIC, **org↔org SOLIDARITY**. RECRUITMENT/EMPLOYMENT stay retired by ADR176 (34) — still present in the frozen Python `EdgeType` enum, never re-minted in the Rust vocabulary; every further edge earns its ceremony. |
| Q16 (D — escalated) | **Full endgame redesign.** The canonical outcomes read the organizational apparatus broadly. A chartered design train on the Director's reserved line (§8). Working interpretation, presented and not objected to at design review: the win-gate repair + one org-seeding golden land first as the floor. |
| Q17 (R) | **Doctrine tree graph-native.** Acquisitions become queryable nodes/edges (CAPABILITY_GRANT-shaped); doctrine is visible, inspectable, targetable world-state. |
| Q18 (R) | **I.21 is live — build the player half** (Educate→centrality, Aid→cutsets, Attack→singleton-exposure), unblocked by the Q15 edge. I.21's bracket gets a PATCH-level status correction. |
| Q19 (D) | **Data tiers 1+2 green-lit; tier 3 withheld.** Facility counts wire into StateApparatus capacity (carceral-only honest scope); QCEW-813 seeds the movement footprint. OLMS/FEC/990 ingestion stays unchartered — ADR176 (38)'s capture vectors run data-light until the Director says otherwise. |
| Q20 (R) | **Clandestinity is a recomputed composition** — derived each tick from line, measured practice, and capacity allocation; never a player dial; losable in both directions. |

**Already ruled, excluded from this spec's scope** (ADR176, never re-opened here): trunk names
and the Autonomist fourth trunk (4, 36); commutative (Major, Minor) union with cost/efficacy
asymmetry, doctrine never hides a verb (5); Major/Minor origin (6); transposition default /
split at extremes (7); nine keys + strike/BUILD/EXPROPRIATE sub-modes (12, 15, 37); chartered
security and metabolic absences (16); standing orders in v1.0 (22); the five edge retirements
(34); funding as a doctrine-pair surface with reformist money the capture vector (14, 38).

---

## 2. Ontology and vocabulary

**One kind of body.** `NodeType/ORGANIZATION` is registered in the first production
`ClosedVocabulary` (the existing Rust registry in `babylon-bsl`'s `vocabulary.rs`, which today
has zero production call sites — the tick driver sets `vocabulary_registry: None`; this contract
is its first production consumer and its wiring is part of this contract's work; the Python
`NodeType.ORGANIZATION` enum member is the frozen estate, not this one). Churches, parties, unions, newspapers, firms, police departments: all one node
type, distinguished by fields and edges, never by kernel kind.

**Kind is content.** `organization/kind` is an enum-typed declared field (`state-apparatus` /
`business` / `political-faction` / `civil-society`), consuming the Q12 enum row. The eligibility
matrix (which kinds may EMPLOY, INFILTRATE, LOCKOUT…) is BSL content per R8 — the pedagogy (a
reading group cannot LOCKOUT) survives as data, not as a type system.

**Two hyperedge constructs, both Amendment AG shapes:**

- **The coalition** (united front): an attributed-membership hyperedge over member orgs. The
  player can be inside one without controlling its other members. Per-member payload (role,
  commitment) defers to Slice 4's first consumer per standing ruling.
- **The apparatus** (the re-referred `Institution`): the religious / educational /
  communications / legal / repressive apparatus of a *bloc* — a hyperedge over the organizations
  that constitute it, with each membership pair carrying that org's **integration degree**. This
  is Althusser's ISA read literally (the ISA is the plural field, not the individual body) and
  Gramsci's `apparato dell'egemonia` (Q8 §179). The 1:1 `HOUSES` containment edge **retires**
  (refuted four ways in the memo: many-to-many, degree-valued, contested, reversible).

**Institutionality is computed, signed, and never stored as a type.** An organization's
`institutionality` w.r.t. a bloc is derived from four textually-grounded channels — consent
produced for the bloc, personnel selected into it, material sustenance drawn from it, protection
received from it — combined by **rank-order under budget** (the Lane A precedent; no threshold
constant, per the no-imposed-forms law). It is **signed by bloc**: deep integration into a
revolutionary bloc's apparatus is the same mathematics as transformismo with the opposite sign.
An unsigned scalar would make council-building read as co-optation — a theory bug, not a
modeling shortcut.

## 3. The Q3 ruling in full (the theory study)

Six readers over the primary texts (Italian Notebooks 7/8/16/17/27/28, the ISA essay, the
pre-prison writings, the transcribed *Selections* chapters) converged unanimously on **(d)**:
neither Gramsci nor Althusser opposes organization to institution as kinds; the operative
concepts are *position* and *function* inside a bloc's hegemonic apparatus, and integration is
continuous, contested, reversible, many-to-many, and formally invisible ("What matters is how
they function" — Althusser; the State "is not always to be sought where it would seem to be
'institutionally'" — Q8 §233). Constitution I.16's "formalization" is precisely the criterion
both authors reject. Full evidence and adjudication: the memo. Consequences adopted here:
§2's apparatus construct, the retirement of `HOUSES`, the computed signed `institutionality`,
and the I.16 amendment motion (§9). The memo's residue (channel weighting, the
apparatus-as-object engineering call's fine grain, de-institutionalization as a live path)
stays open in §10. Corpus gap on record: Notebooks 12–13 were unavailable locally.

## 4. Composition — the three strata

Adopted from Q7 §77 (confirmed independently by two readers) as the contract's composition
model, replacing the flat membership picture:

1. **Social base** — the classes an org claims and draws from: MEMBERSHIP edges into
   `social_class`, weight deliberately `< 1` ("the surge is real, the power is not").
2. **Mass membership** — the mobilizable magnitude, distinct from the base it comes from.
3. **Cadre-staff** — the operating core; ADR184's capacity-bearing tier; where doctrine
   reproduction lives ("no formation of leaders without the theoretical, doctrinal activity of
   parties").

**Hollowing-out** is a *derived pathology*, never a stored flag: staff persisting while the base
decays — the org "voided of its social content… suspended in mid-air." Observed by the
write-log and the inspector surface; measured, not thresholded.

**The mandate variable.** Leadership carries a bound-vs-free mandate value (the Turin councils'
"fixed and conditional mandate") on the COMMAND/leadership surface. It feeds the capture
mathematics: bribability of key figures scales **inversely with cadre depth** (the memo's
molecular-transformismo rate law), and a bound mandate is the anti-transformismo lever the
player can actually build.

## 5. The player's seat

- `player_org_id` (graph singleton) is the only player marker. `is_player` retires everywhere:
  the field, the two reference docs that teach it, and the three test suites that encode it.
- The player org's **nature is its doctrine trajectory**: Major chosen at campaign setup
  (county, name, Major trunk), Minor earned from measured practice and ratified at the first
  congress. The twelve identities are the run's discovery, not a character-select.
- **One player org, forever.** A line-split mints an NPC rival carrying the shed line — the
  ADR137 cost made visible and persistent. Coalitions give breadth without control.
- **Two-channel progression:** doctrine (graph-native, §7) gates attempts; material state
  (budget, cadre, capacity) gates landing. A doctrine cannot buy a treasury; a treasury cannot
  buy a line.
- Player/AI symmetry holds at the dispatch level (same contract, different decide-driver). The
  port must **explicitly decide** each existing player-only carve-out (the INVESTIGATE intel
  bonus, the receptivity gate) rather than inherit them silently — each is either ruled
  symmetric or recorded as a deliberate asymmetry in the pack that lands it.

## 6. Lifecycle

**Minting paths — exactly three.** Campaign setup (the player org); splits (NPC rivals, per Q6);
Institution `SpawningBlueprint`s (world growth). No founding verb exists; the tenth-verb wall
stands.

**Death is real and arithmetic.** An org with no capacity, no members, and no edges is dead —
the write-log records it, narration names it (Q10), and the node is removed through the same
cascade ADR185 R2 pinned. The finished liquidation rule covers decapitation (its four
reachability defects — the REPRESS flattening, the coherence/cohesion mismatch, the
`is_singleton` producer, the dead defines constant — are repaired at the port). The absorbing
states (derecognition, liquidationism, the Allende geometry) remain the partial deaths. The
historical corpus's other destruction modes (volunteer-labor exhaustion, governance starvation,
the professionalization ratchet) are content-expressible through the same arithmetic — an org
whose replenishment sources dry up dies of the drying, no bespoke mechanic.

**Player collapse is game over.** The epochs' stake, ruled binding.

## 7. Verbs, capability, doctrine

- **The nine keys stand** (ruled; out of scope here). Sub-modes are the depth axis;
  DoctrineCapability gates them (attempt channel), material gates land them (Q7).
- **The state's differentiated menu is contract, day one.** RAID, LIQUIDATE, AUDIT, REVOKE,
  INVEST, REZONE, DISPLACE, NEGLECT, FUND, STAFF — the Develop estate Article V promises. The
  port lands the frozen engine's REPRESS collapse first (port-as-is), recorded as a defect with
  its D-record; the differentiation train is chartered (§9) and the state's menu ships before
  the Organization estate is called complete.
- **Doctrine goes graph-native** (Q17): acquisitions become nodes/edges a `when` guard queries
  (CAPABILITY_GRANT-shaped); eligibility becomes BSL content; an org's capabilities are facts
  about the world — visible under ADR182's earned depth, targetable by rivals.
- **I.21's player half gets built** (Q18): Educate creates centrality, Aid strengthens cutsets,
  Attack exposes singletons — over the org↔org SOLIDARITY edge minted in the founding
  vocabulary. I.21's bracket receives its PATCH correction.
- **Clandestinity is a recomputed composition** (Q20): derived each tick from the line, measured
  practice, and capacity allocation. The player steers it only through what they actually do.
  Both failure directions are real: legal-only dies of decapitation exposure, illegal-only dies
  of isolation — the corpus's bidirectional loss, kept.

## 8. Money, capture, consequence

**Budget is graph-visible** (Q13): total plus source composition — base-sourced (dues) vs
bloc-sourced (institutional money: campaign finance, foundation grants, officeholder salaries)
vs illegal-economy vs party-enterprise — as declared Currency fields (typed-Currency storage's
second consumer). `Capacity` stays the kernel's deterministic spend-side allocator, reading
replenishment from the declared fields. Ship-simple licenses a thin first landing: total + one
base/bloc split.

**The funding mix IS the integration measurement** (§2's money channel). ADR176 (38)'s capture
vector is the theory's own mechanism, with two speeds from transformismo: *molecular* (key
figures bought; price scales inversely with cadre depth, gated by the mandate variable) and
*bloc* (funding-mix dominance moving the org's integration by small stages onto the
liquidationism absorbing state). Reversible: cut the grant and integration decays; a fully
bloc-funded org remains a site of struggle. Formal legal status carries zero mechanical weight.

**Consequence** (Q16): floor first — the revolutionary-victory gate counts org-sourced
SOLIDARITY edges (repairing the verified silent exclusion), and one canonical org-seeding
scenario enters both byte gates (`qa:regression` and the golden vault), closing the
`org_count=0` blind spot. Above the floor, the **full endgame redesign** is chartered: the five
canonical outcomes read the organizational apparatus broadly. That train is its own design
cycle with the Director (her reserved line); candidate inputs already on record include the
harvest's signed Caesarism index.

**Strain and fingerprints** (Q14): the profile a declared line implies is derived from the
trunk's own tag_deltas; strain is the measured gap between practice and that derived profile.
The twelve-fingerprint distinguishability criterion is a test-estate gate with a declared
distance — thresholds in conformance vectors, never in mechanics.

**Data grounding** (Q19): `fact_coercive_infrastructure` wires into StateApparatus
capacity/replenishment (honest scope: carceral rows only); the QCEW NAICS-813 tree seeds the
movement/civil-society footprint (honest scope: paid staff, not members; cell-anonymized).
OLMS/FEC/990 ingestion is **not chartered** — until it is, the capture-vector mechanics run on
authored magnitudes and the packs that use them say so in their `:material-basis` strings.

## 9. Chartered work this spec creates

| Train | Content | Gate |
|---|---|---|
| **Enum deffield row** | Spec chapter (§2.5-adjacent), loader, canonical hash layout for validated symbols; first consumer `organization/kind` | Language change — Director reviews the spec text; lands before the Organization pack needs a kind guard |
| **Vocabulary ceremonies** | `NodeType/ORGANIZATION` production registration; the six founding edges; the two hyperedge kinds (coalition, apparatus) | Each re-mint/mint is a declared ceremony per ADR176 (34) / AE (ii) |
| **Typed-Currency second consumer** | `organization/budget-*` fields on the Half-2 lane (already un-deferred) | Rides the Half-2 train's own ceremony |
| **Slice 4 first consumer** | Apparatus/coalition membership payload (integration degree, member role) — charters when the first mechanic reads a pair's payload | Director ADR per the standing hash-widening ruling |
| **`.bscn` hyperedge seeding** | Gap 5 closes with the apparatus construct (an initial apparatus roster must be authorable) | Lands with the apparatus pack |
| **State-verb differentiation** | The ten-sub-verb state estate, replacing the ported REPRESS collapse | Chartered now; D-record links the collapse defect to this train |
| **Endgame redesign** | The five outcomes read the organizational apparatus; Caesarism-index candidate | Director-led design cycle (reserved line); floor (win-gate repair + org golden) lands first |
| **I.16 amendment** | Replace "Organizations become institutions through formalization" with the memo §4.5 relational text | Amendment ceremony — escalated to the Director with the memo as evidence |
| **I.21 PATCH** | Status-bracket correction (half shipped, half live-and-buildable) | Amendment P drift-correction path |
| **Data wiring tiers 1+2** | Facility→StateApparatus; QCEW-813→movement footprint | ADR109 typed motions with sentinel rows |

## 10. What this spec does not decide

1. **Channel weighting** for institutionality (consent vs personnel vs money vs protection) —
   the texts give no arithmetic; the first apparatus pack proposes, the Director disposes.
2. **The apparatus object's fine grain** — hyperedge per (bloc × function) is the shape; how
   many functions ship in v1.0 is pack scoping.
3. **De-institutionalization** as a live path (the memo's reverse direction) — recorded, not
   designed.
4. **The endgame redesign's content** — deliberately left to its own Director-led cycle.
5. **The Notebooks 12–13 gap** — if acquired later, the I.16 amendment text gets one
   confirmation pass before ratification.
6. **OLMS/FEC/990** — withheld by ruling; revisit belongs to the Director.

## 11. Verification obligations

- The **org-seeding canonical scenario** in both byte gates is this contract's hash anchor —
  no Organization pack merges before it exists (the `org_count=0` blind spot closes first).
- Per-verb effect contracts (the dossier's layer C, `tests/contract/verbs/`) transcribe to
  conformance vectors at the port; the AS1/AS2/AS3 ordering guarantees get language-agnostic
  vectors (the mock-patching pin does not meet the rewrite-test bar).
- The twelve-fingerprint gate (Q14) and the hollowing-out/mandate observables land with their
  packs' vectors.
- Every ruling in §1 that changes a published doc (organizations.rst, the design standard's §2
  sub-mode text, I.16/I.21) carries a doc correction in the train that implements it — no
  teaching-the-wrong-answer intervals.
