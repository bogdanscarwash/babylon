# P27 Conformance Corpus Transcription — Delta Ledger

**Task 17 of the Phase 1 plan** (`docs/superpowers/plans/2026-07-29-program-27-phase-1-language-and-kernel.md`).
Source corpora, read end-to-end before transcription (F1 discipline):

- `tests/unit/domain/doctrine/test_mechanics.py` — 271 lines (the trap-DSL evaluator + doctrine mechanics)
- `tests/unit/engine/test_event_evaluator.py` — 628 lines (the event-template evaluator)

Rust side: `rust/crates/babylon-bsl/tests/conformance_corpus.rs` + `tests/conformance/*.bsl`
fixtures, loaded through the composed pipeline (`babylon_bsl::rule_pipeline::load_rule`,
built by this task — the first point where every load gate composes, in §4.6 class order).

**The four M8 correction sites were re-verified against the live file before
transcription — all four line numbers hold exactly** (`event_evaluator.py:103`,
`:313`, `:405`, `:439`; no drift since spec-writing).

**Phase-1 scope law applied throughout:** fold/query *execution* needs the Phase-2
query evaluator, so aggregation vectors pin load-time verdicts here (parse,
resolution, §3.4 typecheck, §3.7 bound) and their runtime values ride the Phase-2
vector re-run. Everything expressible in the Task 14 expression core executes for
real. Rows below say which.

## The four III.11 corrections (spec §5 "Grammar-superset honesty", M8)

| # | Python site | Old behavior | New behavior | Rust test |
|---|-------------|--------------|--------------|-----------|
| 1 | `event_evaluator.py:313` | unknown graph metric silently reads `0.0` | unregistered `:metric` is **E-LOAD-011** at content load — the rule never loads | `correction_1_unknown_metric_is_e_load_011_not_zero` (`unknown_metric.bsl`) |
| 2 | `event_evaluator.py:439` | unknown aggregation falls through to `False` — the condition silently never fires | off-set fold operator is a loud typecheck rejection at load | `correction_2_unknown_aggregation_is_a_loud_load_error_not_false` |
| 3 | `event_evaluator.py:405` | unknown comparison operator falls through to `False` | a token outside the closed operator set is not even an atom — **E-LEX-003** at read | `correction_3_unknown_comparison_operator_is_a_lex_error_not_false` |
| 4 | `event_evaluator.py:103` | empty precondition set silently returns `True` | `(when)` is a loud rejection (E-PARSE-020); "always" is written by **omitting** the clause — explicit intent | `correction_4_empty_when_is_rejected_and_omission_is_the_legal_always` (`empty_when.bsl`, `unconditional.bsl`) |

## Documented tightening beyond the four (the §3.4 law, not a correction row)

Python's `sum_strength`/`avg_strength` edge metrics (`test_event_evaluator.py:307-323`)
and unweighted `avg` over intensity fields silently commit the recorded
intensive-aggregation variance error. Under §3.4 these are **E-TYPE-041/042** —
pinned by `intensive_aggregation_is_rejected_where_python_allowed_it`. A weighted
mean with an extensive `:weight` remains legal (`event_wealth_aggregates.bsl`).

## Disposition — `test_mechanics.py` (271 lines)

| Python test (file:lines) | Disposition | Rust side |
|---|---|---|
| `TestRealMvpConditions` ×4 (45–64) | **Executes** — the two MVP trap conditions as loaded rules, `<when>` evaluated against supplied envs | `real_mvp_conditions` (`doctrine_adventurism.bsl`, `doctrine_liquidationism.bsl`) |
| `TestMissingTagIsZero` ×2 (67–75) | **Executes** — absent-reads-0 becomes a DECLARED `:optional :default 0`, carried on `DEFAULT_ALLOWLIST` (6 rows, §3.5 item 4) | `missing_tag_reads_the_declared_default` |
| `TestMeasuredPracticeVocabulary` ×8 (97–144) | **Executes** — practice variables + `@coeff` → `:field`/`:const` bindings (the `@` sigil does not survive); unknown coeff/variable are **load** errors (E-LOAD-010), matching "fails loud" with an earlier failure time | `liquidation_absorbing_state`, `unknown_coefficient_and_variable_fail_loud_at_load` (`doctrine_liquidation_absorbing.bsl`) |
| `TestFullGrammar` ×2 incl. 11 vectors (147–181) | **Executes** — all six comparisons, and/or/not, grouping; Python's infix precedence is STRUCTURAL in s-expressions, so the precedence test transcribes as explicit nesting | `full_condition_grammar` |
| `TestMalformedRaises` ×1/7 params (184–202) | **Executes** — each malformed shape fails through its §4.6 class: empty→read error, dangling operator→E-PARSE-040 arity, missing comparison→E-LOAD-021, `UNKNOWN_TAG`→E-LEX-003 (SCREAMING_SNAKE is no atom class), free RHS symbol→E-LOAD-010, unterminated→read error | `malformed_conditions_fail_loud` |
| `TestCanAcquire` ×6 (205–238), `TestAcquire` ×2 (241–246) | **Out of BSL scope** — doctrine-tree acquisition mechanics are domain code over loaded JSON data, not rule content; they port with `babylon-domain` (Phase 2), not as BSL | ledger row only |
| `TestTheoreticalLabourAccrual` ×3 (249–258) | **Executes** — surplus×allocation with the negative floor and unit-interval clamp written as explicit `if` content (only SILENT clamping is banned, §3.3) | `theoretical_labor_accrual_vectors` |
| `TestTagDecay` ×3 (261–271) | **Executes** — multiplicative decay as an expression; expected values computed by the SAME IEEE-754 formula, no approx literals | `tag_decay_vectors` |

## Disposition — `test_event_evaluator.py` (628 lines)

| Python test (file:lines) | Disposition | Rust side |
|---|---|---|
| `TestCompare` ×6 (124–151) | **Executes** — six-operator vectors incl. the binary64 equality pair | `compare_vectors` |
| `TestGetNestedValue` ×6 (154–179) | **Transcribed as the honest-null law** — dotted paths are qname field paths; missing-key-returns-`None` does NOT transcribe: absence is a load decision (E-LOAD-010 or a declared default), never a skipped `None`; int-to-float coercion is §3.3 promotion | `nested_paths_are_qnames_and_absence_is_declared` |
| `TestAggregateAndCompare` ×7 (182–214) | **Load-level** — any→`exists`, all→`forall`, count/sum/max/min/weighted-mean folds load, typecheck and bound; runtime values ride Phase 2 | `aggregation_fixtures_load_and_bound` (4 fixtures) |
| `TestFilterNodes` ×4 (217–241) | **Partially transcribed** — type filtering is the query's `<enum-ref>`; role filtering is an `it`-predicate (Phase-2 execution); `id_pattern` REGEX is deliberately not expressible in BSL (no string operations, §2.8 Prohibited) — a permanent delta, not a gap | ledger row + query fixtures |
| `TestEvaluateNodeCondition` ×4 (244–283) | **Load-level** — `exists`/`forall` fixtures with threshold predicates | `aggregation_fixtures_load_and_bound` (`event_node_condition.bsl`, `event_forall.bsl`) |
| `TestEvaluateEdgeCondition` ×4 (286–323) | **Load-level** for count; sum/avg strength are the §3.4 tightening above | `event_edge_count.bsl`, `intensive_aggregation_is_rejected_…` |
| `TestEvaluateGraphCondition` ×3 (326–351) | **Executes** — the six Python metrics are the registered `:metric` set; the conjunction evaluates in the expression core with metric values supplied | `metric_conditions_load_and_evaluate` (`event_metric_conditions.bsl`) |
| `TestCalculateGraphMetric` ×3 (354–369) | Metric *computation* is kernel/engine work (Phase 2/3 — a registered metric's value arrives via the binding); the unknown-metric case is Correction 1 | `correction_1_…` |
| `TestEvaluatePreconditions` ×3 (372–417) | **Executes** — "all"→`and`, "any"→`or` (§2.4 coverage table); the empty case is Correction 4 | `precondition_logic_vectors` |
| `TestEvaluateTemplate` ×4 (420–590) | **Executes (resolution selection)** — Python's conditioned Resolutions become guards in ONE effect list, executed against a real substrate both ways (`bifurcation_routes_by_solidarity_density`); preconditions-unmet ≙ `<when>` false. **Cooldown_ticks does not transcribe to BSL** — it is scheduler state, engine-side (Phase 3 anchor registry), recorded here, not silently dropped | `event_bifurcation.bsl` |
| `TestGetMatchingNodesForResolution` ×1 (593–628) | **Engine-side** — target selection (`${node_id}` interpolation) is the engine's per-node rule application (`self` iteration), not rule content; no string interpolation exists in payloads (§2.8) | ledger row only |

## `DEFAULT_ALLOWLIST` population (Task 15's lint, §3.5 item 4)

Six rows — one per transcribed absent-reads-as-0 binding: `mass-link`
(adventurism); `class-analysis`, `militancy` (liquidationism); `solidarity-mass`,
`co-optive-share`, `petty-bourgeois-drift` (the U11 absorbing state). Authority:
spec §5's migration-corpus enumeration + the honest-null reading
`test_mechanics.py:67-75` pins. The corpus lints clean
(`missing_tag_reads_the_declared_default` asserts zero findings).

## Built by this task

- `babylon_bsl::rule_pipeline` — the composed load entry point (read → surface →
  bindings → §3.4 fold typecheck → anchors → resolution → free variables → §3.7
  bound), plus `bind_environment` (the §3.5 evaluation-side half: required-missing
  is loud, `:optional` takes its declared default — no rule observes absence).
- The fold-typecheck adapter resolves binding-name bodies through their `:field`
  sources; compound fold bodies are rejected loudly as Phase-1-unverifiable
  rather than passed unchecked (III.11).
