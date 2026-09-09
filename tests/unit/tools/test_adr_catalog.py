"""Behavioral contracts for the disposable ADR SQLite read model."""

from __future__ import annotations

import json
import sqlite3
import sys
from pathlib import Path

import pytest

TOOLS_DIR = Path(__file__).resolve().parents[3] / "tools"
sys.path.insert(0, str(TOOLS_DIR))

import adr_catalog  # type: ignore[import-not-found]  # noqa: E402
from adr_catalog import (  # type: ignore[import-not-found]  # noqa: E402
    MAX_TITLE_JSON_BYTES,
    CacheIntegrityError,
    DuplicateAdrError,
    SourceChangedError,
    SourceParseError,
    build_cache,
    capture_snapshot,
    check_catalog,
    main,
    search,
    show,
)

pytestmark = pytest.mark.unit


def _write(repo: Path, relative: str, content: str) -> None:
    target = repo / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(content, encoding="utf-8")


def _write_index(repo: Path, statuses: dict[str, str]) -> None:
    if len(statuses) > 32:
        raise ValueError("test index exceeds its fixed bound")
    lines = ["meta:", "  version: 1", "decisions:"]
    for key, status in list(statuses.items())[:32]:
        lines.extend(
            [
                f"  {key}:",
                f"    title: Index title for {key}",
                f"    status: {status}",
            ]
        )
    _write(repo, "ai/decisions/index.yaml", "\n".join(lines) + "\n")


def _fixture_repo(tmp_path: Path) -> Path:
    repo = tmp_path / "repo"
    _write(
        repo,
        "ai/decisions/ADR001_wrapper.yaml",
        """ADR001_wrapper:
  status: accepted
  title: Wrapper decision
  date: '2026-08-20'
  decision: Wrapper ruling.
  supersedes:
    - ADR000 only for the obsolete clause
""",
    )
    _write(
        repo,
        "ai/decisions/ADR002_flat.yaml",
        """id: ADR002
status: accepted
title: Flat decision
date: '2026-08-21'
decision: Flat ruling.
""",
    )
    _write(
        repo,
        "ai/decisions/ADR003_alternate.yaml",
        """ADR003:
  status: proposed
  title: Alternate wrapper
  decisions:
    graph:
      action: Everything is nodes and edges.
  consensus:
    supersedes:
      - ADR002 only for the graph deferral
""",
    )
    _write(
        repo,
        "ai/decisions/ADR004_comments.yaml",
        """# ADR-004: Comment metadata decision
# Status: ACCEPTED
context: Legacy unwrapped record.
decision: Comment-backed ruling.
""",
    )
    _write_index(
        repo,
        {
            "ADR001_wrapper": "accepted",
            "ADR002_flat": "proposed",
            "ADR003_alternate": "proposed",
            "ADR004_comments": "accepted",
        },
    )
    return repo


def test_imports_current_record_shapes_without_promoting_comment_status(
    tmp_path: Path,
) -> None:
    repo = _fixture_repo(tmp_path)
    cache = tmp_path / "adr.sqlite3"

    summary = build_cache(repo, cache)

    assert summary.record_count == 4
    assert show(cache, "ADR001")["record"]["selector"] == "ADR001_wrapper"
    assert show(cache, "ADR002")["record"]["selector"] == "$"
    assert show(cache, "ADR003")["record"]["selector"] == "ADR003"
    legacy = show(cache, "ADR004")
    assert legacy["record"]["status"] is None
    assert legacy["record"]["index_status"] == "accepted"
    assert legacy["record"]["title_source"] == "index"
    assert [item["code"] for item in legacy["diagnostics"]] == ["missing-structured-status"]


@pytest.mark.parametrize(
    ("relative", "content", "error_type"),
    [
        (
            "ai/decisions/ADR002_flat.yaml",
            "id: ADR099\nstatus: accepted\ntitle: Wrong identity\n",
            SourceParseError,
        ),
        (
            "ai/decisions/ADR002_flat.yaml",
            "id: ADR0020\nstatus: accepted\ntitle: Truncated identity\n",
            SourceParseError,
        ),
        (
            "ai/decisions/ADR001_collision.yaml",
            "id: ADR001\nstatus: proposed\ntitle: Collision\n",
            DuplicateAdrError,
        ),
    ],
)
def test_declared_id_mismatch_and_duplicate_number_fail(
    tmp_path: Path,
    relative: str,
    content: str,
    error_type: type[Exception],
) -> None:
    repo = _fixture_repo(tmp_path)
    _write(repo, relative, content)

    with pytest.raises(error_type):
        build_cache(repo, tmp_path / "adr.sqlite3")


def test_source_index_conflict_stays_explicit(tmp_path: Path) -> None:
    repo = _fixture_repo(tmp_path)
    cache = tmp_path / "adr.sqlite3"
    build_cache(repo, cache)

    result = show(cache, "ADR002")

    assert result["record"]["status"] == "accepted"
    assert result["record"]["index_status"] == "proposed"
    assert [item["code"] for item in result["diagnostics"]] == ["index-status-conflict"]


def test_check_rejects_source_index_membership_drift(tmp_path: Path) -> None:
    repo = _fixture_repo(tmp_path)
    _write_index(
        repo,
        {
            "ADR001_wrapper": "accepted",
            "ADR002_flat": "proposed",
            "ADR003_alternate": "proposed",
            "ADR099_orphan": "accepted",
        },
    )

    with pytest.raises(SourceParseError, match="source/index membership"):
        check_catalog(repo)


def test_nested_and_partial_supersession_is_queryable(tmp_path: Path) -> None:
    repo = _fixture_repo(tmp_path)
    cache = tmp_path / "adr.sqlite3"
    build_cache(repo, cache)

    wrapper_edges = show(cache, "ADR001")["supersession"]
    flat_edges = show(cache, "ADR002")["supersession"]

    assert wrapper_edges == [
        {
            "source_id": "ADR001",
            "kind": "supersedes",
            "target_id": "ADR000",
            "scope": "ADR000 only for the obsolete clause",
        }
    ]
    assert flat_edges == [
        {
            "source_id": "ADR003",
            "kind": "supersedes",
            "target_id": "ADR002",
            "scope": "ADR002 only for the graph deferral",
        }
    ]


def test_live_partial_supersession_keys_preserve_explicit_adr_edges(tmp_path: Path) -> None:
    repo = _fixture_repo(tmp_path)
    _write(
        repo,
        "ai/decisions/ADR003_alternate.yaml",
        """ADR003:
  status: proposed
  title: Alternate wrapper
  partially_supersedes:
    - ADR002 only for the graph deferral
    - the unnamed legacy encoder claim
""",
    )
    _write(
        repo,
        "ai/decisions/ADR004_comments.yaml",
        """status: accepted
title: Scope handoff
supersedes_scope_of: ADR001 only for the deferred implementation choice
""",
    )
    cache = tmp_path / "adr.sqlite3"

    build_cache(repo, cache)

    assert show(cache, "ADR002")["supersession"] == [
        {
            "source_id": "ADR003",
            "kind": "partially_supersedes",
            "target_id": "ADR002",
            "scope": "ADR002 only for the graph deferral",
        }
    ]
    assert {
        "source_id": "ADR004",
        "kind": "supersedes_scope_of",
        "target_id": "ADR001",
        "scope": "ADR001 only for the deferred implementation choice",
    } in show(cache, "ADR001")["supersession"]


@pytest.mark.parametrize("target", ["ADR1000", "ADR002oops"])
def test_malformed_supersession_identifier_fails_instead_of_truncating(
    tmp_path: Path, target: str
) -> None:
    repo = _fixture_repo(tmp_path)
    _write(
        repo,
        "ai/decisions/ADR001_wrapper.yaml",
        f"ADR001_wrapper:\n  status: accepted\n  title: Wrapper decision\n  supersedes: {target}\n",
    )

    with pytest.raises(SourceParseError, match="supersession target"):
        build_cache(repo, tmp_path / "adr.sqlite3")


def test_literal_body_search_finds_nonstandard_decisions_without_returning_body(
    tmp_path: Path,
) -> None:
    repo = _fixture_repo(tmp_path)
    cache = tmp_path / "adr.sqlite3"
    build_cache(repo, cache)

    result = search(cache, "Everything is nodes and edges")

    assert [item["id"] for item in result["results"]] == ["ADR003"]
    assert result["results"][0]["match_location"] == "body"
    assert "body" not in result["results"][0]
    assert search(cache, "%")["results"] == []


def test_search_uses_unicode_casefolding(tmp_path: Path) -> None:
    repo = _fixture_repo(tmp_path)
    _write(
        repo,
        "ai/decisions/ADR002_flat.yaml",
        "id: ADR002\n"
        "status: accepted\n"
        "title: Lukács decision\n"
        "decision: Preserve the accented name.\n",
    )
    cache = tmp_path / "adr.sqlite3"
    build_cache(repo, cache)

    assert [item["id"] for item in search(cache, "LUKÁCS")["results"]] == ["ADR002"]


def test_search_orders_newer_body_match_before_older_title_match(tmp_path: Path) -> None:
    repo = _fixture_repo(tmp_path)
    _write(
        repo,
        "ai/decisions/ADR001_wrapper.yaml",
        "ADR001_wrapper:\n  status: accepted\n  title: Governance title match\n",
    )
    _write(
        repo,
        "ai/decisions/ADR003_alternate.yaml",
        "ADR003:\n  status: proposed\n  title: Newer record\n  decision: Governance body match.\n",
    )
    cache = tmp_path / "adr.sqlite3"
    build_cache(repo, cache)

    assert [item["id"] for item in search(cache, "governance")["results"]] == [
        "ADR003",
        "ADR001",
    ]


def test_search_reports_and_pages_bounded_matches(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    repo = _fixture_repo(tmp_path)
    statuses = {
        "ADR001_wrapper": "accepted",
        "ADR002_flat": "proposed",
        "ADR003_alternate": "proposed",
        "ADR004_comments": "accepted",
    }
    for number in range(5, 11):
        adr_id = f"ADR{number:03d}"
        key = f"{adr_id}_merge"
        statuses[key] = "accepted"
        _write(
            repo,
            f"ai/decisions/{key}.yaml",
            f"{key}:\n  status: accepted\n  title: Merge policy {number} {'🧱' * 500}\n",
        )
    _write_index(repo, statuses)
    cache = tmp_path / "adr.sqlite3"
    build_cache(repo, cache)

    first = search(cache, "merge", limit=2, offset=0)
    second = search(cache, "merge", limit=2, offset=2)

    assert first["match_total"] == 6
    assert first["results_truncated"] is True
    assert first["next_offset"] == 2
    assert [item["id"] for item in first["results"]] == ["ADR010", "ADR009"]
    assert [item["id"] for item in second["results"]] == ["ADR008", "ADR007"]

    assert (
        main(
            [
                "--repo",
                str(repo),
                "--cache",
                str(cache),
                "search",
                "merge",
                "--limit",
                "2",
                "--offset",
                "2",
            ]
        )
        == 0
    )
    assert [item["id"] for item in json.loads(capsys.readouterr().out)["results"]] == [
        "ADR008",
        "ADR007",
    ]

    assert (
        main(
            [
                "--repo",
                str(repo),
                "--cache",
                str(cache),
                "search",
                "merge",
                "--limit",
                "5",
            ]
        )
        == 0
    )
    assert len(capsys.readouterr().out.encode("utf-8")) <= 4096


@pytest.mark.parametrize("location", ["source", "index"])
def test_status_fields_are_bounded_at_ingestion(tmp_path: Path, location: str) -> None:
    repo = _fixture_repo(tmp_path)
    oversized = "x" * 5000
    if location == "source":
        _write(
            repo,
            "ai/decisions/ADR002_flat.yaml",
            f"id: ADR002\nstatus: {oversized}\ntitle: Flat decision\n",
        )
    else:
        _write_index(
            repo,
            {
                "ADR001_wrapper": "accepted",
                "ADR002_flat": oversized,
                "ADR003_alternate": "proposed",
                "ADR004_comments": "accepted",
            },
        )

    with pytest.raises(SourceParseError, match="status is too long"):
        build_cache(repo, tmp_path / "adr.sqlite3")


def test_multibyte_scope_is_bounded_by_output_bytes(tmp_path: Path) -> None:
    repo = _fixture_repo(tmp_path)
    _write(
        repo,
        "ai/decisions/ADR001_wrapper.yaml",
        "ADR001_wrapper:\n"
        "  status: accepted\n"
        "  title: Wrapper decision\n"
        f"  supersedes: ADR002 {'🧱' * 280}\n",
    )

    with pytest.raises(SourceParseError, match="scope is too long"):
        build_cache(repo, tmp_path / "adr.sqlite3")


@pytest.mark.parametrize(
    ("content", "message"),
    [
        (
            f"id: ADR002\nstatus: accepted\ntitle: Flat decision\ndate: {'d' * 5000}\n",
            "date is too long",
        ),
        (
            f"ADR001_{'r' * 400}:\n  status: accepted\n  title: Wrapper decision\n",
            "root key is too long",
        ),
    ],
)
def test_show_metadata_fields_are_bounded_at_ingestion(
    tmp_path: Path, content: str, message: str
) -> None:
    repo = _fixture_repo(tmp_path)
    relative = (
        "ai/decisions/ADR002_flat.yaml"
        if content.startswith("id:")
        else "ai/decisions/ADR001_wrapper.yaml"
    )
    _write(repo, relative, content)

    with pytest.raises(SourceParseError, match=message):
        build_cache(repo, tmp_path / "adr.sqlite3")


def test_failed_rebuild_preserves_the_completed_cache(tmp_path: Path) -> None:
    repo = _fixture_repo(tmp_path)
    cache = tmp_path / "adr.sqlite3"
    original = build_cache(repo, cache)
    _write(repo, "ai/decisions/ADR002_flat.yaml", "id: [unterminated\n")

    with pytest.raises(SourceParseError, match="ADR002_flat.yaml"):
        build_cache(repo, cache)

    assert show(cache, "ADR002")["source_digest"] == original.source_digest


@pytest.mark.parametrize(
    "relative",
    [
        "ai/decisions/index.yaml",
        "ai/decisions/ADR001_wrapper.yaml",
    ],
)
def test_cache_target_cannot_replace_authoritative_adr_sources(
    tmp_path: Path,
    relative: str,
) -> None:
    repo = _fixture_repo(tmp_path)
    target = repo / relative
    original = target.read_bytes()

    with pytest.raises(CacheIntegrityError, match="outside authoritative ADR sources"):
        build_cache(repo, target)

    assert target.read_bytes() == original


def test_source_capture_rejects_symlinks_outside_the_adr_estate(tmp_path: Path) -> None:
    repo = _fixture_repo(tmp_path)
    outside = tmp_path / "outside.yaml"
    _write(tmp_path, "outside.yaml", "id: ADR005\nstatus: accepted\ntitle: Outside\n")
    (repo / "ai/decisions/ADR005_outside.yaml").symlink_to(outside)

    with pytest.raises(SourceParseError, match="non-symlink regular file"):
        capture_snapshot(repo)


def test_source_capture_rejects_a_symlinked_authority_directory(tmp_path: Path) -> None:
    repo = tmp_path / "repo"
    outside = tmp_path / "outside-decisions"
    _write(outside, "ADR001_outside.yaml", "id: ADR001\nstatus: accepted\n")
    _write(outside, "index.yaml", "decisions: {}\n")
    (repo / "ai").mkdir(parents=True)
    (repo / "ai/decisions").symlink_to(outside, target_is_directory=True)

    with pytest.raises(SourceParseError, match="non-symlink directory"):
        capture_snapshot(repo)


def test_cache_symlink_inside_authority_cannot_bypass_lexical_boundary(
    tmp_path: Path,
) -> None:
    repo = _fixture_repo(tmp_path)
    outside = tmp_path / "outside-cache.sqlite3"
    target = repo / "ai/decisions/cache.sqlite3"
    target.symlink_to(outside)

    with pytest.raises(CacheIntegrityError, match="outside authoritative ADR sources"):
        build_cache(repo, target)

    assert target.is_symlink()
    assert not outside.exists()


@pytest.mark.parametrize(
    ("relative", "content"),
    [
        (
            "ai/decisions/ADR002_flat.yaml",
            "id: ADR002\nstatus: accepted\nstatus: proposed\ntitle: Duplicate status\n",
        ),
        (
            "ai/decisions/index.yaml",
            "decisions:\n"
            "  ADR001_wrapper:\n"
            "    status: accepted\n"
            "  ADR001_wrapper:\n"
            "    status: proposed\n",
        ),
    ],
)
def test_duplicate_yaml_keys_fail_instead_of_using_last_value(
    tmp_path: Path,
    relative: str,
    content: str,
) -> None:
    repo = _fixture_repo(tmp_path)
    _write(repo, relative, content)

    with pytest.raises(SourceParseError, match="duplicate key"):
        build_cache(repo, tmp_path / "adr.sqlite3")


def test_cli_rebuild_discards_a_tampered_local_catalog(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    repo = _fixture_repo(tmp_path)
    cache = tmp_path / "adr.sqlite3"
    build_cache(repo, cache)
    connection = sqlite3.connect(cache)
    try:
        connection.execute("UPDATE adr SET title = 'tampered' WHERE id = 'ADR001'")
        connection.commit()
    finally:
        connection.close()

    assert main(["--repo", str(repo), "--cache", str(cache), "show", "ADR001"]) == 0
    result = json.loads(capsys.readouterr().out)

    assert result["record"]["title"] == "Wrapper decision"


def test_source_change_during_build_preserves_prior_cache(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    repo = _fixture_repo(tmp_path)
    cache = tmp_path / "adr.sqlite3"
    original = build_cache(repo, cache)
    initial = capture_snapshot(repo)
    changed = adr_catalog.dataclasses.replace(initial, source_digest="changed")
    snapshots = iter((initial, changed))
    monkeypatch.setattr(adr_catalog, "capture_snapshot", lambda _repo: next(snapshots))

    with pytest.raises(SourceChangedError):
        build_cache(repo, cache)

    assert show(cache, "ADR001")["source_digest"] == original.source_digest


def test_cli_show_search_and_check_are_bounded(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    repo = _fixture_repo(tmp_path)
    cache = tmp_path / "adr.sqlite3"

    assert main(["--repo", str(repo), "--cache", str(cache), "show", "ADR001"]) == 0
    shown = capsys.readouterr()
    assert json.loads(shown.out)["record"]["id"] == "ADR001"
    assert "body" not in shown.out
    assert len(shown.out.encode("utf-8")) <= 4096

    assert main(["--repo", str(repo), "--cache", str(cache), "search", "graph"]) == 0
    found = capsys.readouterr()
    assert all("body" not in item for item in json.loads(found.out)["results"])
    assert len(found.out.encode("utf-8")) <= 4096

    assert main(["--repo", str(repo), "check"]) == 0
    checked = capsys.readouterr()
    assert json.loads(checked.out) == {
        "conflicts": 1,
        "missing_status": 1,
        "records": 4,
        "status": "ok",
    }


def test_worst_case_show_output_stays_inside_its_contract(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    repo = _fixture_repo(tmp_path)
    cache = tmp_path / "adr.sqlite3"
    scopes = "\n".join(
        [
            f"    - ADR010 {'a' * 280}",
            f"    - ADR011 {'b' * 280}",
            f"    - ADR012 {'c' * 280}",
            f"    - ADR013 {'d' * 280}",
            f"    - ADR014 {'e' * 280}",
            f"    - ADR015 {'f' * 280}",
            f"    - ADR016 {'g' * 280}",
            f"    - ADR017 {'h' * 280}",
        ]
    )
    _write(
        repo,
        "ai/decisions/ADR001_wrapper.yaml",
        f"ADR001_wrapper:\n  status: accepted\n  title: {'🧱' * 1000}\n  supersedes:\n{scopes}\n",
    )

    exit_code = main(["--repo", str(repo), "--cache", str(cache), "show", "ADR001"])
    captured = capsys.readouterr()

    assert exit_code == 0
    assert len(captured.out.encode("utf-8")) <= 4096
    title = json.loads(captured.out)["record"]["title"]
    assert len(json.dumps(title, ensure_ascii=False).encode("utf-8")) <= MAX_TITLE_JSON_BYTES


def test_build_rejects_metadata_whose_aggregate_show_output_exceeds_contract(
    tmp_path: Path,
) -> None:
    repo = _fixture_repo(tmp_path)
    selector = "ADR001_" + "s" * 311
    source_status = "a" * 94
    index_status = "b" * 94
    scopes = "\n".join(f"    - ADR{number:03d} {'x' * 311}" for number in range(10, 18))
    _write(
        repo,
        "ai/decisions/ADR001_wrapper.yaml",
        (
            f"{selector}:\n"
            f"  status: {source_status}\n"
            f"  title: {'t' * 318}\n"
            f"  date: {'d' * 94}\n"
            f"  supersedes:\n{scopes}\n"
        ),
    )
    _write_index(
        repo,
        {
            "ADR001_wrapper": index_status,
            "ADR002_flat": "proposed",
            "ADR003_alternate": "proposed",
            "ADR004_comments": "accepted",
        },
    )

    with pytest.raises(SourceParseError, match="show output exceeds 4096 bytes"):
        build_cache(repo, tmp_path / "adr.sqlite3")


def test_live_estate_retains_the_historical_adr_floor() -> None:
    repo = Path(__file__).resolve().parents[3]

    assert len(capture_snapshot(repo).files) >= 219
