"""Executable structure and continuity contract for Constitution v4.1.0.

The live constitution stays short.  The transition ledger in ADR221 carries
the exhaustive historical mapping from the two pinned predecessor snapshots.
"""

from __future__ import annotations

import hashlib
import re
import subprocess
from pathlib import Path
from typing import Final

import pytest
import yaml

pytestmark = [pytest.mark.unit]

_ROOT: Final[Path] = Path(__file__).resolve().parents[3]
_CONSTITUTION: Final[Path] = _ROOT / "CONSTITUTION.md"
_ADR_STEM: Final[str] = "ADR221_game_first_refoundation_v4"
_ADR_PATH: Final[Path] = _ROOT / "ai" / "decisions" / f"{_ADR_STEM}.yaml"
_INDEX_PATH: Final[Path] = _ROOT / "ai" / "decisions" / "index.yaml"
_MANIFEST_PATH: Final[Path] = (
    _ROOT / "tests" / "fixtures" / "governance" / "constitution_v3_2_manifest.yaml"
)
_V3_1_BLOB: Final[str] = "a265b85120ed2a90be40c72e63ee5bf27fc6e703"
_V3_2_BLOB: Final[str] = "e905e90d66bddc6e4eca36a3896428f5ce63de5b"
_V3_1_COMMIT: Final[str] = "3acd1089b6b4e68177c99b4f4cec245e7b74317c"
_V3_2_COMMIT: Final[str] = "cbfc67921283ccb6e00c4b0278288a232281440a"
_V3_2_BYTE_SIZE: Final[int] = 118_006
_V3_2_SHA256: Final[str] = "f9f295504f50b5c9f99323cea7d329aa0bdc46a07c0c4032bfb784810e7e3193"
_MAX_CONSTITUTION_BLOB_BYTES: Final[int] = 128_000
_MAX_MANIFEST_BYTES: Final[int] = 16_384
_MAX_PREDECESSOR_LINES: Final[int] = 1_024

_LIVE_ORIENTATION_PATHS: Final[tuple[str, ...]] = (
    "NORTH_STAR.md",
    "CLAUDE.md",
    "CONTRIBUTORS.md",
    "docs/agents/governance.md",
    ".github/copilot-instructions.md",
    ".opencode/skills/specification-discover/SKILL.md",
    ".opencode/skills/specification-validate/SKILL.md",
    ".serena/memories/project_overview.md",
    "README.md",
    "SETUP_GUIDE.md",
    "docs/index.rst",
    "docs/docs-pdf-index.rst",
    "docs/commentary/design-philosophy.rst",
    "docs/concepts/architecture.rst",
    "docs/concepts/index.rst",
    "ai/mantras.yaml",
)
_BEHAVIORAL_CONTRACT_PATHS: Final[tuple[str, ...]] = (
    "tests/unit/sentinels/test_determinism.py",
    "tests/integration/system/test_phase2_game_loop.py",
    "tests/constants.py",
    "tests/unit/engine/systems/test_metabolism.py",
)
_CANONICAL_ORIENTATION_PATHS: Final[tuple[str, ...]] = (
    "NORTH_STAR.md",
    "CLAUDE.md",
    "README.md",
    "docs/index.rst",
    "CONTRIBUTORS.md",
    "docs/agents/governance.md",
)
_MACHINE_ORIENTATION_PATH: Final[str] = "ai/mantras.yaml"
_HUMAN_ORIENTATION_PATH: Final[str] = "NORTH_STAR.md"
_ROUTED_ORIENTATION_PATHS: Final[tuple[str, ...]] = tuple(
    path
    for path in _LIVE_ORIENTATION_PATHS
    if path not in {_MACHINE_ORIENTATION_PATH, _HUMAN_ORIENTATION_PATH}
)
_CONTROL_SURFACE_PATHS: Final[tuple[str, ...]] = (
    "CLAUDE.md",
    "CONTRIBUTORS.md",
    "docs/agents/governance.md",
    ".github/copilot-instructions.md",
)
_CONTROL_AUTHORITY_PATH: Final[str] = "docs/agents/governance.md"
_ORIENTATION_CONCEPTS: Final[tuple[tuple[str, ...], ...]] = (
    ("entertainment-first",),
    ("emergent political-economy game",),
    ("not a forecast",),
    ("scientific reproduction",),
    ("theory constrains",),
    ("does not predetermine", "without predetermining"),
    ("computational identity",),
    ("scientific truth",),
    ("causal signatures",),
    ("counterfactual",),
    ("admin/viewer", "administrative viewer"),
    ("no player action", "no committed player action"),
)
_NEXT_GATES: Final[tuple[str, ...]] = (
    "PostgreSQL/H3/Archive decision-loop slice",
    "Productive & distributive circuit",
    "Player agency",
    "COVID emergence benchmark",
)
_SUPERSEDED_PRODUCT_PHRASES: Final[tuple[str, ...]] = (
    "graph + math = history",
    "modeling the collapse of american hegemony",
    "models class struggle as a deterministic output",
    "class struggle is the deterministic output",
    "history is the deterministic output",
    "collapse is certain",
    "tragedy of inevitability",
    "the default trajectory toward necropolis",
    "the only gate that matters",
    "must reproduce observed",
    "failure = theory or implementation wrong",
    "project #8 is the sole board",
)
_CATEGORICAL_OUTCOME_PATTERNS: Final[tuple[re.Pattern[str], ...]] = (
    re.compile(
        r"\b(?:agitation|conditions?|crises?|crisis|shocks?|theory|history|"
        r"material relations?)\b[^.\n]{0,160}\b(?:produces?|determines?|dictates?|"
        r"guarantees?|ensures?|inevitably (?:causes?|leads? to)|must (?:cause|lead to|"
        r"end in)|always (?:causes?|becomes?|leads? to))\b[^.\n]{0,100}\b"
        r"(?:fascism|revolution|collapse|victory|defeat|outcomes?|history)\b",
        re.IGNORECASE,
    ),
    re.compile(
        r"\b(?:fascism|revolution|collapse|victory|defeat|history|outcomes?)\b"
        r"[^.\n]{0,100}\b(?:is|are) (?:the )?(?:deterministic|inevitable|guaranteed)"
        r" (?:result|output|outcome)\b",
        re.IGNORECASE,
    ),
)
_NEGATED_CATEGORICAL_PREDICATE: Final[re.Pattern[str]] = re.compile(
    r"\b(?:(?:does?|do|did|can|could|will|would|must|may|might) not|"
    r"cannot|never)\s+(?:produces?|determines?|dictates?|guarantees?|ensures?|"
    r"inevitably (?:causes?|leads? to)|always (?:causes?|becomes?|leads? to))\b",
    re.IGNORECASE,
)
_OLD_DOTTED_ARTICLE_REFERENCE: Final[re.Pattern[str]] = re.compile(
    r"\b(?:I|II|III|IV|V|VI|VII|VIII|IX|X)\.\d+(?:\.\d+)?\b"
)

_ARTICLE_HEADINGS: Final[tuple[str, ...]] = (
    "## Article I — Purpose and Player Promise",
    "## Article II — Theoretical Commitments",
    "## Article III — Emergence and Agency",
    "## Article IV — Evidence and Design Liberties",
    "## Article V — Deterministic Architecture",
    "## Article VI — Political-Economy Circuit",
    "## Article VII — Player Knowledge and Presentation",
    "## Article VIII — Behavioral Validation and Governance",
)

_UNDOTTED_SECTIONS: Final[tuple[str, ...]] = (
    "I.19.Apex",
    "IV",
    "V.Player",
    "V.Investigate",
    "V.State-AI",
)
_DISPOSITIONS: Final[frozenset[str]] = frozenset({"Retained", "Rewritten", "Re-homed", "Retired"})
_ARTICLE_REFERENCE: Final[re.Pattern[str]] = re.compile(r"Article (?:I|II|III|IV|V|VI|VII|VIII)")
_TRANSITION_REFERENCE: Final[re.Pattern[str]] = re.compile(
    r"transition\.(?:sources\.(?:v3_1|v3_2)|clauses(?:\.[A-Z0-9.a-z]+)?|"
    r"undotted_sections(?:\.[A-Z0-9.-]+)?|amendments(?:\.[A-Z]{1,2})?)"
)
_ARTICLE_HEADING: Final[re.Pattern[str]] = re.compile(
    r"^## (?P<article>I|II|III|IV|V|VI|VII|VIII|IX|X)\."
)
_EXPLICIT_CLAUSE: Final[re.Pattern[str]] = re.compile(
    r"^\*\*(?P<number>\d+\.\d+|\d+[a-z]?)(?:\.| )"
)
_ARTICLE_IV_CLAUSE: Final[re.Pattern[str]] = re.compile(r"^### IV\.(?P<number>\d+) ")
_AMENDMENT_HEADING: Final[re.Pattern[str]] = re.compile(r"^\*\*Amendment (?P<letter>[A-Z]{1,2}) —")
_AMENDMENT_REGISTRATION: Final[re.Pattern[str]] = re.compile(
    r"^Bump Rationale: [A-Z]+ — Amendments? (?P<letter>[A-Z]{1,2})"
)
_PLAYER_ROSTER: Final[re.Pattern[str]] = re.compile(
    r"### Player \(9 verbs\)\n\n\*\*(?P<verbs>[A-Za-z, ]+)\*\*\."
)


def _constitution_text() -> str:
    """Return the live constitutional text."""
    return _CONSTITUTION.read_text(encoding="utf-8")


def _repository_text(relative_path: str) -> str:
    """Return one text file from the bounded live-governance corpus."""
    return (_ROOT / relative_path).read_text(encoding="utf-8")


def _categorical_outcome_matches(text: str) -> tuple[str, ...]:
    """Return affirmative categorical outcomes from one bounded prose record."""
    return tuple(
        match.group(0)
        for pattern in _CATEGORICAL_OUTCOME_PATTERNS
        for match in pattern.finditer(text)
        if _NEGATED_CATEGORICAL_PREDICATE.search(match.group(0)) is None
    )


def _adr_entry() -> dict[str, object]:
    """Return ADR221's single keyed record."""
    document = yaml.safe_load(_ADR_PATH.read_text(encoding="utf-8"))
    assert list(document) == [_ADR_STEM]
    return document[_ADR_STEM]


def _transition() -> dict[str, object]:
    """Return ADR221's transition ledger."""
    transition = _adr_entry()["transition"]
    assert isinstance(transition, dict)
    return transition


def _manifest() -> dict[str, object]:
    """Return the checked, size-bounded v3.2 continuity manifest."""
    assert 0 < _MANIFEST_PATH.stat().st_size <= _MAX_MANIFEST_BYTES
    document = yaml.safe_load(_MANIFEST_PATH.read_text(encoding="utf-8"))
    assert isinstance(document, dict)
    return document


def _manifest_family(name: str) -> tuple[str, ...]:
    """Return one ordered family from the independent v3.2 manifest."""
    families = _manifest()["families"]
    assert isinstance(families, dict)
    identifiers = families[name]
    assert isinstance(identifiers, list)
    assert all(isinstance(identifier, str) for identifier in identifiers)
    return tuple(identifiers)


def _mapped_rows() -> tuple[dict[str, str], ...]:
    """Return every dotted, undotted, and amendment disposition row."""
    transition = _transition()
    rows: list[dict[str, str]] = []
    for family in ("clauses", "undotted_sections", "amendments"):
        mapping = transition[family]
        assert isinstance(mapping, dict)
        rows.extend(mapping.values())
    return tuple(rows)


def _git_blob_text(blob: str) -> str:
    """Read one size-bounded Git blob from the repository object store."""
    size_result = subprocess.run(  # noqa: S603
        ["git", "cat-file", "-s", blob],  # noqa: S607
        cwd=_ROOT,
        capture_output=True,
        text=True,
        check=True,
        timeout=5,
    )
    size = int(size_result.stdout.strip())
    assert 0 < size <= _MAX_CONSTITUTION_BLOB_BYTES
    blob_result = subprocess.run(  # noqa: S603
        ["git", "cat-file", "-p", blob],  # noqa: S607
        cwd=_ROOT,
        capture_output=True,
        text=True,
        check=True,
        timeout=5,
    )
    assert len(blob_result.stdout.encode("utf-8")) == size
    return blob_result.stdout


def _git_object_exists(spec: str) -> bool:
    """Return whether one Git object spec resolves without fetching."""
    result = subprocess.run(  # noqa: S603
        ["git", "cat-file", "-e", spec],  # noqa: S607
        cwd=_ROOT,
        capture_output=True,
        check=False,
        timeout=5,
    )
    return result.returncode == 0


def _git_object_id(spec: str) -> str:
    """Resolve one bounded predecessor path to its Git blob identity."""
    result = subprocess.run(  # noqa: S603
        ["git", "rev-parse", spec],  # noqa: S607
        cwd=_ROOT,
        capture_output=True,
        text=True,
        check=True,
        timeout=5,
    )
    return result.stdout.strip()


def _extract_v3_2_manifest(text: str) -> dict[str, object]:
    """Independently extract transition families and the player roster."""
    lines = text.splitlines()
    assert 0 < len(lines) <= _MAX_PREDECESSOR_LINES
    dotted: list[str] = []
    named: list[str] = []
    amendments: set[str] = set()
    list_counts = {"VI": 0, "VII": 0, "VIII": 0}
    article = ""
    for line in lines[:_MAX_PREDECESSOR_LINES]:
        article_match = _ARTICLE_HEADING.match(line)
        if article_match is not None:
            article = article_match.group("article")
            if article == "IV":
                named.append("IV")
            continue
        explicit_match = _EXPLICIT_CLAUSE.match(line)
        if article in {"I", "II", "III", "IX", "X"} and explicit_match is not None:
            dotted.append(f"{article}.{explicit_match.group('number')}")
        if article in list_counts and line.startswith("1. **"):
            list_counts[article] += 1
            dotted.append(f"{article}.{list_counts[article]}")
        article_iv_match = _ARTICLE_IV_CLAUSE.match(line)
        if article == "IV" and article_iv_match is not None:
            dotted.append(f"IV.{article_iv_match.group('number')}")
        if article == "I" and line.startswith("**Apex Abstraction clause**"):
            named.append("I.19.Apex")
        if article == "V" and line.startswith("### Player (9 verbs)"):
            named.append("V.Player")
        if article == "V" and line.startswith("**Investigate Sub-Verbs**"):
            named.append("V.Investigate")
        if article == "V" and line.startswith("### State AI (6 verbs)"):
            named.append("V.State-AI")
        amendment_match = _AMENDMENT_HEADING.match(line)
        if article == "IX" and amendment_match is not None:
            amendments.add(amendment_match.group("letter"))
        registration_match = _AMENDMENT_REGISTRATION.match(line)
        if registration_match is not None:
            amendments.add(registration_match.group("letter"))

    roster_match = _PLAYER_ROSTER.search(text)
    assert roster_match is not None
    roster = [verb.strip() for verb in roster_match.group("verbs").split(",")]
    return {
        "player_verbs": roster,
        "families": {
            "dotted_clauses": dotted,
            "named_sections": named,
            "registered_amendments": sorted(amendments, key=lambda value: (len(value), value)),
        },
    }


def test_live_orientation_corpus_is_exact_bounded_and_present() -> None:
    """The v4 guard covers only the declared current entry points."""
    expected = (
        "NORTH_STAR.md",
        "CLAUDE.md",
        "CONTRIBUTORS.md",
        "docs/agents/governance.md",
        ".github/copilot-instructions.md",
        ".opencode/skills/specification-discover/SKILL.md",
        ".opencode/skills/specification-validate/SKILL.md",
        ".serena/memories/project_overview.md",
        "README.md",
        "SETUP_GUIDE.md",
        "docs/index.rst",
        "docs/docs-pdf-index.rst",
        "docs/commentary/design-philosophy.rst",
        "docs/concepts/architecture.rst",
        "docs/concepts/index.rst",
        "ai/mantras.yaml",
    )
    assert expected == _LIVE_ORIENTATION_PATHS
    assert len(_LIVE_ORIENTATION_PATHS) == len(set(_LIVE_ORIENTATION_PATHS))
    assert all((_ROOT / path).is_file() for path in _LIVE_ORIENTATION_PATHS)


def test_human_orientation_states_the_v4_semantic_contract() -> None:
    """The human north star explains the machine-readable orientation."""
    text = " ".join(_repository_text(_HUMAN_ORIENTATION_PATH).lower().split())
    missing = tuple(
        alternatives
        for alternatives in _ORIENTATION_CONCEPTS
        if not any(alternative in text for alternative in alternatives)
    )
    assert missing == (), f"{_HUMAN_ORIENTATION_PATH} is missing {missing}"


@pytest.mark.parametrize("relative_path", _CANONICAL_ORIENTATION_PATHS)
def test_canonical_entry_points_order_the_next_gates(relative_path: str) -> None:
    """Canonical execution records state the ADR250 next gates in order."""
    text = _repository_text(relative_path)
    positions = tuple(text.find(gate) for gate in _NEXT_GATES)
    assert all(position >= 0 for position in positions), (
        f"{relative_path} is missing gates at positions {positions}"
    )
    assert positions == tuple(sorted(positions)), f"{relative_path} has reordered gates"


@pytest.mark.parametrize("relative_path", _ROUTED_ORIENTATION_PATHS)
def test_live_entry_points_route_to_v4_orientation(relative_path: str) -> None:
    """Narrow entry points route readers instead of duplicating orientation prose."""
    text = _repository_text(relative_path).lower()
    authorities = ("constitution.md", "constitution v4", "north_star.md")
    assert any(authority in text for authority in authorities), relative_path


def test_mantras_publish_machine_readable_v4_orientation() -> None:
    """One canonical data record exposes purpose, validation, status, and gates."""
    document = yaml.safe_load(_repository_text(_MACHINE_ORIENTATION_PATH))
    meta = document["meta"]
    assert meta["authority"] == "CONSTITUTION.md v4.1.0"
    assert meta["product"] == "entertainment-first emergent political-economy game"
    assert meta["forecast"] is False
    assert meta["scientific_reproduction"] is False
    assert meta["theory"] == "constrains the causal model without predetermining outcomes"
    assert meta["determinism_proves"] == "computational identity"
    assert tuple(meta["historical_cases_test"]) == (
        "causal signatures",
        "counterfactual behavior",
    )
    assert meta["current_client"] == "Bevy admin/viewer; no player action"
    assert tuple(meta["next_gates"]) == _NEXT_GATES


@pytest.mark.parametrize("relative_path", _CONTROL_SURFACE_PATHS)
def test_contributor_entry_points_route_work_tracking_to_linear(relative_path: str) -> None:
    """Each contributor control entry routes current work through Linear."""
    text = " ".join(_repository_text(relative_path).lower().split())
    required = ("linear", "canonical", "github", "project #7", "project #8")
    missing = tuple(phrase for phrase in required if phrase not in text)
    assert missing == (), f"{relative_path} is missing {missing}"
    assert "migration is complete" in text


def test_control_authority_states_the_linear_github_boundary() -> None:
    """The control authority defines ownership without duplicating the charter."""
    text = " ".join(_repository_text(_CONTROL_AUTHORITY_PATH).lower().split())
    required = (
        "babylon v1 — playable political economy",
        "per-5",
        "per-15",
        "issue identity",
        "scope",
        "status",
        "priority",
        "dependencies",
        "horizon",
        "milestones",
        "schedule",
        "current work",
        "source control",
        "pull requests",
        "reviews",
        "historical evidence",
        "project #7",
        "project #8",
        "team closed github project #7 and project #8",
        "historical inputs",
        "per-15 is complete",
        "ai/state.yaml",
        "historical implementation evidence",
        "project/",
        "non-live context",
    )
    missing = tuple(phrase for phrase in required if phrase not in text)
    assert missing == (), f"{_CONTROL_AUTHORITY_PATH} is missing {missing}"


def test_live_orientation_does_not_route_to_pending_architecture_record() -> None:
    """Entry points defer the stale architecture catalog until Unit 2B aligns it."""
    found = tuple(
        path
        for path in _LIVE_ORIENTATION_PATHS
        if "ai/architecture.yaml" in _repository_text(path).lower()
    )
    assert found == ()


def test_architecture_reference_marks_current_cutover_and_remaining_gate_3_work() -> None:
    """The detailed reference separates the landed cutover from the playable slice."""
    text = " ".join(_repository_text("docs/concepts/architecture.rst").lower().split())
    required = (
        "live rust bsl rules",
        "executable shocks",
        "identifiedtickreportv2",
        "frozen python",
        "runtimedatabase",
        "committedtickenvelopev2",
        "marker-last transaction",
        "``tick_commit``",
        "``babylon_meta``",
        "semantic archive worker",
        "per-48 is decided",
        "the one-way cutover is complete",
        "rust owns authoritative game-managed postgres",
        "python continues",
    )
    missing = tuple(phrase for phrase in required if phrase not in text)
    assert missing == (), f"docs/concepts/architecture.rst is missing {missing}"


@pytest.mark.parametrize("relative_path", ("CLAUDE.md", "docs/concepts/architecture.rst"))
def test_live_guidance_records_the_decided_postgres_cutover(relative_path: str) -> None:
    """The implemented boundary keeps one writer and a bounded Python periphery."""
    text = " ".join(_repository_text(relative_path).lower().split())
    required = (
        "per-48 is decided",
        "the one-way cutover is complete",
        "rust owns authoritative game-managed postgres",
        "python continues",
    )
    prohibited = (
        "per-48 must resolve",
        "per-48 is in progress",
        "per-48 blocks per-20",
        "has not resolved which language",
    )
    missing = tuple(phrase for phrase in required if phrase not in text)
    stale = tuple(phrase for phrase in prohibited if phrase in text)
    assert missing == (), f"{relative_path} is missing {missing}"
    assert stale == (), f"{relative_path} retains {stale}"


@pytest.mark.parametrize(
    "relative_path",
    ("docs/concepts/architecture.rst", "NORTH_STAR.md"),
)
def test_live_architecture_limits_the_downstream_write_prohibition_to_shocks(
    relative_path: str,
) -> None:
    """Ordinary BSL derivation remains legal while shocks cannot author effects."""
    text = " ".join(_repository_text(relative_path).lower().split())
    assert "shock" in text
    assert "must not write" in text
    assert "downstream result" in text
    assert "ordinary bsl rules" in text
    assert "derive" in text
    assert "write world" in text
    assert "bsl must not write a downstream result" not in text


def test_claude_keeps_gate_4_and_gate_5_work_out_of_gate_3() -> None:
    """The persistence slice cannot absorb later shock or player-action gates."""
    text = " ".join(_repository_text("CLAUDE.md").split())
    gate_3_start = text.index("Gate 3 now has")
    gate_3_end = text.index(".", gate_3_start)
    gate_3_sentence = text[gate_3_start:gate_3_end].lower()
    assert "executable shocks" not in gate_3_sentence
    assert "player actions" not in gate_3_sentence
    assert "gate 4" in text.lower()
    assert "gate 5" in text.lower()


def test_contributor_guide_preserves_branch_and_hotfix_law() -> None:
    """Normal lane flexibility cannot erase the Director-only emergency path."""
    text = " ".join(_repository_text(_CONTROL_AUTHORITY_PATH).lower().split())
    required = (
        ("`codex/`",),
        ("`per-",),
        ("critical hotfix",),
        ("director-only", "only the director"),
        ("directly to `main`",),
        ("mandatory backport",),
        ("`dev`",),
    )
    missing = tuple(
        alternatives
        for alternatives in required
        if not any(phrase in text for phrase in alternatives)
    )
    assert missing == (), f"{_CONTROL_AUTHORITY_PATH} is missing {missing}"


def test_claude_preserves_exact_operational_gotchas() -> None:
    """The compact agent guide retains the failure-prevention contracts."""
    text = _repository_text("CLAUDE.md")
    folded = " ".join(text.lower().split())
    required = (
        "docs/agents/gotchas.md",
        "dynamic_hex_state",
        "v_hex_state_asof",
        "max(tick)",
        "tick_commit",
        "check:vocabulary",
        'pythonpath="$pwd/src"',
        "end-of-file-fixer",
        "trailing newline",
        "workflow `args`",
        "stringified",
        "hard fallback",
    )
    missing = tuple(phrase for phrase in required if phrase not in folded)
    assert missing == (), f"CLAUDE.md is missing {missing}"
    assert len(text.splitlines()) < 200


def test_setup_names_native_host_and_bevy_prerequisites() -> None:
    """Fresh-clone setup declares native dependencies before the game entrypoint."""
    text = " ".join(_repository_text("SETUP_GUIDE.md").lower().split())
    required = (
        "docker compose",
        "host prerequisite",
        "bevy",
        "libx11-dev",
        "libasound2-dev",
        "libvulkan-dev",
        "rustup",
        "rust/rust-toolchain.toml",
        "mise run play",
    )
    missing = tuple(phrase for phrase in required if phrase not in text)
    assert missing == (), f"SETUP_GUIDE.md is missing {missing}"


def test_north_star_names_and_lists_all_seven_abstract_parts() -> None:
    """The main abstraction keeps its declared count and enumerated parts aligned."""
    text = _repository_text("NORTH_STAR.md")
    section = text.split("## The system without political economy", 1)[1].split(
        "## Live path and planned cycle", 1
    )[0]
    parts = tuple(line for line in section.splitlines() if line.startswith("- "))
    assert "general system has seven parts:" in section
    assert len(parts) == 7


def test_readme_distinguishes_both_live_sqlite_roles() -> None:
    """The reference artifact does not erase mutable Python runtime SQLite."""
    text = " ".join(_repository_text("README.md").lower().split())
    required = (
        "deterministic reference sqlite",
        "mutable `runtimedatabase` sqlite",
    )
    missing = tuple(phrase for phrase in required if phrase not in text)
    assert missing == (), f"README.md is missing {missing}"


def test_readme_runs_bevy_through_the_native_campaign_launcher() -> None:
    """The documented game entrypoint composes durable runtime and native client."""
    text = _repository_text("README.md")
    assert "\nmise run play\n" in text
    assert "mise run nix" not in text
    assert "\nmise run rust:client-dev-dylib\n" not in text


def test_north_star_marks_future_circuit_mechanics_as_planned() -> None:
    """Allocation and clearing ownership cannot masquerade as live behavior."""
    text = " ".join(_repository_text("NORTH_STAR.md").split())
    assert "Rust allocates, routes, settles, clears" not in text
    assert (
        re.search(
            r"planned[^.]{0,100}Rust[^.]{0,100}allocat[^.]{0,100}rout[^.]{0,100}"
            r"settl[^.]{0,100}clear",
            text,
            re.IGNORECASE,
        )
        is not None
    )


def test_game_first_transition_uses_adr221_without_rewriting_history() -> None:
    """The game-first ledger yields ADR220 to the accepted persistence boundary."""
    expected = {
        "CONSTITUTION.md": "ADR221_game_first_refoundation_v4.yaml",
        "NORTH_STAR.md": "ADR221 records",
        "CLAUDE.md": "ADR221 maps",
        ".opencode/skills/specification-discover/SKILL.md": "Use ADR221",
    }
    for relative_path, phrase in expected.items():
        assert phrase in _repository_text(relative_path), relative_path


def test_live_entry_points_name_only_the_existing_legacy_web_client() -> None:
    """Current orientation never resurrects the deleted React client tree."""
    found = tuple(
        path
        for path in _LIVE_ORIENTATION_PATHS
        if "src/frontend" in _repository_text(path).lower()
        or "react" in _repository_text(path).lower()
    )
    assert found == ()


def test_skill_frontmatter_keeps_literal_identifiers() -> None:
    """Tool discovery sees literal skill names, not escaped lookalikes."""
    expected = {
        ".opencode/skills/specification-discover/SKILL.md": "specification-discover",
        ".opencode/skills/specification-validate/SKILL.md": "specification-validate",
    }
    for relative_path, identifier in expected.items():
        text = _repository_text(relative_path)
        assert text.startswith("---\n"), relative_path
        closing_delimiter = text.find("\n---\n", 4, 1_028)
        assert closing_delimiter > 4, relative_path
        frontmatter = text[4:closing_delimiter]
        document = yaml.safe_load(frontmatter)
        assert document["name"] == identifier
        assert f"name: {identifier}" in frontmatter
        assert "\\u" not in frontmatter


def test_mantras_describe_tendency_and_session_state_honestly() -> None:
    """Mantras stay memorable without making false outcome or statelessness claims."""
    document = yaml.safe_load(_repository_text(_MACHINE_ORIENTATION_PATH))
    mantras = document["mantras"]
    state_meaning = mantras["state_is_data"]["meaning"].lower()
    tendency = " ".join(
        (
            mantras["agitation_without_solidarity"]["text"],
            mantras["agitation_without_solidarity"]["meaning"],
        )
    ).lower()
    assert "ticksession" in state_meaning
    assert "no state in engine classes" not in state_meaning
    assert "pressure" in tendency
    assert "does not predetermine" in tendency


def test_copilot_restore_key_guidance_is_scoped_to_python_venv_caches() -> None:
    """The cache warning does not prohibit valid non-venv restore fallbacks."""
    text = " ".join(_repository_text(".github/copilot-instructions.md").lower().split())
    required = ("python virtual-environment caches", "restore-keys")
    missing = tuple(phrase for phrase in required if phrase not in text)
    assert missing == (), f".github/copilot-instructions.md is missing {missing}"


def test_control_authority_documents_per_2_delivery_automation() -> None:
    """The guide distinguishes partial delivery from final issue completion."""
    text = " ".join(_repository_text(_CONTROL_AUTHORITY_PATH).lower().split())
    required = (
        "per-2",
        "part of per-n",
        "fixes per-n",
        "non-closing",
        "closing",
        "multi-pull-request",
        "automation does not require github project fields",
    )
    missing = tuple(phrase for phrase in required if phrase not in text)
    assert missing == (), f"{_CONTROL_AUTHORITY_PATH} is missing {missing}"
    assert "team has not verified automation" not in text


@pytest.mark.parametrize("relative_path", _LIVE_ORIENTATION_PATHS + _BEHAVIORAL_CONTRACT_PATHS)
def test_live_v4_corpus_rejects_superseded_product_promises(relative_path: str) -> None:
    """Current authority and contracts never promise predetermined history."""
    text = _repository_text(relative_path).lower()
    found = tuple(phrase for phrase in _SUPERSEDED_PRODUCT_PHRASES if phrase in text)
    assert found == (), f"{relative_path} retains {found}"


@pytest.mark.parametrize("relative_path", _LIVE_ORIENTATION_PATHS + _BEHAVIORAL_CONTRACT_PATHS)
def test_live_v4_corpus_rejects_categorical_historical_outcomes(relative_path: str) -> None:
    """Causal pressure may shape tendencies but cannot guarantee a political result."""
    found = _categorical_outcome_matches(_repository_text(relative_path))
    assert found == (), f"{relative_path} retains categorical outcomes {found}"


@pytest.mark.parametrize(
    "statement",
    (
        "A shock does not guarantee victory.",
        "Theory never dictates outcomes.",
    ),
)
def test_categorical_outcome_guard_accepts_negated_law(statement: str) -> None:
    """Accurate prohibitions are negative controls, not categorical promises."""
    assert _categorical_outcome_matches(statement) == ()


@pytest.mark.parametrize(
    "statement",
    (
        "A shock guarantees victory.",
        "Theory dictates outcomes.",
    ),
)
def test_categorical_outcome_guard_rejects_affirmative_guarantees(statement: str) -> None:
    """The semantic guard continues to reject affirmative outcome promises."""
    assert _categorical_outcome_matches(statement)


@pytest.mark.parametrize("relative_path", _LIVE_ORIENTATION_PATHS + _BEHAVIORAL_CONTRACT_PATHS)
def test_live_v4_corpus_has_no_broken_dotted_article_references(relative_path: str) -> None:
    """Current prose cites v4 Articles or ADRs, never retired dotted clauses."""
    references = tuple(_OLD_DOTTED_ARTICLE_REFERENCE.findall(_repository_text(relative_path)))
    assert references == (), f"{relative_path} retains {references}"


def test_constitution_has_exactly_the_approved_eight_articles() -> None:
    """Article names and order are the v4 public orientation contract."""
    headings = tuple(
        line for line in _constitution_text().splitlines() if line.startswith("## Article ")
    )
    assert headings == _ARTICLE_HEADINGS


def test_constitution_states_the_game_first_promise_and_evidence_classes() -> None:
    """A fresh reader can recover purpose, method, and the validation standard."""
    text = " ".join(_constitution_text().split())
    required = (
        "Version 4.1.0",
        "entertainment",
        "expressive",
        "emergent",
        "not a forecast",
        "not a scientific reproduction",
        "Observed",
        "Derived",
        "Calibrated",
        "Designed",
        "computational identity",
        "scientific truth",
        "decision question",
        "causal signatures",
        "heterogeneity",
        "hysteresis",
        "counterfactual responsiveness",
        "Amendment AH — Game-First Refoundation",
        "Amendment AJ — Finite Material Transition Kernels",
    )
    missing = tuple(phrase for phrase in required if phrase not in text)
    assert missing == ()


def test_constitution_pins_the_exact_v4_primitive_and_retained_invariants() -> None:
    """AH carries each retained formal invariant in the live law."""
    raw_text = _constitution_text()
    text = " ".join(raw_text.split())
    required = (
        "D = (A, Ā, w, T, σ)",
        "material position and conceived interest",
        "Nothing can abstract over dialectical motion",
        "law, a falsifiable prediction, or a running computation",
        "must never expand into a pairwise clique",
        "Every materialized projection must be deterministic",
    )
    assert tuple(phrase for phrase in required if phrase not in text) == ()
    assert "D = (A, A-bar, w, T, s)" not in raw_text


def test_material_action_effects_always_become_next_week_intents() -> None:
    """Only epistemic feedback, never material mutation, can resolve now."""
    text = " ".join(_constitution_text().split())
    assert "All material effects become next-week intents" in text
    assert "only non-material knowledge, previews, and receipts" in text
    assert "unless an explicitly governed rule requires immediate resolution" not in text


def test_constitution_rejects_superseded_product_promises() -> None:
    """Deterministic computation must never become predetermined history."""
    text = _constitution_text()
    prohibited = (
        "Collapse is default",
        "Player shapes character, not outcome",
        "testing MLM-TW political economy against empirical data",
        "MUST reproduce observed",
        "Failure = theory or implementation wrong",
        "the only gate that matters",
    )
    found = tuple(phrase for phrase in prohibited if phrase in text)
    assert found == ()


def test_transition_adr_pins_both_predecessor_blobs_and_resolves_citations() -> None:
    """The refoundation preserves v3.1 and the immediate v3.2/AG authority."""
    transition = _transition()
    sources = transition["sources"]
    assert isinstance(sources, dict)
    assert set(sources) == {"v3_1", "v3_2"}
    assert sources["v3_1"] == {
        "version": "3.1.0",
        "blob": _V3_1_BLOB,
        "note": "Requested AF-era predecessor snapshot.",
    }
    assert sources["v3_2"] == {
        "version": "3.2.0",
        "blob": _V3_2_BLOB,
        "note": "Immediate predecessor and source of Amendment AG authority.",
    }
    resolution = transition["citation_resolution"]
    assert isinstance(resolution, dict)
    assert set(resolution) == {"dotted_clauses", "named_sections", "amendments", "post_ah"}
    for family in ("dotted_clauses", "named_sections", "amendments"):
        assert "pinned v3.2" in resolution[family]


def test_transition_adr_records_the_primitive_notation_ruling_exactly() -> None:
    """AH changes v3.2's ``s`` to ``sigma`` without rewriting history."""
    notation = _transition()["primitive_notation"]
    assert isinstance(notation, dict)
    assert set(notation) == {"v3_2", "v4", "ruling"}
    assert notation["v3_2"] == "D = (A, Ā, w, T, s)"
    assert notation["v4"] == "D = (A, Ā, w, T, σ)"
    assert "Amendment AH" in notation["ruling"]
    assert "does not rewrite" in notation["ruling"]


def test_v3_2_manifest_is_complete_unique_and_exactly_pinned() -> None:
    """The checked manifest is the CI-safe independent continuity oracle."""
    manifest = _manifest()
    assert set(manifest) == {"source", "families"}
    source = manifest["source"]
    assert source == {
        "version": "3.2.0",
        "blob": _V3_2_BLOB,
        "byte_size": _V3_2_BYTE_SIZE,
        "sha256": _V3_2_SHA256,
        "player_verbs": [
            "Educate",
            "Aid",
            "Attack",
            "Mobilize",
            "Campaign",
            "Move",
            "Investigate",
            "Reproduce",
            "Negotiate",
        ],
    }
    families = manifest["families"]
    assert isinstance(families, dict)
    assert set(families) == {"dotted_clauses", "named_sections", "registered_amendments"}
    for family_name in ("dotted_clauses", "named_sections", "registered_amendments"):
        identifiers = _manifest_family(family_name)
        assert identifiers
        assert len(identifiers) == len(set(identifiers))
    assert _manifest_family("named_sections") == _UNDOTTED_SECTIONS


def test_checked_manifest_player_roster_matches_the_governed_matrix() -> None:
    """The normal unit gate needs no Git history to protect the v3.2 roster."""
    from babylon.game.actions.matrix import verbs_in_matrix

    source = _manifest()["source"]
    assert isinstance(source, dict)
    roster = source["player_verbs"]
    assert isinstance(roster, list)
    normalized = tuple(sorted(verb.lower() for verb in roster))
    assert normalized == verbs_in_matrix()


@pytest.mark.skipif(not (_ROOT / ".git").exists(), reason="Git metadata absent from source archive")
def test_checked_manifest_matches_the_supplied_predecessor_commits() -> None:
    """Repository checkouts fail loudly if CI omits the bounded history supply."""
    v3_1_spec = f"{_V3_1_COMMIT}:CONSTITUTION.md"
    v3_2_spec = f"{_V3_2_COMMIT}:CONSTITUTION.md"
    assert _git_object_exists(v3_1_spec), (
        "repository checkout is missing the pinned v3.1 Constitution commit; "
        "the test-unit workflow must run its bounded predecessor fetch"
    )
    assert _git_object_exists(v3_2_spec), (
        "repository checkout is missing the pinned v3.2 Constitution commit; "
        "the test-unit workflow must run its bounded predecessor fetch"
    )
    assert _git_object_id(v3_1_spec) == _V3_1_BLOB
    assert _git_object_id(v3_2_spec) == _V3_2_BLOB

    predecessor = _git_blob_text(v3_2_spec)
    predecessor_bytes = predecessor.encode("utf-8")
    source = _manifest()["source"]
    assert isinstance(source, dict)
    assert len(predecessor_bytes) == source["byte_size"]
    assert hashlib.sha256(predecessor_bytes).hexdigest() == source["sha256"]
    extracted = _extract_v3_2_manifest(predecessor)
    assert extracted["player_verbs"] == source["player_verbs"]
    assert extracted["families"] == _manifest()["families"]


def test_transition_adr_maps_every_old_dotted_clause_exactly_once() -> None:
    """No clause disappears through an article-level summary."""
    transition = _transition()
    clauses = transition["clauses"]
    assert isinstance(clauses, dict)
    assert tuple(clauses) == _manifest_family("dotted_clauses")


def test_transition_adr_maps_every_old_undotted_section_exactly_once() -> None:
    """Named v3 sections receive the same explicit treatment as clauses."""
    sections = _transition()["undotted_sections"]
    assert isinstance(sections, dict)
    assert tuple(sections) == _manifest_family("named_sections")


def test_transition_adr_maps_every_registered_amendment_exactly_once() -> None:
    """AG survives; AB is absent because no Amendment AB was ever registered."""
    transition = _transition()
    amendments = transition["amendments"]
    assert isinstance(amendments, dict)
    assert tuple(amendments) == _manifest_family("registered_amendments")
    assert amendments["AG"]["disposition"] == "Rewritten"


def test_materially_changed_player_and_projection_law_is_rewritten() -> None:
    """The ledger never calls a narrowed or re-bound obligation retained."""
    transition = _transition()
    clauses = transition["clauses"]
    amendments = transition["amendments"]
    assert isinstance(clauses, dict)
    assert isinstance(amendments, dict)
    for clause in ("I.11", "II.5", "II.8"):
        assert clauses[clause]["disposition"] == "Rewritten"
    for amendment in ("V", "W"):
        assert amendments[amendment]["disposition"] == "Rewritten"


def test_every_transition_row_is_a_complete_auditable_record() -> None:
    """No mapping family permits blank authority or unexplained disposition."""
    for row in _mapped_rows():
        assert set(row) == {"disposition", "destination", "rationale"}
        assert row["disposition"] in _DISPOSITIONS
        assert row["destination"].strip()
        assert row["rationale"].strip()


def test_every_destination_resolves_to_an_article_path_or_transition_anchor() -> None:
    """Every disposition names resolvable authority, not a vague category."""
    for row in _mapped_rows():
        references = tuple(part.strip() for part in row["destination"].split(";"))
        assert references
        for reference in references:
            if _ARTICLE_REFERENCE.fullmatch(reference):
                continue
            if _TRANSITION_REFERENCE.fullmatch(reference):
                continue
            assert (_ROOT / reference.partition(" §")[0]).is_file(), reference


def test_adr_is_catalogued_with_the_modern_record_shape() -> None:
    """The ADR filename, key, required fields, and index entry stay in lockstep."""
    entry = _adr_entry()
    required = {
        "status",
        "date",
        "title",
        "context",
        "decision",
        "transition",
        "consequences",
        "supersedes",
        "related",
    }
    assert required <= set(entry)
    assert entry["status"] == "accepted"
    index = yaml.safe_load(_INDEX_PATH.read_text(encoding="utf-8"))
    assert index["decisions"][_ADR_STEM] == {
        "title": entry["title"],
        "status": "accepted",
        "date": "2026-08-22",
        "file": f"{_ADR_STEM}.yaml",
    }
