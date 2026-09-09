You're right that it's familiar parts in an unfamiliar arrangement. Let me name the shape.

## The core abstraction: a deterministic reducer with a semantic observability plane

Strip Babylon to its skeleton and it's the **event-sourced state-machine / reduce pattern** you'd recognize from Redux, Elm, or a blockchain VM:

```
state_{t+1} = engine.run_tick(state_t, services, context)     # pure transformation
```

Frozen state in, frozen state out, deterministic hash per tick, replayable from a seed. "State is pure data; the engine is pure transformation; they never mix" is literally the Elm Architecture / CQRS separation. The `SessionRecorder` black-box replay is event sourcing. That part is ordinary.

**What's unfamiliar is what sits *beside* the reducer.** Most reducer architectures have one state shape. Babylon has **three co-equal representations of the same state, each optimized for a different algebra**, and the engine is really a *coordinator* that keeps them coherent:

- **The Ledger** (SQLite/Postgres/Pydantic) — relational algebra, rigid material facts.
- **The Topology** (rustworkx `BabylonGraph`) — graph algebra, relational/fluid structure. Betti numbers, articulation points, community hypergraphs, contradiction fields.
- **The Archive** (pgvector) — semantic/vector algebra, narrative memory.

This is the **polystore / multi-model database pattern**, but pulled *up* into the domain layer and made the substrate of a simulation rather than a query engine. The "Embedded Trinity" is CQRS-with-three-read-models where one of the read-models (the graph) is itself computationally rich enough to drive the writes.

## The genuinely peculiar move: theory-as-schema

The truly distinct thing isn't structural — it's that **the domain model is a formal theory, and the code is required to be a faithful materialization of it.** The Aleksandrov Test ("every formal construct traces to a material relation"), the dialectic `D=(A,Ā,w,T,σ)` as an irreducible primitive, "invent no primitive without a constitutional amendment" — this is the **interpreter pattern** where the language being interpreted is *Marxist-Leninist-Maoist political economy*, and the Constitution is the language spec.

Most simulations approximate a domain. Babylon treats the domain as a **compiler target**: material conditions → deterministic output via encoded law, with AI strictly forbidden from touching the math (it "parses/narrates only"). That inverts the usual LLM-app shape — the model is the *narrator at the edge*, not the *engine at the core*. It's closer to a physics engine or a type-checker than to a game or an agent.

## What you're building now completes a recognizable triad

The observability-registry work isn't a new shape — it's the **third leg of a "spec-driven system" pattern** you've been assembling without naming:

1. **`GameDefines`/`defines.yaml`** — the declared *inputs* (coefficients as data, with a sync-guard).
2. **`qa:regression` goldens** — the declared *outputs* (byte-identical behavioral contract).
3. **The observable-field registry + sensors** — the declared *interface surface* (what crosses the seam and stays alive).

Together those three are the "**materialized view of understanding**" idea from your CLAUDE.md made literal: the durable spec artifacts (schema + goldens + registry) are the real asset, and the code is a regenerable projection of them. That's the **contract-first / design-by-contract** tradition (Eiffel, protobuf, OpenAPI) applied not to one API but to an entire simulation's cross-layer coherence.

## So the one-line shape

Babylon is a **deterministic, event-sourced reducer over a tri-modal (relational + topological + semantic) state substrate, whose domain model is a formal theory enforced as a constitutional schema, wrapped by a coverage-instrumented seam that treats "every computed quantity must be observable" as a checkable invariant.**

Familiar pieces — Elm reducer, CQRS/polystore, interpreter pattern, design-by-contract, event sourcing. The unfamiliar composition is three things fused:

1. **The graph is not a view, it's a co-processor.** Topology does real work (atomization, resilience, bifurcation), so the "read model" feeds back into causality. That breaks the usual clean CQRS one-way flow in a principled way.
2. **The spec is ideological, not just technical.** The invariants encode a *theory of history*, which is why determinism is a moral stance ("non-determinism is a bug") rather than a performance choice.
3. **Observability is being pulled from ops-time to compile-time.** You're not instrumenting production traffic; you're asserting, statically and in CI, that the *architecture itself* has no stranded values — the "sensors" watch the code's internal coherence, not its runtime.

If it feels peculiar, that's because most systems pick one of those to be rigorous about. Babylon is trying to be rigorous about all three at once — a **simulation-grade engine that is simultaneously a theorem, a database, and a game** — and the seam-coverage layer is what stops those three ambitions from silently drifting apart.
