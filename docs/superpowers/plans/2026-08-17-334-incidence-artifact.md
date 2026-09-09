# Plan — #334 Phase 0: the national-incidence artifact + data program

**Authority:** ADR171 (2026-07-28, National Question ruled — OQ0 MIM+MLP, OQ1 **B+C+I with a declared
overlap policy**, OQ10 E-primary/Λ-secondary, "EXECUTION: Phase 0 + Phase 1 authorized");
ADR208 R29 (2026-08-17, #334 **FLIPPED OPEN** — "its chartered artifact exists nowhere … an
Aleksandrov failure"); ADR208 R2 (ADR171 scope ruled **NARROW** — the Community floors owe a
dedicated ruling, decision memo owed next sitting); **Director ruling, 2026-08-17 second sitting** —
Community's `SUBSTRATE_FLOOR_DEFAULTS` **re-derive from this artifact** before entering BSL content
(this selects option (c) of the sibling memo `checkpoint-a/memo-r2-floors-and-valve.md:176-178`),
which promotes #334 onto the **Checkpoint A critical path**.

**Construction spec:** `reports/national-oppression-proposal.md` §4 Phase 0 (lines 243-250), §2.1
(lines 84-193).
**Motion class:** W-P projection (data production only; no engine contact, no `BoundOpposition`, no
lens). Phase 2 stays BLOCKED per ADR171.
**Status:** plan, nothing built.

---

## §0 — Input-table verification verdict (VERIFIED, no blockers)

Every input the charter names IS registered. Verdict: **GREEN — proceed.**

| input | `data-artifacts.yaml` | rows | mode | catalog |
|---|---|---|---|---|
| `fact_census_poverty` | **:726-736** (sha `6ec12391668d…31ab4e`) | 26,576,550 | `generate` | `data-catalog.yaml:1092-1099` |
| `dim_race` | **:479-489** (sha `e7fe6e449…d227a02d`) | 10 | `generate` | `data-catalog.yaml:771` |
| `dim_county` | **:291-301** (sha `130b7679d…6c72032`) | 3,285 | `generate` | — |
| `dim_poverty_category` | **:469-478** (sha `9849ea803…4812506`) | 60 | `generate` | `data-catalog.yaml:763` |
| `dim_time` | registered (governed table; `time_id=23` ⇒ 2019 confirmed) | — | `generate` | — |

**Key/dimension semantics verified directly against the read-only reference DB this pass**
(`data/sqlite/marxist-data-3NF.sqlite`, `mode=ro`; ORM at `src/babylon/reference/schema.py:1026-1045`,
`:709-741`, `:572-582`):

- `fact_census_poverty` PK = `(county_id, source_id, category_id, time_id, race_id)`,
  `person_count` **NOT NULL** — so an ACS-suppressed cell can only appear as an **absent row** or as
  a literal **0**, never as NULL. That fact is what makes guard G2 necessary rather than decorative.
- `dim_race`: `race_id` 1=`T` Total, 2=`A` White alone, 3=`B` Black, 4=`C` AIAN (`is_indigenous=1`),
  9=`H` White-non-Hispanic, 10=`I` Hispanic (`is_hispanic_ethnicity=1`).
  **Pole map: B→`race_id 3`, C(hicano)→`race_id 10`, I(ndigenous)→`race_id 4`, settler reference
  H→`race_id 9`.** (Note the charter's "B/C/I" pole letters are *nation* names, NOT census race
  codes — Chicano resolves to census `I`, Indigenous to census `C`. Do not let the letters collide.)
- `dim_poverty_category`: `category_id 1` = `B17001_001` ("Total" — the population for whom poverty
  status is determined = the **universe** `u`); `category_id 2` = `B17001_002`
  (`is_below_poverty=1` = **below** `b`).

**Non-blocking pre-existing conditions the plan absorbs (all already declared, none new):**

1. `tools/make_reference_subset.py:405` — `"fact_census_poverty": TablePolicy("skip", _UNREFERENCED_REASON)`.
   No guard **on this table** can run in CI today. Flipping to `full` is 26.5M rows and is
   **owner-gated (ADR169)**. → The plan does **not** request the flip: every guard runs against
   checked-in fixtures (T3a/T3b) plus a local pinned-devshell full run (T6). Recorded as a
   Director-visible ingestion cost, unbudgeted here.
2. `data-catalog.yaml:1096` — `extractor: deleted @ 4ce7c96a^ (src/babylon/data/census/loader_3nf.py)`,
   `disposition: fill`. **No vintage re-cut is possible without reconstructing that extractor.** →
   OQ11 already ruled **pinned vintage** (determinism first), so this plan pins `time_id=23` (2019)
   and never re-cuts. Recorded, not solved.
3. `dist/` is **gitignored** (`.gitignore:183`) and only 5 parquets exist in `dist/data-artifacts/`
   locally — `fact_census_poverty.parquet` is **absent on this box**. → T2 step 1 produces it via a
   targeted `make_data_artifacts.export_table_parquet` call and **sha-verifies it against the
   manifest pin** before any derivation reads it. See T2.

---

## §1 — New measured findings from this planning pass (read-only, no mutations)

These change the plan's shape and belong in the artifact's metadata. All computed at
`time_id=23`, `category_id ∈ {1,2}`, over the **unrestricted 3,218-county** row set (the universe
restriction is T2's job; these are magnitude anchors, not final numbers).

**F1 — The ruled B+C+I partition has never been computed. Its ratio is ~1.87, not 1.55.**

| quantity | B+C+I (the RULED partition) | ADR171 context cites (T−H, demoted) |
|---|---|---|
| `Σu_o` / `Σb_o` | 103,140,228 / 22,634,466 | 124,083,302 / 23,975,901 |
| `q̄` demonstrated floor | **0.219453325** | 0.193224234 |
| `Σu_s` / `Σb_s` (H) | 192,640,740 / 18,536,549 | 192,609,332 / 18,525,178 |
| `p̄` oppressor norm | **0.096223410** | 0.096180065 |
| `ΣE` deprivation mass | **12,709,962 persons** | 12,041,561 |
| `ΣΩ` bribe mass | **23,739,102 persons** | 18,691,613 |
| **ratio Ω : E** | **1.868** | 1.552 |

ADR171's `context` block quotes 1.55 as the instrument-validation figure — computed under `T−H`,
the partition the same ADR **demotes to instrument validation only**. The artifact must publish the
B+C+I number and state plainly that it differs from the 1.55 in the ADR **by construction, not by
defect**. Do not "reconcile" them. (`ai/decisions/ADR171_national_question_rulings.yaml`, context +
OQ1.)

**F2 — ADR171's overlap-policy obligation is DISCHARGEABLE: a computable upper bound exists.**
`B17001` has no B∩I or C∩I cross-tab, so the exact intersection is **not computable**. But
`I − (A − H)` = Hispanic persons who are not White-alone is exactly computable and bounds the
double-count from above: **20,877,584 persons = 20.24% of the B+C+I universe**. That satisfies
"measured where computable and DISCLOSED in the artifact metadata, never silently netted" (OQ1)
without netting anything.

**F3 — The per-pole zero-denominator absence class is ~33× the proposal's figure.** The proposal
budgeted "14/16 cells" — that is *whole-county* absence at the T/H level. Per **pole**, at
`category_id=1`, counties with `u = 0`: **B 164, C(AIAN) 287, I(Hispanic) 15, H 0 → 466 pole-county
cells**. Below-poverty zeros: B 462, AIAN 1,021, Hispanic 182, H 9. The honest-absence law (G5) must
be budgeted at **466 + 14/16**, not 14/16. And note the correlation the proposal names: the largest
absence class is the **Indigenous** pole — the same pole whose single most load-bearing county
(46102 Pine Ridge) is a total hole.

**F4 — The T−pole exactness law is a real, exactly-holding invariant.** `T == Σ(A..G)` per county
holds for **all 3,218 counties** at `category_id=1`, `time_id=23`, max residual **0**. G6 is
therefore a hard equality guard, not a tolerance guard.

**F5 — Floor re-derivation is arithmetically reachable, and one candidate shape lands remarkably
close to the frozen table.** With `p̄ = 0.096223410`:

| CommunityType | census pole | `q_pole` | **F-A** = `q_pole` | **F-B** = `q_pole − p̄` | **F-B′** = `(q−p̄)/(1−p̄)` | frozen |
|---|---|---|---|---|---|---|
| NEW_AFRIKAN | B (3) | 0.232620 | 0.2326 | **0.1364** | 0.1509 | 0.12 |
| FIRST_NATIONS | C/AIAN (4) | 0.247566 | 0.2476 | **0.1513** | 0.1675 | 0.12 |
| CHICANO | I/Hispanic (10) | 0.209543 | 0.2095 | **0.1133** | 0.1254 | 0.08 |
| SETTLER | H (9) | 0.096223 | 0.0962 | **0.0000** | 0.0000 | 0.00 |

**F-B preserves `SETTLER = 0.0` exactly by construction** (the settler pole *is* the norm, so its
excess over the norm is identically zero) — i.e. the structural hegemonic-default claim survives a
data derivation rather than being contradicted by it, which **F-A would contradict** (F-A puts
SETTLER at 0.096, a line change). F-B also breaks the frozen 0.12/0.12 **tie**, ranking
FIRST_NATIONS above NEW_AFRIKAN. Both facts are for the Director, not the workforce (§6).

**F6 — The artifact can speak to 4 of 14 CommunityTypes, and to INCARCERATED not even in
principle.** Poles exist for NEW_AFRIKAN / FIRST_NATIONS / CHICANO / SETTLER. The other ten
(PATRIARCHAL, WOMEN, TRANS, DISABLED, QUEER, UNDOCUMENTED, INCARCERATED, YOUTH, ADULT, ELDER) have
**no B17001 pole**. Worse for one of them: `B17001_001` **structurally excludes the institutionalized
group-quarters population**, so the artifact cannot reach INCARCERATED even in principle — and
INCARCERATED currently carries the table's **highest** floor (0.18,
`consciousness.py:371-377`). §6 must present that as a named non-derivability, never paper over it.

**F7 — `CHICANO ← census "Hispanic or Latino"` is an ADR167-class proxy compression.** ADR171 named
the *Chicano nation*; the census gives a pan-ethnic "Hispanic or Latino" iteration that folds in
Puerto Rican, Cuban, Dominican, Central and South American populations. This is the same class of
disclosure as OQ2's `H`-for-settler-nation and P26's `latin_america`→Non-OECD proxy (`ADR167:33-39`),
and it must be printed in `material_relation`, not assumed. It is a **disclosure**, not a new
ideological choice — but if the Director reads it as a partition question it escalates.

---

## §2 — What gets built: three artifacts, all second-order, none in the reference DB

**ADR098 circularity, resolved explicitly (the proposal's own instruction, line 248):** these are
**SECOND-ORDER PRODUCTS**. They are derived from **registered parquet sources** (never from the
sqlite build product), they get **no `schema.sql` table**, and they are therefore **outside** the
`data:build-db` fixed point. Consequence: `mise run data:build-db` byte identity is **unaffected by
construction** — the reproducibility contract for these three is (a) the generator's own double-run
byte-identity, (b) the manifest sha pin, (c) a regeneration-wipe tripwire test. This is exactly the
**ADR121 hand-maintained-artifact pattern** already carrying `faf_bloc_trade_tons`,
`mit_countypres_rep_share` and the LODES tail (`data-artifacts.yaml:1-30` EXCEPTION note;
generator precedent `tools/make_faf_bloc_tons_artifact.py`; tripwire precedent
`tests/unit/tools/test_faf_artifact_manifest_entry.py`).

### A1 — `county_fips_vintage_crosswalk`
`src/babylon/data/reference/national/county_fips_vintage_crosswalk.csv`
Columns: `fips_engine, fips_acs2019, relation, vintage_note, recoverable`.
Covers the 13 recoverable absences (3 AK reorganizations 02063/02066/02158, 9 CT planning regions
09110…09190 → 09001…09015, 51515 Bedford city VA) **plus** the three retired FIPS the
`scopes.py:163 _load_national_fips` resolver admits and `us_county_territories.json` does not
(02261 Valdez-Cordova, 02270 Wade Hampton, 46113 Shannon), **plus one declared hole row**:
`46102` Oglala Lakota (Pine Ridge), `relation=DECLARED_HOLE`, `recoverable=false` — zero rows at
every `time_id`; predecessor `46113` carries rows only 2010-2014. Pine Ridge is **never** imputed.

### A2 — `national_incidence_county_pole`
`src/babylon/data/reference/national/national_incidence_county_pole.csv.gz`
One row per `(fips, pole)` over the declared universe × 4 poles. Columns:

```
fips, pole, pole_role, universe_u, below_b, rate,
w, sigma_damped, damping_weight,
mass_vs_settler_norm,        # b − u·p̄   (positive ⇒ deprivation; E on oppressed poles)
mass_vs_demonstrated_floor,  # u·q̄ − b   (positive ⇒ premium;    Ω on the settler pole)
lambda_per_capita,           # mass_vs_settler_norm / U(i)
omega_hat_per_capita,        # mass_vs_demonstrated_floor / U(i)
absence_class,               # PRESENT | ZERO_DENOMINATOR | ROW_ABSENT | DECLARED_HOLE | SUPPRESSED
fips_source_vintage          # from A1; "native" when no crosswalk applied
```

`pole_role ∈ {oppressed, settler_reference}`. Both mass columns are computed for **every** pole so
the Appalachian settler negatives fall out of the same arithmetic instead of a special case, and so
no column is null-by-design.
**Absence is a value, never 0.0** — G2. Rows in a non-`PRESENT` absence class carry **empty**
measure cells, not zeros.

### A3 — `national_reproduction_floor`
`src/babylon/data/reference/national/national_reproduction_floor.csv`
Aggregate rows, one per `(pole, universe_variant)` plus pooled rows. Columns:
`pole, universe_variant, counties_present, counties_absent, sum_u, sum_b, rate,
p_bar, q_bar, sum_mass_vs_settler_norm, sum_mass_vs_demonstrated_floor, ratio_bribe_to_deprivation,
overlap_upper_bound, overlap_bound_share, vintage_time_id, notes`.
`universe_variant ∈ {artifact_3153, scopes_3140, unrestricted_3218}` — **the lens declares which it
renders** (proposal lines 121-126); all three ship, the ~1.3×10⁻⁶ `p̄` difference is disclosed rather
than hidden. **This is the file §6's floor memo reads.** Pooled row carries F1's B+C+I figures and
F2's overlap bound.

---

## §3 — The derivation

Pure-function core in the generator module; every guard is an explicit, separately-tested transform.
No pandas index tricks — plain `pyarrow` read + dict aggregation, mirroring
`make_faf_bloc_tons_artifact.py`'s streaming discipline (the poverty parquet is 26.5M rows: filter
`time_id=23 AND category_id IN (1,2) AND race_id IN (1,2,3,4,9,10)` at the pyarrow read, never
materialize whole).

```
step 1  resolve universe        → three declared FIPS sets (A1 applied; DECLARED_HOLE excluded)
step 2  pull cells              → {(fips, race_id, category_id): person_count}
step 3  G6 T−pole exactness     → assert T == Σ(A..G) per county, exact (F4)
step 4  G4 suppression classify → absence_class per (fips, pole) BEFORE any arithmetic
step 5  G1 ratio of sums        → p̄ = Σb_s/Σu_s ; q̄ = Σb_o/Σu_o   (B+C+I pooled)
step 6  G8 overlap disclose     → I − (A − H), per county and pooled; NEVER subtracted
step 7  per-county measures     → w, masses, Λ, Ω̂  — G2 gates every division
step 8  G3 damping              → sigma_damped = |w| · damp(u); damping_weight published
step 9  G5 absence accounting   → counts per absence_class must reconcile to the universe size
step 10 emit                    → A2, A3 deterministically (sorted keys, pinned gzip mtime=0)
```

### Guard register — each an explicit transform with its own test module and mutation legs

Per the standing rule (*sentinel every error CLASS, mutation-validated*) and the in-repo pattern of
**in-suite mutation legs** (`tests/unit/sentinels/test_superstructure.py:5-7` — "The mutation legs
prove each gating rule actually fires"), **not** a `[tool.mutmut]` config change: `paths_to_mutate`
is `src/babylon`-only across all 64 entries (`pyproject.toml:461-526`) and `tests_dir` is three
`tests/unit/` dirs (`:527-531`). Do **not** widen either — a `tools/` entry would drag the whole
`tests/unit/tools/` dir into every mutmut run. Each guard ships a `TestMutation<G>` class whose legs
neuter the guard (or feed a violating fixture) and assert red.

| # | guard | law | red leg (the mutation that must fail) |
|---|---|---|---|
| **G1** | ratio-of-sums | `p̄`/`q̄` = `Σb/Σu`; **`mean(rate_i)` forbidden anywhere** | fixture where ratio-of-sums ≠ mean-of-ratios; swap to the mean ⇒ test reds. Also an AST leg: no `mean`/`statistics`/`/len(` over a per-county rate sequence in the module |
| **G2** | zero-denominator = **ABSENCE** | `u == 0` ⇒ `absence_class=ZERO_DENOMINATOR`, all measure cells **EMPTY**. `0.0` on a diverging ramp renders *at the settler norm* — a fabricated data point (III.11) | replace the guard with `rate = 0.0 if u == 0 else b/u` ⇒ test reds on the 466-cell class (F3). Same error class as `check:aggregation` (`.mise.toml:222-224`: "all-masked group input must yield None, never a fabricated 0.0") |
| **G3** | small-count damping | `σ = \|w\| · damp(u)`, `damp` monotone non-decreasing in `u`, `damp(0)` undefined (G2 already fired), `damp → 1` as `u → ∞`. **Fixtures: Loving TX (u=15), King TX (u=78), Elliott KY (u=31)** — all three saturate `\|w\|` undamped | set `damp ≡ 1` ⇒ the three fixtures re-enter the top of the σ ranking ⇒ test reds. Second leg: non-monotone `damp` ⇒ reds |
| **G4** | ACS suppression policy | a literal `0` `person_count` is **not** proof of a true zero; classify `SUPPRESSED` vs `PRESENT` by a declared, published rule; an absent row is `ROW_ABSENT`, never imputed | feed a fixture with `u>0, b=0` and one with the row missing; collapse the two classes ⇒ test reds |
| **G5** | honest absence | absence counts reconcile **exactly** to `universe_size = counties_present + counties_absent`; budgeted at **466 pole-cells + 14/16 whole-county** (F3), not 14/16; `46102` present as `DECLARED_HOLE` in every universe variant | drop one absence class from the reconciliation ⇒ test reds. Pine-Ridge leg: impute `46102` from `46113` ⇒ reds |
| **G6** | T−pole exactness | `T == Σ(A..G)` per county, **exact equality** (F4: max residual 0 over 3,218); `H ≤ A`; `I ≤ T` | introduce a ±1 residual tolerance ⇒ reds |
| **G7** | FIPS vintage crosswalk | A1 is a partial function, injective on `fips_acs2019`, and **never** maps into `DECLARED_HOLE`; every non-native `fips_source_vintage` in A2 resolves to an A1 row | add a second crosswalk row targeting `46102` ⇒ reds |
| **G8** | overlap disclosed, never netted | the bound `I − (A − H)` is computed and published; **no code path subtracts it from any pole count** | subtract the bound from `Σu_o` ⇒ reds (both the value leg and an AST leg on the pole-sum expression) |

---

## §4 — Registration

1. **Three `data-artifacts.yaml` entries**, hand-maintained (ADR121 pattern), each with:
   `format: csv` / `csv.gz`; `source_table: "N/A — no sqlite table; second-order product derived
   from registered parquet sources (ADR098 disposition, ADR171 Phase 0)"`;
   `generator: tools/make_national_incidence_artifact.py`; `mode: generate`; `rows`; `sha256`;
   `home`; `material_relation` carrying **the disclosures**: the ruled B+C+I partition + pole map,
   F7's Chicano proxy compression, OQ2's H-settler proxy, the point-in-time + carceral-exclusion
   bias with its **conservative direction** (OQ7), the pinned vintage (OQ11), F1's ratio and why it
   is not 1.55, F2's overlap bound, and Pine Ridge as a declared hole.
2. **`data-catalog.yaml` rows** — three `kind: artifact` rows with `source: Census_ACS`,
   `disposition: keep`, `subset_policy: n/a (no sqlite table)`, and a `consumers:` list that starts
   empty and is honest about it.
3. **No `schema.sql` change. No `dim_*`/`fact_*` table.** `data:schema`, `data:build-db`,
   `data:verify-build`, `data:verify-roundtrip`, `data:subset` are all untouched — state this
   explicitly in the ADR so no future reader looks for a table.
4. **Regeneration-wipe tripwire** — `tests/unit/tools/test_national_incidence_manifest_entries.py`,
   modelled line-for-line on `tests/unit/tools/test_faf_artifact_manifest_entry.py:45-70`: pins
   `rows`/`sha256`/`home`/`generator`/`mode` for all three entries and asserts none of the three
   names appears in `make_data_artifacts.ARTIFACTS`. `make_data_artifacts.main()` (no `--check`)
   rewrites the whole `artifacts:` list from its own tuple and **would silently drop these rows**
   (`data-artifacts.yaml:19-30` KNOWN RISK).
5. **Never hand-type a sha256** (`docs/how-to/reference-data-pipeline.rst:63-65`) — the generator
   prints the manifest block; paste it, and the tripwire compares computed-vs-computed thereafter.

---

## §5 — Tasks (one implementer dispatch each)

Environment note that applies to **T2, T6** (and only those): the reference-DB toolchain hard-gates
on **SQLite 3.53.1** and must run in the pinned vendored flake —
`mise run nix -- <cmd>`, or `nix develop .#dataBuild` for builder-class work
(`docs/how-to/reference-data-pipeline.rst:10-17`, `:74-76`;
`.github/workflows/weekly-rebuild-verify.yml:51`). T1/T3a/T3b/T4/T5/T7/T8/T9 are fixture- and
text-only and run on the host venv.

---

### T1 — `county_fips_vintage_crosswalk` (A1) + G7
**Blocked-by:** nothing. Start here; T4 needs it.
**Files:** `tools/make_fips_vintage_crosswalk.py` (new);
`src/babylon/data/reference/national/county_fips_vintage_crosswalk.csv` (new);
`tests/unit/tools/test_fips_vintage_crosswalk.py` (new).
**Steps**
1. Enumerate the engine universe from `src/babylon/data/game/us_county_territories.json` and the
   resolver universe from `_load_national_fips` (`scopes.py:163-176`, `substr(fips,1,2)<'60' AND
   substr(fips,3,3)!='999'`). Publish both sizes; the three-FIPS delta (02261/02270/46113) is a
   crosswalk row each, not a silent drop.
2. Hand-author the 17 rows (13 recoverable + 3 retired + 1 declared hole) from the proposal's
   absence table (lines 127-136) with a `vintage_note` per row citing the reorganization.
3. Red-phase `TestMutationG7` first: injectivity leg, `DECLARED_HOLE`-is-never-a-target leg,
   partial-function leg.
4. Emit deterministically (sorted by `fips_engine`, `lineterminator="\n"`); print the manifest block.
**Verify:** `mise run test:q -- tests/unit/tools/test_fips_vintage_crosswalk.py`; re-run the
generator twice, sha256 identical.
**DoD:** 17 rows, G7 green with all three mutation legs red-on-neuter, `46102` present as
`DECLARED_HOLE`, generator idempotent.

---

### T2 — sha-pinned source access + the derivation skeleton (steps 1-2, no measures yet)
**Blocked-by:** T1 (for the universe resolution).
**Files:** `tools/make_national_incidence_artifact.py` (new — skeleton);
`tests/unit/tools/test_national_incidence_sources.py` (new).
**Steps**
1. **Materialize the canonical parquet source.** `dist/data-artifacts/fact_census_poverty.parquet`
   is absent on a fresh box (`dist/` gitignored). Add a documented `--export-source` mode that
   opens `data/sqlite/marxist-data-3NF.sqlite` **read-only** (`mode=ro`, `uri=True`) and calls
   `make_data_artifacts.export_table_parquet(conn, "fact_census_poverty", dest)` —
   `tools/make_data_artifacts.py:389-409`, the ONLY sanctioned parquet-writing path. Run it inside
   `mise run nix -- …`.
2. **Hard-fail on provenance drift.** Before any read, compute the parquet's sha256 and compare it
   to the `data-artifacts.yaml:732` pin. Mismatch ⇒ `ArtifactGenerationError`, loud, no fallback.
   Same for `dim_race`, `dim_county`, `dim_poverty_category`. **There is no `--from-sqlite`
   derivation path** — the sqlite file is a build product and reading it for derivation is the
   circularity ADR098 forbids; it appears only inside `--export-source`, which *produces* the
   registered source rather than consuming the product.
3. Filtered pyarrow read: `time_id=23`, `category_id ∈ {1,2}`, `race_id ∈ {1,2,3,4,9,10}`;
   `fips` resolved via `dim_county`. Never materialize the full 26.5M rows.
4. Resolve the three `universe_variant` FIPS sets, applying A1.
**Verify:** `mise run nix -- mise run test:q -- tests/unit/tools/test_national_incidence_sources.py`;
a deliberately-corrupted fixture parquet must abort with the sha-mismatch error, not proceed.
**DoD:** sources sha-pinned and loud on drift; three universe sets resolved with published sizes;
skeleton computes nothing yet.

---

### T3a — arithmetic-law guards: G1, G2, G6, G8
**Blocked-by:** T2.
**Files:** `tools/make_national_incidence_artifact.py` (guard transforms);
`tests/unit/tools/test_national_incidence_guards_arithmetic.py` (new);
`tests/fixtures/national_incidence/` (new — small hand-built cell fixtures).
**Steps**
1. Red phase: write all four guards' tests **and** their `TestMutationG{1,2,6,8}` legs first.
2. Implement `ratio_of_sums`, `classify_zero_denominator`, `assert_t_pole_exactness`,
   `overlap_upper_bound`. Each is a separate pure function; none exceeds ~40 lines.
3. G1's AST leg: parse the module and assert no mean-over-per-county-rates expression exists.
4. G8's AST leg: assert the pole-sum expression contains no subtraction of the overlap bound.
**Verify:** `mise run test:q -- tests/unit/tools/test_national_incidence_guards_arithmetic.py`.
**DoD:** four guards green; every mutation leg reds when the guard is neutered; fixtures include one
case where ratio-of-sums and mean-of-ratios provably differ.

---

### T3b — absence + small-count guards: G3, G4, G5
**Blocked-by:** T3a.
**Files:** same generator; `tests/unit/tools/test_national_incidence_guards_absence.py` (new).
**Steps**
1. Red phase first, including the three named small-count fixtures — **Loving TX `u=15`, Elliott KY
   `u=31`, King TX `u=78`** (proposal lines 181-184).
2. Declare `damp(u)` explicitly with its shape stated at declaration and a **written derivation** in
   the module docstring. It is a **measure**, not a stipulated functional form — no imposed sigmoid
   (ADR172 ruling 5; standing no-imposed-forms line). Publish `damping_weight` per row so the
   damping is auditable rather than baked into `sigma_damped`.
3. Implement the suppression classifier with its rule **published in the docstring and the
   `material_relation`**, and the absence reconciliation (`present + absent == universe_size`,
   exact).
4. G5's Pine Ridge leg: any imputation of `46102` from `46113` reds.
**Verify:** `mise run test:q -- tests/unit/tools/test_national_incidence_guards_absence.py`.
**DoD:** three guards green with mutation legs; `damp` monotone and derivation-documented; absence
budget reconciles at **466 pole-cells + 14/16 whole-county**; the three saturating counties are
demoted out of the σ top ranking and the test proves it.

---

### T4 — emit A2 + A3 deterministically
**Blocked-by:** T1, T3b.
**Files:** same generator (emitters);
`src/babylon/data/reference/national/national_incidence_county_pole.csv.gz` (new);
`src/babylon/data/reference/national/national_reproduction_floor.csv` (new);
`tests/unit/tools/test_national_incidence_emission.py` (new).
**Steps**
1. `_open_deterministic_gzip_text` with `mtime=0`, copied from
   `make_faf_bloc_tons_artifact.py:185-193` (which copies
   `make_lodes_tri_county_artifact.py`) — the byte-identity precondition.
2. Sort by `(fips, pole)` / `(pole, universe_variant)`; fixed column set; fixed float formatting
   (choose one `f"{v:.9f}"`-class format and pin it in a module constant — an ad-hoc `repr` is a
   cross-run byte hazard).
3. A3 carries F1's pooled figures, F2's overlap bound + share, and the three universe variants.
4. Print the three manifest blocks (rows + sha256 + home) for hand-entry.
**Verify:** run the generator twice inside the same env, `sha256sum` identical for all three files;
`mise run test:q -- tests/unit/tools/test_national_incidence_emission.py`.
**DoD:** three files emitted, byte-identical across two runs; A3's `ratio_bribe_to_deprivation` on
the ruled partition lands near **1.87** (F1) under the unrestricted variant, and the value under the
declared variant is whatever it is — **do not tune toward 1.55**.

---

### T5 — registration + tripwire
**Blocked-by:** T4.
**Files:** `data-artifacts.yaml` (3 hand entries appended to the EXCEPTION tail);
`data-catalog.yaml` (3 rows); `tests/unit/tools/test_national_incidence_manifest_entries.py` (new).
**Steps**
1. Paste the generator-printed blocks; **never hand-type a sha**.
2. Write the tripwire against `test_faf_artifact_manifest_entry.py`'s two-test shape: pinned-content
   test + "not managed by `make_data_artifacts`" test.
3. Add a one-line pointer to the three new names in the `data-artifacts.yaml:6-18` EXCEPTION note so
   the next reader of the KNOWN RISK block sees them.
**Verify:** `uv run python tools/make_data_artifacts.py --check` (hash verify only, no rewrite);
`mise run test:q -- tests/unit/tools/test_national_incidence_manifest_entries.py`; `mise run check`.
**DoD:** three entries registered; tripwire green; `--check` clean; the EXCEPTION note updated.

---

### T6 — pinned-devshell reproducibility proof + the data lane
**Blocked-by:** T5.
**Files:** `docs/how-to/reference-data-pipeline.rst` (a short "second-order artifacts" subsection);
`.mise.toml` (one task `data:national-incidence`).
**Steps**
1. `mise run nix -- uv run python tools/make_national_incidence_artifact.py` twice from clean;
   prove the three sha256s match the registered pins **on the pinned toolchain**.
2. Prove the **non-interference** claim explicitly: `mise run nix -- mise run data:build-db` then
   `data:verify-roundtrip` — product sha unchanged, because no source and no schema moved. Record
   the before/after product sha in the ADR.
3. Add `[tasks."data:national-incidence"]` next to `data:artifacts` (`.mise.toml:1132`), described
   as second-order and pinned-devshell-only.
4. Document the second-order class in the how-to: derived from registered parquet sources, no
   `schema.sql` table, outside the build fixed point, tripwire-guarded.
**Verify:** the two shas match; `data:verify-roundtrip` green; `mise run check` green.
**DoD:** reproducibility proven on-pin, non-interference proven, task + docs landed.

---

### T7 — **the floor re-derivation memo** (Director deliverable) — see §6
**Blocked-by:** T4 (needs A3).
**Files:** `reports/memo-substrate-floor-rederivation-2026-08-DD.md` (new).
**DoD:** §6's five sections complete, **no recommendation**, every number traceable to an A3 row.

---

### T8 — ADR + state + evidence
**Blocked-by:** T6, T7.
**Files:** `ai/decisions/ADR2NN_national_incidence_artifact.yaml` + `index.yaml`; `ai/state.yaml`.
**Steps** Record: the ADR098 second-order disposition and why no table exists; the eight guards and
their mutation legs; F1 (the 1.87-vs-1.55 construction difference); F2 (the overlap bound as OQ1's
discharge); F3 (466 + 14/16); F6/F7 (the two disclosure classes); the un-flipped CI subset as a
standing owner-gated cost; the four floor-candidate shapes as **options, unruled**.
**DoD:** ADR accepted-status, `index.yaml` updated, `state.yaml` carries the Checkpoint A dependency.

---

### T9 — issue closure + consequence filing
**Blocked-by:** T8.
**Steps** Comment on **#334** with the artifact paths, manifest line numbers, the guard table and
the mutation-leg evidence; close it. File the §7 consequence note as a **fresh issue** (do not
smuggle it into #334) and cross-reference it from the ADR. Post the ADR208 R2 register-hygiene fix
if not already landed (row 13's `#536` citation → `models/entities/consciousness.py:356`).
**DoD:** #334 closed with evidence; consequence issue filed; register row corrected.

**Task count: 9** (T1, T2, T3a, T3b, T4, T5, T6, T7, T8, T9 — T7 runs in parallel with T5/T6 once
T4 lands; everything else is a chain).

---

## §6 — The floor re-derivation deliverable (T7): shape, not values

**Why it exists:** ADR208 R2 ruled ADR171's scope **NARROW** and ordered "the floor table with
provenance" as a decision memo for the next sitting. The sibling memo
`checkpoint-a/memo-r2-floors-and-valve.md` discharged that order and offered options (a) transcribe
verbatim / (b) transcribe with weak rows flagged / (c) **re-derive from the #334 artifact when it
lands**. The Director's second sitting today **selected (c)**. T7 is therefore the *successor* memo:
it supplies the candidate values (c) presupposed. It does **not** re-litigate (a)/(b) and does not
repeat the sibling memo's §1-§3 (models, verbatim table, application mechanism) — it **cites** them.

**Hard boundary, stated in the memo's own header:** *the artifact BUILD is workforce work; the
resulting floor VALUES are a reserved-line ruling* (Constitution IX.5; ADR171 OQ5 already names any
national input to consciousness a Director escalation, baseline-moving). **The memo presents an
option space and makes no recommendation.**

**Required sections:**

1. **What the artifact can and cannot reach.** F6's table: 4 of 14 CommunityTypes have a pole; ten
   do not; **INCARCERATED is unreachable in principle** because `B17001_001` excludes the
   institutionalized population — and it currently holds the highest frozen floor (0.18). State
   that the ten non-derivable rows need a disposition **from the Director**, and name the three
   available dispositions without choosing: keep frozen with provenance unchanged; keep frozen but
   demote `confidence` to reflect the five `data_sources=["estimated"]` rows; or hold the whole
   table until a second artifact reaches those communities.
2. **Four candidate shapes, each with its full 4-row value table** (recomputed from A3 under **each**
   `universe_variant`, so the Director sees the universe sensitivity):
   - **F-A** `floor = q_pole` — the pole's demonstrated reproduction-failure rate. Consequence to
     print: **SETTLER becomes 0.096**, contradicting the ratified structural claim
     ("hegemonic default: no substrate revolutionary consciousness",
     `consciousness.py:420-426`). This is a line change, flagged as such.
   - **F-B** `floor = q_pole − p̄` — the pole's excess over the settler norm. **Preserves
     `SETTLER = 0.0` exactly by construction**; lands at ≈0.136 / 0.151 / 0.113 against frozen
     0.12 / 0.12 / 0.08.
   - **F-B′** `floor = (q_pole − p̄)/(1 − p̄)` — the same, normalized to the non-poor share:
     ≈0.151 / 0.168 / 0.125.
   - **F-C** `floor = frozen, artifact cited as post-hoc corroboration` — no value moves; the
     `data_sources`/`computation_method` fields are rewritten to cite the artifact instead of
     "midpoint of incarceration + mobility proxy range", and `confidence` is re-graded.
3. **What each shape changes mechanically**, with the mechanism cited rather than re-derived:
   the floor is a post-normalization hard clamp on `r` with proportional `l`/`f` redistribution
   (`formulas/consciousness.py:97-107`), fires only when `org_landscape` is non-empty
   (`community.py:451-462`). So the memo must state: every shape **raises** the three oppressed-pole
   floors, which **raises** the minimum `r` those communities keep through total organizational
   destruction — and F-B **breaks the frozen 0.12/0.12 tie**, ranking FIRST_NATIONS above
   NEW_AFRIKAN. Both are substantive, both are the Director's.
4. **Provenance and disclosure ledger.** Every candidate inherits the artifact's declared biases and
   the memo must print them beside the numbers, not in a footnote: the **point-in-time** proxy MIM
   itself discounts (`mt10.txt:5544-5550`) with its **conservative** direction; the **carceral
   exclusion** on an axis whose own grounding cites prison labor as constitutive (OQ7); F7's
   **Chicano ← "Hispanic or Latino"** compression; OQ2's **H-for-settler-nation** proxy; the
   **pinned 2019 vintage** (OQ11); and Pine Ridge's declared hole sitting in the FIRST_NATIONS pole.
5. **The ask, posed precisely.** One sentence per decision: which shape; the ten non-derivable rows'
   disposition; whether the tie-break is accepted; and whether the values enter BSL content directly
   or via a `defconst` table with a §6.5 ceremony if any downstream golden moves.

**Sizing:** T7 is a single dispatch — it computes nothing new (A3 has the numbers) and writes no
code.

---

## §7 — Consequence note: `community_memberships` seeding is NOT this train's work

**Checked the charter.** Issue #334's body says the artifact "**is the designated seeder** for the
STRUCTURALLY_IMPOSSIBLE `community_memberships` seam (**Phase 2 prerequisite #1**)". ADR171 lists
the same thing under **BLOCKED**: "Phase 2 shadow registration is blocked on THREE named
prerequisites (a production writer for `SocialClass.community_memberships` — the Phase 0 artifact is
its designated seeder; …)". The proposal §4 puts the writer in **Phase 2**, not Phase 0.

**Verdict: out of scope for #334.** The charter names the artifact as the *seeder input*, never as
the writer. Building the writer here would (a) exceed the charter, (b) land in the **frozen Python
estate** — the seam row's write site is
`src/babylon/engine/systems/community.py::CommunitySystem.step`
(`src/babylon/sentinels/seam/registry.py:2171`, `LivenessClass.STRUCTURALLY_IMPOSSIBLE`,
"no scenario builder assigns `SocialClass.community_memberships` anywhere in production… only a
code/data change") — which ADR172/AE freezes and Program 28 is porting to Rust/BSL, and (c) collide
with the Community port train (#536 / port-estate-survey row 6.0, filed **BLOCKED — harder than
originally filed**).

**What the artifact makes possible, filed as a fresh issue in T9:**

1. A membership-weight source: A2's `(fips, pole, universe_u)` rows are the per-county population
   weights a `community_memberships` writer needs for the three COLONIAL_AXIS types
   (`CommunityType.NEW_AFRIKAN` / `FIRST_NATIONS` / `CHICANO`) plus `SETTLER`.
2. It unblocks Phase 2 **prerequisite #1 only**. Two prerequisites remain untouched:
   `county_extraction` has no `BoundOpposition` registration (`CouplingGraph.__init__`,
   `core/coupling.py:120-129`, raises `KeyError` on an unregistered endpoint — the coupling row
   throws at import today), and the pole-shape question follows OQ0/OQ1 (already ruled) plus the
   per-nation-sibling-rows-vs-one-binary-opposition choice.
3. The right home is the **Rust/BSL Community port**, not the Python estate — the writer should be
   designed against attributed membership (Amendment AG: the `(member, hyperedge)` pair is a
   first-class payload-carrying element kind), which is precisely what a weighted membership needs
   and what the Python `list[str]` field cannot carry.

---

## §8 — Gates and definition of done

| gate | applies | why |
|---|---|---|
| `mise run check` | every task | lint + format + typecheck + `test:unit` (`.mise.toml:146-148`) |
| `mise run test:q -- <path>` | T1-T5 | scoped, keeps the cache so `--lf` works |
| generator double-run sha identity | T4, T6 | the only reproducibility contract a second-order artifact has |
| `make_data_artifacts.py --check` | T5 | hash verify **without** rewriting (a bare `main()` would wipe the three hand entries) |
| `mise run nix -- …` on-pin run | T2, T6 | SQLite 3.53.1 hard gate (`reference-data-pipeline.rst:10-17`) |
| `mise run nix -- mise run data:build-db` + `data:verify-roundtrip` | T6 | proves **non-interference**: product sha unchanged |
| **`qa:regression` / `qa:vault-regression-ci`** | **not required** | no engine, economics, defines, or baseline file is touched; nothing enters the tick. If either moves, **STOP** — it means something leaked into the engine path |
| **no `tests/baselines/**` change** | all | therefore **no §6.5 ceremony and no `Baselines: blessed(…)` trailer**. If a baseline moves, the plan was violated |
| commit per unit of work | all | `mise run commit -- "type(scope): msg"`; branch `feature/334-national-incidence-artifact` off `dev` |
| `mise run pr:merge -- N` | landing | the one sanctioned merge path; harvest the Copilot review first (ADR181) |

**Overall DoD:** three artifacts registered and sha-pinned; eight guards green with every mutation
leg proven to red; reproducibility proven on the pinned toolchain; non-interference with
`data:build-db` proven and recorded; the floor memo delivered to the Director as an unruled option
space; the consequence issue filed; #334 closed with evidence; ADR + `state.yaml` updated.

---

## §9 — Blockers, escalations, and standing costs

**No input-data blockers.** All five inputs are registered (§0) and the dimension semantics are
verified against live rows. Proceed.

**Two Director-gated items this train must NOT decide:**

1. **The floor VALUES** (T7's memo). Reserved line. The train may build the artifact and compute the
   candidates; it may not select one, and it may not enter any floor value into BSL content until
   the ruling lands. **This is the Checkpoint A coupling: the Community port's floor transcription
   waits on the ruling, not on the artifact.**
2. **F7 — if the Director reads `CHICANO ← "Hispanic or Latino"` as a partition question rather
   than a disclosure**, OQ1 partially reopens and T4 holds. The workforce's read is that it is an
   ADR167-class disclosure (OQ1 already ruled the *named nations*; the census simply has no
   Chicano iteration), so the plan proceeds on disclosure and flags it in the memo. Flag it in the
   T4 PR body so the Director can stop it cheaply.

**Standing costs recorded, not paid here** (all pre-existing, all in ADR171's `negative`
consequences):

- **CI subset flip** for `fact_census_poverty` (`make_reference_subset.py:405` `skip`) —
  **owner-gated, ADR169**. Guards run on fixtures; a CI-executable full-table guard requires the
  flip and a 26.5M-row re-cut.
- **The deleted census extractor** (`data-catalog.yaml:1096`) — no vintage re-cut is possible
  without reconstructing it. OQ11's pinned vintage keeps this dormant.
- **Pine Ridge 46102** — permanent declared hole until a source exists. Never imputed.
- **AIANNH↔county bridge** (N3) and the **domestic outflow series** (N2's `w_dom`) — still absent;
  neither is needed for Phase 0.

**Anti-goals, stated so an implementer cannot drift into them:** no `BoundOpposition` registration;
no fourth `lens` enum member (the enum is closed; Phase 1 is a contract-amendment-entered static
reference overlay); no `chauvinist_pressure` wiring (OQ5/OQ9); no `community_memberships` writer
(§7); no netting of the overlap bound (G8); no `0.0` where data is absent (G2); no tuning any output
toward ADR171's 1.55 (F1).

---

## §10 — Citation index

| claim | source |
|---|---|
| Charter, guards, Pine Ridge hole, second-order declaration | `gh issue view 334`; `reports/national-oppression-proposal.md:243-250` |
| B+C+I ruled, overlap policy, E-primary, Phase 0 authorized, Phase 2 blocked | `ai/decisions/ADR171_national_question_rulings.yaml` |
| #334 flipped open; ADR171 scope NARROW; floor memo owed | `ai/decisions/ADR208_docket_sitting_2026_08_17.yaml` (R2, R29) |
| "the Aleksandrov failure the Constitution forbids" | `reports/data-gap-audit-2026-08-12.md:142`; also `:253`, `:441` |
| Input registrations | `data-artifacts.yaml:291-301, 469-478, 479-489, 726-736` |
| Catalog rows + deleted extractor | `data-catalog.yaml:763, 771, 1092-1099` |
| CI subset `skip` | `tools/make_reference_subset.py:405` |
| ADR121 hand-maintained pattern + tripwire | `tools/make_faf_bloc_tons_artifact.py`; `tests/unit/tools/test_faf_artifact_manifest_entry.py`; `data-artifacts.yaml:1-30` |
| Parquet-write path; manifest rewrite hazard | `tools/make_data_artifacts.py:389-409, 786-816` |
| Loaders-produce-sources invariant | `tools/loader_to_sources.py:22-23` |
| Pinned toolchain, add-a-table procedure, never-hand-type-a-sha | `docs/how-to/reference-data-pipeline.rst:10-17, 19-37, 60-76` |
| `dataBuild` devshell in CI | `.github/workflows/weekly-rebuild-verify.yml:51-53` |
| Data tasks | `.mise.toml:1132, 1144, 1148, 1161, 1165` |
| Fabricated-0.0 error class precedent | `.mise.toml:222-224` (`check:aggregation`) |
| Mutation-leg pattern; mutmut config bounds | `tests/unit/sentinels/test_superstructure.py:5-7`; `pyproject.toml:460-536` |
| Floors, application mechanism, clamp | `src/babylon/models/entities/consciousness.py:356-455`; `src/babylon/engine/systems/community.py:451-462`; `src/babylon/formulas/consciousness.py:97-107` |
| Sibling memo this one succeeds | `checkpoint-a/memo-r2-floors-and-valve.md` (esp. `:176-178`) |
| Seam row | `src/babylon/sentinels/seam/registry.py:2171` |
| Coupling `KeyError` | `src/babylon/domain/dialectics/core/coupling.py:120-129` (via proposal:259) |
| Universe resolvers | `scopes.py:163-176`; `src/babylon/data/game/us_county_territories.json` |
| F1-F7 measured values | read-only queries against `data/sqlite/marxist-data-3NF.sqlite`, `time_id=23`, this planning pass |
