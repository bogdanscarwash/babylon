"""Behavioral contract for the corpus manifest (ADR107, T5 U3).

Replaces the hardcoded ``MVP_CORPUS`` tuple in ``tools/ingest_corpus.py``
with a declarative manifest (mirrors
:mod:`babylon.intelligence.model_manifest`'s validation style). Unit tests
never touch the real ``~/Documents/ocr/`` tree — every glob-resolution test
below builds its own tmp fixture tree.
"""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Final

import pytest
from pydantic import ValidationError

from babylon.intelligence.corpus_manifest import (
    APOCRYPHA_DIR_NAME,
    CanonStatus,
    CorpusFormat,
    CorpusManifest,
    CorpusRole,
    CorpusRow,
    ExclusionPolicy,
    load_bundled_manifest,
    load_manifest,
    parse_manifest,
)

pytestmark = pytest.mark.unit

_EXPECTED_DIRECTOR_POLICY_SHA256: Final[str] = (
    "2ddfbd127723ea60c6e6cdb993763a3dd02b05dd479e8ddc4c850e5c94a3e243"
)


def _row(**overrides: object) -> dict[str, object]:
    base: dict[str, object] = {
        "path_glob": "author-x/work-y/*.txt",
        "author": "Author X",
        "work": "Work Y",
        "role": ["narrator"],
        "format": "txt",
        "canon_status": "allow",
        "provenance": "test fixture row",
    }
    base.update(overrides)
    return base


def _director_policy_digest(rows: tuple[CorpusRow, CorpusRow, CorpusRow]) -> str:
    """Hash the closed row set through one documented canonical serialization.

    Each row is Pydantic JSON-mode data encoded as compact UTF-8 JSON with
    lexicographically sorted object keys. The three complete row objects sort
    by their encoded bytes, then receive JSON array framing and a versioned
    domain prefix before SHA-256.
    """
    encoded_rows = tuple(
        json.dumps(
            row.model_dump(mode="json"),
            ensure_ascii=False,
            separators=(",", ":"),
            sort_keys=True,
        ).encode("utf-8")
        for row in rows
    )
    canonical = (
        b"babylon.director-exclusion-policy.v1\x00[" + b",".join(sorted(encoded_rows)) + b"]"
    )
    return hashlib.sha256(canonical).hexdigest()


# =============================================================================
# 1. Loader validation — closed vocabularies red loudly (Do item 4)
# =============================================================================


class TestClosedVocabularies:
    def test_valid_row_parses(self) -> None:
        manifest = parse_manifest({"rows": [_row()]})
        assert len(manifest.rows) == 1
        row = manifest.rows[0]
        assert row.canon_status is CanonStatus.ALLOW
        assert row.role == (CorpusRole.NARRATOR,)
        assert row.format is CorpusFormat.TXT

    def test_bad_role_reds_loudly(self) -> None:
        with pytest.raises(ValidationError):
            parse_manifest({"rows": [_row(role=["not_a_real_role"])]})

    def test_bad_canon_status_reds_loudly(self) -> None:
        with pytest.raises(ValidationError):
            parse_manifest({"rows": [_row(canon_status="maybe")]})

    def test_bad_format_reds_loudly(self) -> None:
        with pytest.raises(ValidationError):
            parse_manifest({"rows": [_row(format="pdf")]})

    def test_empty_role_list_reds_loudly(self) -> None:
        with pytest.raises(ValidationError):
            parse_manifest({"rows": [_row(role=[])]})

    def test_multi_valued_role_accepted(self) -> None:
        manifest = parse_manifest({"rows": [_row(role=["narrator", "doctrine", "atlas_cn"])]})
        assert manifest.rows[0].role == (
            CorpusRole.NARRATOR,
            CorpusRole.DOCTRINE,
            CorpusRole.ATLAS_CN,
        )

    def test_director_exclusion_requires_deny_status(self) -> None:
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

    def test_manifest_rejects_unknown_top_level_and_row_fields(self) -> None:
        with pytest.raises(ValidationError, match="extra"):
            parse_manifest({"rows": [_row()], "rowz": []})
        with pytest.raises(ValidationError, match="extra"):
            parse_manifest({"rows": [_row(exclusion_polciy="director")]})

    def test_manifest_row_ceiling_is_loud(self) -> None:
        at_limit = tuple(_row(work=f"Work {index}") for index in range(4_096))
        assert len(parse_manifest({"rows": at_limit}).rows) == 4_096
        with pytest.raises(ValidationError, match="4,096"):
            parse_manifest({"rows": (*at_limit, _row(work="Over limit"))})

    def test_manifest_row_ceiling_is_loud_for_generator_input(self) -> None:
        at_limit = (_row(work=f"Work {index}") for index in range(4_096))
        assert len(parse_manifest({"rows": at_limit}).rows) == 4_096

        over_limit = (_row(work=f"Work {index}") for index in range(4_097))
        with pytest.raises(ValidationError, match="4,096"):
            parse_manifest({"rows": over_limit})

    def test_director_excluded_rows_are_typed_and_exact(self) -> None:
        manifest = load_bundled_manifest()
        rows = manifest.director_excluded_rows()
        expected_fields = (
            "path_glob",
            "author",
            "work",
            "role",
            "format",
            "canon_status",
            "exclusion_policy",
            "provenance",
        )
        expected_provenance = (
            "Director exclusion ruling, 2026-08-23. This row exists solely to prevent ingestion."
        )
        if len(rows) != 3:
            pytest.fail("Director exclusion row count mismatch", pytrace=False)
        first, second, third = rows
        exact_rows = (first, second, third)
        envelope_matches = all(
            (
                row.role == (CorpusRole.DOCTRINE,)
                and row.format is CorpusFormat.TXT
                and row.canon_status is CanonStatus.DENY
                and row.exclusion_policy is ExclusionPolicy.DIRECTOR
                and row.provenance == expected_provenance
            )
            for row in exact_rows
        )
        if tuple(CorpusRow.model_fields) != expected_fields or not envelope_matches:
            pytest.fail("Director exclusion governed envelope mismatch", pytrace=False)
        if len({row.author.casefold() for row in exact_rows}) != 3:
            pytest.fail("Director exclusion author uniqueness mismatch", pytrace=False)
        if len({row.path_glob for row in exact_rows}) != 3:
            pytest.fail("Director exclusion path uniqueness mismatch", pytrace=False)
        if _director_policy_digest(exact_rows) != _EXPECTED_DIRECTOR_POLICY_SHA256:
            pytest.fail("Director exclusion canonical digest mismatch", pytrace=False)


# =============================================================================
# 2. Apocrypha glob-fencing (ADR107) — both directions
# =============================================================================


class TestApocryphaFencing:
    def test_apocryphal_row_must_resolve_into_apocrypha_dir(self) -> None:
        with pytest.raises(ValidationError, match="apocryphal"):
            parse_manifest(
                {"rows": [_row(path_glob="author-x/work-y/*.txt", canon_status="apocryphal")]}
            )

    def test_non_apocryphal_row_forbidden_inside_apocrypha_dir(self) -> None:
        with pytest.raises(ValidationError, match=APOCRYPHA_DIR_NAME):
            parse_manifest(
                {
                    "rows": [
                        _row(path_glob=f"{APOCRYPHA_DIR_NAME}/content.jsonl", canon_status="allow")
                    ]
                }
            )

    def test_apocryphal_row_pointing_into_apocrypha_dir_is_valid(self) -> None:
        manifest = parse_manifest(
            {
                "rows": [
                    _row(
                        path_glob=f"{APOCRYPHA_DIR_NAME}/content.jsonl",
                        canon_status="apocryphal",
                        format="jsonl",
                    )
                ]
            }
        )
        assert manifest.rows[0].canon_status is CanonStatus.APOCRYPHAL


# =============================================================================
# 3. Deny-inside-allow precedence, presence, and deterministic ordering
#    (Do item 4) — every test below builds its own tmp fixture tree.
# =============================================================================


class TestDenyInsideAllowPrecedence:
    def test_deny_row_wins_inside_an_enclosing_allow_glob(self, tmp_path: Path) -> None:
        # One broad allow glob sweeps over several authors' subdirectories;
        # a nested denied-source row must still exclude its files.
        approved_dir = tmp_path / "classics" / "approved"
        denied_dir = tmp_path / "classics" / "denied-author"
        approved_dir.mkdir(parents=True)
        denied_dir.mkdir(parents=True)
        approved_file = approved_dir / "approved.txt"
        denied_file = denied_dir / "denied.txt"
        approved_file.write_text("approved source", encoding="utf-8")
        denied_file.write_text("denied source", encoding="utf-8")

        manifest = parse_manifest(
            {
                "rows": [
                    _row(path_glob="classics/**/*.txt", canon_status="allow"),
                    _row(
                        path_glob="classics/denied-author/**/*.txt",
                        author="Denied Author",
                        work="Denied Work",
                        canon_status="deny",
                    ),
                ]
            }
        )

        resolved = manifest.resolve_ingestible_files(tmp_path)
        assert approved_file in resolved
        assert denied_file not in resolved

    def test_flag_bd_row_never_appears_in_ingestible_files(self, tmp_path: Path) -> None:
        work_dir = tmp_path / "nitzan-bichler" / "capital-as-power"
        work_dir.mkdir(parents=True)
        flagged_file = work_dir / "full.txt"
        flagged_file.write_text("adversarial-only steelman text")

        manifest = parse_manifest(
            {
                "rows": [
                    _row(path_glob="nitzan-bichler/capital-as-power/*.txt", canon_status="flag_bd")
                ]
            }
        )

        assert manifest.resolve_ingestible_files(tmp_path) == ()

    def test_apocryphal_row_never_appears_in_ingestible_files(self, tmp_path: Path) -> None:
        (tmp_path / APOCRYPHA_DIR_NAME).mkdir(parents=True)
        apocrypha_file = tmp_path / APOCRYPHA_DIR_NAME / "content.jsonl"
        apocrypha_file.write_text('{"text": "pastiche"}\n')

        manifest = parse_manifest(
            {
                "rows": [
                    _row(
                        path_glob=f"{APOCRYPHA_DIR_NAME}/content.jsonl",
                        canon_status="apocryphal",
                        format="jsonl",
                    )
                ]
            }
        )

        assert manifest.resolve_ingestible_files(tmp_path) == ()


class TestBroadAllowGlobDoesNotLeakNonAllowFiles:
    """Reviewer-reproduced regression: a broad allow glob whose literal
    string never mentions a non-allow row's directory still RESOLVES into it
    at runtime. ``ingest_targets`` must exclude those files by set difference
    over ALL non-allow rows (not deny rows alone) — a narrow per-row glob
    check on the string is not enough.
    """

    def test_broad_allow_glob_does_not_leak_apocrypha_files(self, tmp_path: Path) -> None:
        (tmp_path / APOCRYPHA_DIR_NAME).mkdir(parents=True)
        apocrypha_file = tmp_path / APOCRYPHA_DIR_NAME / "content.jsonl"
        apocrypha_file.write_text('{"text": "pastiche"}\n')
        real_file = tmp_path / "real.jsonl"
        real_file.write_text('{"text": "canon"}\n')

        manifest = parse_manifest(
            {
                "rows": [
                    _row(path_glob="**/*.jsonl", format="jsonl", canon_status="allow"),
                    _row(
                        path_glob=f"{APOCRYPHA_DIR_NAME}/content.jsonl",
                        canon_status="apocryphal",
                        format="jsonl",
                    ),
                ]
            }
        )

        resolved = manifest.resolve_ingestible_files(tmp_path)
        assert real_file in resolved
        assert apocrypha_file not in resolved

    def test_broad_allow_glob_does_not_leak_flag_bd_files(self, tmp_path: Path) -> None:
        work_dir = tmp_path / "nitzan-bichler" / "capital-as-power"
        work_dir.mkdir(parents=True)
        flagged_file = work_dir / "full.txt"
        flagged_file.write_text("adversarial-only steelman text")
        real_dir = tmp_path / "zak-cope" / "divided-world-divided-class"
        real_dir.mkdir(parents=True)
        real_file = real_dir / "full.txt"
        real_file.write_text("canon body")

        manifest = parse_manifest(
            {
                "rows": [
                    _row(path_glob="**/*.txt", canon_status="allow"),
                    _row(
                        path_glob="nitzan-bichler/capital-as-power/*.txt",
                        canon_status="flag_bd",
                    ),
                ]
            }
        )

        resolved = manifest.resolve_ingestible_files(tmp_path)
        assert real_file in resolved
        assert flagged_file not in resolved

    def test_apocrypha_dir_excluded_structurally_even_without_a_matching_row(
        self, tmp_path: Path
    ) -> None:
        # The independent structural fence (_under_apocrypha): even with NO
        # row declaring canon_status=apocryphal for this exact path (so
        # nothing exists in the per-row exclusion set for it), a resolved
        # file under _apocrypha/ must still never surface.
        (tmp_path / APOCRYPHA_DIR_NAME).mkdir(parents=True)
        apocrypha_file = tmp_path / APOCRYPHA_DIR_NAME / "stray.txt"
        apocrypha_file.write_text("should never surface")
        real_file = tmp_path / "canon.txt"
        real_file.write_text("canon body")

        manifest = parse_manifest({"rows": [_row(path_glob="**/*.txt", canon_status="allow")]})

        resolved = manifest.resolve_ingestible_files(tmp_path)
        assert real_file in resolved
        assert apocrypha_file not in resolved


class TestPresenceReporting:
    def test_absent_row_reports_empty_files_not_an_error(self, tmp_path: Path) -> None:
        # corpus_root exists but this row's work has not been extracted yet —
        # a MANIFEST fact, never an exception.
        manifest = parse_manifest(
            {
                "rows": [
                    _row(path_glob="fanon/the-wretched-of-the-earth/*.txt", canon_status="allow")
                ]
            }
        )

        targets = manifest.ingest_targets(tmp_path)
        assert len(targets) == 1
        assert targets[0].files == ()
        assert targets[0].present is False

    def test_entirely_absent_corpus_root_reports_empty_without_error(self, tmp_path: Path) -> None:
        missing_root = tmp_path / "does-not-exist"
        manifest = parse_manifest(
            {
                "rows": [
                    _row(path_glob="fanon/the-wretched-of-the-earth/*.txt", canon_status="allow")
                ]
            }
        )

        targets = manifest.ingest_targets(missing_root)
        assert targets[0].files == ()

    def test_present_row_reports_matched_files(self, tmp_path: Path) -> None:
        work_dir = tmp_path / "author-x" / "work-y"
        work_dir.mkdir(parents=True)
        (work_dir / "full.txt").write_text("body")

        manifest = parse_manifest({"rows": [_row()]})
        targets = manifest.ingest_targets(tmp_path)
        assert targets[0].present is True
        assert len(targets[0].files) == 1


class TestDeterministicOrdering:
    def test_files_within_a_row_are_sorted_regardless_of_creation_order(
        self, tmp_path: Path
    ) -> None:
        work_dir = tmp_path / "mao" / "selected-works-curated"
        work_dir.mkdir(parents=True)
        # Write chapters out of order on purpose.
        (work_dir / "ch-03.txt").write_text("three")
        (work_dir / "ch-01.txt").write_text("one")
        (work_dir / "ch-02.txt").write_text("two")

        manifest = parse_manifest(
            {"rows": [_row(path_glob="mao/selected-works-curated/*.txt", canon_status="allow")]}
        )

        files = manifest.ingest_targets(tmp_path)[0].files
        assert [f.name for f in files] == ["ch-01.txt", "ch-02.txt", "ch-03.txt"]

    def test_rows_are_returned_in_manifest_declaration_order(self, tmp_path: Path) -> None:
        manifest = parse_manifest(
            {
                "rows": [
                    _row(path_glob="zebra/work/*.txt", work="Zebra Work", canon_status="allow"),
                    _row(path_glob="apple/work/*.txt", work="Apple Work", canon_status="allow"),
                ]
            }
        )

        targets = manifest.ingest_targets(tmp_path)
        assert [t.row.work for t in targets] == ["Zebra Work", "Apple Work"]


# =============================================================================
# 4. load_manifest — the test-injectable file loader (never the real tree)
# =============================================================================


class TestLoadManifestFromPath:
    def test_load_manifest_reads_a_yaml_fixture(self, tmp_path: Path) -> None:
        manifest_path = tmp_path / "manifest.yaml"
        manifest_path.write_text(
            "rows:\n"
            "  - path_glob: 'author-x/work-y/*.txt'\n"
            "    author: 'Author X'\n"
            "    work: 'Work Y'\n"
            "    role: [narrator]\n"
            "    format: txt\n"
            "    canon_status: allow\n"
            "    provenance: 'fixture'\n"
        )

        manifest = load_manifest(manifest_path)
        assert isinstance(manifest, CorpusManifest)
        assert len(manifest.rows) == 1
        assert manifest.rows[0].author == "Author X"


# =============================================================================
# 5. The bundled production manifest — sanity checks against the real file
#    this unit ships (src/babylon/data/corpus/manifest.yaml). Still no I/O
#    against ~/Documents/ocr/ — glob resolution against the real corpus root
#    is exercised only by the presence tests above, on tmp fixtures.
# =============================================================================


class TestBundledManifest:
    def test_bundled_manifest_loads_and_validates(self) -> None:
        manifest = load_bundled_manifest()
        assert isinstance(manifest, CorpusManifest)
        assert len(manifest.rows) > 0

    def test_bundled_manifest_has_the_v1_nine_work_allow_set(self) -> None:
        manifest = load_bundled_manifest()
        assert len(manifest.allow_rows()) == 9

    def test_bundled_manifest_preserves_non_director_deny_rows(self) -> None:
        manifest = load_bundled_manifest()
        denied_authors = {
            row.author
            for row in manifest.deny_rows()
            if row.exclusion_policy is ExclusionPolicy.NONE
        }
        assert denied_authors == {
            "Karl Kautsky",
            "Communist Party USA",
            "Enver Hoxha",
        }

    def test_bundled_manifest_has_exactly_one_apocryphal_row_fenced(self) -> None:
        manifest = load_bundled_manifest()
        apocryphal = manifest.apocryphal_rows()
        assert len(apocryphal) == 1
        assert APOCRYPHA_DIR_NAME in Path(apocryphal[0].path_glob).parts

    def test_bundled_manifest_has_a_flag_bd_row(self) -> None:
        manifest = load_bundled_manifest()
        assert len(manifest.flag_bd_rows()) >= 1

    def test_bundled_manifest_no_row_glob_resolves_into_apocrypha_unless_apocryphal(
        self,
    ) -> None:
        manifest = load_bundled_manifest()
        for row in manifest.rows:
            resolves_into_apocrypha = APOCRYPHA_DIR_NAME in Path(row.path_glob).parts
            assert resolves_into_apocrypha == (row.canon_status is CanonStatus.APOCRYPHAL)
