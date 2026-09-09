<!-- vale off -->

# Neel T0 Theory Refoundation Implementation Plan

> **Source-correction notice (2026-08-25):** This file records the T0 execution
> contract as it stood on 2026-08-23. Its whole-clipping source literals are
> superseded and must not be copied or re-executed. The current authority is
> `ai/theory.yaml` v2.1.0, its human rendering, and the page-complete study in
> `docs/superpowers/research/2026-08-25-neel-source-study.md`.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace Babylon's categorical canonical-theory claims with the approved relational, contingent theory contract; consolidate Director source exclusions in one machine-readable policy; and prove the result with focused behavioral tests.

**Architecture:** `ai/theory.yaml` remains the sole machine-readable theory record and `docs/concepts/theory.rst` remains its human rendering. Within the active T0 policy, theory, and test surfaces, `src/babylon/data/corpus/manifest.yaml` remains the only file allowed to contain Director-excluded source identifiers; a typed `ExclusionPolicy` lets tests derive that set without repeating it. A new focused governance test validates the theory schema, provenance ledger, exclusion hygiene, machine/human parity, and the distinction between theoretical constraint, frozen Python reference behavior, and live Rust law.

**Tech Stack:** Python 3.12, Pydantic v2, PyYAML, pytest, reStructuredText, Vale 3, yamllint, Ruff, mypy, mise.

**Spec:** `docs/superpowers/specs/2026-08-23-neel-relational-territory-practice-design.md`, especially sections 2, 6, and 11.

## Global Constraints

- Implement only Train T0. Do not implement the relational territory dossier, practice circuit, emergence evaluator, gameplay actions, BSL mechanics, coefficients, thresholds, or response curves.
- Do not create a second theory registry. The canonical pair remains `ai/theory.yaml` and `docs/concepts/theory.rst`.
- Preserve the constitutionally reserved Marxist-Leninist-Maoist Third Worldist (MLM-TW) line exactly. T0 corrects deterministic interpretations within that line; it does not rename, replace, or amend it.
- `ai/mantras.yaml` changes only at the verified categorical sentence in `mantras.north_star.meaning`, plus its metadata version.
- Within the active T0 surfaces, Director-excluded identifiers may appear only in canonical denial rows in `src/babylon/data/corpus/manifest.yaml`. Do not repeat them in changed Python, tests, comments, docs, commit messages, or review notes. Immutable history and unrelated tracked source stay outside this train.
- The executor must receive the binding three-item exclusion set from the Director before Task 1. Stop before implementation if that set is unavailable; do not infer identifiers from historical documents.
- Keep standalone narrator/RAG denial of the approved 1935 source unchanged. Its use here is bounded research evidence, not narrator doctrine.
- Every substantive value must use exactly one evidence class: `Observed`, `Derived`, `Calibrated`, or `Designed`.
- The four supplied source digests are immutable inputs. Do not substitute editions or recalculate the two externally supplied PDF digests from a different file.
- Historical formulas that remain in frozen Python are reference or surrogate behavior, never live Rust law.
- Do not add `implemented_in` claims. T0 does not establish a new executable binding.
- Equal canonical source bytes must continue to parse deterministically. Reject malformed policy rows loudly.
- `MAX_CORPUS_MANIFEST_ROWS = 4_096` is a technical parser ceiling. The top-level manifest and every row reject unknown fields, the parser refuses row 4,097, and every new row scan slices to the named maximum before iterating.
- Use no user-specific absolute path in repository content.
- Use TDD for every task: observe the intended RED, make the smallest GREEN change, refactor without changing behavior, then commit.
- PER-51 is the sole T0 implementation owner. Before Task 1, refresh PER-50 and PER-51 from Linear, require PER-50 to remain the umbrella, require PER-51 to be Todo and unblocked, and move PER-51 to In Progress. The regular lane still targets `dev`.
- Do not run Sphinx. Run targeted Vale only on the changed RST page.
- Keep unrelated user changes untouched and stage only the exact files listed by each task.

---

## File Structure

### Create

- `tests/unit/governance/test_theory_contract.py` — the focused T0 behavioral contract. It owns schema, provenance, exclusion, retired-claim, machine/human parity, and orientation assertions.

### Modify

- `src/babylon/intelligence/corpus_manifest.py` — add the typed Director-exclusion marker and validated query method; remove excluded identifiers from generic source prose.
- `src/babylon/data/corpus/manifest.yaml` — mark the complete three-row Director exclusion set. This remains the only permitted literal home for those identifiers within the active T0 surfaces.
- `tests/unit/intelligence/test_corpus_manifest.py` — prove the typed exclusion contract and replace identity-bearing fixtures with neutral denied-source fixtures.
- `tests/unit/tools/test_ingest_corpus.py` — preserve allow-minus-deny coverage with neutral fixture identities.
- `ai/theory.yaml` — replace the stale categorical theory record with the versioned T0 contract and source ledger.
- `docs/concepts/theory.rst` — render the same seven constraints for humans and distinguish theory, frozen reference behavior, and live implementation.
- `ai/mantras.yaml` — correct the one categorical geography sentence and bump its patch version.

### Explicitly unchanged

- `CONSTITUTION.md`, `NORTH_STAR.md`, `docs/concepts/architecture.rst`, every ADR, every BSL file, every Rust crate, frozen Python mechanics, formula documentation outside the canonical pair, corpus ingestion behavior, and Linear rows other than PER-51's status and implementation evidence.

---

### Task 1: Consolidate the Director Exclusion Policy

**Files:**

- Modify: `src/babylon/intelligence/corpus_manifest.py:77-171`
- Modify: `src/babylon/data/corpus/manifest.yaml:12-204`
- Modify: `tests/unit/intelligence/test_corpus_manifest.py:1-150`
- Modify: `tests/unit/tools/test_ingest_corpus.py:1-120`

**Interfaces:**

- Consumes: existing `CanonStatus`, `CorpusRow`, `CorpusManifest`, `parse_manifest()`, and `load_bundled_manifest()`.
- Produces: `ExclusionPolicy`, `CorpusRow.exclusion_policy`, and `CorpusManifest.director_excluded_rows() -> tuple[CorpusRow, ...]`.
- Invariant: `ExclusionPolicy.DIRECTOR` is valid only with `CanonStatus.DENY`.
- Invariant: the bundled manifest returns exactly three unique Director-excluded rows.

- [ ] **Step 0: Establish live Linear ownership**

Refresh PER-50 and PER-51 through the repository's Linear workflow. Require
PER-50 to remain the umbrella, PER-51 to be Todo, and PER-51 to have no open
blocker. Move PER-51 to In Progress and record this plan commit as the execution
basis before the first repository edit. Stop on any ownership or dependency
conflict; do not implement under PER-50.

Resolve the execution base mechanically and record its literal SHA in the
PER-51 comment:

```bash
t0_plan_base_sha="$(git log -1 --format=%H -- docs/superpowers/plans/2026-08-23-neel-t0-theory-refoundation.md)"
test "$(git rev-parse HEAD)" = "$t0_plan_base_sha"
```

The equality must pass before Task 1 edits. Later review commands recompute the
same plan-file commit; they do not use the older design-spec commit as a diff
base.

- [ ] **Step 1: Add RED tests for the typed policy**

In `tests/unit/intelligence/test_corpus_manifest.py`, import `ExclusionPolicy` and `load_bundled_manifest`, then add these tests beside the existing row-schema tests:

```python
def test_director_exclusion_requires_deny_status() -> None:
    with pytest.raises(ValidationError, match="director exclusion"):
        parse_manifest(
            {
                "rows": [
                    _row(
                        canon_status="allow",
                        exclusion_policy="director",
                    )
                ]
            }
        )


def test_manifest_rejects_unknown_top_level_and_row_fields() -> None:
    with pytest.raises(ValidationError, match="extra"):
        parse_manifest({"rows": [_row()], "rowz": []})
    with pytest.raises(ValidationError, match="extra"):
        parse_manifest({"rows": [_row(exclusion_polciy="director")]})


def test_manifest_row_ceiling_is_loud() -> None:
    at_limit = tuple(_row(work=f"Work {index}") for index in range(4_096))
    assert len(parse_manifest({"rows": at_limit}).rows) == 4_096
    with pytest.raises(ValidationError, match="4,096"):
        parse_manifest({"rows": (*at_limit, _row(work="Over limit"))})


def test_director_excluded_rows_are_typed_and_exact() -> None:
    manifest = load_bundled_manifest()
    first, second, third = manifest.director_excluded_rows()

    assert first.exclusion_policy is ExclusionPolicy.DIRECTOR
    assert second.exclusion_policy is ExclusionPolicy.DIRECTOR
    assert third.exclusion_policy is ExclusionPolicy.DIRECTOR
    assert first.canon_status is CanonStatus.DENY
    assert second.canon_status is CanonStatus.DENY
    assert third.canon_status is CanonStatus.DENY
    assert len({first.author.casefold(), second.author.casefold(), third.author.casefold()}) == 3
    assert len({first.path_glob, second.path_glob, third.path_glob}) == 3
```

The three-value unpack is intentional: the test fails for both missing and surplus Director-policy rows without copying any excluded identifier into test code.

- [ ] **Step 2: Run the policy tests and observe RED**

Run:

```bash
mise run test:q -- tests/unit/intelligence/test_corpus_manifest.py -k "director_exclusion or manifest_rejects or manifest_row_ceiling"
```

Expected: collection or assertion failure because `ExclusionPolicy`, `CorpusRow.exclusion_policy`, `director_excluded_rows()`, and the complete three-row manifest policy do not exist yet.

- [ ] **Step 3: Add the minimal typed policy interface**

In `src/babylon/intelligence/corpus_manifest.py`, place the enum immediately after `CanonStatus`:

```python
class ExclusionPolicy(StrEnum):
    """Additional source policy that must remain machine-addressable."""

    NONE = "none"
    DIRECTOR = "director"
```

Add the field to `CorpusRow` after `canon_status`:

```python
    exclusion_policy: ExclusionPolicy = ExclusionPolicy.NONE
```

Set both input models to strict envelopes and add the named ceiling before row
construction:

```python
MAX_CORPUS_MANIFEST_ROWS: Final[int] = 4_096

class CorpusRow(BaseModel):
    model_config = ConfigDict(frozen=True, extra="forbid")

class CorpusManifest(BaseModel):
    model_config = ConfigDict(frozen=True, extra="forbid")
    rows: tuple[CorpusRow, ...] = ()

    @field_validator("rows", mode="before")
    @classmethod
    def _check_row_ceiling(
        cls, rows: object
    ) -> object:
        if isinstance(rows, (list, tuple)) and len(rows) > MAX_CORPUS_MANIFEST_ROWS:
            raise ValueError("corpus manifest exceeds 4,096 rows")
        return rows
```

Import `Final` and `field_validator`. Change `parse_manifest` to
`return CorpusManifest.model_validate(raw)` so unknown top-level keys cannot
disappear through `raw.get`. Keep the empty-document behavior through the
declared `rows = ()` default.

Add this model validator after `_check_role_nonempty`:

```python
    @model_validator(mode="after")
    def _check_director_exclusion_is_denied(self) -> CorpusRow:
        if (
            self.exclusion_policy is ExclusionPolicy.DIRECTOR
            and self.canon_status is not CanonStatus.DENY
        ):
            raise ValueError(
                f"row {self.work!r} declares a director exclusion "
                f"but canon_status is {self.canon_status.value!r}, not deny"
            )
        return self
```

Add this query beside `deny_rows()`:

```python
    def director_excluded_rows(self) -> tuple[CorpusRow, ...]:
        """Return Director-excluded rows without exposing literals elsewhere."""
        return tuple(
            row
            for row in self.rows[:MAX_CORPUS_MANIFEST_ROWS]
            if row.exclusion_policy is ExclusionPolicy.DIRECTOR
        )
```

Keep `ingest_targets()` unchanged: these rows are already excluded because they remain `CanonStatus.DENY`.

- [ ] **Step 4: Make the manifest the complete literal authority**

In `src/babylon/data/corpus/manifest.yaml`:

1. Add `exclusion_policy: director` to the existing row that belongs to the binding three-item Director set.
2. Add the two absent Director rows for the lineages named in the Director's
   2026-08-23 ruling. Resolve each full author name and archive slug
   mechanically from its unique local mirror archive, then map that slug through
   the manifest's existing relative `<author-slug>/**/*.txt` OCR-root
   convention. Stop on a missing or ambiguous archive rather than guessing.
3. Give each of the three rows `role: [doctrine]`, `format: txt`, `canon_status: deny`, and `exclusion_policy: director`.
4. Use this exact provenance sentence for each row: `Director exclusion ruling, 2026-08-23. This row exists solely to prevent ingestion.`
5. Do not add `exclusion_policy` to unrelated deny rows, including the approved 1935 source's standalone-ingestion denial.
6. Replace identity-bearing manifest header comments and cross-row examples with neutral `nested denied-source` language. Each excluded identifier may occur only in its own canonical row's `path_glob` and `author` fields; it must not be repeated in section headings, comments, provenance, or another row.

Within the active T0 surfaces, the literal identities and their path globs must
exist only in these canonical manifest rows.

- [ ] **Step 5: Remove identity leakage from generic code and test fixtures**

In `CorpusManifest.ingest_targets()` documentation, replace the identity-bearing parenthetical example with `a nested denied-source case`.

In `tests/unit/intelligence/test_corpus_manifest.py`, retain the same deny-inside-allow behavior but use this neutral fixture layout:

```python
approved_dir = tmp_path / "classics" / "approved"
denied_dir = tmp_path / "classics" / "denied-author"
approved_dir.mkdir(parents=True)
denied_dir.mkdir(parents=True)
approved_file = approved_dir / "approved.txt"
denied_file = denied_dir / "denied.txt"
approved_file.write_text("approved source", encoding="utf-8")
denied_file.write_text("denied source", encoding="utf-8")
```

The neutral deny row is:

```python
_row(
    path_glob="classics/denied-author/**/*.txt",
    author="Denied Author",
    work="Denied Work",
    canon_status="deny",
)
```

Apply the same neutral names and paths to the equivalent fixture in `tests/unit/tools/test_ingest_corpus.py`. Preserve every existing behavioral assertion: the approved body is imported, the denied body is absent, and missing allow rows remain non-errors.

Replace the bundled manifest's existing exact all-denied-author set assertion
with two disjoint checks: `director_excluded_rows()` supplies exactly the typed
three-row set from Step 1, while the non-Director `deny_rows()` subset retains
the existing three unrelated denied authors. The test may name those unrelated
authors, but it must not repeat a Director-excluded literal outside the
manifest.

- [ ] **Step 6: Run focused tests and observe GREEN**

Run:

```bash
mise run test:q -- tests/unit/intelligence/test_corpus_manifest.py tests/unit/tools/test_ingest_corpus.py
```

Expected: 39 tests pass, the 35-test baseline plus exactly the four new
policy/schema tests in Step 1.

- [ ] **Step 7: Refactor and statically check the task**

Run:

```bash
uv run ruff check src/babylon/intelligence/corpus_manifest.py tests/unit/intelligence/test_corpus_manifest.py tests/unit/tools/test_ingest_corpus.py
uv run ruff format --check src/babylon/intelligence/corpus_manifest.py tests/unit/intelligence/test_corpus_manifest.py tests/unit/tools/test_ingest_corpus.py
uv run mypy src/babylon/intelligence/corpus_manifest.py
uv run yamllint -c .yamllint.yaml src/babylon/data/corpus/manifest.yaml
```

Expected: zero errors. Do not alter ingestion ordering, allow-minus-deny behavior, or apocrypha fencing during refactor.

- [ ] **Step 8: Commit the independent source-policy landing**

```bash
git add src/babylon/intelligence/corpus_manifest.py src/babylon/data/corpus/manifest.yaml tests/unit/intelligence/test_corpus_manifest.py tests/unit/tools/test_ingest_corpus.py
mise run commit -- "fix(corpus): consolidate Director source exclusions"
```

Expected: one commit containing only the four listed files.

---

### Task 2: Refound the Machine-Readable Theory Contract

**Files:**

- Create: `tests/unit/governance/test_theory_contract.py`
- Modify: `ai/theory.yaml:1-212`

**Interfaces:**

- Consumes: `load_bundled_manifest().director_excluded_rows()` from Task 1 and the four constitutional evidence classes.
- Produces: a version `2.0.0` machine theory record with exact top-level keys `meta`, `theory_boundary`, `constraints`, `reference_behavior`, `source_policy`, `sources`, and `ai_assistant_guidelines`.
- Produces these exact constraint IDs:
  - `accumulation_outcomes_are_contingent`
  - `imperial_rent_changes_relations_not_destiny`
  - `survival_is_a_heterogeneous_aggregate`
  - `class_subjectivity_is_historical`
  - `consciousness_is_relational_and_multidirectional`
  - `outcomes_are_history_recognizers`
  - `ecology_constrains_without_predetermining`
- Produces these exact source IDs:
  - `neel_hinterland_2018`
  - `neel_hellworld_2025`
  - `party_practice_clipping`
  - `cpusa_organizers_manual_ch3_1935`

- [ ] **Step 1: Create the focused test module and its fixed contract constants**

Create `tests/unit/governance/test_theory_contract.py` with this header and constants:

```python
"""Behavioral contract for the canonical T0 theory representation."""

from __future__ import annotations

from pathlib import Path
from typing import Final

import pytest
import yaml

from babylon.intelligence.corpus_manifest import load_bundled_manifest

pytestmark = pytest.mark.unit

_ROOT: Final[Path] = Path(__file__).parents[3]
_MACHINE_THEORY: Final[Path] = _ROOT / "ai" / "theory.yaml"
_HUMAN_THEORY: Final[Path] = _ROOT / "docs" / "concepts" / "theory.rst"
_MANTRAS: Final[Path] = _ROOT / "ai" / "mantras.yaml"
_CORPUS_POLICY_MODULE: Final[Path] = (
    _ROOT / "src" / "babylon" / "intelligence" / "corpus_manifest.py"
)
_CORPUS_POLICY_TEST: Final[Path] = (
    _ROOT / "tests" / "unit" / "intelligence" / "test_corpus_manifest.py"
)
_INGEST_CORPUS_TEST: Final[Path] = (
    _ROOT / "tests" / "unit" / "tools" / "test_ingest_corpus.py"
)
_EXCLUSION_HYGIENE_PATHS: Final[tuple[Path, ...]] = (
    _MACHINE_THEORY,
    _HUMAN_THEORY,
    _MANTRAS,
    _CORPUS_POLICY_MODULE,
    _CORPUS_POLICY_TEST,
    _INGEST_CORPUS_TEST,
)

_CONSTRAINT_IDS: Final[frozenset[str]] = frozenset(
    {
        "accumulation_outcomes_are_contingent",
        "imperial_rent_changes_relations_not_destiny",
        "survival_is_a_heterogeneous_aggregate",
        "class_subjectivity_is_historical",
        "consciousness_is_relational_and_multidirectional",
        "outcomes_are_history_recognizers",
        "ecology_constrains_without_predetermining",
    }
)
_EVIDENCE_CLASSES: Final[frozenset[str]] = frozenset(
    {"Observed", "Derived", "Calibrated", "Designed"}
)
_SOURCE_HASHES: Final[tuple[tuple[str, str], ...]] = (
    (
        "neel_hinterland_2018",
        "2799eb76f267551afa04a6bb76ffed4a89c5e1fc387c3744fcca3be3b00b4525",
    ),
    (
        "neel_hellworld_2025",
        "43127a54390f9fb798cb644f0e5af0f8228b79cc5c392b1b472b5dc96be8fe1e",
    ),
    (
        "party_practice_clipping",
        "373c2b594f932cbc7fcf590a784e6b48b9031a9bf7363e9b33a58fdc074454b1",
    ),
    (
        "cpusa_organizers_manual_ch3_1935",
        "6d27b580c657f68f35e8d4b5b2ac6ea6b076050b1de7a82cb0b615cce12f44fb",
    ),
)

_EXPECTED_CONSTRAINTS: Final[dict[str, dict[str, str]]] = {
    "accumulation_outcomes_are_contingent": {
        "evidence_class": "Derived",
        "executable_status": "theoretical_constraint",
        "statement": (
            "Accumulation produces pressures and limits; paths and outcomes "
            "remain contingent."
        ),
    },
    "imperial_rent_changes_relations_not_destiny": {
        "evidence_class": "Derived",
        "executable_status": "theoretical_constraint",
        "statement": (
            "Imperial rent changes incentives and causal pathways; organization, "
            "crisis, coercion, solidarity, and countervailing relations remain live "
            "variables."
        ),
    },
    "survival_is_a_heterogeneous_aggregate": {
        "evidence_class": "Derived",
        "executable_status": "theoretical_constraint",
        "statement": (
            "Survival is an aggregate over heterogeneous material distributions "
            "and relations. No fixed response curve is lawful."
        ),
    },
    "class_subjectivity_is_historical": {
        "evidence_class": "Derived",
        "executable_status": "theoretical_constraint",
        "statement": (
            "Classes are positions and relations. Political practice and "
            "subjectivity are historical results."
        ),
    },
    "consciousness_is_relational_and_multidirectional": {
        "evidence_class": "Derived",
        "executable_status": "theoretical_constraint",
        "statement": (
            "Consciousness and line travel through attributed organization and "
            "solidarity relations in multiple directions."
        ),
    },
    "outcomes_are_history_recognizers": {
        "evidence_class": "Derived",
        "executable_status": "theoretical_constraint",
        "statement": (
            "Outcomes are recognizers over histories, not downstream writes or "
            "promised verdicts."
        ),
    },
    "ecology_constrains_without_predetermining": {
        "evidence_class": "Derived",
        "executable_status": "theoretical_constraint",
        "statement": (
            "Ecological degradation and care capacity constrain choices; "
            "construction, repair, and redistribution can change consequences "
            "without promising equilibrium."
        ),
    },
}

_EXPECTED_SOURCES: Final[dict[str, dict[str, object]]] = {
    "neel_hinterland_2018": {
        "title": "Hinterland: America's New Landscape of Class and Conflict",
        "edition": "Reaktion, 2018, supplied PDF",
        "sha256": _SOURCE_HASHES[0][1],
        "evidence_class": "Observed",
        "executable_authority": False,
        "availability": "supplied_external_artifact",
        "scope": (
            "Constrains relational territorial ontology. Supplies no coefficient, "
            "threshold, curve, or guaranteed outcome."
        ),
        "anchors": ["PDF p. 18 (printed p. 17)"],
    },
    "neel_hellworld_2025": {
        "title": "Hellworld: The Human Species and the Planetary Factory",
        "edition": "Brill, 2025, supplied PDF",
        "sha256": _SOURCE_HASHES[1][1],
        "evidence_class": "Observed",
        "executable_authority": False,
        "availability": "supplied_external_artifact",
        "scope": (
            "Constrains relations among production, circulation, reproduction, "
            "ecology, finance, and state power. Supplies no executable value."
        ),
        "anchors": [
            "PDF pp. 170-171 (printed pp. 143-144)",
            "PDF p. 239 (printed p. 212)",
        ],
    },
    "party_practice_clipping": {
        "title": "theory-of-the-party-ill-will.md supplied clipping",
        "edition": "complete supplied clipping",
        "sha256": _SOURCE_HASHES[2][1],
        "evidence_class": "Observed",
        "executable_authority": False,
        "availability": "repository_file",
        "repository_path": "ai/_inbox/archive/theory-of-the-party-ill-will.md",
        "scope": (
            "Supports organization and subjectivity as products of situated "
            "practice. Does not authorize a party score, universal form, or "
            "scripted subject."
        ),
        "anchors": ["complete supplied clipping"],
    },
    "cpusa_organizers_manual_ch3_1935": {
        "title": "Organizers' Manual, chapter 3",
        "edition": "Communist Party USA, 1935, local HTML",
        "sha256": _SOURCE_HASHES[3][1],
        "evidence_class": "Observed",
        "executable_authority": False,
        "availability": "optional_local_mirror",
        "relative_locator": (
            "history/usa/parties/cpusa/1935/07/organisers-manual/ch03.htm"
        ),
        "scope": (
            "Supports rooted work and iterative evaluation. Hierarchy, fractions, "
            "secrecy rules, membership thresholds, and numeric guidance remain "
            "historical particulars rather than Babylon universals."
        ),
        "anchors": [
            "HTML lines 45-61",
            "HTML lines 265-271",
            "HTML lines 464-505",
            "HTML lines 1175-1190",
        ],
    },
}
_SOURCE_BLOCK_MARKERS: Final[tuple[tuple[str, str], ...]] = (
    ("neel_hinterland_2018", "``neel_hellworld_2025``"),
    ("neel_hellworld_2025", "``party_practice_clipping``"),
    ("party_practice_clipping", "``cpusa_organizers_manual_ch3_1935``"),
    ("cpusa_organizers_manual_ch3_1935", "Source Policy"),
)


def _machine_document() -> dict[str, object]:
    document = yaml.safe_load(_MACHINE_THEORY.read_text(encoding="utf-8"))
    assert isinstance(document, dict)
    return document


def _normalized_text(path: Path) -> str:
    return " ".join(path.read_text(encoding="utf-8").casefold().split())


def _normalized_value(value: str) -> str:
    return " ".join(value.casefold().split())


def _director_excluded_tokens() -> tuple[str, ...]:
    first, second, third = load_bundled_manifest().director_excluded_rows()
    rows = (first, second, third)
    return tuple(
        token
        for row in rows
        for token in (
            row.author.casefold(),
            row.author.rpartition(" ")[2].casefold(),
            Path(row.path_glob).parts[0].casefold(),
        )
    )
```

- [ ] **Step 2: Add RED tests for schema, authority, and source provenance**

Append:

```python
def test_machine_theory_has_exact_t0_schema_and_authority() -> None:
    document = _machine_document()

    assert tuple(document) == (
        "meta",
        "theory_boundary",
        "constraints",
        "reference_behavior",
        "source_policy",
        "sources",
        "ai_assistant_guidelines",
    )
    meta = document["meta"]
    assert isinstance(meta, dict)
    assert meta["version"] == "2.0.0"
    assert meta["authority"] == "CONSTITUTION.md v4.0.0"
    assert meta["architecture"] == "docs/concepts/architecture.rst"
    assert meta["reserved_line"] == (
        "Marxist-Leninist-Maoist Third Worldist (MLM-TW)"
    )


def test_machine_theory_declares_exact_constraint_set() -> None:
    constraints = _machine_document()["constraints"]
    assert isinstance(constraints, dict)
    assert constraints == _EXPECTED_CONSTRAINTS


@pytest.mark.parametrize("constraint_id", tuple(sorted(_CONSTRAINT_IDS)))
def test_each_constraint_is_derived_and_non_executable(constraint_id: str) -> None:
    constraints = _machine_document()["constraints"]
    assert isinstance(constraints, dict)
    constraint = constraints[constraint_id]
    assert isinstance(constraint, dict)
    assert constraint["evidence_class"] == "Derived"
    assert constraint["executable_status"] == "theoretical_constraint"
    assert constraint["evidence_class"] in _EVIDENCE_CLASSES


@pytest.mark.parametrize(("source_id", "expected_hash"), _SOURCE_HASHES)
def test_source_ledger_pins_bounded_observed_evidence(
    source_id: str,
    expected_hash: str,
) -> None:
    sources = _machine_document()["sources"]
    assert isinstance(sources, dict)
    source = sources[source_id]
    assert isinstance(source, dict)
    assert source["sha256"] == expected_hash
    assert source["evidence_class"] == "Observed"
    assert source["executable_authority"] is False
    assert source["scope"]
    assert source["anchors"]


def test_source_ledger_has_exact_structured_rows() -> None:
    assert _machine_document()["sources"] == _EXPECTED_SOURCES
```

- [ ] **Step 3: Add RED tests for source policy, stale bindings, and excluded sources**

Append:

```python
def test_theory_source_policy_routes_to_one_canonical_denial_manifest() -> None:
    policy = _machine_document()["source_policy"]
    assert isinstance(policy, dict)
    assert policy == {
        "director_exclusions": "src/babylon/data/corpus/manifest.yaml",
        "narrator_ingestion": "unchanged",
        "approved_research_exception": "cpusa_organizers_manual_ch3_1935",
        "research_exception_scope": "bounded_research_evidence_only",
    }


def test_machine_theory_makes_frozen_reference_status_explicit() -> None:
    reference = _machine_document()["reference_behavior"]
    assert isinstance(reference, dict)
    assert reference == {
        "frozen_python": {
            "status": "frozen_reference",
            "authority": "behavioral_reference_not_live_rust_law",
            "architecture": "docs/concepts/architecture.rst",
        },
        "historical_formulas": {
            "status": "reference_or_surrogate_only",
            "executable_binding_claimed": False,
        },
    }


def test_machine_theory_has_no_stale_implementation_claims() -> None:
    text = _MACHINE_THEORY.read_text(encoding="utf-8")
    assert "implemented_in:" not in text
    assert "src/babylon/systems/formulas/" not in text
    assert "src/babylon/engine/systems/" not in text


@pytest.mark.parametrize("token", _director_excluded_tokens())
@pytest.mark.parametrize("path", _EXCLUSION_HYGIENE_PATHS)
def test_active_policy_and_theory_surfaces_exclude_director_denied_tokens(
    token: str,
    path: Path,
) -> None:
    assert token not in _normalized_text(path)
```

- [ ] **Step 4: Add RED tests for the seven retired categorical claims**

Append:

```python
_RETIRED_CATEGORICAL_PHRASES = (
    "revolution in the imperial core is structurally impossible",
    "revolutionary potential is concentrated in the periphery",
    "p(s|a) = sigmoid",
    "consciousness flows unidirectionally",
    "the tragedy of inevitability",
    "only delays inevitable",
    "terminal bifurcation",
    "the default is fascism",
)


@pytest.mark.parametrize("retired_phrase", _RETIRED_CATEGORICAL_PHRASES)
def test_machine_theory_rejects_retired_claims(retired_phrase: str) -> None:
    assert retired_phrase not in _normalized_text(_MACHINE_THEORY)
```

These assertions target the exact stale statements. They do not ban discussion of pressure, tendency, historical cases, or heterogeneous aggregation.

- [ ] **Step 5: Run the new machine tests and observe RED**

Run:

```bash
mise run test:q -- tests/unit/governance/test_theory_contract.py -k "machine or source_ledger or source_policy or retired_claims"
```

Expected: failures on the old top-level schema, old version, missing constraint IDs, missing structured source ledger, stale implementation paths, and old categorical claims.

- [ ] **Step 6: Replace `ai/theory.yaml` with the minimal versioned contract**

Use this exact top-level shape and statements:

```yaml
---
meta:
  name: Babylon Canonical Theory Constraints
  version: "2.0.0"
  updated: "2026-08-23"
  authority: CONSTITUTION.md v4.0.0
  architecture: docs/concepts/architecture.rst
  reserved_line: "Marxist-Leninist-Maoist Third Worldist (MLM-TW)"
  human_rendering: docs/concepts/theory.rst
  machine_orientation: ai/mantras.yaml
  purpose: >-
    Constrain causal questions without predetermining play outcomes or claiming
    executable authority.

theory_boundary:
  constrains: represented_relations_and_causal_questions
  does_not:
    - predetermine_outcomes
    - impose_response_curves
    - create_executable_rules
    - make_geography_or_class_an_essence
  evidence_classes: [Observed, Derived, Calibrated, Designed]

constraints:
  accumulation_outcomes_are_contingent:
    evidence_class: Derived
    executable_status: theoretical_constraint
    statement: >-
      Accumulation produces pressures and limits; paths and outcomes remain
      contingent.
  imperial_rent_changes_relations_not_destiny:
    evidence_class: Derived
    executable_status: theoretical_constraint
    statement: >-
      Imperial rent changes incentives and causal pathways; organization,
      crisis, coercion, solidarity, and countervailing relations remain live
      variables.
  survival_is_a_heterogeneous_aggregate:
    evidence_class: Derived
    executable_status: theoretical_constraint
    statement: >-
      Survival is an aggregate over heterogeneous material distributions and
      relations. No fixed response curve is lawful.
  class_subjectivity_is_historical:
    evidence_class: Derived
    executable_status: theoretical_constraint
    statement: >-
      Classes are positions and relations. Political practice and subjectivity
      are historical results.
  consciousness_is_relational_and_multidirectional:
    evidence_class: Derived
    executable_status: theoretical_constraint
    statement: >-
      Consciousness and line travel through attributed organization and
      solidarity relations in multiple directions.
  outcomes_are_history_recognizers:
    evidence_class: Derived
    executable_status: theoretical_constraint
    statement: >-
      Outcomes are recognizers over histories, not downstream writes or
      promised verdicts.
  ecology_constrains_without_predetermining:
    evidence_class: Derived
    executable_status: theoretical_constraint
    statement: >-
      Ecological degradation and care capacity constrain choices;
      construction, repair, and redistribution can change consequences without
      promising equilibrium.

reference_behavior:
  frozen_python:
    status: frozen_reference
    authority: behavioral_reference_not_live_rust_law
    architecture: docs/concepts/architecture.rst
  historical_formulas:
    status: reference_or_surrogate_only
    executable_binding_claimed: false

source_policy:
  director_exclusions: src/babylon/data/corpus/manifest.yaml
  narrator_ingestion: unchanged
  approved_research_exception: cpusa_organizers_manual_ch3_1935
  research_exception_scope: bounded_research_evidence_only
```

Add these exact source rows and assistant guidelines after `source_policy`:

```yaml
sources:
  neel_hinterland_2018:
    title: "Hinterland: America's New Landscape of Class and Conflict"
    edition: "Reaktion, 2018, supplied PDF"
    sha256: "2799eb76f267551afa04a6bb76ffed4a89c5e1fc387c3744fcca3be3b00b4525"
    evidence_class: Observed
    executable_authority: false
    availability: supplied_external_artifact
    scope: >-
      Constrains relational territorial ontology. Supplies no coefficient,
      threshold, curve, or guaranteed outcome.
    anchors:
      - "PDF p. 18 (printed p. 17)"
  neel_hellworld_2025:
    title: "Hellworld: The Human Species and the Planetary Factory"
    edition: "Brill, 2025, supplied PDF"
    sha256: "43127a54390f9fb798cb644f0e5af0f8228b79cc5c392b1b472b5dc96be8fe1e"
    evidence_class: Observed
    executable_authority: false
    availability: supplied_external_artifact
    scope: >-
      Constrains relations among production, circulation, reproduction,
      ecology, finance, and state power. Supplies no executable value.
    anchors:
      - "PDF pp. 170-171 (printed pp. 143-144)"
      - "PDF p. 239 (printed p. 212)"
  party_practice_clipping:
    title: "theory-of-the-party-ill-will.md supplied clipping"
    edition: "complete supplied clipping"
    sha256: "373c2b594f932cbc7fcf590a784e6b48b9031a9bf7363e9b33a58fdc074454b1"
    evidence_class: Observed
    executable_authority: false
    availability: repository_file
    repository_path: ai/_inbox/archive/theory-of-the-party-ill-will.md
    scope: >-
      Supports organization and subjectivity as products of situated practice.
      Does not authorize a party score, universal form, or scripted subject.
    anchors:
      - "complete supplied clipping"
  cpusa_organizers_manual_ch3_1935:
    title: "Organizers' Manual, chapter 3"
    edition: "Communist Party USA, 1935, local HTML"
    sha256: "6d27b580c657f68f35e8d4b5b2ac6ea6b076050b1de7a82cb0b615cce12f44fb"
    evidence_class: Observed
    executable_authority: false
    availability: optional_local_mirror
    relative_locator: history/usa/parties/cpusa/1935/07/organisers-manual/ch03.htm
    scope: >-
      Supports rooted work and iterative evaluation. Hierarchy, fractions,
      secrecy rules, membership thresholds, and numeric guidance remain
      historical particulars rather than Babylon universals.
    anchors:
      - "HTML lines 45-61"
      - "HTML lines 265-271"
      - "HTML lines 464-505"
      - "HTML lines 1175-1190"

ai_assistant_guidelines:
  - "Model material relations as constraints on causal questions, not outcomes."
  - "Ask which classes and organizations benefit from each represented relation."
  - "Use dialectical analysis to inspect contradictions and transformations."
  - "Do not use theory to predetermine a result."
```

Do not retain any old class-potential, collapse, terminal-crisis,
survival-formula, or directional-consciousness sections.

- [ ] **Step 7: Run the machine contract and observe GREEN**

Run:

```bash
mise run test:q -- tests/unit/governance/test_theory_contract.py -k "machine or source_ledger or source_policy or retired_claims"
```

Expected: all selected tests pass. Human-parity and mantra tests added in Task 3 are not present yet.

- [ ] **Step 8: Validate and refactor the machine record**

Run:

```bash
uv run yamllint -c .yamllint.yaml ai/theory.yaml
uv run ruff check tests/unit/governance/test_theory_contract.py
uv run ruff format --check tests/unit/governance/test_theory_contract.py
```

Expected: zero errors. Preserve the exact seven IDs, four source IDs, four hashes, and top-level key order during refactor.

- [ ] **Step 9: Commit the independent machine-theory landing**

```bash
git add tests/unit/governance/test_theory_contract.py ai/theory.yaml
mise run commit -- "docs(theory): refound canonical machine contract"
```

Expected: one commit containing only the new focused test and `ai/theory.yaml`.

---

### Task 3: Align the Human Theory Page and Machine Orientation

**Files:**

- Modify: `tests/unit/governance/test_theory_contract.py`
- Modify: `docs/concepts/theory.rst:1-258`
- Modify: `ai/mantras.yaml:1-46`

**Interfaces:**

- Consumes: `_CONSTRAINT_IDS`, `_SOURCE_HASHES`, `_normalized_text()`, and the version `2.0.0` machine record from Task 2.
- Produces: one human section per exact constraint ID, the same four source digests, explicit live/frozen status prose, and a contingent `mantras.north_star.meaning`.
- Invariant: `docs/concepts/theory.rst` explains the YAML; it does not add another rule, value, source, or executable claim.

- [ ] **Step 1: Add RED human-parity tests**

Append to `tests/unit/governance/test_theory_contract.py`:

```python
@pytest.mark.parametrize("retired_phrase", _RETIRED_CATEGORICAL_PHRASES)
@pytest.mark.parametrize("path", (_HUMAN_THEORY, _MANTRAS))
def test_human_and_orientation_surfaces_reject_retired_claims(
    retired_phrase: str,
    path: Path,
) -> None:
    assert retired_phrase not in _normalized_text(path)


def test_human_theory_routes_to_live_authority() -> None:
    text = _HUMAN_THEORY.read_text(encoding="utf-8")
    assert "<../../CONSTITUTION.md>" in text
    assert "Marxist-Leninist-Maoist Third Worldist" in text
    assert ":doc:`architecture`" in text
    assert "frozen Python reference" in text
    assert "not the live Rust law" in text


@pytest.mark.parametrize("constraint_id", tuple(sorted(_CONSTRAINT_IDS)))
def test_human_theory_renders_each_machine_constraint(constraint_id: str) -> None:
    expected = _EXPECTED_CONSTRAINTS[constraint_id]
    rendered = _normalized_text(_HUMAN_THEORY)
    exact_block = _normalized_value(
        f"``{constraint_id}`` Evidence class: {expected['evidence_class']} "
        f"Executable status: {expected['executable_status']} "
        f"Statement: {expected['statement']}"
    )
    assert exact_block in rendered


@pytest.mark.parametrize(("source_id", "expected_hash"), _SOURCE_HASHES)
def test_human_source_ledger_matches_machine_hashes(
    source_id: str,
    expected_hash: str,
) -> None:
    text = _HUMAN_THEORY.read_text(encoding="utf-8")
    assert f"``{source_id}``" in text
    assert expected_hash in text


@pytest.mark.parametrize(("source_id", "end_marker"), _SOURCE_BLOCK_MARKERS)
def test_human_source_ledger_renders_every_exact_source_field(
    source_id: str,
    end_marker: str,
) -> None:
    expected = _EXPECTED_SOURCES[source_id]
    text = _HUMAN_THEORY.read_text(encoding="utf-8")
    start = text.index(f"``{source_id}``")
    end = text.index(end_marker, start + len(source_id) + 4)
    rendered = _normalized_value(text[start:end])
    scalar_fields = (
        "title",
        "edition",
        "sha256",
        "evidence_class",
        "availability",
        "scope",
    )
    for field in scalar_fields:
        assert _normalized_value(str(expected[field])) in rendered
    assert _normalized_value(str(expected["executable_authority"])) in rendered
    for locator_field in ("repository_path", "relative_locator"):
        if locator_field in expected:
            assert _normalized_value(str(expected[locator_field])) in rendered
    anchors = expected["anchors"]
    assert isinstance(anchors, list)
    for anchor in anchors[:4]:
        assert _normalized_value(str(anchor)) in rendered


def test_mantra_north_star_describes_contingent_political_possibility() -> None:
    document = yaml.safe_load(_MANTRAS.read_text(encoding="utf-8"))
    north_star = document["mantras"]["north_star"]["meaning"].casefold()

    assert "why revolution happens in the periphery, not the core" not in north_star
    assert "how organization and solidarity can redirect political possibilities" in north_star
```

- [ ] **Step 2: Run the new human tests and observe RED**

Run:

```bash
mise run test:q -- tests/unit/governance/test_theory_contract.py -k "human or mantra"
```

Expected: failures because the RST page lacks the seven stable IDs, authority/status contract, source digests, and the mantra still contains its categorical sentence.

- [ ] **Step 3: Replace the human page with a faithful rendering**

Rewrite `docs/concepts/theory.rst` with these sections in this order:

1. `Relational Theory Constraints`
2. `Authority and Scope`
3. `Governing Constraints`
4. `Theory and Executable Status`
5. `Evidence and Source Ledger`
6. `Source Policy`
7. `See Also`

Use this exact authority paragraph:

```rst
`CONSTITUTION.md v4.0.0 <../../CONSTITUTION.md>`__ governs this page and
reserves Babylon's Marxist-Leninist-Maoist Third Worldist theoretical line.
:doc:`architecture` separates the live Rust engine from the frozen Python
reference. This page corrects deterministic interpretations within the
reserved line; it does not rename or amend that line. It constrains represented
relations and causal questions. It does not create an executable rule,
coefficient, threshold, response curve, geographic essence, class essence, or
promised outcome.
```

Under `Governing Constraints`, give each of the seven exact IDs its own
subsection. Begin each subsection with one uninterrupted field block in this
exact form so the human rendering remains mechanically associated with its
machine row:

```rst
``constraint_id``
Evidence class: Derived
Executable status: theoretical_constraint
Statement: Exact machine statement.
```

Copy the corresponding `statement` from `ai/theory.yaml` exactly, then add no
more than one explanatory paragraph that answers the relevant causal question
without adding a value or executable claim.

Use this exact status paragraph:

```rst
Historical formulas that remain in Python belong to the frozen Python
reference. They preserve reference or surrogate behavior and are not the live
Rust law. Live Rust behavior exists only where the architecture and executable
source establish it. This page claims no new implementation binding.
```

Under `Evidence and Source Ledger`, render all four source IDs, editions,
digests, evidence class, bounded scope, and exact page/line anchors from the
machine record. State explicitly that the two PDFs are supplied external
artifacts, the clipping is a repository file, and the 1935 chapter is an
optional local-mirror source. Do not claim CI can open an artifact that is not
stored in the repository. Keep the four source blocks in `_SOURCE_BLOCK_MARKERS`
order. Within each ID's block, include every exact scalar, locator when present,
and anchor before the next source ID; this association is a tested contract,
not a bag-of-strings check.

Under `Source Policy`, state that Director exclusions are governed only by
`src/babylon/data/corpus/manifest.yaml`, narrator ingestion remains unchanged,
and the approved 1935 chapter is bounded research evidence rather than
standalone narrator doctrine. Do not repeat excluded identifiers.

Under `See Also`, link only the RST role ``:doc:`architecture``` and the literal
``CONSTITUTION.md`` v4.0.0, and formula pages that are labeled explicitly as
frozen-reference context. Remove language that presents a legacy formula page
as current executable authority.

- [ ] **Step 4: Correct the one machine-orientation inconsistency**

In `ai/mantras.yaml`:

1. Bump `meta.version` from `2.0.0` to `2.0.1`; keep `meta.updated` at `2026-08-23`.
2. Replace only the categorical sentence in `mantras.north_star.meaning` with:

```yaml
      A person who hasn't read Marx plays this game and starts asking
      the right questions about imperial rent, class formation, and how
      organization and solidarity can redirect political possibilities in any
      territory.
```

3. Leave `causal_emergence`, `state_is_data`, `agitation_without_solidarity`, and every other mantra unchanged.

- [ ] **Step 5: Run the complete T0 contract and observe GREEN**

Run:

```bash
mise run test:q -- tests/unit/governance/test_theory_contract.py
mise run test:q -- tests/unit/governance/test_constitution_v4.py
```

Expected: all focused T0 tests pass, and the existing Constitution v4 corpus remains green.

- [ ] **Step 6: Run prose and YAML gates**

Run Vale from the project environment so `rst2html` from the locked `docutils` dependency is on `PATH`:

```bash
uv run vale docs/concepts/theory.rst
uv run yamllint -c .yamllint.yaml ai/theory.yaml ai/mantras.yaml src/babylon/data/corpus/manifest.yaml
```

Expected: zero Vale errors, warnings, or suggestions and zero yamllint errors. A plain host `vale` invocation that fails because `rst2html` is absent is not a valid gate result. Vale treats these YAML files as zero input files, so yamllint remains the YAML validator.

- [ ] **Step 7: Refactor without widening scope**

Read the complete diff for `ai/theory.yaml`, `docs/concepts/theory.rst`, and
`ai/mantras.yaml`. Remove duplicated prose between the machine statements and
their human explanations. Preserve the exact IDs, hashes, evidence classes,
source-policy boundary, and single mantra edit.

Re-run:

```bash
mise run test:q -- tests/unit/governance/test_theory_contract.py tests/unit/governance/test_constitution_v4.py
uv run vale docs/concepts/theory.rst
```

Expected: all tests pass and Vale remains at zero findings.

- [ ] **Step 8: Commit the independent human-parity landing**

```bash
git add tests/unit/governance/test_theory_contract.py docs/concepts/theory.rst ai/mantras.yaml
mise run commit -- "docs(theory): align human theory and orientation"
```

Expected: one commit containing only the three listed files.

---

### Task 4: Verify the Complete T0 Landing and Self-Review It

**Files:**

- Review: every file listed in this plan's `Create` and `Modify` sections.
- Modify only if a review or gate finds a T0 defect: the exact file that contains that defect.

**Interfaces:**

- Consumes: the three independently green commits from Tasks 1-3.
- Produces: evidence that T0 satisfies spec lines 212-219 without claiming T1, T2, T3, or full PER-50 completion.

- [ ] **Step 1: Prove the scoped file set**

Run:

```bash
t0_plan_base_sha="$(git log -1 --format=%H -- docs/superpowers/plans/2026-08-23-neel-t0-theory-refoundation.md)"
git diff --name-only "$t0_plan_base_sha"...HEAD
git status --short
```

Expected diff paths are exactly the eight T0 implementation files:

```text
ai/mantras.yaml
ai/theory.yaml
docs/concepts/theory.rst
src/babylon/data/corpus/manifest.yaml
src/babylon/intelligence/corpus_manifest.py
tests/unit/governance/test_theory_contract.py
tests/unit/intelligence/test_corpus_manifest.py
tests/unit/tools/test_ingest_corpus.py
```

Stop and report any unrelated path. Do not delete, revert, format, or stage it.

- [ ] **Step 2: Run the complete targeted behavioral suite**

Run:

```bash
mise run test:q -- tests/unit/governance/test_theory_contract.py tests/unit/governance/test_constitution_v4.py tests/unit/intelligence/test_corpus_manifest.py tests/unit/tools/test_ingest_corpus.py
```

Expected: all tests pass with no unexpected skip or xfail.

- [ ] **Step 3: Run static, source, and prose validation**

Run:

```bash
uv run ruff check src/babylon/intelligence/corpus_manifest.py tests/unit/governance/test_theory_contract.py tests/unit/intelligence/test_corpus_manifest.py tests/unit/tools/test_ingest_corpus.py
uv run ruff format --check src/babylon/intelligence/corpus_manifest.py tests/unit/governance/test_theory_contract.py tests/unit/intelligence/test_corpus_manifest.py tests/unit/tools/test_ingest_corpus.py
uv run mypy src/babylon/intelligence/corpus_manifest.py
uv run yamllint -c .yamllint.yaml ai/theory.yaml ai/mantras.yaml src/babylon/data/corpus/manifest.yaml
uv run vale docs/concepts/theory.rst
mise run check:vocabulary
git diff --check
```

Expected: every command exits zero; Vale reports zero findings; YAML parses cleanly; Git reports no whitespace errors.

- [ ] **Step 4: Run the required Python repository gate**

Ensure no other heavy gate is running, then run uncapped:

```bash
mise run check
```

Expected: the complete Python gate passes. This train changes no Rust, engine economics, `GameDefines`, or golden baseline, so `rust:check`, regression, vault regression, and baseline ceremony are outside scope.

- [ ] **Step 5: Perform the spec-coverage self-review**

Read spec sections 2, 6, and 11 and confirm all of the following against the diff:

- the canonical pair is the only theory registry;
- all seven governing replacements are present in machine and human form;
- no fixed survival curve or inevitability claim remains in the canonical pair;
- no geographic or class-potential essence remains;
- theoretical constraint, frozen Python reference, and live Rust status are distinct;
- all four sources have exact digests, bounded anchors, and evidence classes;
- Director-excluded identifiers occur only in the canonical manifest rows
  among the active T0 surfaces;
- the approved research exception does not change narrator ingestion;
- the human page links the live Constitution and architecture;
- targeted Vale is clean;
- no downstream train or existing Linear owner's scope was absorbed.

If any item is false, fix only the responsible T0 file, add or strengthen the focused test that would have caught it, and repeat Steps 2-4.

- [ ] **Step 6: Perform the type and naming self-review**

Confirm these names match exactly across production, tests, YAML, and RST:

- `ExclusionPolicy.DIRECTOR`
- `CorpusRow.exclusion_policy`
- `CorpusManifest.director_excluded_rows()`
- all seven `_CONSTRAINT_IDS`
- all four `_SOURCE_HASHES` source IDs
- `reference_behavior`
- `source_policy`

Confirm every test helper has a declared return type and no function exceeds 100 lines.

- [ ] **Step 7: Inspect the final diff and create a correction commit only if needed**

Run:

```bash
t0_plan_base_sha="$(git log -1 --format=%H -- docs/superpowers/plans/2026-08-23-neel-t0-theory-refoundation.md)"
git diff --stat "$t0_plan_base_sha"...HEAD
git diff "$t0_plan_base_sha"...HEAD -- ai/mantras.yaml ai/theory.yaml docs/concepts/theory.rst src/babylon/data/corpus/manifest.yaml src/babylon/intelligence/corpus_manifest.py tests/unit/governance/test_theory_contract.py tests/unit/intelligence/test_corpus_manifest.py tests/unit/tools/test_ingest_corpus.py
git status --short
```

Expected: only the approved T0 surface, no identity leakage outside the manifest
among those surfaces, no stale implementation claims, and a clean worktree.

If review required a correction after Task 3, stage only its exact files and commit:

```bash
mise run commit -- "fix(theory): close T0 contract review findings"
```

Do not create an empty correction commit.

- [ ] **Step 8: Publish the PER-51 completion handoff.** Resolve the final T0
  branch SHA with `git rev-parse HEAD`, require the approved T0 surface to be
  clean, and post one PER-51 completion comment. The comment records the plan
  base SHA, every T0 implementation commit SHA, the final branch SHA, every
  command and result in Completion Evidence, and the explicit T1-T3 exclusions.
  Only after that evidence exists, move PER-51 from In Progress to Done,
  refresh it, and require the returned state to be Done before T1 starts.

---

## Scope Cuts

- Do not rewrite adjacent legacy theory pages. T0 may label links to them as frozen-reference context; later work needs its own owner.
- Do not change formulas, Python engine behavior, Rust behavior, BSL grammar/content, rule attribution, effects, manifests unrelated to source denial, or corpus ingestion output.
- Do not add a party type, party score, geographic rank, core/periphery field, class-potential field, survival scalar, terminal outcome, historical stage, or response curve.
- Do not add source artifacts, copy external PDFs into the repository, or hardcode a local mirror's absolute path.
- Do not change the standalone narrator/RAG disposition of the approved research chapter.
- Do not claim PER-50 complete. This plan completes only its scoped T0 child after all gates pass.
- Do not run Sphinx, Rust gates, economic regression, vault regression, or baseline ceremony for this train.

## Completion Evidence

T0 is complete only when the final handoff records:

- PER-51, its PER-50 parent, the verified In Progress transition, and the final
  evidence-backed Done transition;
- the three implementation commit SHAs;
- the exact targeted pytest command and passing result;
- zero-result Ruff, mypy, yamllint, Vale, vocabulary, and `git diff --check` gates;
- the passing `mise run check` result;
- confirmation that the diff after the recorded T0 plan commit contains only the eight T0 implementation files;
- confirmation that no excluded identifier escaped the canonical denial manifest
  among the active T0 surfaces;
- explicit wording that T1-T3 and full PER-50 remain incomplete.

Continue autonomously with `superpowers:subagent-driven-development`: use one
fresh implementation worker and two-stage review per task, preserve the Linear
preflight, and stop only for a constitutional or Linear ownership conflict.

<!-- vale on -->
