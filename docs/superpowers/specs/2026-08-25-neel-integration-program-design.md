<!-- Vale: this normative design preserves governed identifiers and source terms. -->
<!-- vale Vale.Spelling = NO -->
<!-- vale ste.UnapprovedWords = NO -->
<!-- vale ste.NounClusters = NO -->
<!-- The normative contract, literal APIs, wire layouts, citations, and failure
     predicates require the same local heuristic exemptions as executable plans. -->
<!-- vale Vale.Terms = NO -->
<!-- vale ste.Ambiguity = NO -->
<!-- vale ste.Dictionary = NO -->
<!-- vale ste.Gerunds = NO -->
<!-- vale ste.LatinAbbrev = NO -->
<!-- vale ste.Modals = NO -->
<!-- vale ste.OneInstruction = NO -->
<!-- vale ste.PassiveVoice = NO -->
<!-- vale ste.ProcedureLength = NO -->
<!-- vale ste.Semicolon = NO -->
<!-- vale ste.SentenceLength = NO -->
<!-- vale ste.ThisPronoun = NO -->
<!-- vale strunk.ActiveVoice = NO -->
<!-- vale strunk.CommonlyMisused = NO -->
<!-- vale write-good.ThereIs = NO -->
<!-- vale write-good.TooWordy = NO -->
<!-- vale write-good.Weasel = NO -->

# Neel Integration Program Design

**Status:** Director-approved design, 2026-08-25. Governed by ADR232. Not
implemented.

**Authority:** `CONSTITUTION.md` v4.0.0, `NORTH_STAR.md`, ADR189, ADR220,
ADR223, ADR224, ADR226, ADR227, ADR230, ADR232, and the canonical Linear
portfolio. This design introduces no mathematical primitive, weakens no
prohibition, and does not change the reserved theoretical line.

**Purpose:** Convert the verified findings from Phil Neel's *Hinterland* and
*Hellworld*, together with the permitted Theory of the Party evidence, into one
playable Rust/BSL causal program. The program must produce organization,
territorial leverage, repression, and long-run construction through material
relations and attributed practice. It must never author the political answer
that play is meant to discover.

This is a program-level architecture specification. Each implementation slice
gets its own Linear owner, red-green-refactor loop, and behavioral contract.
The specification does not claim that the planned mechanics are live.

## 1. Current truth and problem

The current checkout implements Gate 2. Rust can adjudicate a detached weekly
tick, publish identity-free audit receipts, and compute graph and nominal-world
hashes. The following boundaries remain planned:

- the Rust `CommittedTickEnvelope` and PostgreSQL durability;
- an accepted next-week input ledger;
- durable actor-bearing action receipts and Archive outbox rows;
- player actions in Bevy;
- attributed membership payload access and hashing;
- live orders, inventory, freight, realization, and reproduction carriers;
- fog-safe gameplay territory projections;
- production consumers for dossiers, capacity allocation, affected
  populations, and Backfire;
- a live slow-fast-slow evidence trace.

Python remains the sole live database writer until the accepted one-way Rust
cutover. Frozen Python is a behavioral reference and periphery. It cannot own
or certify new gameplay mechanics.

The current organization content is a conformance fixture. Current production
can add aggregate output directly to class wealth. `RelationalTerritoryDossierV1`
is administrative and in-memory. Current T3 evidence is synthetic. A passing
content-set test proves fixture coverage, not runtime producer-consumer
closure.

The current sealed contracts have narrower authority than this program needs:

- graph-state layout v4 encodes sections `0x01` through `0x07`; the
  member-hyperedge payload slot is empty, private, and absent from graph bytes;
- nominal world-state layout v1 binds only graph hash, completed tick,
  allocator cursors, and the governed phase-schedule digest;
- Practice V1 admits only Organize, Agitate, and Mutual Aid against a
  `SOCIAL_CLASS` target, and all three return the stable unwired refusal in the
  live path;
- RTD V1 rejects actor, player-knowledge, fog, action, receipt, and Archive
  fields;
- ordinary `Mechanic` rules do not yet have default-deny exact footprints;
- the current dossier scope exposes resolved nodes, while the exposure helper
  can traverse true edges among them without checking `resolved_edges`.

The program must replace those gaps with a connected material circuit. It must
also prevent the predecessor design from reintroducing direct solidarity,
scalar recruitment, solidarity-funded action throughput, or membership-only
emergence as live authority.

## 2. Source authority and quarantine

The complete evidence ledger is
`docs/superpowers/research/2026-08-25-neel-source-study.md`. It records findings
H1-H10 and W1-W104, page anchors, Babylon questions, negative rulings, reading
receipts, and source digests.

The admissible sources are:

| Source | Digest and authority | Permitted use |
|---|---|---|
| Phil Neel, *Hinterland* (Reaktion, 2018), supplied PDF | SHA-256 `2799eb76f267551afa04a6bb76ffed4a89c5e1fc387c3744fcca3be3b00b4525`; `Observed` research evidence, not executable authority | Relational territory, logistics, uneven state capacity, situated events, reproduction, and organization. |
| Phil Neel, *Hellworld* (Brill, 2025), supplied PDF | SHA-256 `43127a54390f9fb798cb644f0e5af0f8228b79cc5c392b1b472b5dc96be8fe1e`; `Observed` research evidence, not executable authority | Production, circulation, labor, reproduction, ecology, infrastructure, repression, practical leadership, and construction. |
| Supplied Theory of the Party clipping | SHA-256 `373c2b594f932cbc7fcf590a784e6b48b9031a9bf7363e9b33a58fdc074454b1`; mixed admissibility | Lines 30-42 only, and only where H8, W10, W20, W36-W40, W45, or W62-W65 independently corroborates situated practice. |
| CPUSA *Organizers' Manual*, chapter 3 (1935), local HTML | SHA-256 `6d27b580c657f68f35e8d4b5b2ac6ea6b076050b1de7a82cb0b615cce12f44fb`; bounded research exception | Workplace and neighborhood rooting, local initiative, outside support without remote command, observation, deliberation, action, evaluation, and revision. |

Pannekoek, Bordiga, and Trotsky are excluded from source authority, design
rationale, vocabulary, coefficients, and mechanics. The exclusion is
transitive: omitting a name does not make a dependent concept admissible.

The Theory of the Party clipping's lines 44-46 and 74-130 remain quarantined.
Its historical or eternal party, invariant program, conclave, party ecosystem,
meta-organization, supra-subjectivity, noosphere, and social-brain ontology are
unusable. Fixed-program and phase-transition claims outside those line ranges
are also unusable.

The CPUSA exception does not authorize official hierarchy, Comintern command,
fractions, secrecy, entryism, member thresholds, prescribed unit sizes,
industrial essentialism, or guaranteed leadership. It does not change the
narrator/RAG denial of standalone CPUSA ingestion.

Every implemented value or relation must be `Observed`, `Derived`,
`Calibrated`, or `Designed`. A citation can constrain a causal question. It
cannot supply an executable coefficient without its own governed evidence and
contract.

The evidence ledger supplies this minimum mechanic-to-finding traceability:

- relational territory and dossiers: H1-H6, W57, W60, and W61;
- labor processes, strike proposals, and independent participation: H6-H9,
  W10, W36, W37, W42, W51, W66, W95, W98, and W99;
- the production-to-reproduction circuit: W1, W5, W11-W19, W23, W47, W49,
  W52-W56, W73-W76, W83, W87-W94, and W102-W104;
- situated organization and practical leadership: H8-H10, W20, W27-W45,
  W62-W67, W78, and W95-W99;
- long-run construction: W8, W9, W18, W19, W21, W22, W47, W53-W55,
  W72-W76, W81-W89, W97, and W102-W104.

These references constrain causal questions and exclusions. They do not make
the findings executable, prove a coefficient, or guarantee an outcome.

## 3. Supersession and preserved contracts

This design is the live successor to
`2026-08-23-neel-relational-territory-practice-design.md`. It preserves that
document as history and makes these dispositions explicit:

| Predecessor surface | Disposition |
|---|---|
| T0 theory corrections and the source boundary | Preserve. `ai/theory.yaml` 2.1.0 remains the current machine-readable boundary. |
| `RelationalTerritoryDossierV1` bytes, validation, administrative fixture, and typed gaps | Freeze. V1 remains administrative and cannot become a player dossier by relaxing its validator. |
| `PracticeIntentV1`, `PracticeInputAuthorityV1`, budget records, refusal records, and their exact vectors | Freeze as V1 contracts. Do not widen their enum or target bytes. |
| V1 Organize writing `SOLIDARITY`, solidarity-funded budget replenishment, and fixed solidarity decay | Supersede for the live Neel path. They remain unwired reference semantics and cannot enter campaign authority. |
| V1 Agitate writing the existing agitation accumulator | Preserve no automatic authority. A V2 practice may produce a bounded intermediate agitation pressure only when an exact downstream consumer exists and the rule cannot select solidarity, membership, tendency, legitimacy, or another political result. |
| One-intent-per-organization V1 admission | Do not inherit. V2 permits several distinct proposals when material resources and typed capacities support them. |
| `SfsTraceV1`, its classifier, proof profiles, mutations, and synthetic vectors | Freeze as generic synthetic evidence contracts. They do not prove live emergence. |
| Membership-only slow-fast-slow observable | Supersede as the program-level emergence claim. Live evidence must cover the whole material circuit. |
| `organization/p5-command-response` and `territory/command-pressure` | Retire from the future gameplay path. Repression must use dossiers, governed candidates, finite capacity, actual affected populations, and Backfire. |
| `territory/rooted-capacity` | Keep only as fixture-local conformance state. It is not event-attributed competence, leadership, political power, or a live gameplay resource. |
| Dyadic organization-to-class `MEMBERSHIP`, `community/c03*`, seeded `cadre-level`, seeded `cohesion`, and declared tendency attribution | Do not treat them as the ADR189 membership path or as consumers of new event products. A governed successor must consume native attributed membership and receipt evidence explicitly. |
| Existing generic structural verbs | Preserve their generic boundary. They do not supply organization-formation law or atomic organization/body/membership initialization. |
| Python-era `PoliticalFaction`, `cadre_level`, legitimacy-as-trust, and authored class-character documentation | Keep as historical reference only. It supplies no live Rust/BSL authority. |

All current V1 byte and digest vectors freeze before successor work. A new
decoder refuses a wrong domain, unsupported version, duplicate key,
noncanonical order, malformed length, or trailing bytes. No implementation may
edit a V1 golden to make a successor pass.

## 4. Program laws

1. The player inhabits one materially situated organization. The organization
   form is content, not a privileged kernel kind.
2. Player and non-player organizations use the same eligibility, resource,
   resolution, counteraction, and receipt machinery.
3. Geography is fixed. Claims, jurisdiction, access, occupation, control, and
   organizational reach are actor-identified overlays or relations.
4. Hinterland position is derived for one actor, question, scale, and tick. No
   graph field stores a hinterland type, stage, rank, band, or score.
5. Only a materially connected worker organization may propose a strike at a
   labor process. A proposal never commands worker participation.
6. Production, circulation, realization, and reproduction conserve their
   typed principals. No adjacency proxy can stand in for missing stock,
   freight, or delivery.
7. Strike, blockade, occupation, damage, and capital strike act through
   different immediate carriers.
8. Dossiers contain only lawfully resolved nodes and relations. A known node
   does not expose a hidden edge.
9. Centrality and topology measurements inform attention and capacity
   allocation. They never write an outcome.
10. Practice can produce attributed competence, repertoire, trust evidence,
    and ties. It cannot directly write membership, solidarity, legitimacy,
    leadership, victory, or political control.
11. Long-run communist construction is a reversible reorganization of
    production and reproduction. It is not a regime flag, progress variable,
    societal stage, or terminal latch.
12. Slow-fast-slow is a possible post-commit trace classification. BSL cannot
    read the classifier or an aggregate used to compute it.
13. Every important intermediate output has a real runtime consumer. A test,
    fixture, manifest assertion, chart, or log is not a consumer.
14. Equal canonical input bytes produce equal output bytes and hashes.
15. A contract breach aborts atomically and loudly.

## 5. Causal architecture and timing

The program uses the existing weekly phase schedule. It adds no scheduler
primitive or extra phase. Material intents submitted after tick `N` resolve in
tick `N+1` at the existing legal boundary before Material Base vitality and
production consumers. The phase-schedule digest changes only if the existing
observable ordering contract changes.

```mermaid
flowchart LR
    A["Tick N: actor-scoped proposal"] --> B["Accepted input ledger"]
    B --> C["Tick N+1: ResolvedPracticeBatchV2"]
    C --> D["BSL participation and immediate carrier"]
    D --> E["Labor and production"]
    E --> F["Orders and inventory"]
    F --> G["Freight, delivery, and realization"]
    G --> H["Needs and reproduction"]
    H --> I["PracticeProductReceiptV2"]
    I --> J["Tick N+2 product reducers"]
    J --> K["Future eligibility and independent decisions"]
    L["Actor dossier"] --> C
    M["Counteraction and repression"] --> D
    D --> M
    H --> M
```

`ResolvedPracticeBatchV2` is immutable canonical input to the tick. It binds
the committed input-authority ledger digest, the exact authority row for every
accepted intent, every accepted intent, resolution tick, resource-allocation
contract digest, content identity, and batch digest. The batch sorts by
complete intent identity, not submission order. Duplicate identities, stale
rows, missing rows, a wrong resolution tick, an authority mismatch, or a
ledger-digest mismatch fail before the detached working copy runs.

`PracticeIntentV2` is a separate closed wire contract with these fields in
fixed order:

```text
schema_version = 2
submit_after_tick
resolve_tick
input_authority_id
actor_org_id
practice_id
tagged_target_identity
proposal_nonce
quoted_content_digest
quoted_resource_contract_digest
parameters[]
evidence_digests[]
```

`PracticeIdV2` and the target tags have their own closed discriminant table.
They do not widen or reinterpret V1 bytes. `mobilize:strike` requires a
`LABOR_PROCESS` target. Blockade, occupation, damage, and capital strike use
their distinct typed target tags. A strike parameter cannot name a
participation or withholding value.

The complete intent uses the `babylon.practice-intent.v2` domain, fixed-width
big-endian integers, length-framed nested records, sorted unique evidence
digests, and no trailing bytes. One intent has at most 16 parameters, 64
evidence digests, and 16,384 canonical bytes. The resolved batch has at most
4,096 intents. These are `Designed` memory and fuel ceilings, not actor or
organization quotas. Maximum-plus-one fails before allocation or sorting.

V2 permits multiple proposals from one organization in the same resolving
tick. The unique key is `(resolve_tick, input_authority_id, actor_org_id,
practice_id, target_identity, proposal_nonce)`. A nonce distinguishes
genuinely different proposals but grants no priority. Actual organization
labor, time, knowledge, inventory, money, and typed capacity constrain
execution. A `Designed` static batch ceiling exists only for memory and fuel
safety; it is not a political quota.

Every proposal names its resource reservations or allocation request. The
admission transaction groups proposals by `(resolve_tick, input_authority_id,
actor_org_id)`. An individually malformed or impossible claim receives its own
submission refusal. If two or more otherwise valid proposals in one group make
incompatible exclusive claims that the governed allocator cannot jointly
resolve, the complete authority group receives
`authority_resource_conflict`; none of that group's proposals enters the
accepted ledger. This group refusal prevents submission order, digest order,
or nonce choice from selecting a winner.

Cross-actor competition is not a submission fault. Each structurally valid
request enters the accepted ledger. At material resolution, the versioned
shared reservation, clearing, or capacity-allocation contract consumes the
sealed requests and true available capacity. An unallocated or partly
allocated request becomes a committed material outcome. The allocator's
identity, ordering law, tie law, units, and digest freeze before Practice V2
activation. Canonical sort order never grants material priority.

The existing `ActionBudget` cannot substitute for a material cost or multiply
efficacy. V2 does not inherit solidarity-funded replenishment. If the shared
Gate 5 input rail retains an interface pacing quota, its value is a declared
`Designed` submission limit outside material resolution, and every practice
still pays its real typed costs.

## 6. Versioned state and identity

### 6.1 Graph layout v5

Graph-state layout v4 remains frozen. Layout v5 adds section `0x08` for
attributed membership payload rows. Rows sort by
`(hyperedge_id, member_node_id, qname)` and carry the declared typed value in
canonical form. The section is elided when empty, following the existing
additive-section law. A v4 graph with no attributed payload therefore keeps
identical bytes under v5.

Membership payload is not an ordinary hyperedge attribute and cannot alias a
dyadic edge field. Layout tests cover exact bytes, empty-section compatibility,
insertion-order invariance, backend parity, payload-only mutations, duplicate
keys, invalid types, nonfinite values, negative zero, and maximum-plus-one
refusal.

### 6.2 Nominal world identity v2

Nominal world-state layout v1 remains frozen. `NominalWorldHashV2` retains its
domain with layout version 2 and binds:

- graph-state hash and graph layout identity;
- completed tick and allocator cursors;
- governed phase-schedule and content digests;
- authoritative input-authority-ledger digest;
- accepted-input batch digest;
- every authoritative auxiliary-register digest;
- authoritative event and subject-bearing receipt-set digests;
- replay seed and campaign identity required by Gate 3.

The graph hash remains graph-only. `NominalWorldHashV2` is not a competing
graph hash. World and receipt construction follows this acyclic order:

1. `world_before` is the previous committed `NominalWorldHashV2`, or the
   versioned genesis identity.
2. The engine validates the accepted batch and authority ledger, then executes
   the detached tick.
3. `PostStateBasisDigestV2` binds the completed tick, graph, registers, events,
   cursors, authority ledger, content, schedule, and replay identity. It
   excludes subject-bearing receipts and the final world hash.
4. Each subject-bearing receipt binds `world_before` and
   `PostStateBasisDigestV2`. It never contains the final world hash.
5. The engine computes the canonical receipt-set digest.
6. `NominalWorldHashV2` binds the post-state components, accepted-input digest,
   and receipt-set digest.
7. The `CommittedTickEnvelope` binds that final world identity plus its ordered
   state, event, receipt, conservation, boundary-flow, checkpoint, and
   Archive-outbox rows.

No byte preimage depends on its own digest. The graph, register,
input-authority, receipt, and post-state-basis contracts freeze before the
final World V2 and envelope goldens. The envelope byte contract freezes before
PostgreSQL writer cutover.

### 6.3 Authoritative storage split

The graph stores identity and persistent relations: nodes, dyads, native
hyperedges, attributed membership, claims, access relations, and relational
attributes. Separate typed auxiliary registers store exact high-volume
principals and transitions: inventories, orders, reservations, shipments,
money claims, settlements, needs, care-time flows, facility condition,
maintenance obligations, and production commitments.

Each principal has exactly one authoritative home. Graph relations may point
to a register identity but cannot duplicate its quantity. World identity binds
every register digest.

`ActorKnowledgeRegisterV2` is another authoritative auxiliary register. It
stores bounded observations, not true graph replicas or dossier results. Each
row binds observer actor, observed subject or relation, typed observed value or
uncertainty, provenance, observation tick, validity interval, visibility, and
as-of policy. Rows use a separate canonical domain and sort by full observation
identity. `NominalWorldHashV2` binds the register digest.

Surveillance and other lawful observation producers can add or expire only
declared rows in this register. `RelationalTerritoryDossierV2`, centrality, and
other actor-scoped projections are pure derived views over the committed graph,
auxiliary registers, and one actor's knowledge-register slice. They are not an
authoritative register and cannot write knowledge back into their inputs.

### 6.4 Successor contracts

The program introduces these successors without widening V1:

- `PracticeInputAuthorityV2` and its authoritative ledger;
- `PracticeIntentV2` and `ResolvedPracticeBatchV2`;
- the versioned resource-reservation and capacity-allocation contract;
- graph-state layout v5 with attributed membership;
- `ActorKnowledgeRegisterV2` and its observation-row bytes;
- `NominalWorldHashV2` and the first complete `CommittedTickEnvelope`;
- `PracticeProductReceiptV2` plus fog-safe receipt projections;
- `RelationalTerritoryDossierV2` for gameplay;
- a live evidence-profile successor that binds the complete material trace.

Each successor needs an accepted ADR, exact language-neutral byte layout,
goldens, refusal vectors, and cross-process replay before a dependent mechanic
can land.

## 7. Player habitation and organization identity

Campaign setup binds the player seat to one existing active `ORGANIZATION`.
Habitation grants input authority only. It grants no legitimacy, truth,
vanguard status, command over other organizations, or exception from fog and
costs.

Habitation grants no knowledge beyond the inhabited organization's committed
dossier.

`PracticeInputAuthorityV2` is authoritative campaign state, not a client
setting. Its canonical ledger row binds the campaign, authority kind,
authority identity, actor organization, effective tick interval, and decision
content or policy digest. `NominalWorldHashV2` binds the sorted ledger digest.
An intent's `input_authority_id` and `actor_org_id` must match one active row at
`resolve_tick`.

The row uses the `babylon.practice-input-authority.v2` domain and writes these
fields in fixed order:

```text
schema_version = 2
campaign_id
authority_kind
input_authority_id
actor_org_id
effective_from_tick
effective_through_tick_exclusive
decision_content_digest
```

Rows sort by `(campaign_id, input_authority_id, effective_from_tick)`. Duplicate
keys, overlapping effective intervals for one authority, one authority mapped
to two organizations in the same tick, a wrong authority kind, an empty
interval, or trailing bytes refuse. The closed authority-kind table contains
only `PLAYER_SEAT` and `DETERMINISTIC_POLICY` in V2.

Exactly one active `PLAYER_SEAT` row exists in the first gameplay campaign,
and it maps to exactly the inhabited organization. A `DETERMINISTIC_POLICY`
row is the equivalent non-player source authority and maps one declared policy
to exactly one organization. One authority cannot propose for a second
organization. Every player and non-player proposal passes the same later
material eligibility and resolution checks.

The first gameplay version does not support a mid-campaign controller
reassignment. A later habitation change requires its own committed
organizational process, consent and succession rules, fog-safe receipt, and
versioned contract. Changing a client-side `player_org_id` is never sufficient.
Formation does not mint an input-authority row. A new organization remains
unable to submit proposals until its participants commit a separately
versioned authority assignment.

An organization actor has two joined identities:

1. one `ORGANIZATION` node supplies the actor identity used by intents,
   dossiers, resources, and receipts;
2. one native `ORGANIZATION_BODY` hyperedge contains that node as its
   distinguished actor member and contains every attributed participant cohort
   as separate members.

The sealed graph must map each active organization node to exactly one active
body hyperedge through that shared membership. Participant roles, activity,
commitment, visibility, and other governed fields live on their constitutional
member-hyperedge incidences. The existing dyadic organization-to-class
`MEMBERSHIP` edge is not this object and cannot be relabeled as ADR189
attributed membership.

An organization is materially situated through attributed members and actual
relations such as workplace participation, tenancy, provision, communication,
use, access, and presence. A label such as party, union, tenant formation,
council, mutual-aid network, or affinity group changes narration and lawful
content declarations. It does not mint different physics.

An organization with no current material witness cannot use a stale label to
act at a locus. A remote supporter can propose agitation, provision, defense,
communication, or another practice for which it has its own lawful relation.
It cannot impersonate the organization or workers situated there.

No maximum organization count expresses theory. A finite implementation
ceiling may bound memory, iteration, and denial-of-service exposure. The limit
must be `Designed`, byte-bound, tested at maximum plus one, and must not become
a formation quota, rank, or player-facing political law.

## 8. Labor processes and attributed labor

`LABOR_PROCESS` is a new governed hyperedge kind using the existing native
hyperedge primitive. It does not add mathematics. The hyperedge identity is
the target of a strike proposal.

The first live schema requires:

- at least one worker cohort member;
- at least one material locus member, such as a territory or workplace;
- an active process identity and declared output function;
- worker-incidence payload for labor allocation and withholding;
- provenance and evidence classes for seeded relations.

Facilities, products, owner or employer actors, energy inputs, and routes join
the same hyperedge or point to it through typed relations when their live
consumer lands. Co-location alone never establishes productive linkage.

Worker incidence uses exact integer shares:

```text
0 <= allocation_ppm(c, p) <= 1_000_000
sum(active p, allocation_ppm(c, p)) <= 1_000_000
0 <= withholding_ppm(c, p, t) <= 1_000_000
```

One part per million is a `Designed` representation precision. Values are
stored as exact integers in the attributed-membership contract. A total below
one million means labor capacity outside modeled processes. A total above one
million refuses before execution.

For declared labor-capacity quanta `L(c,t)`, checked integer arithmetic derives:

```text
allocated(c,p,t) = floor(L(c,t) * allocation_ppm(c,p) / 1_000_000)
withheld(c,p,t) = floor(allocated(c,p,t) * withholding_ppm(c,p,t) / 1_000_000)
available(c,p,t) = allocated(c,p,t) - withheld(c,p,t)
```

Division remainders stay in unallocated or available capacity; they never
disappear or become withheld. Overflow, a negative value, or an invalid share
fails atomically.

An organization has the material relation required to propose a strike only
when the sealed snapshot proves all of these facts:

1. the active `PracticeInputAuthorityV2` row maps the submitting authority to
   the actor organization; a player row maps only to the inhabited
   organization, and a non-player policy row maps only to its declared
   organization;
2. the actor node and an affected worker cohort share the actor's unique active
   `ORGANIZATION_BODY` hyperedge, and the cohort's attributed incidence is
   active;
3. that cohort is a worker member of the target `LABOR_PROCESS` with positive
   allocation;
4. the organization has an attributed workplace or territorial presence at a
   material locus of that process.

`WAGES` may record employment, appropriation, or payment. It cannot substitute
for this witness or grant strike authority by itself.

`PRESENCE`, adjacency, centrality, a declared line, a solidarity relation,
player control, or generic class identity cannot substitute for the complete
material-connection witness.

## 9. Strike proposal and worker participation

`mobilize:strike` is a V2 proposal, not an order. Admission proves the
initiating organization's material relation and seals the proposal for the
next tick. It does not set participation or withheld labor.

Eligibility does not require a prior mandate and does not imply one. It grants
only the right to put the proposal before the affected workers through their
shared material relation.

At tick `N+1`, BSL resolves each affected cohort incidence independently. The
initiator cannot supply `withholding_ppm`. The resolver may read only governed,
attributed facts such as grievance, prior practice, directed reliability
evidence, provision, strike funds, replacement risk, lost income, repression,
household needs, concessions, and observed participation. Unknown facts remain
unknown. No rule may read a desired trajectory, elapsed campaign stage, global
strike health, or sigmoid.

Workers may join, continue, reduce, abstain, or leave each week. The state is
the current incidence-level withholding commitment, not a strike stage. A
change from zero total withheld labor to positive creates an attributed strike
episode. A change back to zero ends that episode. A later restart receives a
new episode identity.

Only the worker-participation resolver may write
`withholding_ppm(c,p,t)`. Its completed incidence updates occur before
available labor and production are calculated.

The initiating organization may recommend, withdraw its recommendation,
negotiate, provision, communicate, or defend. It cannot directly end worker
withholding. Repeated calls do not multiply participation. Sympathetic workers
act through their own labor-process relations. A general strike is a derived
projection over several independently active process episodes.

The strike's only immediate material carrier is available labor at the target
process. It cannot directly write output loss, scarcity, price, concessions,
repression, solidarity, membership, legitimacy, victory, or political
control.

Actual withheld functions, replacement labor, inventory buffers,
substitution, freight rerouting, repression, exhaustion, and reproduction
determine later consequences. A highly central supporter without the labor
relation still cannot initiate the strike, although it may continue any
separately eligible support practice.

## 10. Distinct disruptive practices

Every disruptive practice uses the common intent, cost, timing, receipt, fog,
and counteraction rails. Each retains a distinct eligibility predicate and
immediate carrier:

| Practice | Required material relation | Immediate carrier |
|---|---|---|
| Strike | Attributed worker-organization connection to the target labor process | Incidence-level labor withholding and derived available labor. |
| Blockade | Situated participants and means with access to a named route, node, shipment class, or access point | Freight, route, or access capacity at that target. |
| Occupation | Situated participants physically present at the named facility, territory, or access locus, with resources to maintain presence | Use, entry, occupancy, or operation relations; ownership does not change automatically. |
| Damage | Attributed reach or access plus the required means at a specific stock or facility | Installed-stock quantity or condition plus explicit waste and repair requirements. |
| Capital strike | An actor that owns or controls the named investment, credit, procurement, or production commitment | The actual commitment, order, credit, or working-capital carrier. |

No generic `disruption-power` value can substitute for these paths. Equal
nominal cost or magnitude must still yield observably different immediate
receipts and downstream traces.

Termination is also material and distinct. A strike episode ends when its
total withholding reaches zero. A blockade ends when the situated obstruction
or its capacity effect no longer exists. An occupation ends when attributed
occupancy and maintained access reach zero. Damage persists as condition and
repair obligations after the damaging action ends. A capital strike ends only
when the withheld commitment is restored, replaced, canceled, or otherwise
resolved through its own contract.

## 11. Conserved production-to-reproduction circuit

The live path is:

```text
labor and inputs -> production -> inventory -> order reservation
-> dispatch -> freight and loss -> arrival -> delivery and access
-> realization and settlement -> needs and reproduction
```

Production consumes declared input quantities, available labor, energy,
productive knowledge, and operable facility capacity. Output enters typed
inventory. It does not become wealth, welfare, access, or reproduced labor
directly.

Conserved principals use checked integer or fixed-scale quantities with a
declared `unit_id` and scale. Existing Currency uses its checked `i128` lane.
Binary64 may express finite dimensionless intensities, rates, and derived
projections. It cannot be the unaccounted principal of a transfer. Any rounding
remainder enters an explicit residual, waste, or retained-source row.

The required local laws are:

1. Production debits every consumed input and labor-time quantum before it
   credits output and attributed waste.
2. An order creates demand and a claim. It creates no goods.
3. Reservation partitions existing inventory into reserved and unreserved
   quantities without changing their sum.
4. Dispatch debits source inventory and credits an identified in-transit row
   by the same quantity.
5. In-transit opening plus dispatch equals arrival plus loss plus in-transit
   closing for each good and shipment identity.
6. Arrival credits destination inventory before delivery or consumption.
7. Delivery and lawful access precede need fulfillment.
8. A commodity sale realizes only after accepted delivery. Settlement then
   debits a payer and credits a payee or records an explicit default.
9. Care and services consume finite attributed labor time when performed. They
   cannot be stockpiled as goods.
10. Reproduction consumes delivered and accessible goods or performed services
    and changes later health, needs, care burden, labor availability, and
    participation constraints only through named intermediates.
11. Spoilage, damage, freight loss, waste, and ecological residue remain
    attributed destinations.

Substitution and rerouting require a real alternative input, process, route,
inventory, skill, access relation, spare capacity, switching cost, and delay.
An authored `throughput`, `alternate`, `buffer`, or `reproduction` field on an
`ADJACENCY` fixture cannot stand in for those carriers.

For each tick and principal, the committed envelope records a zero residual or
the tick fails. Conservation checks use exact quantities. Derived binary64
projections have a separately documented tolerance policy and never decide
ledger closure.

## 12. Reproduction, access, and affected populations

Needs are typed by population and context. Food, shelter, energy, care,
mobility, health, and other shipped needs remain separate. Babylon does not
collapse them into a universal welfare score.

Access is distinct from ownership, proximity, production, delivery, and legal
eligibility. The first live access modes are mechanically explicit:

- commodity sale;
- entitlement;
- public provision;
- cooperative allocation;
- reciprocal provision;
- common access.

Each mode names its decision relation, obligations, exclusion rules, and
settlement behavior. Actor-scoped planning consists of forecasts, proposals,
reservations, and independently accepted commitments. No omniscient optimizer
chooses for affected populations or organizations.

Every shortage, ration, exclusion, service failure, coercive act, and burden
produces affected-population rows of the form
`(cohort_id, affected_quantity, causal_relation, evidence)`. Equal node counts
cannot substitute for population weight. Missing population attribution makes
the claimed population effect unavailable, not zero.

Reproduction never directly writes consciousness, solidarity, legitimacy,
membership, organization formation, victory, or political control.

Extraction, production, consumption, construction, freight loss, damage, and
repair produce attributed ecological debits, residues, or waste where their
material contracts require them. Regeneration, recycling, stewardship, and
repair consume explicit labor, material, energy, and time. Ecological burdens
change later needs, access, health, route or facility capacity, and production
possibilities through named consumers. They cannot determine a political
result or feed a collapse meter.

## 13. Relational territory and gameplay dossiers

`RelationalTerritoryDossierV1` remains the frozen administrative contract.
`RelationalTerritoryDossierV2` is the gameplay successor. It binds:

- actor identity and dossier digest;
- committed tick and `NominalWorldHashV2`;
- fog policy and knowledge-context digests;
- projection version, scale, relation family, flow family, target, and decision
  question;
- visible signals, uncertainty, eligible actions, expected receipts, and
  Archive subjects;
- a fog-safe receipt projection and projection hash.

The projection computes independent facets from typed relations. A territory
may be central to one flow, peripheral to another, near at one scale, and far
at another. Near and far are human interpretations in the projection. They are
not enum values, persisted fields, rule guards, or bonuses.

The dossier scope consists of resolved nodes **and resolved edges,
hyperedges, member incidences, and attribute observations**. Each visible
relation observation binds its typed value or bounded uncertainty, provenance,
observation tick, and as-of policy. Topology code may traverse and weight only
that projected subgraph. It cannot reread the true relation or current
authoritative weight. Changing a hidden relation, incidence, attribute, or
true weight while the dossier stays identical must not change the actor's
measurement. A full-information dossier must equal the ground-truth
projection.

Gameplay centrality is transient and keyed by actor, dossier digest, committed
tick, projection version, relation family, flow family, target, scale, and
decision question. Lawful measures include flow-weighted betweenness, removal
loss, cut exposure, coordination reach, and replaceability. No single measure
is the canonical territorial score.

Each gameplay surface must answer a decision question and disclose uncertainty.
An administrative RTD display remains visibly exempt and cannot satisfy the
player-agency milestone.

## 14. Repression, capacity, and Backfire

Employers, courts, police, brokers, and other declared actors have separate
dossiers, jurisdictions, access relations, instruments, and finite capacity.
No generic state actor receives omniscient graph truth.

The governed candidate builder derives candidates from the actor's dossier,
lawful jurisdiction, instrument eligibility, observable target relations, and
typed costs. Callers cannot supply arbitrary expected yield. Topology
measurements may rank or filter candidates for attention. They cannot decide
whether repression succeeds or what it causes.

Capacity allocation funds a selected set of candidate-instrument pairs under
checked limits. Instruments remain distinct:

- surveillance can add only bounded observed facts to the actor's dossier;
- an injunction creates a legal burden and enforcement pathway;
- a concession changes real terms or access but cannot settle participation by
  decree;
- replacement consumes reserve labor, money, training, transport, and time
  before it adds usable labor capacity;
- a lockout changes workplace access or available capacity;
- targeted coercion records an attributed incident and actual affected
  population;
- a sweep consumes greater capacity and resolves its broader affected
  population rather than declaring every target affected.

Funded response resolves against true material state, but its receipt reveals
only observations lawful for each audience. The actual population rows and
material effects feed the existing structural Backfire calculation through a
population-weighted successor. Backfire changes later pressures, observation,
trust evidence, participation conditions, and organization practice only
through named consumers.

No instrument or Backfire rule may directly stop a strike, set participation,
or write solidarity, membership, legitimacy, victory, leadership, or political
control. A response observed during tick `N` influences a new actor decision no
earlier than tick `N+1`, unless the actor already possessed that fact.

The legacy `strike` name in `exposure.rs` means coercive decapitation, not labor
withholding. Its successor is named `decapitation_exposure`; it cannot share the
practice identity or receipt vocabulary of a worker strike.

## 15. Attributed practice products

After material execution and before atomic commit, Rust builds one canonical
`PracticeProductReceiptV2` for each accepted practice resolution. It also
builds canonical transition-batch receipts for later independent participant
decisions and one canonical `MembershipTransitionReceiptV1` for each
authoritative membership-incidence mutation. A multi-tick strike keeps its
episode identity while each tick receives distinct batch identities. The
practice-product receipt has one closed cause tag:

- `PROPOSAL_RESOLUTION` binds the originating intent, input authority, and
  proposing organization;
- `PARTICIPANT_TRANSITION_BATCH` binds the episode, tick, target process,
  participation-resolver identity, canonical chunk index, and digest of its
  sorted decision-owner subrecords. Each subrecord binds one worker incidence,
  prior commitment, resolved commitment, decision evidence, and outcome. The
  batch can retain the originating proposal as historical provenance, but
  cannot attribute any transition to its proposer.

The record contains:

- cause, episode, tick, target, and material-locus identities;
- proposing actor identity only for a proposal resolution, or decision-owner
  subrecords only for a participant-transition batch;
- attributed participant cohorts and membership identities;
- performed functions and topology witnesses;
- labor, inventory, money, time, capacity, and route debits;
- delivered effects, failures, counteractions, and affected populations;
- observer visibility and audience policy;
- content, input-batch, graph, register, `world_before`,
  `PostStateBasisDigestV2`, and causal-path digests.

Product reducers attribute competence, repertoire, trust evidence, and ties to
the subjects that performed or lawfully observed the recorded function. A
proposer receives no product for a worker-owned transition unless another
receipt attributes a function that the proposer actually performed. Reducers
process each transition subrecord independently; batching creates no shared
decision, tie, trust evidence, or product attribution.

`PracticeProductReceiptV2` contains no membership result, solidarity result,
victory, legitimacy, leadership, party state, or political-control result. The
existing identity-free `AuditReceipt` remains the proof of BSL role and
footprint. It is not replaced by either subject-bearing receipt.

`MembershipTransitionReceiptV1` is a separate canonical record, not a practice
outcome. It binds one create, update, or end mutation to episode, tick,
participant-owned resolver, organization-body hyperedge, member node,
membership incidence, independent-decision-evidence digest, and canonical
prerequisite-receipt-set digest. A create carries no prior payload and one
resulting attributed-membership payload; an update carries distinct prior and
resulting payloads; an end carries one prior payload and no resulting payload.
Every other presence combination refuses before graph publication.

Its
domain-separated identity commits those fields and both optional payloads in
that fixed order, so the same incidence mutation in another episode or tick
cannot collide. The record has no variable collection: its optional payloads
use the exact graph-layout-v5 membership-payload bytes and inherit their
maximum. The successor byte ADR derives and freezes the resulting receipt-byte
maximum before the complete envelope freezes.

The membership-transition receipt commits in the same envelope as its graph
mutation and counts against the subject-bearing receipt, row, and byte budgets
below. Archive and actor-scoped projections consume it; each projection redacts
unobserved member, organization, payload, and decision evidence. Product and
formation reducers may use the canonical receipt as evidence on a later tick,
but the receipt cannot itself author solidarity, legitimacy, leadership,
party state, political control, or another downstream result.

Campaign sealing bounds active formation and later membership candidates.
Candidate admission reserves one membership-transition receipt and its
worst-case bytes for every incidence the resolver can mutate. If the complete
atomic candidate set cannot fit, admission refuses before participant decision
resolution; execution cannot publish a partial organization, membership set,
or receipt set.

The subject-bearing receipt builders consume actor-bearing event and resolver
payloads before the identity-free audit projection reduces them. The tick
cannot discard actor, participant, target, value, observer, or
affected-population identity before all required receipts succeed.

The authoritative receipt commits in the same envelope as the final world but
does not contain that final world digest. Actor-scoped projections
redact unobserved participants, relations, and effects. A public projection
cannot reveal a private participant merely because the canonical receipt knows
them.

The receipt domain is `babylon.practice-product-receipt.v2`. Its payload writes
the cause tag, common scalar identities, and cause-specific prelude in fixed
order. A transition-batch prelude contains its chunk identity followed by its
sorted decision-owner subrecords. The remaining length-framed arrays follow in
this order: participants, functions, loci, topology witnesses, resource debits,
realized effects, counteractions, affected populations, observers, and Archive
subjects. Every order-insensitive array sorts by complete canonical identity
and rejects duplicates. Audience-ordered display data belongs only in a
projection, never in the authoritative receipt.

One receipt admits at most 4,096 decision-owner subrecords, 4,096 attributed
participant cohorts, 256 performed functions, 1,024 loci, 4,096 topology
witnesses, 65,535 debit rows, 65,535 effect or counteraction rows, 65,535
affected-population rows, 4,096 observer rows, 4,096 Archive subjects, and
16,777,216 canonical bytes. These `Designed` ceilings bound fuel and memory;
they do not limit the number of people a cohort represents. A builder refuses
at maximum plus one and publishes no partial bytes.

One tick admits at most 65,535 subject-bearing receipts, 1,048,576 total nested
receipt rows, and 67,108,864 total canonical receipt bytes. The builder checks
all three cumulative counters with checked arithmetic before it reserves or
appends memory. A maximum-plus-one count, row, byte, or serialization failure
aborts the detached tick and publishes no partial receipt set.

Those downstream caps have matching upstream laws. One sealed world has at
most 262,144 active receipt-producing `(episode_id, participant_incidence_id)`
identities, and one identity can produce at most one participant-transition
record per tick. A transition receipt canonically batches up to 4,096 sorted
decision-owner records with the same episode, tick, target process, and resolver.
The engine sorts the complete record sequence by decision-owner identity and
splits it into consecutive 4,096-record chunks. The zero-based chunk index and
chunk digest do not determine identity alone.

The domain-separated batch
identity commits, in fixed order, the episode, tick, target process, resolver,
zero-based chunk index, and chunk digest. Batching never combines or changes the
independent decisions.

Fog projection evaluates each decision-owner subrecord separately. It can
expose only the lawfully visible subrecords in a separate canonical projection.
That projection exposes no authoritative chunk index, record count, or batch
digest from which an audience could infer hidden gaps. It has its own digest
and cannot stand in for the authoritative batch identity.

Every practice declaration supplies finite worst-case receipt counts, nested
rows, and canonical bytes. Campaign sealing proves that already-active episodes
fit the aggregate tick budget. Accepted-input admission reserves the declared
worst-case remainder for every new proposal and refuses
`receipt_capacity_exceeded` before the ledger if the complete candidate batch
cannot fit. Episode creation carries its reserved recurring footprint until
the episode ends. Thus no valid committed world depends on a later routine
capacity abort; the runtime caps remain a defense against a contract breach.

Outcome codes distinguish applied, partial, no-participation, materially
failed, and countered episodes. They are descriptions of execution, not
success certificates. Pure submission refusals remain non-authoritative
refusal records rather than episode receipts. Each authoritative field must
have a named validator, gameplay consumer, fog projection, or Archive
explanation; decorative receipt fields fail consumer closure.

At tick `N+2` or later, separate BSL reducers consume committed receipts:

- **Competence** retains demonstrated functions, context, required resources,
  failure modes, and later decay or maintenance. It is not XP or one power
  value.
- **Repertoire** records performed, observed, adapted, or transmitted practice
  variants and their material requirements. Availability never guarantees
  efficacy.
- **Directed trust evidence** records witnessed promises, delivery, burden
  sharing, refusal, and harm. It is actor-scoped evidence, not global trust or
  legitimacy.
- **Ties** preserve actual communication, provision, coordination, care,
  defense, or accountability relations. Co-participation alone cannot mint a
  pairwise clique or generic solidarity. A tie declares its endpoints,
  interaction kind, visibility, maintenance or severance law, and consumer.

Each product has a named consumer in later eligibility, execution cost,
worker-participation conditions, organization capacity, reproduction,
dossiers, repression, Backfire, Bevy decisions, or Archive explanation.

## 16. Organization formation, leadership, and party development

Organization formation is a separate participant-owned resolver. A candidate
requires attributed evidence of recurring necessary function, maintained
communication or accountability, committed labor or resources, and a shared
decision relation that persists beyond one episode. Every affected participant
or cohort independently resolves continued coordination.

Eligibility does not create an organization. Only the formation resolver may
atomically mint the `ORGANIZATION` actor node, its unique `ORGANIZATION_BODY`
hyperedge, the distinguished actor incidence, participant incidences, and its
required resource and capacity state after those independent decisions. A
designed recurrence window or resource ceiling is a transparent play
constraint, not a Neel-derived scientific threshold. No generic structural
verb, direct practice effect, or legacy dyadic membership edge may perform a
partial formation.

There is no result-writing `form-organization` intent. Actors may propose the
concrete recurring functions, commitments, or decision relations from which
the participant-owned resolver can later derive a formation candidate.

One event, a shared slogan, a shared enemy, nominal victory, nominal defeat,
or player desire cannot satisfy the recurring-function requirement. The
episode that produces a receipt cannot consume its own uncommitted product as
a formation prerequisite.

Practical leadership is an actor-scoped adoption history:

1. one actor demonstrates or proposes a practice;
2. another actor can lawfully observe it;
3. the second actor independently adopts, adapts, or coordinates through it;
4. later receipts show which need or problem it did or did not solve.

Babylon stores no `leader=true`, leadership rank, influence score, or
centrality-to-legitimacy conversion.

Practical leadership may be mistaken, reactionary, exclusionary, contested,
or short-lived. Its lawful consumers are proposal attention, communication
routing, coordination invitations, and actor-scoped dossier assessment. It
never grants command.

Party development is a fog-safe projection over plural autonomous
organizations' communication, training, delegation, reciprocal provision,
contested deliberation, transmitted repertoires, and coordinated practice.
The projection shows coordination gaps, reliable partners, disagreements,
refusals, and possible proposals. It does not mint a privileged `PARTY` kind,
party relation, unified movement actor, stage, rank, score, meta-organization,
or social brain.

The player-facing decision question is: which organizations can coordinate on
which concrete function, under which commitments and observed reliability,
with which refusals, missing relations, and unresolved disagreements? Shared
names, programs, symbols, enemies, federations, or electoral alliances cannot
merge the actors or answer that question by themselves.

Organizations can persist, split, federate, dissolve, succeed one another, or
change function only through attributed relations and committed practice. They
do not follow an organization lifecycle stage machine.

Defeat can produce competence, ties, and organization. Victory can consume or
destroy them. Neither outcome proves political content.

## 17. Long-run communist construction

Communist construction is the continuing, contested capacity to reproduce
life through more collective and participant-governed relations. The player
cannot invoke `build-communism`. Through the inhabited organization, the
player may propose next-week provisioning, repair, repurposing, construction,
access, or coordination practices.

Affected workers, populations, and organizations independently accept,
modify, or refuse commitments. Execution consumes actual labor, skills,
knowledge, inventories, energy, money, route capacity, access rights, and
time.

Construction is represented by independent facts and receipts:

- proposal and accepted commitments;
- financing and contracts;
- reserved labor, materials, energy, and routes;
- installed stock and connection;
- operable capacity and current utilization;
- maintenance, repair, access, use, debt, waste, and ecological burden.

There is no project-stage enum. Announcement, financing, installation,
ownership, operation, utilization, and social benefit remain different facts.
Installed stock becomes usable only when energy, input, skill, connection,
labor, access, and maintenance predicates hold.

Maintenance and repair consume real resources. Deferred maintenance can reduce
condition and operable capacity or cause cascading failure. Repurposing needs
attributed knowledge, lawful use or access, actual inputs, and independently
accepted commitments. Infrastructure may create capacity, dependence,
exclusion, debt, coercive exposure, and ecological damage at the same time.

Actor-scoped projections may derive which needs were fulfilled through which
access relations, whose decisions governed allocation, what commodity
payments or obligations remained, who was excluded, and who bore each burden.
Commodity dependence or decommodification may be evaluated post-commit from
those relations. It cannot be stored as a BSL-readable progress variable.

There is no `communism`, `autonomy`, `decommodification`, `political-control`,
or revolution-progress field; no societal stage; no monotonic ladder; and no
terminal communist victory latch. Constructed relations may generalize,
stagnate, decay, become exclusive, be captured, or reverse.

## 18. Failure and atomicity

The system distinguishes three classes of failure.

### 18.1 Submission refusal

Malformed, wrong-version, unauthorized, stale, duplicate, noncanonical,
over-limit, wrong-target, over-budget, or structurally ineligible input refuses
before it enters the accepted ledger. The refusal changes no graph, register,
budget, receipt, event, outbox, allocator cursor, or tick.

An `authority_resource_conflict` is also a submission refusal. It rejects the
complete authority-resolution group before any member enters the ledger. It is
not a contract abort and does not select a proposal by canonical order.

A refusal message uses only facts lawfully visible to the submitter. A hidden
route, participant, countermeasure, or stock cannot leak through a precise
error. When a lawful diagnosis would reveal hidden truth, the response reports
that eligibility cannot be established under current knowledge.

### 18.2 Material outcome

A canonical authorized proposal can be valid even when its result is
uncertain. Missing material means, worker nonparticipation, refusal by another
organization, substitution, counteraction, loss, or defeat becomes a committed
partial or failed outcome. It is not an engine error. The receipt records only
lawful preparatory costs, actual material changes, and audience-visible facts.

The engine does not roll back a world merely because the actor's objective
failed.

### 18.3 Contract abort

Unsupported version, malformed committed input, invalid membership share,
nonfinite value, negative principal, overflow, unordered iteration,
undeclared effect, dead permission, conservation residual, dossier mismatch,
content or schedule mismatch, evaluator error, event-reservation error,
receipt-count, receipt-row, or receipt-byte overflow, receipt-builder failure,
persistence error, or outbox error aborts the whole boundary that owns it.

A malformed, wildcard, forbidden, omniscient, duplicate, or dead read/write
declaration rejects content loading before a campaign or tick can start. A
runtime accessor that attempts an undeclared source, field, subject, actor,
dossier, or as-of scope is a contract abort of the detached tick. Neither case
publishes state, events, receipts, cursors, or completed time.

The detached Rust tick publishes no graph, event, receipt, cursor, or completed
time after an adjudication failure. The future persistence transaction
publishes no state, input consumption, receipt, outbox row, or `tick_commit`
after a durability failure. In-memory atomicity and database atomicity remain
separate contracts until Gate 3 lands.

## 19. Exact causal footprints and consumer closure

Every Neel rule declares one existing causal role and one evidence class. The
program adds no new role. Every role, including ordinary `Mechanic`, opts into
exact input-specific and output-specific footprints. Each declared read names
the authoritative source or actor-scoped dossier projection, typed field,
as-of tick, and lawful subject scope. Each permission equals one actual unique
read or effect. Missing, extra, wildcard, duplicate, omniscient, forbidden,
and dead permissions fail before execution.

Direct-write prohibitions apply to every executable rule regardless of causal
role. No role name or exact-footprint declaration creates an exception. A rule
cannot directly write production loss, scarcity, concession outcome,
legitimacy, victory, repression success, leadership, party status, communist
status, or political control.

Named sole-writer boundaries remain narrow. The worker-participation resolver
alone writes withholding. Conserved circuit transitions alone write their
immediate principals. The participant-owned formation resolver alone creates
an organization and its initial attributed membership atomically after the
independent decisions. A participant-owned membership resolver alone can add,
change, or end later membership incidences after the affected participants'
independent decisions. None of those writers can author a downstream political
result.

If solidarity remains in the live game, only a separately governed endogenous
resolver that consumes attributed independent conduct may change it. No
practice-specific gain, clamp, decay, controller flag, or desired trajectory
may write it.

The producer-consumer closure gate derives its graph from executable BSL
declarations, typed Rust transitions, register schemas, and actual call sites.
A manual manifest can classify or constrain an edge but cannot invent a
consumer. The gate rejects:

- an intermediate output with no runtime reader;
- a terminal output with no decision surface, receipt, Archive record, or
  explicit governed retirement;
- a test or fixture presented as a consumer;
- duplicate authoritative channels for one principal;
- a phantom manifest-only edge;
- a disconnected causal subgraph;
- a permission with no actual producer or consumer.

The same gate derives the read graph from executable accessors and call sites.
It rejects an undeclared read, a declared read with no call site, an
authoritative graph read where the rule requires an actor dossier, a true
attribute read where only a projected observation is lawful, and a read whose
subject or as-of scope exceeds its declaration. Consumer closure cannot make
an unlawful read valid.

## 20. Verification and CI cadence

Every implementation PR runs the smallest applicable tests first and retains
these merge-blocking contracts:

- V1 byte compatibility and cross-version decoder refusal;
- successor byte, digest, ordering, cardinality, and backend parity;
- exact read-and-write causal footprints and consumer closure;
- conservation of material, goods, labor time, energy, money, waste, and loss;
- fog noninterference, actor divergence, and full-information parity;
- short control/intervention twins;
- distinct action-signature tests;
- mutation-manifest integrity and a bounded set of executed link-severing
  mutants;
- scoped Rust tests, Clippy with warnings denied, formatting, BSL conformance,
  and `mise run rust:check-no-docs`.

Frozen Python regression, vault, and gate-coverage checks remain reference
evidence where their frozen behavior applies. They cannot certify a new
Rust/BSL mechanic.

Weekly CI owns genuinely long work:

- long-horizon slow-fast-slow, persistence, reversal, and hysteresis traces;
- full sector, geography, access-mode, and disruption-action matrices;
- the full live mutation campaign;
- broad cross-process, graph-backend, PostgreSQL, checkpoint, restart, and
  Archive replay.

The PR target is 15 minutes per job at the measured hosted-run p95. A test that
exceeds that target in three representative runs moves to weekly only after a
bounded merge-blocking surrogate preserves its local contract. If no sound
surrogate exists, the test remains merge-blocking despite its duration. Heavy
Cargo, PostgreSQL, and long evidence gates run serially on the shared host.

The minimum live mutation manifest contains independently named witnesses for:

- severing the organization-to-worker labor relation;
- admitting a lawful strike proposal while the workers refuse participation;
- hiding an observer and therefore preventing that observation from producing
  trust evidence or repertoire;
- removing each labor, stock, money, energy, and route debit;
- severing communication while preserving co-participation;
- preserving co-participation without an attributed interaction and therefore
  producing no tie;
- removing provision, strike funds, replacement labor, rerouting, access,
  maintenance, knowledge, and ecological burden one at a time;
- giving a central actor no independent adoption and a peripheral actor real
  independent adoption;
- recurring function without commitment and commitment without the recurring
  function or material resources;
- swapping nominal victory and defeat labels while preserving the receipts;
- replacing each disruptive practice with an equal nominal magnitude of every
  other practice;
- removing each product, dossier, capacity, affected-population, Backfire, and
  construction consumer;
- attempting every prohibited direct write; and
- actor renaming, canonical-input permutation, restart replay, and graph-backend
  twins.

Each mutant has a predeclared refusal or observable difference. Merely
executing a mutant does not make the proof pass.

## 21. Live emergence evidence

The existing scalar SFS classifier remains a post-commit recognizer. The live
successor samples attributed material channels rather than membership alone:

- labor allocation and withheld functions;
- production input use and output;
- order fill and unfilled demand;
- inventory, substitution, dispatch, freight, loss, and delivery;
- realization and settlement;
- typed need fulfillment and reproduction burden;
- maintenance, exhaustion, ecology, and operable capacity;
- repression, affected populations, Backfire, and practice products.

`LiveSfsEvidenceV2` defines an ordered set of independently classified channel
traces. Each trace binds one channel identity, subject, native `unit_id`,
versioned reduction law, observation cadence, and exact post-commit envelope
source. One trace contains one nonnegative scalar in one unit at each sample.
The unchanged classifier runs separately on each trace. The evidence layer
never sums, averages, normalizes, or weights different principals into a
universal mass or political score.

Each proof profile preregisters its required witness channels, negative-control
channels, and causal ordering, such as withholding before available-labor and
output changes, before the run. The report contains the ordered per-channel
classifications and cross-channel timing evidence. It has no aggregate
slow-fast-slow classification. A run can supply evidence in one or more
declared channels without requiring every channel to share one shape.

The run identity binds the initial complete envelope, content, phase schedule,
accepted-input ledger, actor/fog policy, reference manifests, proof profile,
mutation manifest, and final envelopes. The classifier remains outside the
engine dependency graph.

A preregistered flat cadence is a proof driver only. It is not gameplay timing,
an organization growth law, or a requirement that actors repeat an action.

The live suite must admit continuing, plateau, flat, reversal, fragmentation,
reconvergence, and persistent-separation traces without changing the
classifier. It must show sector and geographic heterogeneity. A slow-fast-slow
trace is evidence for one causal history, not an acceptance requirement for
every run.

Counterfactuals change exactly one declared relation, ledger row, capacity,
knowledge fact, route, maintenance condition, access decision, participation
decision, or repression allocation. Expected downstream channels must change.
An intervention refused before admission preserves world identity. An accepted
material no-op preserves the relevant post-state principals and downstream
channels, while its input, event, receipt, envelope, and world identities can
differ.

Mutation tests sever every claimed causal link. The proof fails if the same
classification survives only because the scenario seeded its cadence, a
stored stage selected effects, a hidden field bypassed a carrier, or an
important output lacked a consumer.

## 22. Architecture dependencies and Linear boundary

Linear alone owns current scope, status, priority, dependencies, milestones,
and work. GitHub owns source, pull requests, review, and historical evidence.
This specification records architecture and required corrections; it does not
replace Linear.

The canonical Linear portfolio maps these architecture dependencies to current
issue identities, owners, status, priority, and schedule. This specification
does not duplicate those live assignments. Linear can reorder, split, or
reassign delivery without changing the causal prerequisites below.

Implementation proceeds from fresh `origin/dev` in independently reviewable
slices. Shared causal-spine, `.mise.toml`, content-manifest, BSL, and CI files
have one active owner at a time. Heavy gates are serialized.

The critical order is:

1. Freeze and inventory every current V1 vector and digest.
2. Define `PracticeInputAuthorityV2`, accepted-input identity, and the
   reservation and allocation contract.
3. Define graph layout v5, attributed-membership access, and the authoritative
   `ActorKnowledgeRegisterV2` bytes.
4. Define labor-process identity and every conserved auxiliary-register byte
   contract.
5. Implement and verify the conserved production, orders, inventory, freight,
   realization, and reproduction transitions that labor withholding affects.
6. Define Practice V2, participant-transition receipt identities, the
   membership-transition receipt contract, and the
   receipt-excluded post-state basis.
7. Freeze `NominalWorldHashV2` and the complete envelope only after all bound
   component layouts are stable.
8. Activate strike proposals and independent worker participation.
9. Add blockade, occupation, damage, and capital-strike carriers.
10. Define and implement `RelationalTerritoryDossierV2` over the frozen
    knowledge register and prove relation-and-attribute fog.
11. Connect the topology measurements, governed candidates, finite capacity,
    repression, affected populations, and Backfire only through that V2
    dossier.
12. Add attributed products, membership-transition emission, formation,
    practical leadership, party projections, and construction practices.
13. Add Bevy decision projections, fog-safe receipt projections, and Archive
    consumers.
14. Run the live counterfactual, mutation, heterogeneity, persistence, and
    slow-fast-slow proof.

Fog/repression and consumer-closure work may proceed in parallel after the
identity boundary. Strike resolution cannot precede attributed membership and
the conserved economic carriers it affects.

## 23. Required ADRs

Implementation cannot rely on this prose alone. ADR232 records the program's
source quarantine, supersession, causal architecture, and Linear boundary.
Before each dependent code slice lands, accepted successor ADRs must record:

1. graph layout v5, attributed-membership bytes, and authoritative actor
   knowledge-register bytes;
2. Practice V2 input authority, accepted-ledger identity, resource reservation,
   capacity allocation, and conflict semantics;
3. Practice V2, resolved-batch, material-connection, participation, timing,
   participant- and membership-transition provenance and receipt identities;
4. auxiliary economic-register ownership, units, conservation,
   post-state-basis composition, `NominalWorldHashV2`, and envelope ordering;
5. gameplay dossier/fog, topology measurement, capacity, repression,
   affected-population, and Backfire boundaries;
6. attributed practice products, membership-transition consumption,
   participant-owned formation, party projection, and long-run construction;
7. the per-channel live evidence profile, mutation scope, and PR-versus-weekly
   CI policy.

Historical ADRs remain unchanged. Each successor ADR names what it supersedes
and which V1 bytes it preserves.

## 24. Prohibited shortcuts

The program refuses:

- a stored hinterland, party, communist, decommodification, control, collapse,
  revolution-progress, or slow-fast-slow state;
- a fixed sigmoid, response table, stage machine, threshold ladder, terminal
  outcome latch, or second authored curve;
- a Capital, Party, Movement, State, or Planner super-agent that owns outcomes;
- an omniscient police graph or planner;
- centrality, infrastructure, ownership, service, output, mutual aid, strike,
  or repression directly producing political authority;
- pairwise expansion of a native hyperedge;
- care as an infinite buffer or stockpiled service;
- stock, delivery, access, realization, and reproduction treated as one fact;
- an aggregate disruption scalar shared by materially different practices;
- a Python gameplay implementation, dual writer, or Python acceptance oracle
  for new Rust/BSL mechanics;
- a fixture, chart, manifest row, or unit test presented as a live consumer;
- any Pannekoek, Bordiga, Trotsky, quarantined Theory clipping, or disallowed
  CPUSA concept as direct or indirect authority.

## 25. Completion definition

The Neel Integration Program is complete only when current-state evidence
proves all of these facts:

1. The player inhabits a materially situated organization and can act through
   the same machinery as non-player organizations.
2. A fog-safe gameplay dossier derives multi-scale territorial relations with
   no stored hinterland classifier.
3. A materially connected worker organization can propose a strike, and
   affected worker cohorts independently determine participation.
4. Withholding changes available labor, which changes production. Downstream
   production, circulation, realization, and reproduction consequences
   propagate only through the conserved inventory, order, freight, delivery,
   realization, and reproduction circuit.
5. Strike, blockade, occupation, damage, and capital strike produce distinct
   immediate carriers and traces.
6. Actor-scoped dossiers and resolved relations feed topology measurements,
   governed candidates, finite capacity, distinct repression instruments,
   actual affected populations, and Backfire without direct outcomes.
7. Committed practice receipts produce competence, repertoire, directed trust
   evidence, and real ties; participant-owned resolution can form an
   organization without a practice directly writing membership.
8. Practical leadership and party development remain actor-scoped projections
   over independent adoption and plural coordination.
9. Construction changes actual provisioning, access, production,
   reproduction, maintenance, ecology, and dependence without a progress score
   or terminal state.
10. Every important output reaches a real consumer.
11. Exact V1 compatibility, successor determinism, conservation, fog,
    counterfactual responsiveness, read-and-write causal footprints, mutations,
    and per-channel live emergence traces all pass against the exact reviewed
    head.
12. Rust/BSL is the only gameplay authority, PostgreSQL writer cutover is
    explicit and single-writer, and Python remains reference/periphery only.
13. Linear scope and dependency records match the implemented architecture,
    the required ADRs are accepted, and Bevy plus Archive expose a real
    decision-to-consequence loop.

Anything less is a staged contribution to the program, not completion.

<!-- vale write-good.Weasel = YES -->
<!-- vale write-good.TooWordy = YES -->
<!-- vale write-good.ThereIs = YES -->
<!-- vale strunk.CommonlyMisused = YES -->
<!-- vale strunk.ActiveVoice = YES -->
<!-- vale ste.ThisPronoun = YES -->
<!-- vale ste.SentenceLength = YES -->
<!-- vale ste.Semicolon = YES -->
<!-- vale ste.ProcedureLength = YES -->
<!-- vale ste.PassiveVoice = YES -->
<!-- vale ste.OneInstruction = YES -->
<!-- vale ste.Modals = YES -->
<!-- vale ste.LatinAbbrev = YES -->
<!-- vale ste.Gerunds = YES -->
<!-- vale ste.Dictionary = YES -->
<!-- vale ste.Ambiguity = YES -->
<!-- vale Vale.Terms = YES -->
<!-- vale ste.NounClusters = YES -->
<!-- vale ste.UnapprovedWords = YES -->
<!-- vale Vale.Spelling = YES -->
