"""Behavioral contract for the repo-hygiene gate (Program 14, Phase 0.5).

The gate (``tools/check_repo_hygiene.py``) enforces three invariants, loudly
(Constitution III.11):

a. every *tracked* top-level entry is on the fixed allowlist;
b. no tracked file matches the ignore rules (``.gitkeep`` convention exempt);
c. no tracked blob at HEAD exceeds 1 MiB unless it is an LFS pointer
   (pointers are ~130 bytes, so blob size alone distinguishes them) or a
   named exemption.

The synthetic-input tests are the red-phase/mutation proof that each check
actually detects its violation class; the integration test pins the repo's
own cleanliness from Phase 0 onward.
"""

from __future__ import annotations

import sys
import tomllib
from pathlib import Path

import pytest

# Mirror the import path used by tools/*.py and its existing unit tests
# (see tests/unit/tools/test_dense_goldens.py).
TOOLS_DIR = Path(__file__).resolve().parents[3] / "tools"
DENY_TOML = Path("rust/deny.toml")
LIVE_HYPERGRAPH_MANIFEST = Path("rust/crates/babylon-graph/Cargo.toml")
OBSOLETE_HYPERGRAPH_MANIFEST = "babylon-tui/Cargo.toml"
sys.path.insert(0, str(TOOLS_DIR))

from check_repo_hygiene import (  # type: ignore[import-not-found]  # noqa: E402
    ALLOWED_TOP_LEVEL_DIRS,
    check_large_non_lfs_blobs,
    check_top_level_allowlist,
    check_tracked_but_ignored,
    main,
)


def _sources_section(deny_toml: str) -> str:
    """Return the source-allowlist portion of a cargo-deny configuration."""
    _, separator, sources = deny_toml.partition("[sources]")
    if not separator:
        raise AssertionError("rust/deny.toml: missing [sources] section")
    return sources


@pytest.mark.unit
class TestSyntheticViolations:
    """Each check must catch a planted violation (detection proof)."""

    def test_allowlist_violation_detected(self) -> None:
        tracked = ["src/babylon/__init__.py", "evil-dir/payload.py", "README.md"]
        violations = check_top_level_allowlist(tracked)
        assert violations == ["evil-dir"]

    def test_allowlist_rejects_unknown_top_level_file(self) -> None:
        tracked = ["stray-notes.txt", "src/babylon/__init__.py"]
        violations = check_top_level_allowlist(tracked)
        assert violations == ["stray-notes.txt"]

    def test_allowlist_clean_input_passes(self) -> None:
        tracked = ["src/a.py", "tests/b.py", "README.md", "pyproject.toml"]
        assert check_top_level_allowlist(tracked) == []

    def test_allowlist_admits_only_the_canonical_mise_lock(self) -> None:
        tracked = ["mise.lock", "mise.lock.bak", "unapproved.lock"]
        assert check_top_level_allowlist(tracked) == ["mise.lock.bak", "unapproved.lock"]

    def test_allowlist_admits_contracts_root_only(self) -> None:
        tracked = [
            "contracts/relational_territory_dossier_v1.yaml",
            "contractz/typo.yaml",
            "unapproved-root/payload.txt",
        ]
        assert "contracts" in ALLOWED_TOP_LEVEL_DIRS
        assert check_top_level_allowlist(tracked) == ["contractz", "unapproved-root"]

    def test_tracked_but_ignored_detected(self) -> None:
        ignored_tracked = ["reports/sim-runs/trace.csv"]
        violations = check_tracked_but_ignored(ignored_tracked)
        assert violations == ["reports/sim-runs/trace.csv"]

    def test_gitkeep_convention_exempt(self) -> None:
        ignored_tracked = ["results/.gitkeep", "logs/.gitkeep"]
        assert check_tracked_but_ignored(ignored_tracked) == []

    def test_large_blob_detected(self) -> None:
        # git ls-tree -r -l line shape: mode SP type SP oid SP size TAB path
        lines = [
            "100644 blob abc123                2097152\ttests/fat_fixture.json",
        ]
        violations = check_large_non_lfs_blobs(lines)
        assert violations == ["tests/fat_fixture.json (2097152 bytes)"]

    @pytest.mark.parametrize("theme", ["phi", "panopticon"])
    def test_embedded_themes_have_an_exact_two_mib_budget(self, theme: str) -> None:
        path = f"assets/music/babylon_theme_{theme}.ogg"
        boundary = 2_097_152
        assert check_large_non_lfs_blobs([f"100644 blob abc123 {boundary}\t{path}"]) == []
        assert check_large_non_lfs_blobs([f"100644 blob abc123 {boundary + 1}\t{path}"]) == [
            f"{path} ({boundary + 1} bytes)"
        ]

    def test_theme_budget_does_not_admit_other_music_or_near_names(self) -> None:
        paths = [
            "assets/music/babylon_theme_phi.ogg.bak",
            "assets/music/babylon_theme_other.ogg",
            "assets/music/unrelated.ogg",
            "other/assets/music/babylon_theme_phi.ogg",
        ]
        size = 1_048_577
        assert check_large_non_lfs_blobs(
            [f"100644 blob abc123 {size}\t{path}" for path in paths]
        ) == sorted(f"{path} ({size} bytes)" for path in paths)

    def test_lfs_pointer_passes(self) -> None:
        lines = [
            "100644 blob def456                    133\tsources/Capital-Volume-I.pdf",
        ]
        assert check_large_non_lfs_blobs(lines) == []

    def test_symlink_entries_are_skipped(self) -> None:
        # symlinks are mode 120000 with the target path as tiny blob content
        lines = ["120000 blob 0ddba11                    33\tdata/sqlite"]
        assert check_large_non_lfs_blobs(lines) == []


@pytest.mark.unit
class TestRepoIsClean:
    """The real repository satisfies the gate from Program 14 Phase 0 onward."""

    def test_gate_passes_on_this_repo(self) -> None:
        assert main() == 0


@pytest.mark.unit
class TestCargoDenyProvenance:
    """The Git-source allowlist identifies its live dependency manifest."""

    def test_hypergraph_allowlist_comment_names_its_live_manifest(self) -> None:
        """Avoid a stale manifest path in the hypergraph-rs allowlist rationale."""
        manifest = tomllib.loads(LIVE_HYPERGRAPH_MANIFEST.read_text(encoding="utf-8"))
        dependency = manifest["dependencies"]["hypergraph-rs"]
        assert dependency["git"] == "https://github.com/percy-raskova/hypergraph-rs.git"

        sources = _sources_section(DENY_TOML.read_text(encoding="utf-8"))
        assert str(LIVE_HYPERGRAPH_MANIFEST) in sources
        assert OBSOLETE_HYPERGRAPH_MANIFEST not in sources

    def test_missing_sources_section_reports_an_actionable_contract_break(self) -> None:
        """A renamed cargo-deny section must not leak an indexing exception."""
        with pytest.raises(
            AssertionError,
            match=r"^rust/deny\.toml: missing \[sources\] section$",
        ):
            _sources_section("[advisories]\n")
