# Territory BSL Surface Facts — Verified 2026-08-12

Scout pass against `dev` @ `4e0faf2` (main checkout,
`/home/user/projects/game/babylon`), read-only (`rg`/`Read` only, no cargo).
Every fact below carries a `file:line` anchor as of this commit and, where the
fact is a shape, a verbatim quote. Where something is absent, the search run
is stated. This supersedes the relevant parts of
`reports/territory-port-phase1-inventory-2026-08-11.md` §3/§6 where noted —
see **CONTRADICTIONS / FLAGS** at the end; read that section first, it is the
highest-value part of this dossier for planning purposes.

---

## 1. `deffield` type vocabulary, current state

**There are TWO deffield dialects in this codebase, and they are NOT the same
surface.**

### 1.0 The RST-canonical `.bsl` dialect — `declarations.rs::FieldRegistry` — UNWIRED, dead code today

`rust/crates/babylon-bsl/src/declarations.rs:290-304` (doc comment on
`FieldRegistry`):

> "This type IS that Phase-2 registry, waiting on its consumer: rule-side
> `deffield` CONTENT PACKS (fields declared alongside `.bsl` rule content
> rather than re-declared per scenario) are what will wire it in — until then
> it stays fully built, fully tested, and correctly unreferenced from any
> live tick."

Verified: `rg -n "FieldRegistry" rust/crates/babylon-tick/src/*.rs
rust/crates/babylon-tick/tests/*.rs` → **zero hits**. `FieldRegistry` is
exported from the crate root (`rust/crates/babylon-bsl/src/lib.rs:41`) but
has no consumer anywhere in `babylon-tick`, the crate that actually drives
`run_once`/`run_once_into`/`TickSession`. **This dialect is real, tested Rust
code, but nothing on the live tick path reads it.**

Its grammar (`declarations.rs:442-516`, `parse_deffield`):
```
(deffield <qname> :type <type-name> :kind intensive|extensive)
(deffield <qname> :type enum :enum-type <EnumTypeName>)
```
`bool` IS legal here — `parse_type_name` (`declarations.rs:646-675`):
```rust
pub fn parse_type_name(name: &str) -> Result<BslType, DeclError> {
    match name {
        "int" => Ok(BslType::Int),
        "bool" => Ok(BslType::Bool),
        "currency" => Ok(BslType::Currency),
        "probability" => Ok(BslType::Probability),
        "intensity" => Ok(BslType::Intensity),
        "coefficient" => Ok(BslType::Coefficient),
        "enum" => Err(...),  // handled by the caller before reaching here
        other => Err(...),
```
A `bool` field declared this way requires `:kind intensive|extensive` too —
only the `enum` branch is exempt from `:kind` (`declarations.rs:502-515`,
527-535: "`:type enum` forbids `:kind`").

### 1.1 The `.bscn` scenario dialect — `scenario.rs::load_deffield` — THE ONE ACTUALLY DRIVING EVERY LANDED RULE PACK

`rust/crates/babylon-bsl/src/scenario.rs:879-951` (`load_deffield`), grammar:
```
(deffield <qname> <type-symbol> <intensive|extensive>)
(deffield <qname> enum <EnumTypeName>)
```
This is what `babylon-tick`'s `prepare_rules` actually reads
(`rust/crates/babylon-tick/src/lib.rs:121,127-130`: `let scenario =
load_scenario(...)`, `let types = TypeEnv { fields: scenario.fields.clone(),
... }`) — the field registry every landed rule pack (metabolism, vitality,
dispossession, lifecycle, organization) is typechecked and evaluated against.

**(a) The enum row's exact syntax** (`.bscn` dialect,
`scenario.rs:866-951`):
```
(deffield <qname> enum <EnumTypeName>)
```
Verbatim landed example, `rust/crates/babylon-tick/content/scenarios/organization-foundation.bscn:44-45`:
```scheme
(defenum OrgKind (STATE_APPARATUS BUSINESS POLITICAL_FACTION CIVIL_SOCIETY))
(deffield organization/kind enum OrgKind)
```
The "enum" token in the 3rd slot is what selects this alternate 4-slot
reading (`scenario.rs:866-873`, comment): "the type-symbol slot itself being
`enum` is what selects the 4th slot's alternate meaning."

**(b) `probability`/`intensity`/`coefficient` exist and arrive as
`Value::Real` through a `:field` binding.**
- Legal in BOTH dialects (`scenario.rs:919-923`; `declarations.rs:651-653`).
- RST type table, `docs/reference/bsl-language.rst:2312-2320` — all three are
  `binary64, [0.0, 1.0]`, "Kernel scalar."
- **Confirmed at the read site.** `rust/crates/babylon-bsl/src/tick.rs:312-328`
  (`bind_field_value`, the function every `:field` binding resolves through
  in `bind_subject`, `tick.rs:279`):
  ```rust
  fn bind_field_value(qname: &str, stored: f64, types: &TypeEnv, enums: &EnumRegistry)
      -> Result<Value, TickError> {
      let Some(decl) = types.fields.get(qname) else {
          return Ok(Value::Real(stored));
      };
      let BslType::Enum(ty) = decl.ty else {
          return Ok(Value::Real(stored));
      };
      // ... enum-only branch renders Value::Enum instead ...
  ```
  Every non-enum-declared field (`int`/`probability`/`intensity`/`coefficient`/
  `currency`) reads back as `Value::Real(stored)` unconditionally — the
  declared type only affects the LOAD-time seeding domain check
  (`scenario.rs::attribute_value_unit_interval`, `scenario.rs:1210-1300+`),
  never the runtime `:field`-read value shape.
- `GraphSubstrate` attribute storage is bare `f64` regardless of declared
  type (`scenario.rs:1044-1054`, doc on `attribute_value`): "`GraphSubstrate`
  attributes are already `f64` in and out... nothing about the trait
  restricts values to integers."

**(c) `bool` fields — split verdict, and this is the load-bearing finding for
the eviction latch.**
- Legal in the RST canon and in `declarations.rs::parse_type_name` (1.0
  above) — `BslType::Bool` exists as a first-class type (`types.rs:214-226`).
- **NOT legal in `scenario.rs::load_deffield`, the dialect every landed
  content pack actually uses.** The `ty.as_str()` match
  (`scenario.rs:918-930`) is exhaustive over exactly five names:
  ```rust
  let ty = match ty.as_str() {
      "int" => BslType::Int,
      "probability" => BslType::Probability,
      "intensity" => BslType::Intensity,
      "coefficient" => BslType::Coefficient,
      "currency" => BslType::Currency,
      other => {
          return Err(err(format!(
              "deffield `{qname}`: unknown type `{other}` — one of \
               int / probability / intensity / coefficient / currency / enum"
          )))
      }
  };
  ```
  `(deffield territory/some-flag bool extensive)` in a `.bscn` file fails to
  load today, citing exactly this message. Search run to confirm no other
  arm exists: `rg -n '"bool"' rust/crates/babylon-bsl/src/scenario.rs` →
  zero hits.
- **`update-node` has no Bool-typed store path at all, independent of the
  deffield question.** `rust/crates/babylon-bsl/src/structural_verbs.rs:2680-2688`
  (test doc comment, `update_node_against_a_selection_result_writes_the_selected_node`):
  > "The reference's own literal example writes `#t` to a `Bool` field
  > (`organization/holds-office`) — `numeric_write_value` (this module) has
  > no `Bool`-typed store path today (`GraphSubstrate::update_node` stores
  > `f64` only; boolean field storage is a separate, pre-existing gap this
  > task does not own)"
  `GraphSubstrate::update_node`'s signature only accepts `f64`
  (`scenario.rs:1026-1030` calls it with an `f64` return from
  `attribute_value`). There is no code path anywhere in `babylon-bsl` that
  converts a `Value::Bool` into a stored node attribute.
- **Implication for Territory's write-once eviction latch:** BOTH halves of
  "declare a bool field, write it via update-node" are unavailable on the
  live pipeline today. The existing precedent every landed pack uses instead
  is an `int`-declared 0/1 flag field (e.g. `social-class/active`,
  `vitality.bsl:35`, guarded `(= active 1)` at `vitality.bsl:66`, and
  `organization/active`, `organization.bsl:28-29`). A Territory latch should
  follow that same `int` 0/1 convention, not `bool`, unless this gap is
  closed first.

**(d) Currency-typed field storage refusal, confirmed at TWO sites (defense
in depth):**
- `scenario.rs:497-548` (`load_defconst`), the `Atom::Currency(_)` arm
  (`scenario.rs:539-548`):
  ```rust
  Atom::Currency(_) => {
      return Err(err(format!(
          "defconst `{qname}`: a Currency coefficient needs typed \
           attribute storage — the Director ruled (2026-08-11) that \
           this lands with Currency's first real consumer — the \
           `:default` and node-attribute paths refuse one for the same \
           reason, and admitting it here alone would make the literal's \
           legality depend on which form it was written in"
      )))
  }
  ```
- `scenario.rs:1055-1087` (`attribute_value`), the `BslType::Currency` arm
  (`scenario.rs:1067`): `BslType::Currency => Err(err(currency_refusal_message(local, field)))`.
  `currency_refusal_message` (`scenario.rs:1273-1280`):
  ```rust
  fn currency_refusal_message(local: &str, field: &str) -> String {
      format!(
          "node `{local}` field `{field}`: Currency attributes need typed \
           attribute storage — the Director ruled (2026-08-11) that this \
           lands with Currency's first real consumer, not this train — \
           f64 cannot hold i128 micro-units, and this refuses rather than \
           casting lossily"
      )
  }
  ```
  Also reached from `attribute_value_int`'s own `Atom::Currency(_)` arm at
  `scenario.rs:1158`.
- Confirmed still true on current `dev` — no change since the
  2026-08-11 Director ruling both sites cite.

---

## 2. The four Territory-shaped e2e vectors — full verbatim BSL

Source: `rust/crates/babylon-tick/tests/query_lane_e2e.rs` (413 lines, read
in full). Scenario: `rust/crates/babylon-tick/content/scenarios/query-lane-e2e.bscn`
(98 lines, read in full).

### (a) Shape B — select-max + language-level tiebreak feeding update-node against a computed ref

`query_lane_e2e.rs:227-243` (`RULE_SINK_SELECTION`):
```scheme
(rule territory/sink-selection-tiebreak-e2e
  :material-basis "priority sink selection with the section-2.7 language-level tiebreak, guarded by exists (Task 6), feeding update-node against the computed reference — Territory blocker table rows 1-2 (_find_sink_node, territory.py:139-194; the population transfer, territory.py:259-267)"
  :fuel 256
  (bindings
    (binding shape :field territory/shape)
    (binding rate :const territory/displacement-rate))
  (when (= shape 1))
  (effects
    (update-node
      (if (exists (neighbors self EdgeType/ADJACENCY :out NodeType/TERRITORY) #t)
          (select-max (neighbors self EdgeType/ADJACENCY :out NodeType/TERRITORY)
                      (field-of it territory/priority))
          self)
      territory/population
      (add (* (field-of self territory/population) rate)))))
```
Tiebreak rule (§2.7, D45, cited at `query_lane_e2e.rs:204-211`): the FIRST
element in ascending id byte order wins — `sink-a` (id 5, declared before
`sink-b` id 6) is selected over an EQUAL-scored `sink-b`. Explicitly flagged
as a DIFFERENT rule from the frozen `_find_sink_node`'s own mode-ordered
`_PRIORITY_BY_MODE` tiebreak — "the Territory port's own D-record owes a
comparison between the two" (`query_lane_e2e.rs:206-211`).

### (b) Shape A — pull-side fold sum over neighbors reading pre-tick state

`query_lane_e2e.rs:142-155` (`RULE_SPILLOVER`):
```scheme
(rule territory/spillover-e2e
  :material-basis "heat spillover via a pull-side fold reading pre-tick neighbour heat — Territory blocker table row 3 (_process_spillover, territory.py:269-316); proves the section-4.2 chapter-C4 pre-state law end to end through the real run_once_into seam"
  :fuel 256
  (bindings
    (binding shape :field territory/shape)
    (binding rate :const territory/heat-spillover-rate))
  (when (= shape 0))
  (effects
    (update-node self territory/heat
      (add (* (fold sum (neighbors self EdgeType/ADJACENCY :any NodeType/TERRITORY)
                    (field-of it territory/heat))
              rate)))))
```
Pre-state proof (`query_lane_e2e.rs:104-112`): all four subjects fire under
ONE rule in ascending id order; each firing's `fold` reads the SAME pre-tick
`heat` values (`t1` reads `t0`'s original `0.25`, never `t0`'s own
already-written `0.275`) — the §4.2 chapter-C4 collect-then-apply law
(D103), proven end to end through `run_once_into`.

### (c) Shape D — for-each writing a TENANCY-incident subject set

`query_lane_e2e.rs:341-351` (`RULE_PENAL_COLONY`):
```scheme
(rule territory/penal-colony-suppression-e2e
  :material-basis "PENAL_COLONY organization suppression via for-each writing the source node's TENANCY :in neighbours — Territory blocker table row 4 (_suppress_organization, territory.py:353-378)"
  :fuel 128
  (bindings
    (binding shape :field territory/shape))
  (when (= shape 4))
  (effects
    (for-each (neighbors self EdgeType/TENANCY :in NodeType/SOCIAL_CLASS)
      (update-node it social-class/organization (set 0)))))
```
`it` is the bound name for each element `for-each` iterates
(`query_lane_e2e.rs:332-337`); no `:as` rename used here (the default
binding name for a `for-each`/query element is `it` when none is given —
confirmed by usage, not separately re-derived from grammar.rs in this pass).

### (d) Shape C — exists guard / empty-ADJACENCY fallback

`query_lane_e2e.rs:289-303` (`RULE_FALLBACK`):
```scheme
(rule territory/fallback-no-sink-e2e
  :material-basis "the exists-guarded selection's fallback branch, never E-EVAL-021, when a territory has no ADJACENCY neighbour — the plan intro's exists requirement, over _process_eviction_pipeline's sink_id-is-None case"
  :fuel 128
  (bindings
    (binding shape :field territory/shape))
  (when (= shape 3))
  (effects
    (update-node
      (if (exists (neighbors self EdgeType/ADJACENCY :out NodeType/TERRITORY) #t)
          (select-max (neighbors self EdgeType/ADJACENCY :out NodeType/TERRITORY)
                      (field-of it territory/priority))
          self)
      territory/priority
      (set 1))))
```
Without the `exists` guard, `select-max` over an empty query is `E-EVAL-021`
(a tick ABORT) — `query_lane_e2e.rs:273-278`.

**Neighbor typing spelling, all four vectors:** `(neighbors <expr>
EdgeType/<MEMBER> <:any|:out|:in> NodeType/<MEMBER>)` — the fourth (result
NodeType) operand is MANDATORY per D51 (confirmed independently in
`bound_checker.rs:510-521`, `neighbors_ceiling`: "the pre-C8 three-operand
form is E-PARSE-042").

**Scenario node-id map** (`query_lane_e2e.rs:80-93`, matches
`query-lane-e2e.bscn:18-24` declaration order): `t0..t3`=0-3 (Shape A chain),
`source-b`=4/`sink-a`=5/`sink-b`=6 (Shape B), `isolated-c`=7 (Shape C),
`penal-colony-d`=8/`tenant-1`=9/`tenant-2`=10/`non-tenant`=11 (Shape D).

---

## 3. System/anchor registration

**The registry is a hard-coded `HashSet<String>` literal in
`babylon-tick`'s driver code — NOT content, NOT hash-bearing.**

Site: `rust/crates/babylon-tick/src/lib.rs:174-203` (`prepare_rules`):
```rust
let systems: HashSet<String> = HashSet::from([
    "economics".to_owned(),
    "vitality".to_owned(),
    "consciousness".to_owned(),
    "lifecycle".to_owned(),
    "dispossession".to_owned(),
    "metabolism".to_owned(),
    "territory".to_owned(),
    "organization".to_owned(),
]);
```

**Currently registered systems (all 8, verbatim from the literal above):**
`economics`, `vitality`, `consciousness`, `lifecycle`, `dispossession`,
`metabolism`, `territory`, `organization`.

**`territory` IS ALREADY REGISTERED — but not for Territory content.**
`lib.rs:190-197`:
```rust
        // NOT a Territory-port system (§2.3's anchor default names a real
        // content pack; this train ships none — see the query-evaluation
        // plan's Task 15, "this task ships no Territory content"). Added
        // solely so `query_lane_e2e.rs`'s four synthetic, Territory-SHAPED
        // vectors have a legal, honestly-named rule-id namespace to anchor
        // under; same class of minimal driver-scaffolding addition as the
        // four above.
        "territory".to_owned(),
```
See CONTRADICTIONS section — this means the port train needs **zero new
registration code** for `territory` as a namespace; the entry already
exists, added defensively by the query-eval train explicitly so it would
NOT be mistaken for Territory content landing.

**Mechanism.** `ctx.systems` (`LoadContext.systems`, `rule_pipeline.rs:68`,
`211-227` construction) is passed into `check_anchor`
(`rust/crates/babylon-bsl/src/mod_anchors.rs:130-212`), called at
`rule_pipeline.rs:304`. Two paths:
- No `(anchor ...)` form: the rule belongs to the system named by its rule
  id's FIRST segment (`mod_anchors.rs:204-211`):
  ```rust
  let first_segment = rule_id.split('/').next().unwrap_or_default();
  if registered_systems.contains(first_segment) {
      Ok(None)
  } else {
      Err(AnchorError::NoSystemForRule { rule_id })
  }
  ```
  This is what every landed pack uses (`organization.bsl:20-22`: "No anchor
  form: `organization` is already a registered system... this rule's own
  `organization/kind-probe` id resolves it from the rule-id's namespace
  prefix").
- Explicit `(anchor :after|:before <system>)` — also checked against
  `registered_systems` (`mod_anchors.rs:193-198`).

**What registering a NEW system (if `territory` needed a fresh, differently
purposed entry, or a real system beyond the query-eval placeholder) requires:**
one string literal added to the `HashSet::from([...])` in
`babylon-tick/src/lib.rs:174-203` — nothing else. It is a load-time-only,
Rust-side gate: not read from content, not part of any `.bscn`/`.bsl` file,
and **not hash-bearing** — it never touches `state_hash`/`CanonicalState`
(confirmed: `systems` is constructed fresh every `prepare_rules` call from a
hard-coded Rust literal, never serialized into or read from graph state).

---

## 4. Landed-pack conventions

Sources read in full: `metabolism.bsl` (412 lines), `vitality.bsl` (91
lines), `dispossession.bsl` (417 lines), `organization.bsl` (31 lines).

### (a) Nested-if clamp idiom

Exemplar, `dispossession.bsl:361-364` (min-then-max double clamp, D-3):
```scheme
(binding intensity-floor :expr
  (if (> raw-intensity 0) raw-intensity (- 0 0c)))
(binding intensity :expr
  (if (< intensity-floor 1) intensity-floor (- 1 0c)))
```
Two `if`s per clamp, floor first then ceiling — "No scalar min/max in the
grammar... 'nested `if` is doctrinally preferable under §3.3'"
(`metabolism.bsl:392-396`, citing the gap report's Appendix item 2). The same
shape recurs 5+ times in `dispossession.bsl` (per-input floors, sum floor,
sum ceiling, transfer-amount ceiling, deadweight-fraction floor/ceiling) and
twice in `metabolism.bsl` (`new-max`, `new-biocapacity`, lines 397-406).

### (b) c-suffix Real-literal convention and the Real-zero-promotion idiom

`dispossession.bsl:356-360` (comment on the idiom, adjacent to (a)'s quote):
> "The `(- 0 0c)`/`(- 1 0c)` forms are Real zero/one — Lifecycle's own
> promotion trick (`lifecycle.bsl:284`'s header) for the same reason: `if`'s
> two branches must share one static type (E-TYPE-020), and a bare `0`/`1`
> Int literal would not match `raw-intensity`'s Real type. The same trick
> recurs at every clamp in this rule."

The `c` suffix marks a `coefficient`-domain scaled literal (`[0,1]`,
`E-LEX-024`-bounded at lex time); `(- 0 0c)` computes `Int(0) - Real(0.0) =
Real(0.0)` — the ELSE branch of a clamp `if` whose THEN branch is already
`Value::Real` (e.g. `raw-intensity`), so both branches share one static type
per `E-TYPE-020`. `metabolism.bsl:360-366` states the same rule with a
different name ("Real-zero promotion trick") for its own `regeneration`
binding's THEN branch.

**Caveat found while reading `metabolism.bsl:367-381`:** the promotion trick
is needed only where a binding's OTHER branch is `:const`-sourced (which CAN
be a bare unsuffixed `Int`, D-1/D-4's own escape hatch, see 4(c) below); a
product of two `:field`-sourced operands is `Real x Real` from the start and
needs no promotion — an earlier PR #501 revision wrongly added one there and
was corrected on adversarial review (`metabolism.bsl:373-381`).

### (c) Coefficient arrival: `defconst` rows

Landed in the `.bscn` scenario (NOT the `.bsl` rule pack — `defconst` is a
scenario-level form per `scenario.rs:456-497`), verbatim,
`rust/crates/babylon-tick/content/scenarios/dispossession-conformance.bscn:29-42`:
```scheme
  (defconst dispossession/foreclosure-rate 0.5c)
  (defconst dispossession/eviction-rate 0.3c)
  (defconst dispossession/displacement-rate 0.2c)
  (defconst dispossession/concentrated-ownership 0.6c)
  (defconst dispossession/absentee-landlord-share 0.4c)

  ; `DispossessionDefines`, `src/babylon/data/defines.yaml:424-431`.
  (defconst dispossession/weight-foreclosure 0.4c)         ; :425
  (defconst dispossession/weight-eviction 0.3c)             ; :426
  (defconst dispossession/weight-displacement 0.15c)        ; :427
  (defconst dispossession/weight-tax-sale 0.05c)            ; :428
  (defconst dispossession/weight-eminent-domain 0.02c)      ; :429
  (defconst dispossession/deadweight-loss-fraction 0.05c)   ; :430
  (defconst dispossession/transfer-scale 0.01c)             ; :431
```
D-record comment style: a one-line citation to the exact `defines.yaml` file
and line range, sometimes with a per-row trailing `; :NNN` line-number
comment as above. The bare-`Int` scaled-coefficient escape hatch (used by
`metabolism.bsl`'s `entropy-factor-x1e6`, see §8 below) is documented at
`dispossession.bsl:110-167` (D-2) as the SAME mechanism, applied to a
different consumer.

### (d) `floor` intrinsic

Declaration site (kernel signature), `rust/crates/babylon-bsl/src/declarations.rs:816-826`:
```rust
pub fn kernel_signature(name: &str) -> Option<(Vec<IntrinsicTypeName>, IntrinsicTypeName)> {
    match name {
        "floor" => Some((
            vec![IntrinsicTypeName::Real],
            IntrinsicTypeName::Scalar(BslType::Int),
        )),
        "exp" | "log" => Some((vec![IntrinsicTypeName::Real], IntrinsicTypeName::Real)),
        _ => None,
    }
}
```
`DECLARABLE_INTRINSICS` (`declarations.rs:110`): `["exp", "log", "floor"]` —
`floor` landed under ADR188 Row 2 (per `declarations.rs:92-96`, citing
"pinned libm crate r21", `f64::floor` is exact IEEE-754, no libm-precision
caveat).

Usage, `rust/crates/babylon-tick/tests/floor_intrinsic_e2e.rs:33-45`:
```scheme
(intrinsic floor :params (real) :returns int :cost 5)
(rule vitality/floor-e2e-count-deaths
  :material-basis "prove the floor intrinsic clears content, load and evaluation"
  :fuel 64
  (bindings
    (binding population :field social-class/population)
    (binding rate :const economy/rate)
    (binding deaths :expr (floor (* population rate))))
  (when (> population 0))
  (effects
    (update-node self social-class/deaths (set deaths))))
```
A content set must DECLARE the intrinsic (`(intrinsic floor :params (real)
:returns int :cost N)`) alongside the rule before calling `(floor ...)` — no
pack currently landed in `content/rules/` declares or calls `floor` (grep
confirms: `rg -n "intrinsic floor" rust/crates/babylon-tick/content/rules/`
→ zero hits; only the e2e test exercises it). **`vitality.bsl:16-19`'s own
header, written before this landed, still records `floor` as the BLOCKING
gap for Vitality's own Grinding Attrition phase** — this is now stale per
the `floor_intrinsic_e2e.rs` proof; see CONTRADICTIONS.

### (e) `:fuel` declaration and CardinalityCeilings

Every landed rule declares `:fuel <N>` as the second keyword clause of the
`(rule <id> :material-basis "..." :fuel <N> ...)` form (e.g.
`metabolism.bsl:349`, `:fuel 4096`; `vitality.bsl:33`, `:fuel 512`; the
query-lane vectors use `:fuel 256`/`:fuel 128`).

`CardinalityCeilings` origin — built PER SCENARIO LOAD from the actually-
minted node/edge population, `rust/crates/babylon-tick/src/lib.rs:160-173`:
```rust
let ceilings = CardinalityCeilings::new(
    scenario.node_types.iter()
        .map(|(member, count)| (format!("NodeType/{member}"), *count))
        .chain(scenario.edge_types.iter()
            .map(|(member, count)| (format!("EdgeType/{member}"), *count)))
        .collect(),
    HashMap::new(),
);
```
`(neighbors ...)` folds are bounded against the LESSER of the queried
EdgeType's ceiling and the result NodeType's (`bound_checker.rs:500-536`,
`neighbors_ceiling`) — so any Territory rule using `neighbors` over
`EdgeType/ADJACENCY`/`TENANCY` needs BOTH a `NodeType/TERRITORY` (or
`SOCIAL_CLASS`) ceiling AND an `EdgeType/ADJACENCY`/`TENANCY` ceiling —
supplied automatically from the scenario's own minted counts, no separate
manifest declaration needed in the slice-1 driver
(`scenario.rs:239-251`, doc on `LoadedScenario.edge_types`: "the FIRST
consumer of `neighbors` through the scenario-driven `run_once_into` path...
so this gap was latent, not exercised, until now" — landed by the
query-evaluation plan's Task 15, the same train that shipped the four
vectors in §2).

---

## 5. Scenario `.bscn` shapes

### (a) Edge declaration syntax — PRESENT in a committed scenario, including ADJACENCY and TENANCY

`query-lane-e2e.bscn:92-98` (the committed scenario, not a test-only
fixture):
```scheme
  (edge EdgeType/ADJACENCY t0 t1 1)
  (edge EdgeType/ADJACENCY t1 t2 1)
  (edge EdgeType/ADJACENCY t2 t3 1)
  (edge EdgeType/ADJACENCY source-b sink-a 1)
  (edge EdgeType/ADJACENCY source-b sink-b 1)
  (edge EdgeType/TENANCY tenant-1 penal-colony-d 1)
  (edge EdgeType/TENANCY tenant-2 penal-colony-d 1))
```
Grammar (`scenario.rs:1282-1284`, doc on `load_edge`): `(edge <enum-ref>
<local-name> <local-name> <int>)` — `<EdgeType/MEMBER> <source-local-name>
<target-local-name> <strength-int>`.

### (b) Fieldless-node legality

`organization-foundation.bscn:53`: `(node county NodeType/TERRITORY)` — no
attribute forms at all. Cited directly in that file's own header
(`organization-foundation.bscn:27-33`):
> "A fieldless node is legal (`county` below carries no attributes at all —
> scenario.rs's own 'No defaults' contract: an unwritten field errors on
> read, and seeding zeros here would defeat that at the easiest place to
> defeat it)"

Confirmed at the loader too: `scenario.rs:56-58` (module doc): "**No
defaults.** A node with no attributes gets no attributes."

### (c) `defenum` + enum-valued node-attribute syntax

`organization-foundation.bscn:40-61`:
```scheme
(scenario organization/foundation
  (defvocabulary NodeType (SOCIAL_CLASS TERRITORY ORGANIZATION))
  (defvocabulary EdgeType
    (MEMBERSHIP PRESENCE COMMAND TRANSACTIONAL SOLIDARISTIC SOLIDARITY))
  (defenum OrgKind (STATE_APPARATUS BUSINESS POLITICAL_FACTION CIVIL_SOCIETY))
  (deffield organization/kind enum OrgKind)
  (deffield organization/active int extensive)
  (deffield social-class/population int extensive)
  ...
  (node reading-group NodeType/ORGANIZATION
    (organization/kind OrgKind/CIVIL_SOCIETY) (organization/active 1))
  (node precinct NodeType/ORGANIZATION
    (organization/kind OrgKind/STATE_APPARATUS) (organization/active 1))
```
`defenum` member order is the declared-order ordinal
(`declarations.rs:556-559` and `types.rs`'s `EnumRegistry`); members are
written BARE (`STATE_APPARATUS`, not `OrgKind/STATE_APPARATUS`) inside the
`defenum` form itself, but a NODE attribute value is the full `<enum-ref>`
(`OrgKind/CIVIL_SOCIETY`) — confirmed by `scenario.rs:1089-1141`
(`attribute_value_enum`): "an enum-typed field is seeded ONLY as
`<EnumType>/<MEMBER>` — the ordinal is never a surface value."

---

## 6. Multi-pack composition

**Rule-id byte-order sort — `prepare_rules` site:**
`rust/crates/babylon-tick/src/lib.rs:229-242`:
```rust
let mut rules = Vec::with_capacity(rule_forms.len());
for (id, form) in rule_forms {
    let loaded = load_rule_form(form, &ctx).map_err(|e| format!("rule {id} rejected: {e}"))?;
    rules.push((id, loaded));
}
rules.sort_by(|(a, _), (b, _)| a.as_bytes().cmp(b.as_bytes()));
```
§4.2/D16 reference — `docs/reference/bsl-language.rst:4685-4687`:
```
* - D16
  - §4.2
  - Rules at one anchor position evaluate in ascending rule-id byte order.
```

**The us-counties demo composes vitality+lifecycle** —
`rust/crates/babylon-tick/tests/us_counties_demo.rs:11-19`:
```rust
const VITALITY: &str = include_str!("../content/rules/vitality.bsl");
const LIFECYCLE: &str = include_str!("../content/rules/lifecycle.bsl");

#[test]
fn the_demo_scenario_loads_and_ticks_both_packs() {
    let rule_src = format!("{VITALITY}\n{LIFECYCLE}");
    let mut session = TickSession::new(SCENARIO, &rule_src, HypergraphStore::new()).expect("load");
    let mut sink = CollectingSink::default();
    let report = session.advance(&mut sink).expect("tick 1");
```
Composition mechanism: literal string concatenation of two whole `.bsl`
pack sources into one `rule_src`; `split_content` (called inside
`prepare_rules`) parses out both `(rule ...)` top-forms; each is loaded
independently against the shared scenario-derived `LoadContext`; the pair is
then sorted into ascending rule-id byte order (`lifecycle/dpd-circuit`
before `vitality/subsistence-and-death`, confirmed by
`us_counties_demo.rs:22-25`, matching D16 above — concatenation order in the
source string is irrelevant to execution order).

**Two rules at ONE anchor position do NOT share pre-state — confirmed, D-row
Q14/D116.**

`docs/reference/bsl-language.rst:5888-5914` (D116, verbatim):
> "**Query-evaluation plan Q14, recorded not fixed — a second, narrower
> instance of D103's same divergence, one anchor level up.** D103 repaired
> the WITHIN-one-rule half of §4.2's own sentence ("rules within one system
> position observe the same pre-state"): every subject firing of ONE rule
> now observes that rule's shared pre-tick state. The RULE-to-rule half is
> still open: `babylon-tick`'s `run_once_into`/`TickSession::advance` run
> each rule in a content set to COMPLETION — collect and apply — before the
> next rule starts, against the SAME mutable graph, so a second rule at the
> same anchor position observes the FIRST rule's already-applied writes from
> this tick, not the tick's shared pre-state. Latent today: every landed
> rule pack keeps its system position to exactly one rule..."

Matching divergence comment in code, `rust/crates/babylon-tick/src/lib.rs:285-302`
(on `run_once_into`):
> "Every rule in `prepared.rules` runs to COMPLETION (every matching
> subject) before the next rule starts... but it is NOT what §4.2 demands...
> This is a divergence to fix in its own train, not a design feature
> 'inherited for free'."

And `rust/crates/babylon-bsl/src/tick.rs:67`: "Recorded as D-row **Q14**
(the query-evaluation plan's...)" (module-doc, same gap named from the
evaluator side).

**Implication for Territory, and how vitality+lifecycle avoid it today:**
they are NOT at the same anchor/system position — `vitality/*` anchors under
the `vitality` system (Material Base @1.0) and `lifecycle/*` under
`lifecycle` (@7.0), two DIFFERENT positions, so D116's gap (same-position
cross-rule mutation-visibility) never triggers for that pair. Separately,
EVERY landed pack keeps its own system position to exactly ONE rule by
design — `vitality.bsl:7-10`'s own header: "ONE rule, not three. §4.2: rules
within one system position observe the same pre-state, so a three-rule
decomposition would have to restate the drain algebra in each downstream
rule." **A Territory pack with multiple rules at the Territory system
position (a single "territory" anchor, single tick-position number) would
hit D116 directly if any two of its rules read state the other rule's
effects also write** — the port plan should either (i) collapse all four
Territory phases into ONE rule (the `vitality.bsl` precedent, using `:expr`
bindings to share intermediates), or (ii) give each phase its own DISTINCT
anchor/system position (like vitality vs. lifecycle), never two rules
sharing one position with cross-visible state.

---

## 7. Enum comparison + write surface

**Comparison — `=`/`!=` only, confirmed at the landed content and the
evaluator.**

`organization.bsl:29` (guard using `=` against an enum literal):
```scheme
(when (and (= active 1) (= kind OrgKind/STATE_APPARATUS)))
```

Evaluator equality arm, `rust/crates/babylon-bsl/src/evaluator.rs:1589-1608`
(`apply_equality`):
```rust
(
    Value::Enum { enum_type: ta, member: ma },
    Value::Enum { enum_type: tb, member: mb },
) => {
    if ta != tb {
        return Err(EvalError::plain(format!(
            "Enum<{ta}> compares only to the same enum type, found \
             Enum<{tb}> (§3.1)"
        )));
    }
    ma == mb
}
```

**`update-node` CAN write an enum-typed field, via `set` only.**

Test proving the write, `structural_verbs.rs:2850-2868`:
```rust
#[test]
fn update_node_writes_an_enum_ref_and_stores_the_declared_ordinal() {
    ...
    collect_then_apply(..., "(effects (update-node self organization/kind (set OrgKind/POLITICAL_FACTION)))", ...)
        .expect("a matching enum-ref write must succeed");
    let stored = graph.node_attribute(id, "organization/kind").unwrap();
    assert!((stored - 2.0).abs() < 1e-12, "POLITICAL_FACTION is declaration-order ordinal 2, stored: {stored}");
}
```
So the write-surface fact is: `(update-node <ref> <enum-field-qname> (set
<EnumType>/<MEMBER>))` is legal and stores the declared-order ordinal as
`f64`. (Territory's own port, per the task brief, never writes an enum
field — this is recorded as a language-surface fact for completeness, not
because Territory needs it.)

**Ordering/arithmetic on enums — REFUSED, at BOTH load time and eval time
(defense in depth), confirmed at 3 sites:**

1. Load-time static check, `typecheck.rs:307-330`
   (`check_no_arithmetic_on_enum_field`, D118):
   ```rust
   if matches!(op.as_str(), "add" | "sub" | "scale") {
       if let Some(decl) = env.fields.get(qname) {
           if matches!(decl.ty, BslType::Enum(_)) {
               return Err(TypeError { code: None, message: format!(
                   "update-node {qname}: ({op} …) is not a coherent \
                    operation on an enum-typed field — Enum<T> \
                    supports no arithmetic (§2.13, D118); only `set` \
                    may write it. ...") });
   ```
2. Eval-time defense-in-depth, `structural_verbs.rs:1288-1300`
   (`refuse_arithmetic_on_enum_field`), error code `E-EVAL-042`
   (`EnumWriteShapeViolation`).
3. Ordering (`<`/`<=`/`>`/`>=`) refusal, `evaluator.rs:1560-1577`
   (`apply_ordering`), the fallback arm:
   ```rust
   _ => {
       return Err(EvalError::plain(format!(
           "({op} {lhs:?} {rhs:?}) — ordering is defined within one \
            numeric lane only (Enum and Bool compare with =/!= alone, \
            §3.1)"
       )))
   }
   ```

`select-max`/`select-min` scoring also refuses an enum-typed score at LOAD
time (`typecheck.rs:188-207`, `check_selection_scores`, D46): "`Bool`,
`Enum<T>`, `Str`, references and sets are `E-TYPE-016`."

---

## 8. The rent-level lane — the entropy_factor D-1 precedent

`metabolism.bsl:20-207` (D-1, full header) is the exact precedent the
prompt asks about: `entropy_factor`'s domain is `(1.0, 3.0]` — genuinely
`> 1.0`-valued, not `[0,1]` — and `Ratio`'s only legal operator is `Currency
× Ratio` (`docs/reference/bsl-language.rst:2404-2407`, `2454-2458`), which
does not fit because the multiplicand here (`raw_extraction`) is a `:field`
read, never `Currency`-typed (no Currency field storage exists at all, per
§1(d) above). The workaround, verbatim:

Declaration (in the scenario, `metabolism-conformance.bscn:34-36`):
```scheme
  ; Currency x Ratio -> Currency (this formula needs Real x Ratio, which
  ; does not exist) — scaled bare-Int workaround, x1,000,000.
  (defconst metabolism/entropy-factor-x1e6 1200000)
```
Arithmetic (`metabolism.bsl:347-387`):
```scheme
    (binding entropy-factor-x1e6 :const metabolism/entropy-factor-x1e6)
    ...
    (binding ecological-cost-scaled :expr (* raw-extraction entropy-factor-x1e6))
    (binding ecological-cost :expr (/ ecological-cost-scaled 1000000))
```
D-record comment, `metabolism.bsl:61-66`:
> "`entropy_factor` is declared as a scaled bare-`Int` `:const` —
> `(defconst metabolism/entropy-factor-x1e6 1200000)`, `x1,000,000` — and
> divided back out inline (`ecological-cost` below). This is the SAME escape
> hatch Dispossession's own D-2/D-4 already use and document: a bare,
> unsuffixed `Int` `:const` carries NO domain check at all (`E-LEX-024` only
> bounds SCALED/suffixed literals)."

The header (`metabolism.bsl:57-207`) is extensive and self-critical: it is
explicitly "NOT bit-exact against the frozen engine", the deviation is
"UNBOUNDED in general" once cancellation is involved (two measured
counterexamples up to 32768 ULP apart, and one clamp-branch-flip case where
the two engines land on opposite sides of `max(0.0, ...)`), and it names the
real fix — a genuine `Real × Ratio` operator or `Ratio`-typed field storage
— as chartered but NOT attempted work (workstream 3 of the post-port
refactor program, GitHub issue #502). **Territory's `rent_level` (float 1.0
init, ×1.5 spikes on eviction, per the port brief) is the exact same shape
as `entropy_factor`: a `:field`-carried multiplier outside `[0,1]`, feeding
a formula where the multiplicand is never `Currency`.** Territory's
`rent_level` will need the SAME `-x1e6`-suffixed bare-`Int` `:const`-or-field
workaround (mutatis mutandis for whichever binding kind rent_level ends up
using — `:const` if uniform, `:field` if genuinely per-territory, per the
Dispossession D-1 "is it PROVABLY uniform" test at `dispossession.bsl:16-47`)
unless workstream 3 lands first.

---

## CONTRADICTIONS / FLAGS against `reports/territory-port-phase1-inventory-2026-08-11.md` §3/§6

The inventory report predates BOTH the query-evaluation train (query-lane
e2e vectors, `neighbors`/`fold`/`select-max`/`for-each`/`exists`/`field-of`
all landed and now EVALUATE) and the org-foundation train (`defenum`,
`deffield ... enum ...`, ADR195/ADR196). Concretely, on current `dev`:

1. **`territory` is already a registered system** (`babylon-tick/src/lib.rs:
   190-197`) — added by the query-eval train SOLELY so its four synthetic
   vectors have a namespace, explicitly marked "NOT a Territory-port
   system... this train ships none." If the inventory report's §3/§6 still
   lists "register the territory system" as outstanding work, that line
   item is now ZERO-COST (one string already present) rather than new
   plumbing — but the port should probably still discuss with the Director
   whether to reuse this exact entry or make the real registration a
   deliberate, separate act (the comment at that call site was written
   specifically to prevent the placeholder from being mistaken for the real
   thing).

2. **The four query-lane blockers the inventory report's §6 table names as
   blocking (sink selection with a tie, population transfer against a
   computed reference, heat spillover reading pre-tick neighbour state,
   `for-each`-driven suppression) are now UNBLOCKED and have working,
   tested reference syntax** — see §2 above, verbatim. If §6 of the
   inventory report still marks these rows as blocked/blocking, that is
   now stale; the port train can transcribe Territory's four phases
   directly against these four templates.

3. **`bool` fields are NOT available on the live `.bscn` pipeline today**,
   despite being a legal RST type (§3.1) and legal in the
   unwired `declarations.rs::FieldRegistry` dialect. If the inventory
   report's §3 assumed `bool` was simply usable because "it's in the spec,"
   that assumption does not hold against the actual scenario loader
   (`scenario.rs::load_deffield`) every landed pack is typechecked through.
   This is the single most important correction for the eviction-latch
   design: use an `int` 0/1 flag (the `active`-field precedent), not `bool`.

4. **`floor` has landed** (ADR188 Row 2, `DECLARABLE_INTRINSICS`
   includes it, `declarations.rs:110`, and `floor_intrinsic_e2e.rs` proves
   it end to end through `run_once_into`) — but `vitality.bsl`'s own header
   (`vitality.bsl:16-19`) still describes `floor`/`trunc` as a blocking gap
   for its own Grinding Attrition phase ("§3.10's rider slate row 2 records
   `floor`/`trunc` as a PROPOSAL"). That comment predates the floor landing
   and is stale in the same file it lives in — worth noting since the same
   kind of staleness could easily be present in the inventory report if it
   was written before or around the same time.

5. **The rule-pack-internal `deffield` dialect (RST-canonical,
   keyword-based, `declarations.rs::FieldRegistry`) is fully implemented,
   fully unit-tested, and has ZERO consumers in `babylon-tick`.** If any
   planning document assumes Territory content can declare its own fields
   inline in a `.bsl` pack (rather than via the `.bscn` scenario's
   `deffield` rows), that assumption is currently false on the wired path —
   all field declarations for a live tick come from the scenario file today
   (`babylon-tick/src/lib.rs:121-130`).

6. **Cross-rule pre-state sharing at one anchor position is a live, open gap
   (D116/Q14)**, not fixed by the query-eval train (which deliberately
   avoided exercising it — each of the four vectors loads exactly one rule
   per tick, `query_lane_e2e.rs:17-25`). Any Territory-port design assuming
   "put heat spillover, eviction pipeline, and necropolitics in one rule
   each, all at the `territory` anchor" needs to either merge them into one
   rule (vitality's own precedent) or place them at genuinely distinct
   anchor/system positions — see §6 above for the full argument.
