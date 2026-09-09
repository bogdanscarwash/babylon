# Bundle-Consequence Closure Audit — Program 28 §7 Criterion 4

**Date:** 2026-08-11. **Auditor:** read-only workforce sweep (live `gh` state + all
issue comments, ADR176/187/190/191/192, the roadmap spec's 2026-08-10 correction,
and direct code/PR archaeology across `src/babylon/`, `rust/`,
`.github/workflows/`, `ai/decisions/`).

**Question:** §7 criterion 4 — "All seven ruling bundles closed with ADRs" — reads,
per the spec's own correction, as *land each bundle's ruled consequences and
close #376–#382 with evidence*. How far along is that, really?

**Headline:** all seven issues are OPEN and **none is ready to close**. Six of
seven are non-trivially incomplete; #381 (narrator) is nearest, with one
consequence remaining. The rulings themselves are complete (ADR176 ruled the §11
queue in full; both named residuals — the Article V 3×3 ratification and the
player-facing trunk names — stand discharged via ADR187/ADR190 and ADR176 item 36
respectively; #383 already closed).

## Per-issue status

| Issue | Bundle | Score | Sole/major remaining work |
|---|---|---|---|
| #376 | endings & verdicts | 0/3 | `LONG_CONTAINMENT` ending, RED_OGV routing fix, all three W-G crisis-sovereign triggers; `sovereignty_type` still bare-string-stamped at `collapse_transition.py:162,234` |
| #377 | doctrine surface | 1/8 | everything substantive: `DoctrineTrunk` still 3 members, no Major/Minor construct, no militancy parameter in `survival_calculus.py`, `is_goal` still adjudicates. Sequenced behind #378 by ADR186 R7 |
| #378 | strike & verb algebra | 0/11 code | drafting fully done (ADR177/187/190); zero resolvers wired — `matrix.py` unchanged, `resolve_build` written but unregistered, no strike lane, no expropriate/restore/security/counter_intel dispatch |
| #379 | multi-res & spatial | 1/7 | Q11(b) per-identity RNG landed pre-freeze; Q9 data half landed (PR #401) but consumer-wiring design-gated; dispersion fields / grain riders / h3 BIGINT / libm goldens all pending Phase 2 |
| #380 | pacing & long-wave | 3/4 | standing orders + gating checks + coefficient-free pacing all verified; **sole open item: ruling 23's three restoration channels** (fascist devaluation, war devaluation, periphery re-division) — L-sized, unblocked, zero code |
| #381 | narrator | 5/6 | prompt voices, §5 repair, grounding filter, envelope contract all code-verified; **sole open item: the four-tier narration ladder**, already its own issue #27 — closing #381 by cross-reference to #27 is the sensible disposition |
| #382 | persistence & CI | 4/9 | P-J/P-H/P-I/P-30-31-34 done (main-branch merge rules verified live); **P-F half-landed** (`synchronous_commit=off` on `test` role only, not the player role — corrected on-issue 2026-08-11); **P-D is a live correctness gap** (`runtime_db.py` still `MAX(tick)`-hydrates against a BINDING checkpoint-only ruling); P-B and W-I not started |

## The freeze-discipline constraint on "unblocked" items

Most engine-side open items (#376 endings/triggers, #377 doctrine, #378 resolvers,
#380 restoration channels) belong to the **frozen** Python engine's estate. Under
ADR172/ADR183 and the port-as-is directive, their implementation home is the
**Rust/BSL port**, not new Python engine code. The audit's tier list re-reads as:

- **Genuinely actionable now (periphery, outside the frozen engine):** #382 P-F
  (player-role `synchronous_commit`), #382 P-B (`ref_digest` re-key of the reference
  tier). Both S-sized.
- **Live correctness gap worth an early call:** #382 P-D (checkpoint-only
  hydration) — the current code contradicts a BINDING ruling; blocked on the
  ruling-31 checkpoint-frame schema design.
- **Everything engine-shaped lands at the port**, sequenced by the tick-order port
  lane (the Phase 2 BSL content track ADR186 R9 charters). That track is the
  shared blocking dependency for the majority of remaining items across #376,
  #377, #378, #379 — **the port lane IS the criterion-4 critical path.**
- **#380's restoration channels** and **#381→#27's narration ladder** are the two
  large items with no upstream blocker — candidates for their own design/impl
  trains when capacity frees.

## Corrections posted

- #382: the issue's "rulings 28, 32, 33 fully implemented" self-report corrected
  on-issue (ruling 32 is half-landed).

## Rulings received (2026-08-11, ADR194)

Four items reached the Director in a second sitting the same day, each on
options this audit (or a sibling port-train STOP) had surfaced without a
recommendation of its own to act on unilaterally. The Director ruled all four
by selecting the workforce's recommended option. Full text:
`ai/decisions/ADR194_director_rulings_batch2_2026_08_11.yaml`.

- **#491** (audit Q3, within-class wealth distribution) — ruled an EMPIRICAL
  QUANTILE SKETCH: data-driven ACS-derived brackets as a first-class field, no
  imposed functional form. This resolves the canonical-form question in
  direction; the concrete field design remains chartered follow-up, and
  Grinding Attrition / the P(S|A) landing stay blocked until it lands. Issue
  closed.
- **#492** (Currency × unbounded runtime coefficient) — ruled a new legal
  DECLARED-DOMAIN SCALE operation: Currency scaled by a defconst-declared
  positive coefficient that declares its domain up front and enforces it
  loudly at load/eval, reading as machinery rather than a mint under
  Amendment AE. This unblocks Territory's eviction pipeline, Metabolism's
  `entropy_factor`, and the two rules the Lifecycle pack leaves verified-inert
  once the spec chapter and Rust implementation land. Issue closed.
- **#382 P-B** (`ref_digest` re-key) — ruled FORWARD-ONLY: new reference data
  keys by the ADR098 build sha256, existing rows keep their keys until they
  naturally regenerate; no migration, no data-loss mode. #382 itself stays
  open — P-D and W-I remain, and P-B's forward-only implementation is not yet
  shipped.
- **Scheduling** — the current focus holds: B2's tick loop + port lane first,
  then the #380 restoration-channels design train charters. **#381 (narrator)
  closes now** by cross-reference to #27, the four-tier narration ladder's own
  tracking issue — the disposition this audit's own table already flagged as
  "the sensible disposition."
