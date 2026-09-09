# Babylon Constitution

<!-- vale ste.UnapprovedWords = NO -->
<!-- vale ste.NounClusters = NO -->
<!-- vale Vale.Spelling = NO -->

**Version 4.2.0**

**Ratified:** 2026-08-22

Babylon's Constitution defines the durable purpose, boundaries, and rules of
the game. Architecture records, design standards, content registries, and
roadmaps contain details that change more frequently.

## Article I — Purpose and Player Promise

Babylon is an emergent political-economy game built for entertainment,
expression, and education. Babylon is not a forecast. Babylon is not a
scientific reproduction of history or a policy oracle. Its scientific
discipline concerns how it uses evidence, declares assumptions, and tests
causal behavior. That discipline serves a game that must engage players,
remain realistic enough to support reasoning, and be powerfully expressive.

The player enters a living world of organizations, classes, firms,
institutions, territories, and ecological limits. The world continues to move
without the player, responds materially to intervention, and preserves the
consequences of prior choices. The player must understand pressures and choose
among meaningful strategies. Neither the scenario author nor the engine
guarantees those trajectories in advance.

Babylon teaches through consequences. It can be rich, difficult, tragic,
funny, surprising, or hopeful, but it must not confuse obscurity with depth or
predestination with rigor. Entertainment value and truthful causal structure
are joint design requirements.

## Article II — Theoretical Commitments

Material relations and Babylon's Marxist-Leninist-Maoist Third Worldist
(MLM-TW) theoretical line constrain the causal model. They constrain which
relations the engine represents, which questions it asks, and which
explanations it can offer. They neither predetermine a play outcome nor impose
a historical script.

The dialectic `D = (A, Ā, w, T, σ)` remains the irreducible formal unit. In
v4, `σ` is the dialectic's sublation predicate. Nothing can abstract over
dialectical motion. Higher formalisms are compositions, coarse-grainings, or
projections of that motion. A proposed peer or superior is a new primitive and
needs an amendment.

Partitions such as class position, core and periphery, or political alignment
must emerge from composed material relations at a declared scale. Any claimed
partition must include an invariance proof under its declared coarse-graining.
An actor's material position and conceived interest remain distinct dimensions.
Their gap is a terrain of political struggle.

Every formal construct must pass the Aleksandrov Test. The test traces each
abstraction to a material process that an implementer can name and inspect. A
formal construct must yield a law, a falsifiable prediction, or a running
computation. Notation alone creates no authority.

Imperial rent, unequal exchange, production and realization, class formation,
organization, state power, social reproduction, ecological metabolism, and
contradiction remain central theoretical concerns. Their exact equations,
vocabularies, and implementation bindings belong in governed standards and
ADRs. Theory can constrain possible mechanics. Designers must not use theory
as a license to hard-code an answer for play to discover.

The spatial substrate is immutable reference geography. Jurisdictions,
political claims, occupation, secession, and control are mutable overlays.
Political change can alter the overlay but must not falsify the ground beneath
it.

## Article III — Emergence and Agency

Conditions pressure outcomes. They do not dictate them. No shock, scenario,
coefficient, or hidden branch can guarantee a victory, collapse, political
alignment, historical path, or terminal pattern. Governed recognizers can name
outcome patterns, but recognition only describes a world state. Recognition
does not force that state to occur.

Player and non-player actors act through the same declared causal machinery.
Actions must have an eligible actor, a target, and a cost or constraint. They
must also give a preview of what the actor can know and a later receipt that
shows what occurred. All material effects become next-period intents and enter
only the next four-week causal interval. The current interval can resolve only
non-material knowledge, previews, and receipts. Those effects must not mutate
authoritative material state.

External events can introduce only their declared exogenous burden, capacity
change, or pressure. They must not directly author downstream unemployment,
shortages, deaths, migration, consciousness, firm failure, bank failure, or a
winner. Those consequences must emerge through the shared systems that also
operate in control and counterfactual runs.

Gameplay must support more than one viable strategy. A dominant zero-cost
choice, an action with no legible consequence, or an outcome that cannot
respond to counterfactual play is a design defect.

## Article IV — Evidence and Design Liberties

Every substantive value, relationship, or rule must carry one of four evidence
classes:

- **Observed:** directly transcribed from a named source and vintage.
- **Derived:** computed from observed or already governed values by a declared
  transformation.
- **Calibrated:** selected to make a declared causal signature or operating
  range plausible, with the target and method recorded.
- **Designed:** chosen as a game-design liberty for pacing, legibility,
  expression, challenge, or entertainment.

Designed values and mechanics are legitimate. Creators must declare them and
keep them causally coherent with the rest of the model. Reviewers must be able
to distinguish them from observed fact and review them like other content. A
design liberty can simplify, compress, or stylize reality. It must not claim
empirical authority that it lacks. It must not bypass the causal circuit to
manufacture an intended conclusion.

Data sources, artifacts, transformations, calibrations, and designed values
must carry enough provenance to reproduce the input state. Historical data can
set envelopes, scales, priors, and comparison cases. It must not silently set a
mechanic's response shape or dictate its result.

## Article V — Deterministic Architecture

Determinism establishes computational identity, not scientific truth. The
engine must produce the same ordered events, state, and world hash from the
same canonical input. That input includes the initial state, content, player
and non-player intents, reference digests, and random seed. Nondeterminism,
unchecked overflow, noncanonical serialization, and silent degradation are
defects.

Required inputs that are absent or invalid must fail loudly. A subsystem must
not silently skip its work, substitute a plausible default, or leave a
declared guardrail inactive.

The Rust engine adjudicates one four-week tick, exactly 28 days, as a pure
transformation over a working copy. Each period has one causal traversal and
one atomic commit, with no internal weekly substeps. Thirteen periods span
one 52-week model year; these periods are not civil calendar months.
BSL supplies governed rules, shocks, policies, and action
semantics as content. Database access, wall-clock time, narration, rendering,
and network services must not take part in adjudication. A failed rule aborts
the whole working copy. No partial tick becomes visible.

Postgres durability occurs only after adjudication. A committed tick records
the graph, auxiliary registers, action receipts, events, digests, and combined
world hash as one atomic envelope. The durable tick advances only after the
transaction acknowledges it. SQLite and pinned columnar artifacts remain
reference-build inputs, never mutable campaign state.

State is data, the engine is transformation, and clients are projections. AI
can observe, retrieve knowledge allowed by fog, and narrate with citations. AI
must not adjudicate mechanics or retrieve hidden ledger truth. A client can
emit intent but must not mutate authoritative state directly.

The graph exposes native hyperedges. Amendment AG keeps attributed membership
as a first-class, typed, hash-covered element. A native n-ary membership must
remain one whole membership object and must never expand into a pairwise clique.

Governed content can mint instances of the ratified scale-lattice and
adjunction schemas. New mathematical kinds remain closed without amendment.
`ai/decisions/ADR189_amendment_ag_attributed_membership_lattice_instances.yaml`
governs ordered membership, effect-only mutation, and weighted aggregation of
intensive fields. Dyadic relations, hyperedges, compact registers, and
transport substrate records must keep their distinct semantics.

Amendment AJ adds one closed mathematical kind subordinate to the dialectic:
the finite material transition kernel
`K : C -> D_f(Delta)`. Its carrier is a bounded, typed set of material effect
bundles. A deterministic mechanic is the Dirac case. A kernel cannot range
over event names, terminal routes, historical outcomes, or undeclared world
mutations. Scheduled kernels compose through the same sequential causal order
as deterministic mechanics. No declaration of parallel independence exists.

Behavioral contracts must outlive any implementation. Canonical byte layouts,
golden traces, schemas, property laws, replay checks, scenario tests, and
boundary contracts form the durable criteria for a rewrite.

Every materialized projection must be deterministic for the same committed
state, knowledge, definitions, templates, and projection version.

Random realization is part of the canonical input and evidence boundary. Every
finite-kernel choice must use the governed replay key, consume one exact
integer ticket draw, and publish a hash-bound transactional choice receipt.
The receipt records both the selected and unselected alternatives. This
separate record cannot influence later mechanics or widen `AuditReceipt`.

## Article VI — Political-Economy Circuit

Babylon must progressively close one political-economy circuit spanning:

1. production and the transformation of inputs.
2. circulation, transport, trade, and delivery.
3. realization through orders, inventory, prices, and settlement.
4. finance through money, claims, liquidity, solvency, and backstops.
5. labor, reserve-army movement, class formation, and social reproduction.
6. organizations, the state, political struggle, and player action.
7. ecology, capacity, depletion, and recovery.

A mechanic is complete only when its important outputs reach a real consumer
or receive a recorded absence or retirement disposition. Ports are
slice-driven: frozen systems receive Port, Adapt, Replace, or Retire decisions
when a playable causal slice needs them. The project can begin play before it
completes every port.

The topology can contain detail without an all-pairs graph. Exact stocks
belong in one authoritative register. Relations carry identity, routes, or
shares without duplicate principal. Resolution expands only when a named
mechanic and evidence source justify more entities or relations.

Conservation laws bind the circuit. Goods cannot arrive before shipment.
Settlement cannot occur before realization. Every money change needs a
declared source or sink. Labor-force movement must close over employment,
reserve, mortality, migration, and inactivity.

## Article VII — Player Knowledge and Presentation

Every player-facing map, chart, graph, topology, table, or visualization must
answer a decision question. Rich detail is welcome when it helps the player
interpret signals, compare options, act, or understand a receipt. Detail that
does not serve a decision belongs in the semantic Archive, a reference view,
or an explicitly labeled administrative or diagnostic surface.

Every shipped gameplay surface must declare a `DecisionSurfaceContract` with:

- its decision question.
- visible signals and their uncertainty.
- knowledge and fog requirements.
- available actions.
- expected feedback receipts.
- linked Archive subjects.
- an explicit `admin_debug_exempt` status.

An administrative or diagnostic exemption must be visible to the user and
cannot meet a gameplay milestone. No surface is exempt by default.

The semantic Archive is part of play, not a decorative encyclopedia. It must
present deterministic, citation-bearing pages and links under campaign and
knowledge context. Unknown subjects appear as honest red links. Search and LLM
retrieval must enforce fog at the data boundary. A page that lags the durable
tick must show its last verified tick.

The core Archive loop is evidence to decision to action to consequence to an
updated dossier. Every important action must return a causal receipt that a
player can inspect through this loop.

## Article VIII — Behavioral Validation and Governance

Historical cases are behavioral benchmarks, not reenactment scripts. They test
causal signatures, heterogeneity across places and sectors, hysteresis after a
shock ends, conservation, and counterfactual responsiveness. Success means the
same declared mechanisms produce plausible, varied, inspectable dynamics in
control, shock, and intervention runs. It does not mean a run must reproduce a
single historical trajectory.

Emergence claims need redundant evidence. The evidence includes control and
shock twins, counterfactuals, and mutation tests that sever causal links. It
also includes geographic and sector variation, persistence after the event,
deterministic replay, and a direct-write audit. The audit must prove that the
scenario did not author its downstream results.

The human Director sets direction, holds the reserved ideological and
theoretical line, approves changes to governed political content, and keeps
final merge authority to `main`. Agents engineer within that line under TDD,
behavioral contracts, deterministic gates, explicit ownership, and loud
failure. A green gate licenses progress. A red gate stops it until the team
resolves the failure.

A new or redefined mathematical primitive, a relaxed constitutional
prohibition, or a change to the reserved theoretical line needs a
constitutional amendment. New governed content vocabulary does not need an
amendment when it uses the current mathematics. It needs the applicable
Director ceremony, schema and conformance updates, and a recording ADR.
Ordinary BSL content that uses the current vocabulary follows the content
review path.

An amendment must state the problem, the changed law, affected commitments,
transition treatment, verification, and version change. Historical records
remain immutable. Later decisions supersede them explicitly. They do not
rewrite what an earlier decision said.

### Amendment AH — Game-First Refoundation

Amendment AH ratifies this eight-article Constitution and the
entertainment-first, emergence-through-play standard. It supersedes the former
live v3.2.0 text and preserves its history and any authority explicitly
retained or re-homed by
`ai/decisions/ADR221_game_first_refoundation_v4.yaml`.

AH restores the Director-approved v4 primitive notation
`D = (A, Ā, w, T, σ)`. The immediate v3.2 predecessor used `Ā` and `s` after
Amendment N. The change is a new v4 notation ruling, not a claim that v3.2
used `σ`.

The transition record pins both the requested v3.1.0 snapshot
`a265b85120ed2a90be40c72e63ee5bf27fc6e703` and the immediate v3.2.0 snapshot
`e905e90d66bddc6e4eca36a3896428f5ce63de5b`. Pre-AH unversioned dotted
clauses, named sections, and amendment references resolve against the pinned
v3.2.0 snapshot through ADR221. Post-AH living authority cites the named v4
article or its stable heading.

### Amendment AJ — Finite Material Transition Kernels

Amendment AJ ratifies a finite material transition kernel
`K : C -> D_f(Delta)` as a new closed mathematical kind. The dialectic remains
Babylon's irreducible ontological primitive. A kernel is subordinate machinery
for bounded material contingency within that ontology, never a peer or a
replacement. Existing deterministic mechanics embed as one-outcome Dirac
kernels, and scheduled rules continue to compose sequentially.

A kernel distributes exact mass over a finite, typed enumeration of bounded
material effect bundles. It cannot distribute over events, victory or defeat,
terminal routes, historical outcomes, or a hidden downstream result. A shared
cause requires one joint kernel over its whole governed carrier. Babylon does
not infer independence between separately authored quantities, and v1 adds no
tensor or parallel-independence declaration. A kernel paired with a finite
projection keeps all material writes carrier-local: it can update only its
literal `self` node. A write to another node, an edge, a hyperedge, or graph
shape requires a joint carrier kernel and is not exactly enumerable in v1.

`Mass`, `Probability`, `Intensity`, and dialectical weight are distinct.
`Mass` is exact allocation input for a finite kernel. `Probability` remains a
bounded material scalar. `Intensity` remains a state level. Dialectical
weight remains part of `D = (A, Ā, w, T, σ)`. No implicit coercion joins
these kinds.

Events remain deterministic observations. A recognizer can observe a
post-transition state but cannot draw randomness. An event declaration owns no
probability. For deterministic recognizer `R`, event likelihood is the measure
of its preimage under the kernel:

`Pr(e | s) = sum_delta K(s, delta) [e in R(s, apply(s, delta))]`.

Only an exact finite, subject-local projection can compute that value. An
implementation must refuse analysis outside the enumerable boundary rather
than sample, approximate, assume independence, or attach an authored
likelihood to an event.

Every realization is replay-keyed, hash-bound, transactional,
evidence-classified, and receipted. The realization key binds replay session,
seed, tick, rule, sample, append-only slot, stable subject, and ordered active
elements. A failed tick publishes no state, event, choice receipt, completed
time, or identity. Event observation and receipt observation must not alter a
material trajectory.

AJ grants no stochastic authority to `Recognizer`, `ExternalEvent`, or
`Intent`. It adds no universal sigmoid, empirical-distribution sampler,
alternate RNG, arbitrary event forecast, or authored terminal branch. The
kernel RNG remains a private realization seam. Exact language, ticket,
projection, receipt, persistence, and migration rules live in
`ai/decisions/ADR248_finite_material_transition_kernels.yaml` and the governed
BSL and determinism references.

### Amendment AK — Four-Week Simulation Periods

The Director chose a coarser four-week simulation tick to reduce repeated
advancement work and improve game pacing. Amendment AK replaces the weekly
interval in Articles III and V. A tick closes one 28-day period through the
existing causal order. It does not batch four weekly transitions or promise
the same delivery and production timing as weekly simulation.

Content must state capacities, labor schedules, delays, retention, and any
rate conversion in the declared period units. Exact stocks, persons,
per-batch recipes, and observed source units do not change merely because the
simulation interval changes. Conservation, deferred material effects,
determinism, and atomic publication remain binding. This amendment adds no
mathematical primitive and changes no reserved theoretical commitment.

New campaign content binds the duration into its admitted identity.
Incompatible weekly campaigns remain preserved but cannot resume under the
new interval. Historical ADRs, contracts, and codec vectors retain their
bytes; ADR257 records the scoped supersession of their weekly unit semantics.
Verification must prove a single transition per advance, explicit duration
admission, current-content replay and restart, conserved accounts, and
matching period labels and receipts. This amendment records the decision;
executable and native acceptance require separate evidence.

AK advances the Constitution from v4.1.0 to v4.2.0. Its behavioral contract is
`contracts/simulation_interval_v1.yaml`; its transition record is
`ai/decisions/ADR257_four_week_simulation_interval.yaml`.

---

**Version:** 4.2.0

**Originally ratified:** 2026-01-30

**Last amended:** 2026-09-06
