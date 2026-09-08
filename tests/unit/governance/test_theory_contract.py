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
_INGEST_CORPUS_TEST: Final[Path] = _ROOT / "tests" / "unit" / "tools" / "test_ingest_corpus.py"
_THEORY_CONTRACT_TEST: Final[Path] = Path(__file__)
_EXCLUSION_HYGIENE_PATHS: Final[tuple[Path, ...]] = (
    _MACHINE_THEORY,
    _HUMAN_THEORY,
    _MANTRAS,
    _CORPUS_POLICY_MODULE,
    _CORPUS_POLICY_TEST,
    _INGEST_CORPUS_TEST,
    _THEORY_CONTRACT_TEST,
)
_EXCLUSION_HYGIENE_SURFACE_IDS: Final[tuple[str, ...]] = (
    "machine-theory",
    "human-theory",
    "machine-orientation",
    "corpus-policy-module",
    "corpus-policy-test",
    "corpus-ingest-test",
    "theory-contract-test",
)
_DIRECTOR_TOKEN_IDS: Final[tuple[str, ...]] = (
    "director-token-1",
    "director-token-2",
    "director-token-3",
    "director-token-4",
    "director-token-5",
    "director-token-6",
    "director-token-7",
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
            "Accumulation produces pressures and limits; paths and outcomes remain contingent."
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
            "Outcomes are recognizers over histories, not downstream writes or promised verdicts."
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
        "edition": "supplied clipping; mixed-admissibility source",
        "sha256": _SOURCE_HASHES[2][1],
        "evidence_class": "Observed",
        "executable_authority": False,
        "availability": "repository_file",
        "repository_path": "ai/_inbox/archive/theory-of-the-party-ill-will.md",
        "scope": (
            "Retains only bounded observations about situated practice when "
            "independently corroborated by permitted book findings. Cannot "
            "authorize organization kinds, party relations, or mechanics."
        ),
        "anchors": ["repository lines 30-42"],
        "admissibility": "bounded_with_independent_corroboration",
        "quarantined_ranges": [
            "repository lines 44-46",
            "repository lines 74-130",
        ],
        "corroborating_findings": [
            "H8",
            "W10",
            "W20",
            "W36",
            "W37",
            "W39",
            "W40",
            "W45",
            "W62-W65",
        ],
    },
    "cpusa_organizers_manual_ch3_1935": {
        "title": "Organizers' Manual, chapter 3",
        "edition": "Communist Party USA, 1935, local HTML",
        "sha256": _SOURCE_HASHES[3][1],
        "evidence_class": "Observed",
        "executable_authority": False,
        "availability": "optional_local_mirror",
        "relative_locator": ("history/usa/parties/cpusa/1935/07/organisers-manual/ch03.htm"),
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
    tokens = tuple(
        token
        for row in rows
        for token in (
            row.author.casefold(),
            row.author.rpartition(" ")[2].casefold(),
            Path(row.path_glob).parts[0].casefold(),
        )
    )
    return tuple(dict.fromkeys(tokens))


def _assert_director_token_absent(*, token: str, path: Path, surface_id: str) -> None:
    if token in _normalized_text(path):
        pytest.fail(
            f"Director-exclusion hygiene violation in {surface_id}",
            pytrace=False,
        )


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
    assert meta["version"] == "2.1.0"
    assert meta["authority"] == "CONSTITUTION.md v4.2.0"
    assert meta["architecture"] == "docs/concepts/architecture.rst"
    assert meta["reserved_line"] == "Marxist-Leninist-Maoist Third Worldist (MLM-TW)"
    assert document["theory_boundary"] == {
        "constrains": "represented_relations_and_causal_questions",
        "does_not": [
            "predetermine_outcomes",
            "impose_response_curves",
            "create_executable_rules",
            "make_geography_or_class_an_essence",
        ],
        "evidence_classes": ["Observed", "Derived", "Calibrated", "Designed"],
    }
    assert document["ai_assistant_guidelines"] == [
        "Model material relations as constraints on causal questions, not outcomes.",
        "Ask which classes and organizations benefit from each represented relation.",
        "Use dialectical analysis to inspect contradictions and transformations.",
        "Do not use theory to predetermine a result.",
    ]


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


@pytest.mark.parametrize(
    "token",
    _director_excluded_tokens(),
    ids=_DIRECTOR_TOKEN_IDS,
)
@pytest.mark.parametrize(
    "path",
    _EXCLUSION_HYGIENE_PATHS,
    ids=_EXCLUSION_HYGIENE_SURFACE_IDS,
)
def test_active_policy_and_theory_surfaces_exclude_director_denied_tokens(
    token: str,
    path: Path,
) -> None:
    surface_id = _EXCLUSION_HYGIENE_SURFACE_IDS[_EXCLUSION_HYGIENE_PATHS.index(path)]
    _assert_director_token_absent(token=token, path=path, surface_id=surface_id)


def test_exclusion_hygiene_failure_does_not_render_sensitive_token(tmp_path: Path) -> None:
    sensitive_token = "sensitive sentinel"
    surface = tmp_path / "neutral-surface.txt"
    surface.write_text(sensitive_token, encoding="utf-8")

    with pytest.raises(pytest.fail.Exception) as failure:
        _assert_director_token_absent(
            token=sensitive_token,
            path=surface,
            surface_id="neutral-surface",
        )

    assert sensitive_token not in str(failure.value)


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
    for mixed_source_field in (
        "admissibility",
        "quarantined_ranges",
        "corroborating_findings",
    ):
        if mixed_source_field not in expected:
            continue
        field_value = expected[mixed_source_field]
        values = field_value if isinstance(field_value, list) else [field_value]
        for value in values:
            assert _normalized_value(str(value)) in rendered


def test_mantra_north_star_describes_contingent_political_possibility() -> None:
    document = yaml.safe_load(_MANTRAS.read_text(encoding="utf-8"))
    north_star = " ".join(document["mantras"]["north_star"]["meaning"].casefold().split())

    assert "why revolution happens in the periphery, not the core" not in north_star
    assert "how organization and solidarity can redirect political possibilities" in north_star
