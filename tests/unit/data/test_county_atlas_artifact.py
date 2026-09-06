"""Pins for the committed county atlas the Bevy client renders (B1 Phase A).

The atlas is a content-hashed binary derived from pinned TIGER 2024 geometry
plus the committed ``county_adjacency.json`` — these tests are the tripwire
that catches a hand-edited or half-regenerated artifact before the Rust reader
ever sees it. They read only the committed file: never the reference DB, never
``dist/data-artifacts``, never the data drive.

Regenerate with ``mise run data:county-atlas``; every number pinned here comes
from that tool's own report.
"""

from __future__ import annotations

import hashlib
import json
import math
import struct
from pathlib import Path

import pytest

from babylon.domain.geography.adjacency import ARTIFACT_PATH as ADJACENCY_PATH
from babylon.domain.geography.adjacency import load_adjacency_pairs

ATLAS_PATH = Path(__file__).resolve().parents[3] / "assets" / "map" / "county_atlas.bin"

MAGIC = b"BABCTY\0\x01"
HEADER_BYTES = 128
HEADER_STRUCT = "<8sII32sdddIIII32s8x"
COUNTY_RECORD_BYTES = 28
RING_RECORD_BYTES = 12

#: Every county with TIGER geometry. Moving this is a declared data change.
EXPECTED_COUNTY_COUNT = 3222

#: 2 x the 9,477 committed adjacency pairs, both directions, zero pairs
#: dropped — every FIPS in the adjacency artifact has a geometry row.
EXPECTED_CSR_NNZ = 2 * 9477

#: Rings and vertices at the 0.001 deg simplification the tool records. Two of
#: the 3,388 source rings are sub-pixel Denver/Jefferson exclaves that collapse
#: on the u16 grid; the rest survive.
EXPECTED_RING_COUNT = 3386
EXPECTED_VERTEX_COUNT = 360064


class Header:
    """The decoded 128-byte atlas header.

    :param raw: the artifact's first 128 bytes.
    """

    def __init__(self, raw: bytes) -> None:
        fields = struct.unpack(HEADER_STRUCT, raw)
        self.magic: bytes = fields[0]
        self.version: int = fields[1]
        self.flags: int = fields[2]
        self.content_hash: bytes = fields[3]
        self.origin_x: float = fields[4]
        self.origin_y: float = fields[5]
        self.scale: float = fields[6]
        self.county_count: int = fields[7]
        self.ring_count: int = fields[8]
        self.vertex_count: int = fields[9]
        self.csr_nnz: int = fields[10]
        self.source_hash: bytes = fields[11]


@pytest.fixture(scope="module")
def atlas_bytes() -> bytes:
    """The committed artifact's bytes.

    :returns: every byte of ``county_atlas.bin``.
    """
    assert ATLAS_PATH.is_file(), f"{ATLAS_PATH} is missing — run 'mise run data:county-atlas'"
    return ATLAS_PATH.read_bytes()


@pytest.fixture(scope="module")
def header(atlas_bytes: bytes) -> Header:
    """The decoded header.

    :param atlas_bytes: the committed artifact.
    :returns: the decoded 128-byte header.
    """
    return Header(atlas_bytes[:HEADER_BYTES])


@pytest.mark.unit
class TestArtifactIntegrity:
    def test_magic_and_version(self, header: Header) -> None:
        assert header.magic == MAGIC
        assert header.version == 1
        assert header.flags == 0

    def test_content_hash_covers_every_later_byte(self, atlas_bytes: bytes, header: Header) -> None:
        # The stamp covers every byte AFTER the content_hash field, which is
        # the rest of the header as well as the whole body.
        recomputed = hashlib.sha256(atlas_bytes[48:]).digest()
        assert recomputed == header.content_hash

    def test_a_tampered_byte_breaks_the_stamp(self, atlas_bytes: bytes, header: Header) -> None:
        # The tripwire only earns its place if it bites. Flip one vertex byte
        # deep in the body and the recomputation must disagree.
        tampered = bytearray(atlas_bytes)
        tampered[-1] ^= 0x01
        assert hashlib.sha256(bytes(tampered[48:])).digest() != header.content_hash

    def test_length_matches_the_header_counts_exactly(
        self, atlas_bytes: bytes, header: Header
    ) -> None:
        # No trailing bytes: a truncated or over-long file means a partial
        # write, and the Rust reader must never have to guess.
        name_offset = (
            HEADER_BYTES
            + header.county_count * COUNTY_RECORD_BYTES
            + header.ring_count * RING_RECORD_BYTES
            + header.vertex_count * 4
            + (header.county_count + 1) * 4
            + header.csr_nnz * 4
        )
        assert name_offset + 4 <= len(atlas_bytes)
        (name_length,) = struct.unpack_from("<I", atlas_bytes, name_offset)
        assert len(atlas_bytes) == name_offset + 4 + name_length


@pytest.mark.unit
class TestPinnedShape:
    def test_county_count(self, header: Header) -> None:
        assert header.county_count == EXPECTED_COUNTY_COUNT

    def test_ring_and_vertex_counts(self, header: Header) -> None:
        assert header.ring_count == EXPECTED_RING_COUNT
        assert header.vertex_count == EXPECTED_VERTEX_COUNT

    def test_csr_nnz_is_both_directions_of_every_pair(self, header: Header) -> None:
        assert header.csr_nnz == EXPECTED_CSR_NNZ
        assert header.csr_nnz == 2 * len(load_adjacency_pairs())

    def test_grid_transform_is_finite_and_positive(self, header: Header) -> None:
        assert header.scale > 0.0
        # One grid unit stays well under the 111 m simplification tolerance
        # the geometry rides on; the tool asserts the round-trip error too.
        assert header.scale < 111.0
        for value in (header.origin_x, header.origin_y):
            assert not math.isnan(value)
            assert abs(value) < 1e9


@pytest.mark.unit
class TestLineage:
    def test_source_hash_tracks_the_live_adjacency_artifact(self, header: Header) -> None:
        # The cross-artifact tripwire: regenerating county_adjacency.json
        # without regenerating the atlas reds this gate.
        stamped = json.loads(ADJACENCY_PATH.read_text())["content_hash"]
        assert header.source_hash.hex() == stamped

    def test_first_county_is_autauga(self, atlas_bytes: bytes, header: Header) -> None:
        # Counties sort by FIPS ascending, so index 0 is 01001. Phase B's
        # reader tests resolve this same county by name.
        fips = atlas_bytes[HEADER_BYTES : HEADER_BYTES + 5].decode("ascii")
        assert fips == "01001"
        names_start = (
            HEADER_BYTES
            + header.county_count * COUNTY_RECORD_BYTES
            + header.ring_count * RING_RECORD_BYTES
            + header.vertex_count * 4
            + (header.county_count + 1) * 4
            + header.csr_nnz * 4
            + 4
        )
        first_name = atlas_bytes[names_start:].split(b"\n", 1)[0].decode("utf-8")
        assert first_name == "Autauga County, AL"
