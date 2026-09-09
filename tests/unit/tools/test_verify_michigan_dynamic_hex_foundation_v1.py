"""Independent verifier laws for Michigan Dynamic-Hex Foundation V1."""

from __future__ import annotations

from copy import deepcopy
from pathlib import Path

import pytest
import tools.verify_michigan_dynamic_hex_foundation_v1 as foundation_verifier
import yaml
from tools.verify_michigan_dynamic_hex_foundation_v1 import (
    verify_foundation_artifact,
    verify_michigan_dynamic_hex_foundation_v1,
)

CONTRACT = Path("contracts/michigan_dynamic_hex_foundation_v1.yaml")


def _contract() -> dict[str, object]:
    loaded = yaml.safe_load(CONTRACT.read_text(encoding="utf-8"))
    assert isinstance(loaded, dict)
    return loaded


def _artifact(contract: dict[str, object]) -> tuple[bytes, tuple[int, ...]]:
    artifact = contract["artifact"]
    assert isinstance(artifact, dict)
    raw_parts = artifact["fixture_parts"]
    assert isinstance(raw_parts, list)
    parts = tuple(Path(str(path)).read_bytes() for path in raw_parts)
    return b"".join(parts), tuple(map(len, parts))


def test_canonical_composite_foundation_verifies_without_builder_authority() -> None:
    assert verify_michigan_dynamic_hex_foundation_v1(CONTRACT) == ()
    source = Path("tools/verify_michigan_dynamic_hex_foundation_v1.py").read_text(encoding="utf-8")
    assert "build_michigan_dynamic_hex_foundation_v1" not in source
    assert "rust/crates" not in source


def test_parent_mutation_exposes_semantic_coverage_and_digest_findings() -> None:
    contract = _contract()
    artifact, part_sizes = _artifact(contract)
    wire = contract["wire"]
    dynamic = contract["dynamic_r7"]
    assert isinstance(wire, dict)
    assert isinstance(dynamic, dict)
    domain = str(wire["foundation_domain_utf8"]).encode()
    dynamic_count = int(dynamic["row_count"])
    r8_domain = str(wire["r8_section_domain_utf8"]).encode()
    first_parent_offset = len(domain) + 4 + 32 * 4 + 8 + dynamic_count * 80 + len(r8_domain) + 8 + 8
    mutated = bytearray(artifact)
    mutated[first_parent_offset : first_parent_offset + 8] = int("872664801ffffff", 16).to_bytes(
        8, "big"
    )

    codes = {
        finding.code for finding in verify_foundation_artifact(contract, bytes(mutated), part_sizes)
    }

    assert {"artifact_digest", "r8_parent", "r8_coverage", "r8_digest"} <= codes


def test_manifest_digest_mutation_cannot_redefine_composite_authority() -> None:
    contract = deepcopy(_contract())
    artifact, part_sizes = _artifact(contract)
    reference = contract["reference_bundle"]
    assert isinstance(reference, dict)
    reference["composite_digest"] = "01" * 32

    codes = {finding.code for finding in verify_foundation_artifact(contract, artifact, part_sizes)}

    assert "reference_bundle_digest" in codes


def test_h3_exceptions_are_normalized_to_stable_semantic_findings(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    contract = _contract()
    artifact, part_sizes = _artifact(contract)

    def refuse_h3(*_args: object) -> object:
        raise ValueError("synthetic H3 refusal")

    monkeypatch.setattr(foundation_verifier.h3, "get_resolution", refuse_h3)
    monkeypatch.setattr(foundation_verifier.h3, "cell_to_children", refuse_h3)

    codes = {finding.code for finding in verify_foundation_artifact(contract, artifact, part_sizes)}

    assert {"dynamic_identity", "r8_parent", "r8_coverage"} <= codes


def test_malformed_audited_identity_is_a_stable_contract_finding() -> None:
    contract = deepcopy(_contract())
    artifact, part_sizes = _artifact(contract)
    audited = contract["audited_identities"]
    assert isinstance(audited, dict)
    audited["r8_child"] = "not-an-h3-cell"

    codes = {finding.code for finding in verify_foundation_artifact(contract, artifact, part_sizes)}

    assert codes == {"invalid_contract"}
