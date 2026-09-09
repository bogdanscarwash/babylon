"""Behavioral contract for tools/ingest_corpus.py's manifest-driven rewire (T5 U3).

The retired ``MVP_CORPUS`` tuple (fuzzy filename matching against an
arbitrary external mirror) is replaced entirely by
:class:`babylon.intelligence.corpus_manifest.CorpusManifest`-driven
enumeration (allow minus deny, existing-files-only, deterministic
ordering). Nothing here touches the real ``~/Documents/ocr/`` tree, calls
an embedding model, or writes to a database — this unit ships file
preparation only.
"""

from __future__ import annotations

from pathlib import Path

import pytest
from tools.ingest_corpus import (
    _dest_filename,
    import_corpus_files,
    import_manifest_work,
)

from babylon.intelligence.corpus_manifest import parse_manifest

pytestmark = pytest.mark.unit


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


class TestNoMvpCorpusSymbolsRemain:
    """Regression sentinel: the old hardcoded corpus must not creep back."""

    def test_mvp_corpus_tuple_is_gone(self) -> None:
        import tools.ingest_corpus as module

        assert not hasattr(module, "MVP_CORPUS")
        assert not hasattr(module, "CorpusText")
        assert not hasattr(module, "fuzzy_match_score")


class TestManifestDrivenEnumeration:
    def test_allow_minus_deny_existing_files_only(self, tmp_path: Path) -> None:
        corpus_root = tmp_path / "ocr"
        corpus_dir = tmp_path / "prepared"

        # Present allow row.
        present_dir = corpus_root / "zak-cope" / "divided-world-divided-class"
        present_dir.mkdir(parents=True)
        (present_dir / "full.txt").write_text("labor aristocracy theory body")

        # Allow row whose work has NOT been extracted onto this box yet.
        absent_row = _row(
            path_glob="fanon/the-wretched-of-the-earth/*.txt",
            author="Frantz Fanon",
            work="The Wretched of the Earth",
            canon_status="allow",
        )

        # A broad allow glob that would sweep in a nested denied source if
        # the deny row were not honored.
        approved_dir = corpus_root / "classics" / "approved"
        denied_dir = corpus_root / "classics" / "denied-author"
        approved_dir.mkdir(parents=True)
        denied_dir.mkdir(parents=True)
        approved_file = approved_dir / "approved.txt"
        denied_file = denied_dir / "denied.txt"
        approved_file.write_text("approved source", encoding="utf-8")
        denied_file.write_text("denied source", encoding="utf-8")

        manifest = parse_manifest(
            {
                "rows": [
                    _row(
                        path_glob="zak-cope/divided-world-divided-class/*.txt",
                        author="Zak Cope",
                        work="Divided World, Divided Class",
                        canon_status="allow",
                    ),
                    absent_row,
                    _row(
                        path_glob="classics/**/*.txt",
                        author="Approved Author",
                        work="Classics",
                        canon_status="allow",
                    ),
                    _row(
                        path_glob="classics/denied-author/**/*.txt",
                        author="Denied Author",
                        work="Denied Work",
                        canon_status="deny",
                    ),
                ]
            }
        )

        imported, missing = import_corpus_files(corpus_root, corpus_dir, manifest)

        assert imported == 2  # Cope plus allowed Classics file; Fanon absent.
        assert missing == ["The Wretched of the Earth (Frantz Fanon)"]

        cope_dest = corpus_dir / _dest_filename(manifest.allow_rows()[0])
        assert cope_dest.exists()
        assert "labor aristocracy theory body" in cope_dest.read_text()

        classics_dest = corpus_dir / _dest_filename(manifest.allow_rows()[2])
        classics_text = classics_dest.read_text()
        assert "approved source" in classics_text
        assert "denied source" not in classics_text

        # No file was ever prepared for the absent Fanon row.
        fanon_dest = corpus_dir / _dest_filename(manifest.allow_rows()[1])
        assert not fanon_dest.exists()

    def test_absent_row_reported_missing_not_raised(self, tmp_path: Path) -> None:
        corpus_root = tmp_path / "ocr"  # never created — entirely absent tree
        corpus_dir = tmp_path / "prepared"
        manifest = parse_manifest(
            {"rows": [_row(path_glob="stalin/foundations-of-leninism/*.txt", canon_status="allow")]}
        )

        imported, missing = import_corpus_files(corpus_root, corpus_dir, manifest)

        assert imported == 0
        assert len(missing) == 1

    def test_deterministic_dest_filename(self) -> None:
        manifest = parse_manifest(
            {"rows": [_row(author="V.I. Lenin", work="State and Revolution")]}
        )
        assert _dest_filename(manifest.rows[0]) == "vi_lenin_state_and_revolution.md"


class TestImportManifestWork:
    def test_txt_format_is_read_verbatim(self, tmp_path: Path) -> None:
        f = tmp_path / "full.txt"
        f.write_text("plain extracted body")
        manifest = parse_manifest({"rows": [_row(format="txt")]})
        row = manifest.rows[0]
        dest = tmp_path / "out.md"

        assert import_manifest_work((f,), row, dest) is True
        content = dest.read_text()
        assert "plain extracted body" in content
        assert f"# {row.work}" in content
        assert f"**Author:** {row.author}" in content

    def test_html_format_is_converted_via_markdownify(self, tmp_path: Path) -> None:
        f = tmp_path / "ch01.htm"
        f.write_text("<html><body><h1>Title</h1><p>Body text</p></body></html>")
        manifest = parse_manifest({"rows": [_row(format="html")]})
        row = manifest.rows[0]
        dest = tmp_path / "out.md"

        assert import_manifest_work((f,), row, dest) is True
        content = dest.read_text()
        assert "Body text" in content
        assert "<html>" not in content

    def test_multiple_files_joined_in_given_order(self, tmp_path: Path) -> None:
        f1 = tmp_path / "ch-01.txt"
        f1.write_text("chapter one")
        f2 = tmp_path / "ch-02.txt"
        f2.write_text("chapter two")
        manifest = parse_manifest({"rows": [_row(format="txt")]})
        row = manifest.rows[0]
        dest = tmp_path / "out.md"

        assert import_manifest_work((f1, f2), row, dest) is True
        content = dest.read_text()
        assert content.index("chapter one") < content.index("chapter two")

    def test_empty_files_tuple_returns_false(self, tmp_path: Path) -> None:
        manifest = parse_manifest({"rows": [_row()]})
        row = manifest.rows[0]
        dest = tmp_path / "out.md"

        assert import_manifest_work((), row, dest) is False
        assert not dest.exists()
