"""Behavioral contract for the bundled model manifest (D3, ADR096).

The manifest ships in the package (data/model_manifest.toml). Entries are
owner-provisioned: until the owner uploads
weights to R2 and flips ``available = true`` with real url/sha256/bytes, no
entry is available and provision is a loud no-op.
"""

from __future__ import annotations

import pytest
from pydantic import ValidationError

from babylon.intelligence.model_manifest import (
    ModelEntry,
    ModelKind,
    ModelManifest,
    load_bundled_manifest,
)


def test_bundled_manifest_loads_and_parses() -> None:
    manifest = load_bundled_manifest()
    assert isinstance(manifest, ModelManifest)
    # Ships one chat + one embed entry (owner-provisioned placeholders).
    assert len(manifest.chat_entries()) >= 1
    assert len(manifest.embed_entries()) >= 1


def test_bundled_entries_are_owner_provisioned_not_yet_available() -> None:
    # Weights not uploaded to R2 yet (verified 2026-07-20) → nothing available.
    manifest = load_bundled_manifest()
    assert manifest.available_entries() == []


def test_available_entry_requires_url_sha256_bytes() -> None:
    with pytest.raises(ValidationError):
        ModelEntry(name="chat", kind=ModelKind.CHAT, available=True)  # missing fields


def test_available_entry_with_full_fields_validates() -> None:
    entry = ModelEntry(
        name="chat",
        kind=ModelKind.CHAT,
        available=True,
        url="https://data.example/chat.gguf",
        sha256="a" * 64,
        bytes=123,
    )
    assert entry.available and entry.url is not None


def test_embed_entry_requires_dims() -> None:
    with pytest.raises(ValidationError):
        ModelEntry(name="embed", kind=ModelKind.EMBED, available=False)  # dims missing


def test_unavailable_chat_entry_allows_empty_source() -> None:
    entry = ModelEntry(name="chat", kind=ModelKind.CHAT, available=False)
    assert entry.url is None and not entry.available
