"""Typed reference data catalog used by artifact exporters and builders."""

from __future__ import annotations

from pathlib import Path
from typing import Literal

import yaml
from pydantic import BaseModel, ConfigDict, model_validator


class CatalogError(ValueError):
    """Reference catalog is unavailable or contains an invalid table declaration."""


#: Repo root (this file is ``<root>/src/babylon/sentinels/coverage/catalog.py``).
_REPO_ROOT: Path = Path(__file__).resolve().parents[3]

#: The canonical catalog location (repo root, Constitution III.4.1).
CATALOG_PATH: Path = _REPO_ROOT / "data-catalog.yaml"

#: The governed reference estate: only ``sqlite_master`` objects with these
#: name prefixes are governed by the catalog, exported as parquet sources,
#: replayed into the build product, and content-compared at roundtrip.
#: Utility bookkeeping tables (``ingest_checkpoint``, ``staging_*``) are
#: OUTSIDE the estate by design — loaders recreate them on demand via
#: ``metadata.create_all``. This is the single source of truth; every sweep
#: surface (catalog sentinel, exporter, schema extractor, roundtrip verifier,
#: subset policy review) must scope through it, never restate it.
GOVERNED_PREFIXES: tuple[str, ...] = ("fact_", "dim_", "bridge_", "view_")


class CatalogTable(BaseModel):
    """One reference-DB table or view declared in ``data-catalog.yaml``.

    Frozen and ``extra="forbid"`` so a malformed row is a loud failure at load
    time (Constitution III.11) rather than a quiet ``None`` at check time.

    :ivar name: the ``sqlite_master`` object name (``fact_*``/``dim_*``/
        ``bridge_*`` table or ``view_*`` view).
    :ivar kind: ``"table"`` or ``"view"`` — drives the empty-base-table probe.
    :ivar source: lineage pointer — an ``id`` from ``categories[].sources[]``
        or ``"derived"``/``"internal"`` for computed/infrastructure objects.
    :ivar extractor: loader/ingest lineage — a live repo path, an archaeology
        note (e.g. ``"deleted @ 4ce7c96a^ tools/etl.py"``), or ``None``.
    :ivar reads: views only — the base tables the view SELECTs from (the
        empty-view probe target). Must be empty for ``kind="table"``.
    :ivar consumers: repo-relative ``.py`` paths that read this object at
        runtime or in tools; existence is asserted by the static sensor.
    :ivar tests: repo-relative test paths guarding this object; existence is
        asserted by the static sensor.
    :ivar disposition: the Data Constitution triage verdict. ``amputate`` is a
        *proposal* — execution always requires an owner ruling.
    :ivar subset_policy: the CI-subset scope; for base tables this MUST equal
        the generator's ``TABLE[name].scope`` (parity-checked); views are never
        copied into subsets and must declare ``"skip"``.
    :ivar material_relation: the material relation the data grounds
        (Aleksandrov Test); required, non-blank.
    :ivar notes: free-text clarification.
    """

    model_config = ConfigDict(frozen=True, extra="forbid")

    name: str
    kind: Literal["table", "view"]
    source: str
    extractor: str | None = None
    reads: tuple[str, ...] = ()
    consumers: tuple[str, ...] = ()
    tests: tuple[str, ...] = ()
    disposition: Literal["keep", "fill", "artifact", "amputate", "investigate"]
    subset_policy: Literal["full", "michigan", "skip"]
    material_relation: str
    notes: str = ""

    @model_validator(mode="after")
    def _validate_shape(self) -> CatalogTable:
        """Reject malformed rows loudly at construction (III.11).

        :returns: ``self`` when valid.
        :raises ValueError: on a blank ``name``/``source``/``material_relation``,
            a view without ``reads`` (or a table with them), a view whose
            ``subset_policy`` is not ``"skip"``, or a non-``.py`` path in
            ``consumers``/``tests``.
        """
        if not self.name.strip():
            raise ValueError("CatalogTable.name must be non-empty")
        if not self.source.strip():
            raise ValueError(f"{self.name!r}: source must be non-empty")
        if not self.material_relation.strip():
            raise ValueError(f"{self.name!r}: material_relation must be non-empty (Aleksandrov)")
        if self.kind == "view" and not self.reads:
            raise ValueError(f"{self.name!r}: a view must declare the base tables it reads")
        if self.kind == "table" and self.reads:
            raise ValueError(f"{self.name!r}: only views declare reads")
        if self.kind == "view" and self.subset_policy != "skip":
            raise ValueError(
                f"{self.name!r}: views are never copied into ci-data subsets — "
                "subset_policy must be 'skip'"
            )
        for field_name, paths in (("consumers", self.consumers), ("tests", self.tests)):
            for entry in paths:
                if not entry.endswith(".py"):
                    raise ValueError(
                        f"{self.name!r}: {field_name} entry {entry!r} is not a .py path"
                    )
        return self


def load_catalog_tables(path: Path = CATALOG_PATH) -> tuple[CatalogTable, ...]:
    """Load the ``tables:`` block of ``data-catalog.yaml`` into frozen rows.

    :param path: Catalog file to read (injectable for efficacy tests).
    :returns: One :class:`CatalogTable` per declared table/view.
    :raises CatalogError: If the file is missing, unparseable, carries no
        ``tables:`` block, or any row fails validation — all infrastructure
        failures (exit 2), never swallowed into a false pass.
    """
    try:
        raw = path.read_text(encoding="utf-8")
    except OSError as exc:
        raise CatalogError(f"cannot read data catalog {path}: {exc}") from exc
    try:
        document = yaml.safe_load(raw)
    except yaml.YAMLError as exc:
        raise CatalogError(f"cannot parse data catalog {path}: {exc}") from exc
    if not isinstance(document, dict) or "tables" not in document:
        raise CatalogError(
            f"data catalog {path} has no 'tables' block — the per-table registry "
            "is missing (Program 21 backfill absent?)"
        )
    rows_raw = document["tables"]
    if not isinstance(rows_raw, list) or not rows_raw:
        raise CatalogError(f"data catalog {path}: 'tables' must be a non-empty list")
    try:
        return tuple(CatalogTable(**row) for row in rows_raw)
    except (TypeError, ValueError) as exc:
        raise CatalogError(f"data catalog {path}: malformed table row — {exc}") from exc
