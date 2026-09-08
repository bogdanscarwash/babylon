Event System Architecture
=========================

.. vale Vale.Spelling = NO
.. vale ste.UnapprovedWords = NO
.. vale ste.NounClusters = NO

Babylon events are deterministic observations of an adjudicated world. They
support explanation, persistence, the semantic Archive, and later narration.
They do not control mechanics. Constitution v4.1.0, Amendment AJ, and ADR248
make this distinction explicit for finite material probability.

The canonical live BSL payload registry is
``docs/reference/event-schema-registry.toml``. The Pydantic model reference at
:doc:`/reference/events` describes the frozen Python evidence. The registry
records its known divergences from live BSL. For replay and receipt identity,
see :doc:`/reference/determinism-contract`.

Events and consequences
-----------------------

``Consequences`` is one causal partition in the four-week phase schedule. An
``event`` is an observational record that an authorized rule can emit from any
governed phase. The concepts are not synonyms. A Consequence mechanic can
change material state without an event. A recognizer can emit an event after
it observes state produced elsewhere.

The live Rust path takes canonical replay input and governed BSL rules into one
detached tick. The tick buffers material state, typed events, audit receipts,
and choice receipts. ``TickPayloadV2`` binds those outputs before the
marker-last Postgres transaction makes them durable for the semantic Archive.

Nothing becomes observable until the detached tick and its identity succeed.
Nothing becomes durable until the exact marker-last transaction succeeds. The
runtime must reconcile an ambiguous commit byte-for-byte.

Causal authority
----------------

Every BSL rule declares one causal role and one evidence class. Events inherit
that attribution from their emitting rule.

``Mechanic``
   Can derive material effects through the ordinary typed effect algebra. A
   Mechanic can also emit an authorized event. Under Amendment AJ, only this
   role can contain a finite material transition kernel.

``Recognizer``
   Deterministically observes a governed state pattern. Its complete effect
   tree is exact-allowlist and default-deny. A finite projection recognizer is
   further restricted to subject-local, emit-only behavior.

``ExternalEvent``
   Remains deterministic and can introduce only an explicitly allowed
   exogenous pressure, burden, or capacity effect. It cannot draw a kernel or
   author downstream results.

``Intent``
   Remains deterministic and can introduce only governed next-period intent
   effects. It cannot draw a kernel.

The independent production attribution and effect-allowance ledgers remain
mandatory. A role name never grants first-party content authority by itself.

Observation is not probability
------------------------------

An authored event owns no probability. Probability belongs to the material
transition that can produce different post-states. A deterministic recognizer
then maps each post-state to zero or more observed events.

For a finite material kernel ``K`` and deterministic recognizer ``R``:

.. code-block:: text

   Pr(e | s) = sum_delta K(s, delta)
               * [e in R(s, apply(s, delta))]

The exact event likelihood is the measure of the recognizer's preimage.
Babylon computes it only for a loaded
``FiniteProjectionV1`` whose recognizer immediately follows its kernel in
resolved schedule order and is subject-local, deterministic, and emit-only.
The linked kernel Mechanic can write material state only on its literal
``self`` carrier. Forecasting applies every branch to detached state, runs the
real linked recognizer, and sums favorable ticket counts. It consumes no
random draw.

``EventLikelihoodV1`` reports:

- event type.
- enum-ordered favorable outcomes.
- exact favorable ticket numerator.
- fixed denominator ``2^64``.

The API refuses arbitrary recognizers, whole-tick enumeration, cross-sample
joins, sequences, conjunctions, and payload distributions. It never substitutes
Monte Carlo sampling or an independence assumption. A state-dependent mass
remains honestly state-dependent when the supplied scenario cannot resolve
it.

Finite realization and receipts
--------------------------------

A finite kernel evaluates every exact branch mass. It allocates the complete
``2^64`` ticket measure and draws one replay-keyed ``u64`` ticket. It applies
only the selected material effect bundle. Branch bodies cannot emit an event.
The ordinary adjacent recognizer observes post-state instead.

Receipt contents
~~~~~~~~~~~~~~~~

- encounter ordinal, rule ID, sample QName, and append-only slot.
- stable subject and ordered active elements.
- every enum-ordered exact mass and ticket interval.
- draw ticket and selected outcome.
- allocation and instance digests.

Material change and no-change outcomes both produce this receipt.
``ChoiceReceiptV1`` stays separate from ``AuditReceipt``. The audit receipt
remains the compact identity-free record of actual event and write effects.
The choice receipt carries the exact realization evidence.

Committed event record
----------------------

``TickPayloadV2`` carries ``SuccessfulEventBatchV2`` and ordered choice
receipts. Each ``SuccessfulEventV2`` carries the metadata described below.
``CommittedTickEnvelopeV2`` persists the ``ChoiceReceipt`` family between the
``Event`` and ``Checkpoint`` families.

A committed event contains two provenance fields outside its authored
payload:

``emitting_rule``
   The exact rule that emitted the event.

``choice_receipt_ref``
   An optional automatically derived reference to the realized choice observed
   by a finite projection. Ordinary events have no reference.

Postgres stores this live shape in ``tick_event_v2`` and
``tick_event_field_v2``. The authored payload contains only material facts from
the event schema. It never contains branch mass, ticket count, likelihood,
draw, allocation digest, or a user-supplied receipt reference. The engine
derives a projection link from the typed adjacent kernel and recognizer.

Ordering and atomicity
----------------------

Within one successful rule, actual emitted events precede actual writes in the
continuous ``AuditReceipt`` sequence. Choice receipts use their own encounter
ordinal and canonical enum order. The tick payload fixes the relative section
order. Persistence writes choice receipts and projected event provenance before
checkpoint and Archive rows. It writes ``babylon_state.tick_commit`` last.

Any error in mass evaluation, allocation, draw, selected effects, or
recognition fails the tick. Event validation, receipt construction, hashing,
storage, and restart comparison also fail the tick. A failed tick publishes no
state, event, receipt, completed time, or identity. Retry must reconstruct
exact ``CommittedTickEnvelopeV2`` bytes.

Observer independence
---------------------

Events, forecasts, receipts, narration, and Archive materialization are
projections. They cannot feed mechanics or become an adjudication input.
Removing an event sink must leave the material trajectory, choice draws,
selected outcomes, and world hashes unchanged. Removing the event from a
content rule is a content change. Removing a downstream observer is not.

The standard post-commit JSONL report can summarize choice count and digest.
Detailed choice logging is opt-in, create-new, administrative output written
only after durable commit. Neither JSONL form is authoritative, and neither
can feed replay, mechanics, or Archive causality.

Frozen Python reference
-----------------------

The frozen Python engine retains its typed Pydantic events and observer paths
as historical behavioral evidence. They do not define the live Rust event
boundary, Envelope V2, choice provenance, or exact forecast API. No adapter
converts the frozen event bus into authoritative campaign state.

Invariants
----------

Deterministic recognition
   A recognizer observes state and consumes no randomness.

Material causality
   A kernel chooses a bounded material effect bundle, never an event name or
   terminal outcome.

Exact likelihood
   Event likelihood is a finite recognizer preimage measure or a loud refusal.

Payload honesty
   Probability and realization evidence never appear in an authored event
   payload.

Transactional evidence
   State, event, choice receipt, completed time, and identity publish together
   or not at all.

Observer isolation
   Event and receipt consumers cannot alter mechanics.

See Also
--------

- :doc:`architecture` — current Rust and persistence boundary.
- :doc:`/reference/events` — frozen Python event models and the live BSL
  schema cross-reference.
- ``docs/reference/event-schema-registry.toml`` — canonical live BSL emit
  payload registry.
- :doc:`/reference/bsl-language` — kernel and projection language.
- :doc:`/reference/determinism-contract` — allocation, keys, hashes, and V2.
- ``ai/decisions/ADR224_bsl_causal_composition_contract.yaml`` — causal roles,
  allowances, and ``AuditReceipt``.
- ``ai/decisions/ADR248_finite_material_transition_kernels.yaml`` — Amendment
  AJ implementation law.

.. vale ste.NounClusters = YES
.. vale ste.UnapprovedWords = YES
.. vale Vale.Spelling = YES
