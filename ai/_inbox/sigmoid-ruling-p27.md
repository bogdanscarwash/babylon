# P27 Task 8 — Transcendental Intrinsics: the Surviving Set and the Ruled Mechanism

**Status: RULED, not open.** Spec §13 open ruling 2 ("sigmoid/transcendental
intrinsics: polynomial approximation vs pinned deterministic libm") was decided
by **ADR176 ruling 21** (2026-07-29): *"P27 Task 8: transcendentals cross via a
PINNED SOFT-FLOAT LIBM crate with golden vectors per intrinsic."* This document
is the analysis artifact Task 8 owes — the surviving-set enumeration ADR173's
reframing requires as its opening move, plus the concrete shape of the ruled
mechanism — so Phase 2 can populate `IntrinsicHost` without reopening anything.
No intrinsic is implemented by this document (Task 8 is not a code deliverable).

## 1. The enumeration (ADR173's required opening)

Fresh call-site census of `src/babylon/` (production code, tests excluded),
2026-07-30, cross-checked against the adversarially-verified sweep in
`reports/p27-proscription-audit-2026-07-29.md` (29 agents; its §"Consequences
for the P27 intrinsic table" is the authority this section condenses):

| Intrinsic | Live Python sites (frozen reference) | Rust-engine disposition |
|---|---|---|
| `sigmoid` (as `exp` in a logistic) | 5 families: `formulas/survival_calculus.py:43`, `domain/economics/reserve_army/calculator.py:52`, `models/entities/precarity_state.py:91`, `domain/bifurcation/consciousness.py:66`, `formulas/reactionary.py:91` | **Not an intrinsic.** Every site is an imposed response curve on a class-mean scalar. P(S|A) does not port (ADR172 r5 + ADR173 r1: the S-curve EMERGES as the measure of members clearing subsistence); the other four are per-mechanic Phase-2 transcription decisions under the same directive, each already dispositioned in the audit. |
| `tanh` | 1: `formulas/market.py:107` (scissors balance) | **Not an intrinsic.** Re-derives to the `(p−v)/(p+v)` ratio algebra other oppositions already use — a transcription decision, not new math. |
| `exp` (non-sigmoid) | `formulas/sustained_exploitation.py:198` (chauvinist Gaussian), `engine/systems/contradiction.py:455` (inverse of a log-ratio), `engine/field_registry.py:194` (saturation), `engine/scenarios/_legacy.py:628` (build-time) | **Survives at most as half of a matched `{exp, log}` pair.** The financialization site is an `exp(log(x))` round-trip eliminable by carrying the ratio; the Gaussian is an audited imposed form; scenarios are build-time. |
| `log` | `domain/economics/monetary/anchor.py:89` (FRED coordinate change), `formulas/consciousness_routing.py:45,470` + `models/entities/consciousness.py:290` (entropy), `formulas/survival_calculus.py:90` (the logistic's own inverse — dies with it), `engine/scenarios/_legacy.py:665` (build-time `log1p`) | **Survives at most as the pair's other half.** The anchor is an empirical coordinate change computable at **data-build time** and hashed as declared input; entropy is projection-lane (Amendment S — outside the tick hash) once the one mechanics leak (`ooda/action_effects.py:95`) is closed. |
| `sqrt` | 12 sites: vector norms (`formulas/politics.py`), geodesy (`domain/geography/`), variance (`temporal/anomaly.py`) | **Needs no policy.** IEEE-754 `sqrt` is a *basic* correctly-rounded operation — bit-exact across conforming implementations. Geodesy is substrate/build-time regardless. |
| `atan2` | 3 sites, all haversine geodesy | Build-time substrate derivation; never tick-time. |

**Net surviving tick-time intrinsic set: `{exp, log}` at most, possibly `{}`.**
The audit's conclusion stands: de-imposition shrinks the table from
`{sigmoid, exp, log, tanh, sqrt, entropy}` to at most a matched `{exp, log}`
pair, and the poly-vs-libm question nearly moots itself. The theory correction
and the determinism budget are the same work.

## 2. The ruled mechanism (ADR176 r21), made concrete for Phase 2

- **Crate:** the pure-Rust `libm` crate (rust-lang/libm, the MUSL port) is the
  recommended pinning target — no platform libm linkage, no OS/arch-dependent
  code paths for `exp`/`log`, `no_std`-clean, version-pinned in
  `Cargo.toml`/`Cargo.lock` like every other kernel dep. (`rlibm`-style
  correctly-rounded research libms remain a fallback if a golden vector ever
  exposes cross-platform drift; none is expected from pure-Rust code built on
  basic IEEE-754 ops.)
- **Golden vectors are the enforcement, not the crate's reputation:** each
  registered intrinsic ships a pinned table of `(input bit-pattern → output
  bit-pattern)` vectors — boundary values, subnormals, sign edges, and
  mid-range points — asserted **bit-exactly** in the kernel test suite and
  recorded in `docs/reference/determinism-contract.rst`. A crate upgrade that
  moves any output bit is a red gate and a declared ceremony, never a silent
  improvement. This is the same pin discipline `quantize`, `Currency`, and the
  RNG conformance vector already use.
- **Where it lands:** Phase 2 populates `babylon-bsl`'s `IntrinsicHost` (the
  Phase-1 trait with zero real implementations) with at most `exp`/`log`.
  `sigmoid`, `tanh`, and `entropy` are **never registered** — the typechecker
  rejects them as unknown intrinsics, which is the III.11-loud way to keep the
  proscribed forms from re-entering as content.

## 3. Remaining Director surface (none of it blocks Phase 1 exit)

Task 18's exit gate needs the *ruling* in hand — it is (r21). What remains is
per-mechanic **transcription** work already chartered by the audit, each a
Phase-2 decision on the theory line: the four non-P(S|A) sigmoid families
(reserve army, precarity twin, bifurcation consciousness, reactionary
defection), the scissors `tanh`, and the chauvinist Gaussian. Each either
re-derives as a population measure (audit §2 gives the derivation shape) or
must be individually defended to the Director as materially grounded — the
default under the standing no-imposed-forms directive is re-derivation.
