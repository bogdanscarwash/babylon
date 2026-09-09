Good question to pin down, because "seam" is doing real conceptual work here and it's borrowed, not invented.

## The everyday engineering meaning (where the word comes from)

The term is popularized by Michael Feathers in *Working Effectively with Legacy Code*:

> **A seam is a place where you can alter behavior in your program without editing in that place.**

The classic example: a function that calls `logger.write(...)` has a seam at that call — you can swap the logger for a fake *without touching the function's body*, by injecting a different object. The seam is the **joint where two components meet and can be separated**. Its companion concept is the *enseam point* — the place where you actually make the substitution.

So in plain engineer lingo, a seam is a **substitutable boundary**: a point where one side doesn't know or care about the other's internals, only the contract between them. Dependency injection exists to create seams. Mocks live at seams. It's literally the tailoring metaphor — the line where two pieces of fabric are stitched, and where you'd cut if you wanted to separate them.

## What "the seam" means specifically in Babylon

Here it's used more narrowly and concretely: **the web↔engine boundary** — the joint between the pure simulation engine (`src/babylon/`) and everything that presents it (`engine_bridge.py` → Django → frontend). Data crosses that stitch via serialization. The recurring bug pattern is that the stitch *drops threads*: a value exists on one side and silently fails to appear on the other. "Lighting the seam," "fixing the seam," "seam-coverage gate" all refer to making that specific joint faithful and verifiable.

## The higher-level / mathematical framing

There isn't a single formal "seam" theorem, but the idea maps cleanly onto several rigorous concepts, and this is where it gets interesting for Babylon:

**1. An interface / boundary as a morphism.** In type theory and category-theoretic terms, a seam is the **arrow between two objects** — a mapping `f: A → B` where `A` is the engine's internal representation and `B` is the wire representation. A *faithful* seam is one where the morphism preserves the structure you care about (no information silently lost). The bugs are cases where `f` is **not injective** — distinct engine states collapse to the same (or a null) wire value, so information dies at the boundary. Your `(payload, wire-key)` keying is literally forcing `f` back toward injectivity.

**2. The "fibered" or round-trip property.** The cleanest formalization: a seam should satisfy a **section/retraction** law. If `serialize: Engine → Wire` and `deserialize: Wire → Engine`, you want `deserialize ∘ serialize = id` on the observable fields. That's exactly the `to_graph`/`from_graph` round-trip you keep testing — a seam is *lossy* precisely when that composition isn't the identity. In algebra this is a **split mono/epi**; the failures are where the round-trip isn't split and data falls into the kernel (gets zeroed).

**3. Cut / interface in a decomposed system.** In systems theory and graph terms, a seam is a **cut** — the edge set you'd sever to separate two subsystems. The "coverage" question ("is everything on side A represented on side B?") is asking whether the cut is a **complete interface**: does every dependency that crosses the boundary have a declared, checked channel? This is why the subrepo-decomposition conversation and the seam-observability conversation are the *same* conversation — a repo boundary is just a seam you've made physical, and both demand the crossing edges be enumerated.

**4. Information-theoretic honesty.** The liveness sensor is really asking whether the **channel** across the seam has capacity actually being used — a field typed `float | None` that's always `None` is a channel with zero mutual information between engine and screen. "Blank Φ" is a seam that type-checks (the channel *exists*) but carries no signal (the channel is *dead*).

## The synthesis

So: **an engineer's seam is a substitutable boundary; the mathematical generalization is a structure-preserving morphism between two representations, and a *good* seam is one where that morphism is faithful (injective on observables) and round-trippable (splits).** Babylon's whole seam-observability project is an attempt to take a concept that's normally informal ("the joint between engine and UI") and hold it to the algebraic standard — prove, mechanically, that the boundary map loses nothing it shouldn't. That's why it feels like more than testing: you're asserting a mathematical property of a boundary, not just checking that code runs.
