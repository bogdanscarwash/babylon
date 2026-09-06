"""The corpus manifest — canon-as-data for the narrator's grounding corpus (ADR107, T5 U3).

Replaces the hardcoded ``MVP_CORPUS`` tuple that used to live in
``tools/ingest_corpus.py`` (a fuzzy-keyword-matching search against an
arbitrary external mirror path) with a declarative manifest of rows —
mirroring :mod:`babylon.intelligence.model_manifest`'s shape (frozen
Pydantic models, closed ``StrEnum`` vocabularies, a pure ``parse_manifest``
function, loud validation).

Each row names a ``path_glob`` that resolves against a *corpus root* — the
durable OCR/extraction tree at ``~/Documents/ocr/`` (see that tree's own
``MANIFEST.yaml`` for extraction provenance) — supplied by the caller at
load/resolve time. This module never hardcodes an absolute, user-specific
path; production code passes the real tree, unit tests build tmp fixtures.

Four canon dispositions (ADR107 widens the original allow/deny/flag_bd
triad to four so the apocrypha row can even be represented):

* ``allow`` — canon; eligible for narrator/RAG ingestion.
* ``deny`` — ruled OUT.
* ``flag_bd`` — adversarial-only (Divergence Channel, ADR072).
* ``apocryphal`` — kept deliberately, structurally fenced: a row may resolve
  into an ``_apocrypha/`` directory only if it declares this status, and a
  row declaring this status may resolve nowhere else. Enforced at load time
  by :meth:`CorpusRow._check_apocrypha_fencing` (rejects the glob *string*
  itself), not by trusting every future ``path_glob`` author to remember a
  deny row (ADR107 consequences).

Every one of ``deny``/``flag_bd``/``apocryphal`` is honored even *inside* an
enclosing allow glob: :meth:`CorpusManifest.ingest_targets` computes the
exclusion as a set-difference over matched *files*, subtracting ALL non-allow
rows' matches (not deny alone), never a directory-level carve-out — so a
denied/flagged/apocryphal sub-path wins over whatever allow glob happens to
sweep over it, however broad that glob is (a nested denied-source case, and
its ``_apocrypha``/``flag_bd`` analogues). The ``_apocrypha/``
directory additionally gets a second, independent structural check at
resolved-file granularity (:func:`_under_apocrypha`), so it stays unreachable
from ``ingest_targets`` even if the per-row exclusion above were ever wrong.

Presence vs. validity: a row whose glob matches nothing on a given box is a
MANIFEST fact (nothing has been extracted there yet), never a validation
error — :meth:`CorpusManifest.ingest_targets` reports empty matches for such
rows rather than raising.
"""

from __future__ import annotations

from collections.abc import Iterable
from enum import StrEnum
from itertools import islice
from pathlib import Path
from typing import Any, Final

import yaml
from pydantic import BaseModel, ConfigDict, field_validator, model_validator

#: Directory name the apocrypha class is fenced to (ADR107). No non-apocryphal
#: row's ``path_glob`` may resolve into it, and no apocryphal row may resolve
#: anywhere else.
APOCRYPHA_DIR_NAME = "_apocrypha"


class CorpusRole(StrEnum):
    """Where a chunk may surface — rides chunk ``metadata`` jsonb for ``@>`` retrieval.

    Orthogonal to :class:`CanonStatus`: role says WHERE a chunk can surface
    once ingested; canon_status says WHETHER it is ever ingested at all.
    """

    NARRATOR = "narrator"
    ATLAS_RU = "atlas_ru"
    ATLAS_CN = "atlas_cn"
    ATLAS_US = "atlas_us"
    CONCEPT_CARD = "concept_card"
    GLOSSARY = "glossary"
    DOCTRINE = "doctrine"


class CanonStatus(StrEnum):
    """The four canon dispositions (ADR107)."""

    ALLOW = "allow"
    DENY = "deny"
    FLAG_BD = "flag_bd"
    APOCRYPHAL = "apocryphal"


class ExclusionPolicy(StrEnum):
    """Additional source policy that must remain machine-addressable."""

    NONE = "none"
    DIRECTOR = "director"


class CorpusFormat(StrEnum):
    """Source format a ``path_glob``'s matched files are stored in."""

    TXT = "txt"
    HTML = "html"
    MARKDOWN = "md"
    JSONL = "jsonl"


MAX_CORPUS_MANIFEST_ROWS: Final[int] = 4_096


class CorpusRow(BaseModel):
    """One row of the corpus manifest: a glob, its canon disposition, provenance."""

    model_config = ConfigDict(frozen=True, extra="forbid")

    path_glob: str
    author: str
    work: str
    role: tuple[CorpusRole, ...]
    format: CorpusFormat
    canon_status: CanonStatus
    exclusion_policy: ExclusionPolicy = ExclusionPolicy.NONE
    provenance: str

    @model_validator(mode="after")
    def _check_role_nonempty(self) -> CorpusRow:
        if not self.role:
            raise ValueError(f"row {self.work!r} ({self.author!r}) must declare at least one role")
        return self

    @model_validator(mode="after")
    def _check_director_exclusion_is_denied(self) -> CorpusRow:
        if (
            self.exclusion_policy is ExclusionPolicy.DIRECTOR
            and self.canon_status is not CanonStatus.DENY
        ):
            raise ValueError(
                f"row {self.work!r} declares a director exclusion "
                f"but canon_status is {self.canon_status.value!r}, not deny"
            )
        return self

    @model_validator(mode="after")
    def _check_apocrypha_fencing(self) -> CorpusRow:
        # ADR107: the _apocrypha/ directory is structurally fenced — a glob
        # resolving into it must be tagged apocryphal, and nothing else may
        # point there. This is enforced here (loud, at load time) rather than
        # trusting every future path_glob author to remember a deny row.
        points_into_apocrypha = APOCRYPHA_DIR_NAME in Path(self.path_glob).parts
        if points_into_apocrypha and self.canon_status is not CanonStatus.APOCRYPHAL:
            raise ValueError(
                f"row {self.work!r} path_glob {self.path_glob!r} resolves into "
                f"{APOCRYPHA_DIR_NAME}/ but canon_status is "
                f"{self.canon_status.value!r}, not apocryphal (ADR107 glob-fencing)"
            )
        if self.canon_status is CanonStatus.APOCRYPHAL and not points_into_apocrypha:
            raise ValueError(
                f"row {self.work!r} is canon_status=apocryphal but path_glob "
                f"{self.path_glob!r} does not resolve into {APOCRYPHA_DIR_NAME}/"
            )
        return self


class ManifestTarget(BaseModel):
    """One ``allow`` row plus its excluded-and-existing-only matched files.

    ``files`` is already sorted (deterministic ordering). An empty tuple is a
    MANIFEST fact — the row's work has not been extracted onto this box yet —
    never an error.
    """

    model_config = ConfigDict(frozen=True)

    row: CorpusRow
    files: tuple[Path, ...]

    @property
    def present(self) -> bool:
        """Whether this row currently matches any file on disk."""
        return len(self.files) > 0


class CorpusManifest(BaseModel):
    """A validated, ordered set of corpus rows."""

    model_config = ConfigDict(frozen=True, extra="forbid")

    rows: tuple[CorpusRow, ...] = ()

    @field_validator("rows", mode="before")
    @classmethod
    def _check_row_ceiling(cls, rows: object) -> object:
        if isinstance(rows, (list, tuple)):
            bounded_rows = rows
        elif isinstance(rows, Iterable):
            bounded_rows = tuple(islice(rows, MAX_CORPUS_MANIFEST_ROWS + 1))
        else:
            return rows

        if len(bounded_rows) > MAX_CORPUS_MANIFEST_ROWS:
            raise ValueError("corpus manifest exceeds 4,096 rows")
        return bounded_rows

    def allow_rows(self) -> tuple[CorpusRow, ...]:
        return tuple(r for r in self.rows if r.canon_status is CanonStatus.ALLOW)

    def deny_rows(self) -> tuple[CorpusRow, ...]:
        return tuple(r for r in self.rows if r.canon_status is CanonStatus.DENY)

    def director_excluded_rows(self) -> tuple[CorpusRow, ...]:
        """Return Director-excluded rows without exposing literals elsewhere."""
        return tuple(
            row
            for row in self.rows[:MAX_CORPUS_MANIFEST_ROWS]
            if row.exclusion_policy is ExclusionPolicy.DIRECTOR
        )

    def flag_bd_rows(self) -> tuple[CorpusRow, ...]:
        return tuple(r for r in self.rows if r.canon_status is CanonStatus.FLAG_BD)

    def apocryphal_rows(self) -> tuple[CorpusRow, ...]:
        return tuple(r for r in self.rows if r.canon_status is CanonStatus.APOCRYPHAL)

    def ingest_targets(self, corpus_root: Path) -> tuple[ManifestTarget, ...]:
        """Allow rows (in manifest order), each with excluded/existing files resolved.

        This is the enumeration contract that replaces ``MVP_CORPUS``: allow
        minus every OTHER canon_status (deny, flag_bd, apocryphal) —
        existing-files-only, deterministic ordering. The exclusion is a set
        difference over matched files, never a directory-level carve-out, so
        a narrower deny/flag_bd/apocryphal glob nested inside a broader allow
        glob still wins (a nested denied-source case) — and this
        holds for a broad allow glob too (e.g. a raw marxists.org mirror
        import, or ``**/*.jsonl``): it can never sweep a flag_bd or
        apocryphal row's files back into the ingestible set, because ANY
        non-allow row's matches are excluded, not just deny rows'.

        On top of that per-row exclusion, every resolved file is also checked
        structurally against :data:`APOCRYPHA_DIR_NAME` (see
        :func:`_under_apocrypha`) — a second, independent fence so the
        ``_apocrypha/`` directory stays unreachable even if the exclusion set
        above were ever computed wrong.

        A row whose matched-files tuple ends up empty is a MANIFEST fact
        (nothing extracted onto this box yet), never an error.
        """
        excluded_files: set[Path] = set()
        for row in self.rows:
            if row.canon_status is not CanonStatus.ALLOW:
                excluded_files.update(_glob_matches(corpus_root, row.path_glob))

        targets: list[ManifestTarget] = []
        for row in self.allow_rows():
            matched = tuple(
                f
                for f in _glob_matches(corpus_root, row.path_glob)
                if f not in excluded_files and not _under_apocrypha(corpus_root, f)
            )
            targets.append(ManifestTarget(row=row, files=matched))
        return tuple(targets)

    def resolve_ingestible_files(self, corpus_root: Path) -> tuple[Path, ...]:
        """Flat, deterministically sorted union of every allow row's matched files."""
        files: set[Path] = set()
        for target in self.ingest_targets(corpus_root):
            files.update(target.files)
        return tuple(sorted(files))


def _glob_matches(corpus_root: Path, path_glob: str) -> list[Path]:
    """Resolve one ``path_glob`` against ``corpus_root``.

    An absent corpus_root (or a glob matching nothing) returns an empty list
    rather than raising — presence is reported separately from validity.
    """
    if not corpus_root.exists():
        return []
    return sorted(corpus_root.glob(path_glob))


def _under_apocrypha(corpus_root: Path, file_path: Path) -> bool:
    """Whether ``file_path`` (a match under ``corpus_root``) lives under ``_apocrypha/``.

    Structural fence (ADR107), applied unconditionally in
    :meth:`CorpusManifest.ingest_targets` regardless of which row's glob
    matched the file — the apocrypha directory is never bulk-ingestible via
    any allow row, however broad. ``file_path`` always comes from
    ``corpus_root.glob(...)`` (see :func:`_glob_matches`), so it is always
    relative to ``corpus_root``.
    """
    return APOCRYPHA_DIR_NAME in file_path.relative_to(corpus_root).parts


def parse_manifest(raw: dict[str, Any]) -> CorpusManifest:
    """Parse a YAML mapping (a ``rows:`` list) into a validated manifest."""
    return CorpusManifest.model_validate(raw)


def load_manifest(path: Path) -> CorpusManifest:
    """Load and validate a corpus manifest from an explicit YAML path.

    Test-injectable by design: production code loads the bundled manifest via
    :func:`load_bundled_manifest`; unit tests pass a tmp-fixture path instead
    of ever touching the real ``~/Documents/ocr/`` tree.
    """
    with path.open("r", encoding="utf-8") as handle:
        raw = yaml.safe_load(handle)
    return parse_manifest(raw or {})


def load_bundled_manifest() -> CorpusManifest:
    """Load the manifest shipped in the package (``babylon/data/corpus/manifest.yaml``).

    Loaded via a filesystem-relative path from this module's own location —
    the same convention every other consumer under ``babylon/data/`` uses
    (e.g. ``GameDefines.default_yaml_path`` in
    :mod:`babylon.config.defines._assembler`), rather than
    :mod:`importlib.resources`. ``babylon/data/`` and ``babylon/data/corpus/``
    carry no ``__init__.py`` (unlike :mod:`babylon.intelligence.data`, the one
    true ``importlib.resources`` package in this tree, which
    :mod:`babylon.intelligence.model_manifest` reads from) — resolving them as
    ``importlib.resources`` packages would lean on implicit namespace-package
    behavior that the rest of ``babylon/data/`` deliberately does not depend
    on for a packaged wheel.
    """
    manifest_path = Path(__file__).parent.parent / "data" / "corpus" / "manifest.yaml"
    return load_manifest(manifest_path)
