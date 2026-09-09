"""Sync guard for the content-set manifest (``rust/crates/babylon-tick/content/
content-sets.toml``, issue #652 Task 4, plan §4).

Every ``.bscn`` scenario in ``babylon-tick``'s content estate is loaded
together with a specific rule pack (and sometimes a shared declaration
prelude) by one or more ``include_str!`` call sites scattered across the
crate's ``src/`` and ``tests/`` directories. That pairing was previously
recoverable only by reading the Rust source: two files can ``include_str!``
the SAME scenario and rule pack (siblings), one file can pair one scenario
with several rule files, and a prelude's presence is discoverable only from
a header comment. Convention (filename stems) does not recover it — see
plan §4.2's own counterexamples. The manifest makes the pairing an explicit,
transcribed fact; this guard keeps it from drifting silently out of sync
with the Rust source it was transcribed from.

Today the Rust call sites are the source and the manifest is the one-time
transcription (Task 4.2). Going forward the manifest is the declaration and
the call sites are its mirror — the two containments below (rows 2 and 3)
together mean neither side can drift without the other noticing:

1. every ``scenario``/``prelude``/``rules``/``consumers`` path a row names
   exists on disk;
2. **Rust ⊆ manifest** — every content path an ``include_str!`` call site
   under ``rust/crates/babylon-tick/{src,tests}/*.rs`` declares is covered
   by the union of every manifest row that names that file as a consumer;
3. **manifest ⊆ Rust** — every row's ``consumers`` files exist and each one
   really does ``include_str!`` that row's ``scenario``, every one of its
   ``prelude`` entries, and every one of its ``rules`` (an empty
   ``consumers`` list requires a non-empty ``note`` explaining why);
4. **no invisible content** — every ``.bsl``/``.bscn`` file under
   ``content/**`` is claimed by at least one row (as ``scenario``,
   ``prelude``, or a ``rules`` entry) or is named in ``[orphans]`` with a
   reason — this is how a file nobody wires up yet (the #646 landmine
   class) stays VISIBLE instead of latent;
5. **ids are unique and appear in ascending byte order** — a duplicate id
   is a collision caught mechanically rather than by a reviewer noticing,
   and a stable sort order keeps diffs to the manifest small and readable.

Rows 2 and 3 are set-containment checks per consumer file, not literal
tuple-formation: a file's `include_str!` constants might pair scenario A
with rule pack B in one test and scenario C with rule pack D in another
(``tick_goldens.rs`` is the extreme case — nine independent pairs in one
file), so the guard does not try to rediscover WHICH scenario goes with
WHICH rule from the Rust source; that pairing is exactly the fact the
manifest transcribes by hand. What the guard verifies mechanically is that
the set of paths a file actually ``include_str!``s and the set of paths the
rows naming that file as a consumer claim for it are the same set — either
side having something the other lacks is drift.

Known blind spot of that reading (task-4 review, 2026-08-18): two disjoint
single-rule rows sharing one consumer file could have their ``rules`` values
SWAPPED — each row then lies about its scenario's pairing while the per-file
union stays identical, and every row here still passes. No such crossed
pairing exists in the committed data; a future editor adding sibling rows
under one consumer should not rely on this guard to catch a cross-wiring.

Modelled on ``test_bsl_grammar_sync.py``'s containment discipline (:13-33).
"""

from __future__ import annotations

import re
import tomllib
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[3]
CRATE_ROOT = REPO_ROOT / "rust" / "crates" / "babylon-tick"
CONTENT_ROOT = CRATE_ROOT / "content"
MANIFEST = CONTENT_ROOT / "content-sets.toml"

#: The exact rg pattern plan §4.3 derives the transcription budget from:
#: ``rg -c 'include_str!\("\.\./content' rust/crates/babylon-tick/{src,tests}/*.rs``.
#: Non-recursive by design — only the top-level files of ``src/`` and
#: ``tests/``, matching the census command exactly.
INCLUDE_STR_RE = re.compile(r'include_str!\("\.\./content/([^"]+)"\)')

ORGANIZATION_PRACTICE_PRELUDE = "declarations/organization-practice.bscn"
WORLDVIEW_PRELUDE = "declarations/worldview.bscn"
ORGANIZATION_PRACTICE_CONTRACT_CONSUMER = (
    "rust/crates/babylon-tick/tests/organization_practice_contract.rs"
)
PROMOTED_PRACTICE_SET_IDS = (
    "organization/foundation",
    "community/carrier-collision",
    "community/conformance",
    "community/cost-modifier",
    "community/decay-arc",
    "community/degenerate",
    "community/empty",
    "community/floor",
    "community/solidarity-seam",
    "community/tie",
    "consciousness/ternary-conformance",
)


def _read_manifest() -> dict[str, object]:
    assert MANIFEST.exists(), f"{MANIFEST} is missing"
    with MANIFEST.open("rb") as fh:
        return tomllib.load(fh)


def _rows() -> list[dict[str, object]]:
    data = _read_manifest()
    rows = data.get("set", [])
    assert isinstance(rows, list), "content-sets.toml's `set` key must be an array of tables"
    return rows


def _orphans() -> dict[str, str]:
    data = _read_manifest()
    orphans = data.get("orphans", {})
    assert isinstance(orphans, dict), "content-sets.toml's `[orphans]` must be a table"
    return orphans


def _kernel_slots() -> list[dict[str, object]]:
    data = _read_manifest()
    rows = data.get("kernel_slot", [])
    assert isinstance(rows, list), (
        "content-sets.toml's `kernel_slot` key must be an array of tables"
    )
    return rows


def _row_content_paths(row: dict[str, object]) -> set[str]:
    """A row's own claimed content paths: its scenario, prelude, and rules."""
    scenario = row["scenario"]
    prelude = row["prelude"]
    rules = row["rules"]
    assert isinstance(scenario, str)
    assert isinstance(prelude, list)
    assert isinstance(rules, list)
    return {scenario, *prelude, *rules}


def _rust_declared_paths() -> dict[str, set[str]]:
    """Every ``.rs`` file's own set of ``include_str!("../content/...")`` paths.

    Scoped to the top-level files of ``src/`` and ``tests/`` only — the same
    scope the census command (plan §4.3) uses, and the only directories any
    content-set consumer lives in today.
    """
    files = sorted((CRATE_ROOT / "src").glob("*.rs")) + sorted((CRATE_ROOT / "tests").glob("*.rs"))
    declared: dict[str, set[str]] = {}
    for path in files:
        paths = set(INCLUDE_STR_RE.findall(path.read_text(encoding="utf-8")))
        if paths:
            declared[str(path.relative_to(REPO_ROOT))] = paths
    return declared


def _manifest_paths_by_consumer() -> dict[str, set[str]]:
    """Every consumer file's claimed paths: the union over every row naming it."""
    by_consumer: dict[str, set[str]] = {}
    for row in _rows():
        consumers = row["consumers"]
        assert isinstance(consumers, list)
        row_paths = _row_content_paths(row)
        for consumer in consumers:
            assert isinstance(consumer, str)
            by_consumer.setdefault(consumer, set()).update(row_paths)
    return by_consumer


class TestEveryManifestPathExistsOnDisk:
    """Row 1 — a row naming a path that does not exist is a typo, not content."""

    def test_every_scenario_prelude_and_rule_path_exists(self) -> None:
        missing: list[str] = []
        for row in _rows():
            for content_path in sorted(_row_content_paths(row)):
                if not (CONTENT_ROOT / content_path).exists():
                    missing.append(f"{row['id']}: {content_path}")
        assert not missing, f"manifest rows name content paths that do not exist: {missing}"

    def test_every_consumer_path_exists(self) -> None:
        missing: list[str] = []
        for row in _rows():
            for consumer in sorted(row["consumers"]):
                if not (REPO_ROOT / consumer).exists():
                    missing.append(f"{row['id']}: {consumer}")
        assert not missing, f"manifest rows name consumer files that do not exist: {missing}"


class TestRustContentIsCoveredByTheManifest:
    """Row 2 — Rust ⊆ manifest.

    Every path an ``include_str!`` call site declares must be covered by the
    union of manifest rows naming that file as a consumer. A path the file
    declares but no row (naming that file) claims is content the manifest
    forgot — exactly what deleting a row simulates (Task 4.4).
    """

    def test_every_declared_path_is_covered_by_a_row_naming_that_consumer(self) -> None:
        rust_index = _rust_declared_paths()
        manifest_index = _manifest_paths_by_consumer()
        gaps: list[str] = []
        for consumer, declared in sorted(rust_index.items()):
            covered = manifest_index.get(consumer, set())
            missing = declared - covered
            if missing:
                gaps.append(f"{consumer}: {sorted(missing)}")
        assert not gaps, (
            "Rust files include_str! content paths no manifest row (naming that "
            f"file as a consumer) accounts for: {gaps}"
        )


class TestTheManifestIsCoveredByRust:
    """Row 3 — manifest ⊆ Rust.

    Every row's consumers must actually ``include_str!`` that row's full
    content-path set (scenario, prelude, and every rule). A row whose
    ``scenario`` (or a rule) points somewhere its consumers never load is
    drift from the other direction — exactly what redirecting a row's
    ``scenario`` at a sibling simulates (Task 4.4). An empty ``consumers``
    list is only legal with a non-empty ``note`` explaining why the row has
    none.
    """

    def test_every_row_with_consumers_is_fully_included_by_each_one(self) -> None:
        rust_index = _rust_declared_paths()
        gaps: list[str] = []
        for row in _rows():
            consumers = row["consumers"]
            if not consumers:
                continue
            row_paths = _row_content_paths(row)
            for consumer in sorted(consumers):
                declared = rust_index.get(consumer, set())
                missing = row_paths - declared
                if missing:
                    gaps.append(f"{row['id']} / {consumer}: {sorted(missing)}")
        assert not gaps, f"manifest rows claim content a named consumer never include_str!s: {gaps}"

    def test_a_row_with_no_consumers_carries_a_non_empty_note(self) -> None:
        bare: list[str] = []
        for row in _rows():
            if not row["consumers"] and not row.get("note"):
                bare.append(str(row["id"]))
        assert not bare, (
            f"rows with an empty `consumers` list and no `note` explaining why: {sorted(bare)}"
        )


class TestNoContentIsInvisible:
    """Row 4 — every ``.bsl``/``.bscn`` under ``content/**`` is claimed or orphaned.

    A file that appears in neither a row nor ``[orphans]`` is exactly the
    #646 landmine class: content nobody wires up and nothing flags.
    """

    def test_every_content_file_is_claimed_by_a_row_or_declared_as_an_orphan(self) -> None:
        all_content = {
            str(path.relative_to(CONTENT_ROOT))
            for path in CONTENT_ROOT.rglob("*")
            if path.suffix in (".bsl", ".bscn")
        }
        claimed: set[str] = set()
        for row in _rows():
            claimed |= _row_content_paths(row)
        orphans = _orphans()
        claimed |= set(orphans)
        invisible = all_content - claimed
        assert not invisible, (
            f"content files claimed by no row and not listed in [orphans]: {sorted(invisible)}"
        )

    def test_orphans_and_rows_do_not_both_claim_the_same_file(self) -> None:
        claimed: set[str] = set()
        for row in _rows():
            claimed |= _row_content_paths(row)
        orphans = set(_orphans())
        contradictions = claimed & orphans
        assert not contradictions, (
            "files listed in [orphans] that a row ALSO claims (the orphan reason is "
            f"stale): {sorted(contradictions)}"
        )

    def test_every_orphan_reason_is_non_empty(self) -> None:
        empty = [path for path, reason in _orphans().items() if not str(reason).strip()]
        assert not empty, f"[orphans] entries with an empty reason: {sorted(empty)}"


class TestRowIdsAreUniqueAndSorted:
    """Row 5 — deterministic diffs, and a collision caught mechanically.

    Modelled on ``TestTheDraftRulingRegisterHasNoDuplicateRowNumbers``
    (``test_bsl_grammar_sync.py:726-753``).
    """

    def test_no_row_id_is_duplicated(self) -> None:
        ids = [str(row["id"]) for row in _rows()]
        duplicates = sorted({i for i in ids if ids.count(i) > 1})
        assert not duplicates, f"duplicate [[set]] ids: {duplicates}"

    def test_row_ids_appear_in_ascending_byte_order(self) -> None:
        ids = [str(row["id"]) for row in _rows()]
        assert ids == sorted(ids), (
            "[[set]] rows are not in ascending id byte order — reorder them for deterministic diffs"
        )


class TestTheManifestHasRowsAtAll:
    """A manifest that parses but is empty would pass every row above vacuously."""

    def test_at_least_one_row_exists(self) -> None:
        assert _rows(), "content-sets.toml has no [[set]] rows"

    def test_the_schema_version_is_declared(self) -> None:
        data = _read_manifest()
        assert data.get("schema") == 2, "content-sets.toml must declare `schema = 2`"

    def test_official_kernel_slot_ledger_is_pinned(self) -> None:
        assert _kernel_slots() == [
            {
                "ordinal": 0,
                "rule": "struggle/spark-mechanic",
                "sample": "struggle/spark",
                "slot": 0,
            }
        ]


class TestOrganizationPracticePreludePromotion:
    """The eleven promoted sets must each declare the shared practice prelude."""

    def test_each_promoted_set_declares_the_practice_prelude(self) -> None:
        rows = {str(row["id"]): row for row in _rows()}
        for set_id in PROMOTED_PRACTICE_SET_IDS:
            assert set_id in rows, f"missing promoted content set: {set_id}"
            prelude = rows[set_id]["prelude"]
            assert isinstance(prelude, list)
            assert ORGANIZATION_PRACTICE_PRELUDE in prelude, (
                f"{set_id} must declare {ORGANIZATION_PRACTICE_PRELUDE}"
            )

    def test_dual_prelude_sets_preserve_exact_dependency_order(self) -> None:
        rows = {str(row["id"]): row for row in _rows()}
        expected = [ORGANIZATION_PRACTICE_PRELUDE, WORLDVIEW_PRELUDE]
        for set_id in ("community/tie", "consciousness/ternary-conformance"):
            assert rows[set_id]["prelude"] == expected, set_id

    def test_practice_contract_row_is_the_exact_no_rule_witness(self) -> None:
        rows = {str(row["id"]): row for row in _rows()}
        row = rows["organization/practice-contract"]
        assert row["scenario"] == "scenarios/organization-practice-contract.bscn"
        assert row["prelude"] == [ORGANIZATION_PRACTICE_PRELUDE]
        assert row["rules"] == []
        assert row["consumers"] == [ORGANIZATION_PRACTICE_CONTRACT_CONSUMER]
