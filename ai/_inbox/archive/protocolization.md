Good instinct to ask — it's a small typing thing, but it does touch the seam.

## The pattern in question

All over `engine_bridge.py`, the bridge calls optional persistence methods defensively, like this:

```python
query = getattr(self._persistence, "query_tick_events", None)
if callable(query):
    try:
        event_rows = query(session_id, state.tick)
    except Exception:
        logger.exception(...)
        event_rows = []
```

It uses `getattr(..., None)` because `self._persistence` can be **either** a full `PostgresRuntime` (which *has* `query_tick_events`, `query_tick_summary_series`, etc.) **or** a leaner SQLite `RuntimeDatabase` (which doesn't). So the bridge asks "do you have this method?" at runtime rather than assuming it.

**Why Pyright complains:** the result of `getattr(x, "name", None)` is typed as `object | None`. Pyright can't know `query(...)` returns a `list[dict]`, so assigning it to `event_rows: list[dict[str, Any]]` trips `reportAssignmentType`. **mypy** (your actual gate) tolerates it because it treats the dynamic result more permissively. So it's a real code smell, not a real bug — the runtime works.

## What "a typed Protocol" would mean

A `Protocol` is Python's structural-typing interface — "any object that has these methods, with these signatures." Instead of the stringly-typed `getattr`, you'd declare the optional query surface once:

```python
from typing import Protocol, runtime_checkable

@runtime_checkable
class TickQueryCapable(Protocol):
    def query_tick_events(self, session_id: UUID, tick: int) -> list[dict[str, Any]]: ...
    def query_tick_summary_series(self, session_id: UUID) -> list[dict[str, Any]]: ...
```

Then the call site becomes typed:

```python
if isinstance(self._persistence, TickQueryCapable):
    event_rows = self._persistence.query_tick_events(session_id, state.tick)
```

Now the return type is *known* (`list[dict[str, Any]]`), Pyright is satisfied, and — more importantly — the **contract of what an optional persistence backend must provide is written down in one place** rather than implied by ~8 scattered `getattr` strings.

## Why it might be genuinely relevant to you

This connects to two things you're already thinking about:

1. **The contract seam.** The `getattr`-by-string pattern is exactly the kind of implicit, un-typed coupling that makes the seam fragile — a method rename or signature drift fails silently (the `getattr` just returns `None` and the feature quietly no-ops). A Protocol turns that into a *checked* contract: rename `query_tick_events` and every call site lights up red at type-check time. That's the same "make the seam explicit and verifiable" discipline that the whole Program 17 work has been about.

2. **The subrepo decomposition.** If the engine/persistence layer ever moves behind a repo boundary, a named `Protocol` (in `kernel/services.py` or `persistence/protocols.py`, where your existing `RuntimePersistence`/`TraceCollector` protocols already live) becomes the *published interface* between the sides — far better than each caller guessing method names via strings. The project already favors this: your memory notes "Protocol + Default impl" as a house pattern (`RuntimePersistence`, `PostgresRuntimeExtensions`, `VectorStoreProtocol`).

## The honest caveats

- It's **not a bug fix** — purely tightening types + silencing Pyright. mypy is already green, so it's optional polish, not correctness.
- There's a subtlety: `@runtime_checkable` Protocols only check method *names* at runtime, not signatures — so `isinstance` narrowing is slightly weaker than it looks, though the static-analysis benefit is the real prize.
- Scope: doing it properly means defining the protocol next to the existing persistence protocols and converting the ~8 `getattr` sites — a small, self-contained refactor, worth bundling with the persistence/contracts work rather than as a one-off.

So: worth doing, ideally as part of formalizing the persistence contract (especially if the subrepo split goes ahead) — but it's discipline/hygiene, not a defect, which is why I flagged it as "say the word" rather than doing it unprompted.
