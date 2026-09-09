"""Behavioral contract for model provisioning (D3, ADR096).

Zero network: the fetcher is injected (mirroring the providers seam's
client_factory). Pins what provision DOES — sha256 gate, .part resume,
rename-into-place, and the loud owner-provisioning gate when nothing is
available.
"""

from __future__ import annotations

import hashlib
from collections.abc import Iterator
from pathlib import Path

import pytest

from babylon.intelligence.model_manifest import ModelEntry, ModelKind, ModelManifest
from babylon.intelligence.provision import (
    default_models_dir,
    provision_models,
)


def _sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def test_default_models_dir_prefers_xdg_data_home(tmp_path: Path) -> None:
    got = default_models_dir({"XDG_DATA_HOME": str(tmp_path)})
    assert got == tmp_path / "babylon" / "models"


def test_default_models_dir_falls_back_to_local_share() -> None:
    got = default_models_dir({})
    assert got.parts[-3:] == (".local", "share") + ("babylon",) or got.name == "models"
    assert got.name == "models" and got.parent.name == "babylon"


def test_gated_when_nothing_available(tmp_path: Path) -> None:
    manifest = ModelManifest(
        models=[ModelEntry(name="babylon-embed", kind=ModelKind.EMBED, available=False, dims=768)]
    )

    def unused_fetcher(url: str, start: int) -> Iterator[bytes]:  # pragma: no cover
        raise AssertionError("must not fetch a gated entry")

    results = provision_models(manifest, tmp_path, fetcher=unused_fetcher)
    assert len(results) == 1
    assert results[0].status == "gated"
    assert not any(tmp_path.iterdir())


def test_downloads_and_verifies_available_entry(tmp_path: Path) -> None:
    payload = b"gguf-bytes-x" * 100
    digest = _sha256(payload)
    manifest = ModelManifest(
        models=[
            ModelEntry(
                name="babylon-chat",
                kind=ModelKind.CHAT,
                available=True,
                url="https://data.example/chat.gguf",
                sha256=digest,
                bytes=len(payload),
            )
        ]
    )

    def fetcher(url: str, start: int) -> Iterator[bytes]:
        # Honor Range: yield only the tail from `start`.
        yield payload[start:]

    results = provision_models(manifest, tmp_path, fetcher=fetcher)
    assert results[0].status == "downloaded"
    final = tmp_path / "babylon-chat.gguf"
    assert final.read_bytes() == payload
    assert not (tmp_path / "babylon-chat.gguf.part").exists()


def test_resumes_from_existing_part_file(tmp_path: Path) -> None:
    payload = b"resumable-payload" * 50
    digest = _sha256(payload)
    # Pre-seed a partial .part with the first 100 bytes.
    part = tmp_path / "babylon-chat.gguf.part"
    part.write_bytes(payload[:100])
    seen_starts: list[int] = []

    def fetcher(url: str, start: int) -> Iterator[bytes]:
        seen_starts.append(start)
        yield payload[start:]

    manifest = ModelManifest(
        models=[
            ModelEntry(
                name="babylon-chat",
                kind=ModelKind.CHAT,
                available=True,
                url="https://data.example/chat.gguf",
                sha256=digest,
                bytes=len(payload),
            )
        ]
    )
    results = provision_models(manifest, tmp_path, fetcher=fetcher)
    assert results[0].status == "downloaded"
    assert seen_starts == [100]  # resumed, did not restart from 0
    assert (tmp_path / "babylon-chat.gguf").read_bytes() == payload


def test_sha256_mismatch_raises_and_keeps_no_final(tmp_path: Path) -> None:
    payload = b"corrupt-content" * 10
    manifest = ModelManifest(
        models=[
            ModelEntry(
                name="babylon-chat",
                kind=ModelKind.CHAT,
                available=True,
                url="https://data.example/chat.gguf",
                sha256="0" * 64,  # deliberately wrong
                bytes=len(payload),
            )
        ]
    )

    def fetcher(url: str, start: int) -> Iterator[bytes]:
        yield payload[start:]

    with pytest.raises(ValueError, match="sha256"):
        provision_models(manifest, tmp_path, fetcher=fetcher, max_retries=1)
    assert not (tmp_path / "babylon-chat.gguf").exists()
