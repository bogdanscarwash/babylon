You're describing a **coverage contract for the seam** — a way to prove that every quantity the engine produces is (a) carried across the bridge, and (b) consumed by the frontend, with a test that *fails loudly* when something is stranded. Let me give it a shape.

## What you're actually asking for

Three claims you want mechanically guaranteed, not eyeballed:

1. **Nothing the engine computes dies at the bridge** — every meaningful graph-node / snapshot field has a serialization path in `engine_bridge.py`.
2. **Nothing the bridge emits is unused** — every field in the wire payload is read by *some* frontend code.
3. **Nothing rendered is faked** — what the frontend displays traces back to a real engine value, not a fixture or a relabel (your honest-Φ discipline).

This is exactly the failure mode you keep hitting: `imperial_rent` computed but NULL at the bridge; `profit_rate` computed, written to the graph, read by the bridge, but dark because its *input* was unseeded; events computed but capped at 14 of 88 types. Each was a **silent gap between layers**, and you found them by accident, live, one at a time. What you want is a sensor that trips the moment such a gap opens.

## The layer of abstraction: a field registry / manifest

The missing abstraction is a **single declared inventory of "observable quantities"** — the canonical list of things the engine produces that are *meant* to reach a player. Something like a registry where each entry declares:

- **name** (`imperial_rent`, `profit_rate`, `solidarity_index`…)
- **engine source** — where it's computed / which graph attr holds it (`tick_profit_rate`)
- **bridge field** — the wire key it serializes to
- **frontend sink** — the lens / inspector / panel that consumes it
- **liveness policy** — "must be non-null by tick N" vs. "legitimately null until a fascist faction exists"

Right now that knowledge is *implicit*, scattered across three codebases and a few docstrings. Making it a **declared artifact** is the abstraction. It's the same move as your `EventType` enum or `GameDefines` — turn tribal knowledge into one typed source of truth. Once it exists, it's the spec that the rewrite-test in your CLAUDE.md talks about: the thing that would let you prove a reimplementation correct.

## The testing you aren't doing: three sensor layers

The registry alone is just a list. The power is the tests that check reality *against* it:

**Sensor 1 — Structural coverage (static, cheap).**
Assert the three sets line up: every registry entry has a real engine producer, a real bridge serializer, and a real frontend consumer. This catches "computed but never serialized" and "serialized but never rendered." It's essentially a set-difference test:

```
engine_fields  →  bridge_fields  →  frontend_consumed_fields
```

Any field present in one set and absent from the next is a **stranded value** → test fails. You already do a primitive version of this (the event classifier handles all 79 types); this generalizes it to *every* observable. On the frontend side it can be as blunt as grepping the wire-type keys against the codebase, or as principled as the generated `@babylon/api-types` you've discussed — tsc becomes the drift detector.

**Sensor 2 — Liveness / population coverage (dynamic, the one that actually bites).**
This is the sensor you were missing when Φ shipped blank. Run a real deterministic tick, then for every registry field assert its **value is non-null (or non-default) by its declared deadline** — *unless* the registry marks it as legitimately-null-until-condition. This is the difference between "the column exists and is typed `float | None`" (which schema tools happily bless) and "the column actually has a number in it." It's what would have screamed:

- `imperial_rent` NULL at tick 1 → **fail** (should be lit)
- `profit_rate` NULL at tick 60 → **fail** (should be lit → sends you straight to the unseeded-employment root cause)
- `fascist_drift` NULL in a scenario with no fascist faction → **pass** (declared conditional)

That last clause is the crucial design point: naive "everything must be non-null" is noise. The registry encodes *when* each field is expected to be alive, so the test distinguishes "honestly dark" from "silently broken." This is the layer your golden/behavioral-contract philosophy already argues for — it snapshots *values a real tick produces*, not just shapes.

**Sensor 3 — Provenance / honesty (guards the relabel).**
The subtler one: assert that a rendered value's *source* is the field it claims to be. This is what stops `extraction_intensity` being painted as Φ, or a fixture leaking into a live view. Practically: forbid frontend display components from reading anything but the typed wire model (no second raw-DB read path — this is the same reason Datasette is wrong for in-game), and have golden tests pin the *actual field values* a canonical tick emits so a relabel changes a byte and trips the baseline.

## Why this is more than lint

The reason it feels like "an observability engine with sensors" is that it *is* one — but pointed inward at your own code rather than at production traffic. The registry is the schema of what should be observable; Sensor 1 is a wiring continuity check; Sensor 2 is a liveness probe; Sensor 3 is integrity. Together they convert your current workflow ("build it, then discover live and by luck that a layer dropped it") into "the CI gate tells you the instant a value is computed-but-stranded, serialized-but-unrendered, or rendered-but-fake."

## Honest caveats

- **The registry is load-bearing and must not rot.** If it's hand-maintained it drifts; the discipline is that adding an engine observable *requires* a registry entry, enforced by Sensor 1 failing on unregistered-but-serialized fields. It's the same maintenance cost as `defines.yaml` sync.
- **Frontend "is it consumed" is the hardest set to compute rigorously** — "a key appears in a `.ts` file" is weak (dead code, commented-out). The strong version needs the generated-types approach so the compiler, not grep, proves consumption.
- **Scope it to a curated observable set, not literally every field.** Internal scratch attributes (`res6_parent`, tick bookkeeping) aren't player-observable; forcing them through the sensors is noise. The registry *is* the curation.
- This is real infrastructure — a spec's worth of work, adjacent to the two-tier web↔engine contract mechanism already on your deferred list. It's the natural home for it: Tier 1 (shape) = Sensor 1, Tier 2 (value/behavior golden) = Sensor 2, and the honesty guard = Sensor 3.

So your instinct is correct and it has a name: a **declared observable-field registry + a three-tier seam-coverage gate (continuity, liveness, provenance)**. It's the generalization of every seam bug you've fought — instead of finding them one live run at a time, the sensor trips in CI the moment engine, bridge, and frontend fall out of agreement.
