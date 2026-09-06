"""Keep the independent V4 wire fixtures reproducible and read-only by default."""

from __future__ import annotations

import sys
from pathlib import Path

import pytest
from tools import generate_production_evidence_v4_vectors as vectors


def test_registered_vectors_equal_independent_generator() -> None:
    assert vectors.DESTINATION.read_bytes() == vectors.fixture_bytes()


def test_normal_verification_refuses_drift_without_rewriting(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    path = tmp_path / "changed.json"
    original = b'{"deliberate": "drift"}\n'
    path.write_bytes(original)
    monkeypatch.setattr(sys, "argv", ["vectors", "--output", str(path)])
    with pytest.raises(SystemExit, match="V4 wire fixture differs"):
        vectors.main()
    assert path.read_bytes() == original
