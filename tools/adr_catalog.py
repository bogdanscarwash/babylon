#!/usr/bin/env python3
"""Build and query Babylon's disposable local ADR catalog.

Git-tracked YAML remains authoritative. SQLite stores a validated search copy
and returns only bounded metadata; it never promotes status or emits ADR bodies.
"""

from __future__ import annotations

import argparse
import dataclasses
import datetime
import hashlib
import itertools
import json
import os
import re
import sqlite3
import stat
import sys
import tempfile
from collections import deque
from collections.abc import Mapping, Sequence
from pathlib import Path
from typing import Any, Final

import yaml

SCHEMA_VERSION: Final = "2"
MAX_DIRECTORY_ENTRIES: Final = 512
MAX_ADR_FILES: Final = 512
MAX_DIAGNOSTICS: Final = MAX_ADR_FILES * 4
MAX_FILE_BYTES: Final = 1_000_000
MAX_INDEX_ROWS: Final = 512
MAX_CONTAINER_ITEMS: Final = 512
MAX_YAML_NODES: Final = 4096
MAX_SUPERSESSION_PER_ADR: Final = 8
MAX_SHOW_EDGES: Final = 8
MAX_DIAGNOSTICS_PER_ADR: Final = 8
MAX_SEARCH_RESULTS: Final = 5
MAX_SEARCH_OFFSET: Final = MAX_ADR_FILES
MAX_QUERY_CHARS: Final = 200
MAX_SCOPE_JSON_BYTES: Final = 320
MAX_TITLE_JSON_BYTES: Final = 320
MAX_SEARCH_TITLE_JSON_BYTES: Final = 180
MAX_STATUS_JSON_BYTES: Final = 96
MAX_DATE_JSON_BYTES: Final = 96
MAX_SELECTOR_JSON_BYTES: Final = 320
MAX_QUERY_JSON_BYTES: Final = 240
MAX_TRUNCATION_STEPS: Final = MAX_TITLE_JSON_BYTES
MAX_CLI_OUTPUT_BYTES: Final = 4096
MAX_QUERYABLE_ADR_IDS: Final = 1000

DEFAULT_REPO_ROOT: Final = Path(__file__).resolve().parents[1]
DEFAULT_CACHE: Final = Path(".cache/babylon/adr-catalog.sqlite3")

_ADR_FILE_RE = re.compile(r"^ADR(?P<number>\d{3})_[A-Za-z0-9_]+\.yaml$")
_ADR_ID_RE = re.compile(r"ADR[-_ ]?(?P<number>\d{1,3})(?!\d)", re.IGNORECASE)
_ADR_TOKEN_RE = re.compile(r"(?<![A-Za-z0-9_])ADR[-_ ]?[A-Za-z0-9_]+", re.IGNORECASE)
_DECLARED_ID_RE = re.compile(r"ADR[-_ ]?(?P<number>\d{1,3})(?!\d)", re.IGNORECASE)
_WRAPPED_ID_RE = re.compile(r"ADR[-_ ]?(?P<number>\d{1,3})(?!\d)(?:_[A-Za-z0-9_]+)?", re.IGNORECASE)
_QUERY_ID_RE = re.compile(r"(?:ADR[-_ ]?)?(?P<number>\d{1,3})", re.IGNORECASE)
_SUPERSESSION_KEYS: Final = frozenset(
    {
        "partially_supersedes",
        "superseded_by",
        "supersedes",
        "supersedes_scope_of",
    }
)
_OPTIONAL_TARGET_SUPERSESSION_KEYS: Final = frozenset({"partially_supersedes"})


class AdrCatalogError(Exception):
    """Base class for catalog failures."""


class SourceParseError(AdrCatalogError):
    """An ADR source cannot be interpreted safely."""


class DuplicateAdrError(AdrCatalogError):
    """Two files claim the same ADR number."""


class SourceChangedError(AdrCatalogError):
    """Source bytes changed during a build."""


class CacheIntegrityError(AdrCatalogError):
    """A completed cache does not match its recorded digest."""


class QueryError(AdrCatalogError):
    """A query violates the bounded output contract."""


class _UniqueKeySafeLoader(yaml.SafeLoader):
    """Safe YAML loader that rejects ambiguous duplicate mapping keys."""


def _construct_unique_mapping(
    loader: yaml.SafeLoader,
    node: yaml.MappingNode,
    deep: bool = False,
) -> dict[object, object]:
    loader.flatten_mapping(node)
    mapping: dict[object, object] = {}
    for key_node, value_node in node.value:
        key = loader.construct_object(key_node, deep=deep)
        try:
            duplicate = key in mapping
        except TypeError as error:
            raise yaml.constructor.ConstructorError(
                "while constructing a mapping",
                node.start_mark,
                "found an unhashable mapping key",
                key_node.start_mark,
            ) from error
        if duplicate:
            raise yaml.constructor.ConstructorError(
                "while constructing a mapping",
                node.start_mark,
                f"found duplicate key {key!r}",
                key_node.start_mark,
            )
        mapping[key] = loader.construct_object(value_node, deep=deep)
    return mapping


_UniqueKeySafeLoader.add_constructor(
    yaml.resolver.BaseResolver.DEFAULT_MAPPING_TAG,
    _construct_unique_mapping,
)


@dataclasses.dataclass(frozen=True)
class SourceFile:
    relative_path: str
    adr_id: str
    number: int
    content: bytes
    text: str
    sha256: str


@dataclasses.dataclass(frozen=True)
class SourceSnapshot:
    repo_root: Path
    files: tuple[SourceFile, ...]
    index_content: bytes
    source_digest: str


@dataclasses.dataclass(frozen=True)
class IndexRow:
    status: str | None
    title: str | None


@dataclasses.dataclass(frozen=True)
class Supersession:
    kind: str
    target_id: str
    scope: str


@dataclasses.dataclass(frozen=True)
class ParsedRecord:
    source: SourceFile
    root_key: str | None
    status: str | None
    title: str | None
    title_source: str
    record_date: str | None
    index_status: str | None
    supersession: tuple[Supersession, ...]


@dataclasses.dataclass(frozen=True)
class Diagnostic:
    adr_id: str
    code: str
    detail: str


@dataclasses.dataclass(frozen=True)
class BuildSummary:
    record_count: int
    missing_status: int
    conflicts: int
    membership_errors: int
    warning_count: int
    source_digest: str


def _bounded_bytes(path: Path) -> bytes:
    if path.is_symlink():
        raise SourceParseError(f"{path}: must be a non-symlink regular file")
    flags = os.O_RDONLY | getattr(os, "O_NONBLOCK", 0) | getattr(os, "O_NOFOLLOW", 0)
    try:
        descriptor = os.open(path, flags)
    except OSError as error:
        raise SourceParseError(f"{path}: must be a non-symlink regular file") from error
    try:
        metadata = os.fstat(descriptor)
        if not stat.S_ISREG(metadata.st_mode):
            raise SourceParseError(f"{path}: must be a non-symlink regular file")
        with os.fdopen(descriptor, "rb", closefd=False) as stream:
            content = stream.read(MAX_FILE_BYTES + 1)
    finally:
        os.close(descriptor)
    if len(content) > MAX_FILE_BYTES:
        raise SourceParseError(f"{path}: source exceeds {MAX_FILE_BYTES} bytes")
    return content


def capture_snapshot(repo_root: Path) -> SourceSnapshot:
    """Read each authoritative byte sequence once for one build attempt."""
    root = repo_root.resolve()
    directory = root / "ai/decisions"
    if not directory.is_dir() or directory.resolve() != directory:
        raise SourceParseError(
            f"ADR directory must be a non-symlink directory inside the repository: {directory}"
        )
    entries = list(itertools.islice(directory.iterdir(), MAX_DIRECTORY_ENTRIES + 1))
    if len(entries) > MAX_DIRECTORY_ENTRIES:
        raise SourceParseError(f"{directory}: too many directory entries")
    paths = sorted(
        (entry for entry in entries[:MAX_DIRECTORY_ENTRIES] if _ADR_FILE_RE.match(entry.name)),
        key=lambda item: item.name,
    )
    if not paths or len(paths) > MAX_ADR_FILES:
        raise SourceParseError(f"expected 1..{MAX_ADR_FILES} ADR files; found {len(paths)}")

    digest = hashlib.sha256()
    sources: list[SourceFile] = []
    seen: set[int] = set()
    for path in paths[:MAX_ADR_FILES]:
        match = _ADR_FILE_RE.match(path.name)
        if match is None:
            raise SourceParseError(f"invalid ADR filename: {path.name}")
        number = int(match.group("number"))
        if number in seen:
            raise DuplicateAdrError(f"ADR{number:03d} appears more than once")
        seen.add(number)
        content = _bounded_bytes(path)
        try:
            text = content.decode("utf-8")
        except UnicodeDecodeError as error:
            raise SourceParseError(f"{path}: source is not UTF-8") from error
        relative = path.relative_to(root).as_posix()
        digest.update(relative.encode())
        digest.update(b"\0")
        digest.update(content)
        digest.update(b"\0")
        sources.append(
            SourceFile(
                relative_path=relative,
                adr_id=f"ADR{number:03d}",
                number=number,
                content=content,
                text=text,
                sha256=hashlib.sha256(content).hexdigest(),
            )
        )

    index_path = directory / "index.yaml"
    if not index_path.is_file():
        raise SourceParseError(f"missing legacy index: {index_path}")
    index_content = _bounded_bytes(index_path)
    digest.update(b"ai/decisions/index.yaml\0")
    digest.update(index_content)
    return SourceSnapshot(root, tuple(sources), index_content, digest.hexdigest())


def _load_yaml(content: bytes, source_path: str) -> Any:
    loader = _UniqueKeySafeLoader(content)
    try:
        return loader.get_single_data()
    except yaml.YAMLError as error:
        raise SourceParseError(f"{source_path}: invalid YAML: {error}") from error
    finally:
        loader.dispose()


def _scalar(value: object, field: str, source_path: str) -> str | None:
    if value is None:
        return None
    if isinstance(value, str):
        return value.strip() or None
    if isinstance(value, (int, float)) and not isinstance(value, bool):
        return str(value)
    if isinstance(value, (datetime.date, datetime.datetime)):
        return value.isoformat()
    raise SourceParseError(f"{source_path}: {field} must be scalar")


def _json_string_bytes(value: str) -> int:
    return len(json.dumps(value, ensure_ascii=False, separators=(",", ":")).encode("utf-8"))


def _encoded_payload(payload: object) -> bytes:
    output = json.dumps(payload, sort_keys=True, separators=(",", ":"), ensure_ascii=False)
    return (output + "\n").encode()


def _bounded_output_text(value: str, field: str, source_path: str, json_byte_limit: int) -> str:
    if _json_string_bytes(value) > json_byte_limit:
        raise SourceParseError(f"{source_path}: {field} is too long")
    return value


def _status(value: object, field: str, source_path: str) -> str | None:
    text = _scalar(value, field, source_path)
    if text is None:
        return None
    return _bounded_output_text(text.casefold(), field, source_path, MAX_STATUS_JSON_BYTES)


def _structured_id(value: str, source_path: str, field: str, *, allow_suffix: bool = False) -> str:
    pattern = _WRAPPED_ID_RE if allow_suffix else _DECLARED_ID_RE
    match = pattern.fullmatch(value)
    if match is None:
        raise SourceParseError(f"{source_path}: {field} is not a complete ADR identifier")
    return f"ADR{int(match.group('number')):03d}"


def _index_rows(snapshot: SourceSnapshot) -> dict[str, IndexRow]:
    document = _load_yaml(snapshot.index_content, "ai/decisions/index.yaml")
    if not isinstance(document, Mapping):
        raise SourceParseError("ai/decisions/index.yaml: root must be a mapping")
    decisions = document.get("decisions")
    if not isinstance(decisions, Mapping):
        raise SourceParseError("ai/decisions/index.yaml: decisions must be a mapping")
    items = list(decisions.items())
    if len(items) > MAX_INDEX_ROWS:
        raise SourceParseError("ai/decisions/index.yaml: too many rows")
    rows: dict[str, IndexRow] = {}
    for raw_key, raw_value in items[:MAX_INDEX_ROWS]:
        key = str(raw_key)
        adr_id = _structured_id(key, "ai/decisions/index.yaml", key, allow_suffix=True)
        if adr_id in rows or not isinstance(raw_value, Mapping):
            raise SourceParseError(f"ai/decisions/index.yaml: malformed duplicate {key}")
        rows[adr_id] = IndexRow(
            status=_status(raw_value.get("status"), f"{key}.status", "index.yaml"),
            title=_scalar(raw_value.get("title"), f"{key}.title", "index.yaml"),
        )
    return rows


def _select_record(
    document: object, source: SourceFile
) -> tuple[Mapping[object, object], str | None]:
    if not isinstance(document, Mapping):
        raise SourceParseError(f"{source.relative_path}: root must be a mapping")
    items = list(document.items())
    if len(items) > MAX_CONTAINER_ITEMS:
        raise SourceParseError(f"{source.relative_path}: root mapping is too large")
    if len(items) == 1 and isinstance(items[0][1], Mapping):
        return items[0][1], str(items[0][0])
    if isinstance(document.get("meta"), Mapping):
        candidates = [item for item in items[:MAX_CONTAINER_ITEMS] if item[0] != "meta"]
        if len(candidates) == 1 and isinstance(candidates[0][1], Mapping):
            return candidates[0][1], str(candidates[0][0])
    return document, None


def _validate_identity(
    document: Mapping[object, object],
    record: Mapping[object, object],
    root_key: str | None,
    source: SourceFile,
) -> None:
    declarations: list[tuple[str, str]] = []
    if root_key is not None:
        declarations.append(("root key", root_key))
    if "id" in record:
        value = _scalar(record.get("id"), "id", source.relative_path)
        if value is not None:
            declarations.append(("id", value))
    meta = document.get("meta")
    if isinstance(meta, Mapping) and "id" in meta:
        value = _scalar(meta.get("id"), "meta.id", source.relative_path)
        if value is not None:
            declarations.append(("meta.id", value))
    for field, value in declarations[:3]:
        declared = _structured_id(
            value,
            source.relative_path,
            field,
            allow_suffix=True,
        )
        if declared != source.adr_id:
            raise SourceParseError(
                f"{source.relative_path}: {field} declares {declared}, expected {source.adr_id}"
            )


def _relation_scopes(value: object, source_path: str, field: str) -> list[str]:
    if value is None:
        return []
    if isinstance(value, str):
        values: list[object] = [value]
    elif isinstance(value, Sequence) and not isinstance(value, (str, bytes)):
        if len(value) > MAX_SUPERSESSION_PER_ADR:
            raise SourceParseError(f"{source_path}: {field} has too many entries")
        values = list(value)[:MAX_SUPERSESSION_PER_ADR]
    else:
        raise SourceParseError(f"{source_path}: {field} must be scalar or a list")
    scopes: list[str] = []
    for item in values[:MAX_SUPERSESSION_PER_ADR]:
        text = _scalar(item, field, source_path)
        if text is not None:
            collapsed = " ".join(text.split())
            scopes.append(
                _bounded_output_text(
                    collapsed,
                    f"{field} scope",
                    source_path,
                    MAX_SCOPE_JSON_BYTES,
                )
            )
    return scopes


def _supersession(record: Mapping[object, object], source: SourceFile) -> tuple[Supersession, ...]:
    queue: deque[object] = deque([record])
    seen_nodes: set[int] = set()
    edges: list[Supersession] = []
    seen_edges: set[tuple[str, str, str]] = set()
    for _ in range(MAX_YAML_NODES):
        if not queue:
            break
        node = queue.popleft()
        if id(node) in seen_nodes:
            continue
        seen_nodes.add(id(node))
        if isinstance(node, Mapping):
            items = list(node.items())
            if len(items) > MAX_CONTAINER_ITEMS:
                raise SourceParseError(f"{source.relative_path}: mapping is too large")
            for raw_key, value in items[:MAX_CONTAINER_ITEMS]:
                key = str(raw_key)
                if key in _SUPERSESSION_KEYS:
                    for scope in _relation_scopes(value, source.relative_path, key):
                        tokens = list(
                            itertools.islice(
                                _ADR_TOKEN_RE.finditer(scope),
                                MAX_SUPERSESSION_PER_ADR + 1,
                            )
                        )
                        if not tokens and key in _OPTIONAL_TARGET_SUPERSESSION_KEYS:
                            continue
                        if not tokens:
                            raise SourceParseError(
                                f"{source.relative_path}: {key} has no valid supersession target"
                            )
                        if len(tokens) > MAX_SUPERSESSION_PER_ADR:
                            raise SourceParseError(
                                f"{source.relative_path}: too many supersession targets"
                            )
                        for token in tokens[:MAX_SUPERSESSION_PER_ADR]:
                            match = _WRAPPED_ID_RE.fullmatch(token.group())
                            if match is None:
                                raise SourceParseError(
                                    f"{source.relative_path}: invalid supersession target "
                                    f"{token.group()!r}"
                                )
                            target = f"ADR{int(match.group('number')):03d}"
                            identity = (key, target, scope)
                            if identity not in seen_edges:
                                seen_edges.add(identity)
                                edges.append(Supersession(key, target, scope))
                if isinstance(value, Mapping) or (
                    isinstance(value, Sequence) and not isinstance(value, (str, bytes))
                ):
                    queue.append(value)
        elif isinstance(node, Sequence) and not isinstance(node, (str, bytes)):
            if len(node) > MAX_CONTAINER_ITEMS:
                raise SourceParseError(f"{source.relative_path}: list is too large")
            queue.extend(list(node)[:MAX_CONTAINER_ITEMS])
        if len(edges) > MAX_SUPERSESSION_PER_ADR:
            raise SourceParseError(f"{source.relative_path}: too many supersession edges")
    if queue:
        raise SourceParseError(f"{source.relative_path}: YAML node budget exceeded")
    return tuple(edges)


def _parse_record(source: SourceFile, index: IndexRow | None) -> ParsedRecord:
    document = _load_yaml(source.content, source.relative_path)
    record, root_key = _select_record(document, source)
    if not isinstance(document, Mapping):
        raise SourceParseError(f"{source.relative_path}: root must be a mapping")
    _validate_identity(document, record, root_key, source)
    status = _status(record.get("status"), "status", source.relative_path)
    source_title = _scalar(record.get("title"), "title", source.relative_path)
    if source_title is not None:
        title = source_title
        title_source = "source"
    elif index is not None and index.title is not None:
        title = index.title
        title_source = "index"
    else:
        title = None
        title_source = "missing"
    record_date = _scalar(record.get("date"), "date", source.relative_path)
    if record_date is not None:
        record_date = _bounded_output_text(
            record_date, "date", source.relative_path, MAX_DATE_JSON_BYTES
        )
    if root_key is not None:
        root_key = _bounded_output_text(
            root_key, "root key", source.relative_path, MAX_SELECTOR_JSON_BYTES
        )
    return ParsedRecord(
        source=source,
        root_key=root_key,
        status=status,
        title=title,
        title_source=title_source,
        record_date=record_date,
        index_status=index.status if index is not None else None,
        supersession=_supersession(record, source),
    )


def _diagnostics(
    records: tuple[ParsedRecord, ...], indexes: Mapping[str, IndexRow]
) -> tuple[Diagnostic, ...]:
    findings: list[Diagnostic] = []
    source_ids = {record.source.adr_id for record in records[:MAX_ADR_FILES]}
    for record in records[:MAX_ADR_FILES]:
        if record.status is None:
            findings.append(
                Diagnostic(
                    record.source.adr_id, "missing-structured-status", "YAML status is absent"
                )
            )
        if (
            record.status is not None
            and record.index_status is not None
            and record.status != record.index_status
        ):
            findings.append(
                Diagnostic(
                    record.source.adr_id,
                    "index-status-conflict",
                    f"source={record.status}; index={record.index_status}",
                )
            )
        if record.title is None:
            findings.append(Diagnostic(record.source.adr_id, "missing-title", "title is absent"))
        if record.source.adr_id not in indexes:
            findings.append(
                Diagnostic(record.source.adr_id, "missing-index-row", "legacy index row is absent")
            )
    for adr_id in sorted(indexes)[:MAX_INDEX_ROWS]:
        if adr_id not in source_ids:
            findings.append(Diagnostic(adr_id, "orphan-index-row", "ADR source is absent"))
    return tuple(findings)


_SCHEMA_SQL: Final = """
PRAGMA foreign_keys = ON;
CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL) WITHOUT ROWID;
CREATE TABLE adr (
    id TEXT PRIMARY KEY,
    number INTEGER NOT NULL UNIQUE,
    path TEXT NOT NULL UNIQUE,
    root_key TEXT,
    source_sha256 TEXT NOT NULL,
    status TEXT,
    title TEXT,
    title_source TEXT NOT NULL,
    record_date TEXT,
    index_status TEXT,
    body TEXT NOT NULL
) WITHOUT ROWID;
CREATE TABLE supersession (
    source_id TEXT NOT NULL REFERENCES adr(id),
    ordinal INTEGER NOT NULL,
    kind TEXT NOT NULL,
    target_id TEXT NOT NULL,
    scope TEXT NOT NULL,
    PRIMARY KEY (source_id, ordinal)
) WITHOUT ROWID;
CREATE TABLE diagnostic (
    ordinal INTEGER PRIMARY KEY,
    adr_id TEXT NOT NULL,
    code TEXT NOT NULL,
    detail TEXT NOT NULL
);
"""


def _insert_records(connection: sqlite3.Connection, records: tuple[ParsedRecord, ...]) -> None:
    for record in records[:MAX_ADR_FILES]:
        connection.execute(
            "INSERT INTO adr VALUES (?,?,?,?,?,?,?,?,?,?,?)",
            (
                record.source.adr_id,
                record.source.number,
                record.source.relative_path,
                record.root_key,
                record.source.sha256,
                record.status,
                record.title,
                record.title_source,
                record.record_date,
                record.index_status,
                record.source.text,
            ),
        )
        for ordinal, edge in enumerate(record.supersession[:MAX_SUPERSESSION_PER_ADR]):
            connection.execute(
                "INSERT INTO supersession VALUES (?,?,?,?,?)",
                (record.source.adr_id, ordinal, edge.kind, edge.target_id, edge.scope),
            )


def _metadata(connection: sqlite3.Connection) -> dict[str, str]:
    result = connection.execute("PRAGMA quick_check").fetchone()
    if result is None or result[0] != "ok":
        raise CacheIntegrityError(f"SQLite quick_check failed: {result}")
    rows = connection.execute("SELECT key,value FROM meta").fetchmany(5)
    metadata = {str(row[0]): str(row[1]) for row in rows[:4]}
    if metadata.get("schema_version") != SCHEMA_VERSION:
        raise CacheIntegrityError("cache schema version is stale")
    count = int(connection.execute("SELECT COUNT(*) FROM adr").fetchone()[0])
    if metadata.get("source_count") != str(count):
        raise CacheIntegrityError("cache source count does not match its rows")
    return metadata


def _write_database(
    path: Path,
    snapshot: SourceSnapshot,
    records: tuple[ParsedRecord, ...],
    diagnostics: tuple[Diagnostic, ...],
) -> None:
    connection = sqlite3.connect(path)
    connection.row_factory = sqlite3.Row
    try:
        connection.execute("PRAGMA journal_mode = DELETE")
        connection.executescript(_SCHEMA_SQL)
        connection.executemany(
            "INSERT INTO meta VALUES (?,?)",
            (
                ("schema_version", SCHEMA_VERSION),
                ("source_digest", snapshot.source_digest),
                ("source_count", str(len(records))),
            ),
        )
        _insert_records(connection, records)
        for ordinal, diagnostic in enumerate(diagnostics[:MAX_DIAGNOSTICS]):
            connection.execute(
                "INSERT INTO diagnostic VALUES (?,?,?,?)",
                (ordinal, diagnostic.adr_id, diagnostic.code, diagnostic.detail),
            )
        connection.commit()
        _metadata(connection)
        _validate_show_contract(connection)
    finally:
        connection.close()


def _summary(
    snapshot: SourceSnapshot,
    records: tuple[ParsedRecord, ...],
    diagnostics: tuple[Diagnostic, ...],
) -> BuildSummary:
    missing = sum(
        1 for item in diagnostics[:MAX_DIAGNOSTICS] if item.code == "missing-structured-status"
    )
    conflicts = sum(
        1 for item in diagnostics[:MAX_DIAGNOSTICS] if item.code == "index-status-conflict"
    )
    membership_errors = sum(
        1
        for item in diagnostics[:MAX_DIAGNOSTICS]
        if item.code in {"missing-index-row", "orphan-index-row"}
    )
    return BuildSummary(
        record_count=len(records),
        missing_status=missing,
        conflicts=conflicts,
        membership_errors=membership_errors,
        warning_count=len(diagnostics),
        source_digest=snapshot.source_digest,
    )


def build_cache(
    repo_root: Path,
    cache_path: Path,
    *,
    snapshot: SourceSnapshot | None = None,
) -> BuildSummary:
    """Build, validate, recheck source bytes, and atomically replace the cache."""
    captured = snapshot if snapshot is not None else capture_snapshot(repo_root)
    indexes = _index_rows(captured)
    records = tuple(
        _parse_record(source, indexes.get(source.adr_id))
        for source in captured.files[:MAX_ADR_FILES]
    )
    diagnostics = _diagnostics(records, indexes)
    target = cache_path if cache_path.is_absolute() else captured.repo_root / cache_path
    authority_root = (captured.repo_root / "ai/decisions").resolve()
    lexical_target = Path(os.path.abspath(target))
    resolved_target = target.resolve(strict=False)
    if (
        lexical_target == authority_root
        or lexical_target.is_relative_to(authority_root)
        or resolved_target == authority_root
        or resolved_target.is_relative_to(authority_root)
    ):
        raise CacheIntegrityError(
            f"cache target must be outside authoritative ADR sources: {target}"
        )
    target.parent.mkdir(parents=True, exist_ok=True)
    handle = tempfile.NamedTemporaryFile(
        prefix=f".{target.name}.", suffix=".tmp", dir=target.parent, delete=False
    )
    temporary = Path(handle.name)
    handle.close()
    try:
        _write_database(temporary, captured, records, diagnostics)
        if capture_snapshot(captured.repo_root).source_digest != captured.source_digest:
            raise SourceChangedError("ADR source changed during cache build")
        os.replace(temporary, target)
    finally:
        if temporary.exists():
            temporary.unlink()
    return _summary(captured, records, diagnostics)


def _open_read_only(cache_path: Path) -> sqlite3.Connection:
    connection = sqlite3.connect(f"{cache_path.resolve().as_uri()}?mode=ro", uri=True)
    connection.row_factory = sqlite3.Row
    connection.create_function("unicode_casefold", 1, _sqlite_casefold, deterministic=True)
    return connection


def _sqlite_casefold(value: object) -> str:
    if value is None:
        return ""
    if not isinstance(value, str):
        raise TypeError("unicode_casefold accepts SQLite text")
    return value.casefold()


def _query_id(value: str) -> str:
    match = _QUERY_ID_RE.fullmatch(value.strip())
    if match is None:
        raise QueryError(f"invalid ADR identifier: {value!r}")
    number = int(match.group("number"))
    if not 0 <= number <= 999:
        raise QueryError(f"ADR number outside 000..999: {number}")
    return f"ADR{number:03d}"


def _truncate(value: str | None, json_byte_limit: int) -> tuple[str | None, bool]:
    if value is None or _json_string_bytes(value) <= json_byte_limit:
        return value, False
    max_chars = min(len(value), MAX_TRUNCATION_STEPS)
    for step in range(MAX_TRUNCATION_STEPS + 1):
        if step > max_chars:
            break
        candidate = value[: max_chars - step] + "…"
        if _json_string_bytes(candidate) <= json_byte_limit:
            return candidate, True
    raise CacheIntegrityError("string cannot fit the bounded JSON output")


def _show_payload(
    connection: sqlite3.Connection, normalized: str, source_digest: str
) -> dict[str, object]:
    row = connection.execute(
        "SELECT id,status,index_status,title,title_source,record_date,path,root_key,source_sha256 "
        "FROM adr WHERE id=?",
        (normalized,),
    ).fetchone()
    edge_total = int(
        connection.execute(
            "SELECT COUNT(*) FROM supersession WHERE source_id=? OR target_id=?",
            (normalized, normalized),
        ).fetchone()[0]
    )
    edge_rows = connection.execute(
        "SELECT source_id,kind,target_id,scope FROM supersession "
        "WHERE source_id=? OR target_id=? ORDER BY source_id,ordinal LIMIT ?",
        (normalized, normalized, MAX_SHOW_EDGES),
    ).fetchall()
    diagnostic_rows = connection.execute(
        "SELECT code,detail FROM diagnostic WHERE adr_id=? ORDER BY ordinal LIMIT ?",
        (normalized, MAX_DIAGNOSTICS_PER_ADR + 1),
    ).fetchall()
    edges = edge_rows[:MAX_SHOW_EDGES]
    diagnostics = diagnostic_rows[:MAX_DIAGNOSTICS_PER_ADR]
    record = None
    if row is not None:
        title, title_truncated = _truncate(row["title"], MAX_TITLE_JSON_BYTES)
        record = {
            "id": row["id"],
            "status": row["status"],
            "index_status": row["index_status"],
            "title": title,
            "title_truncated": title_truncated,
            "title_source": row["title_source"],
            "date": row["record_date"],
            "source_path": row["path"],
            "source_sha256": row["source_sha256"],
            "selector": row["root_key"] or "$",
        }
    return {
        "source_digest": source_digest,
        "record": record,
        "supersession": [dict(item) for item in edges],
        "supersession_total": edge_total,
        "supersession_truncated": edge_total > len(edges),
        "diagnostics": [dict(item) for item in diagnostics],
        "diagnostics_truncated": len(diagnostic_rows) > MAX_DIAGNOSTICS_PER_ADR,
    }


def _validate_show_contract(connection: sqlite3.Connection) -> None:
    metadata = _metadata(connection)
    rows = connection.execute(
        "SELECT id FROM (SELECT id FROM adr UNION SELECT target_id AS id FROM supersession "
        "UNION SELECT adr_id AS id FROM diagnostic) ORDER BY id LIMIT ?",
        (MAX_QUERYABLE_ADR_IDS + 1,),
    ).fetchall()
    if len(rows) > MAX_QUERYABLE_ADR_IDS:
        raise CacheIntegrityError("catalog exceeds the queryable ADR identifier space")
    for row in rows[:MAX_QUERYABLE_ADR_IDS]:
        adr_id = str(row["id"])
        payload = _show_payload(connection, adr_id, metadata["source_digest"])
        if len(_encoded_payload(payload)) <= MAX_CLI_OUTPUT_BYTES:
            continue
        record = payload["record"]
        source = record["source_path"] if isinstance(record, Mapping) else adr_id
        raise SourceParseError(f"{source}: show output exceeds {MAX_CLI_OUTPUT_BYTES} bytes")


def show(cache_path: Path, adr_id: str) -> dict[str, object]:
    normalized = _query_id(adr_id)
    connection = _open_read_only(cache_path)
    try:
        metadata = _metadata(connection)
        return _show_payload(connection, normalized, metadata["source_digest"])
    finally:
        connection.close()


def search(
    cache_path: Path,
    query: str,
    limit: int = MAX_SEARCH_RESULTS,
    offset: int = 0,
) -> dict[str, object]:
    term = query.strip()
    if not term or len(term) > MAX_QUERY_CHARS:
        raise QueryError(f"search must contain 1..{MAX_QUERY_CHARS} characters")
    if _json_string_bytes(term) > MAX_QUERY_JSON_BYTES:
        raise QueryError(f"search JSON string must fit {MAX_QUERY_JSON_BYTES} bytes")
    if not 1 <= limit <= MAX_SEARCH_RESULTS:
        raise QueryError(f"limit must be 1..{MAX_SEARCH_RESULTS}")
    if not 0 <= offset <= MAX_SEARCH_OFFSET:
        raise QueryError(f"offset must be 0..{MAX_SEARCH_OFFSET}")
    folded = term.casefold()
    connection = _open_read_only(cache_path)
    try:
        metadata = _metadata(connection)
        match_sql = (
            "instr(unicode_casefold(COALESCE(title,'')),?)>0 OR instr(unicode_casefold(body),?)>0"
        )
        match_total = int(
            connection.execute(
                f"SELECT COUNT(*) FROM adr WHERE {match_sql}",  # noqa: S608
                (folded, folded),
            ).fetchone()[0]
        )
        rows = connection.execute(
            "SELECT id,status,title,title_source,path,"
            "CASE WHEN instr(unicode_casefold(COALESCE(title,'')),?)>0 "
            "THEN 'title' ELSE 'body' END AS match_location FROM adr WHERE "
            f"{match_sql} ORDER BY number DESC, "  # noqa: S608
            "CASE match_location WHEN 'title' THEN 0 ELSE 1 END LIMIT ? OFFSET ?",
            (folded, folded, folded, limit, offset),
        ).fetchall()
    finally:
        connection.close()
    results: list[dict[str, object]] = []
    for row in rows[:MAX_SEARCH_RESULTS]:
        item = dict(row)
        title, title_truncated = _truncate(item["title"], MAX_SEARCH_TITLE_JSON_BYTES)
        item["title"] = title
        item["title_truncated"] = title_truncated
        results.append(item)
    return {
        "source_digest": metadata["source_digest"],
        "query": term,
        "match_total": match_total,
        "offset": offset,
        "results_truncated": offset + len(results) < match_total,
        "next_offset": offset + len(results) if offset + len(results) < match_total else None,
        "results": results,
    }


def check_catalog(repo_root: Path) -> BuildSummary:
    with tempfile.TemporaryDirectory(prefix="babylon-adr-check-") as temporary:
        summary = build_cache(repo_root, Path(temporary) / "catalog.sqlite3")
    if summary.membership_errors:
        raise SourceParseError(
            f"source/index membership differs in {summary.membership_errors} row(s)"
        )
    return summary


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, default=DEFAULT_REPO_ROOT)
    parser.add_argument("--cache", type=Path, default=DEFAULT_CACHE)
    commands = parser.add_subparsers(dest="command", required=True)
    exact = commands.add_parser("show", help="show bounded metadata for one ADR")
    exact.add_argument("adr_id")
    finder = commands.add_parser("search", help="search titles and stored source bytes")
    finder.add_argument("query")
    finder.add_argument("--limit", type=int, default=MAX_SEARCH_RESULTS)
    finder.add_argument("--offset", type=int, default=0)
    commands.add_parser("check", help="build and validate a temporary catalog")
    return parser


def _emit(payload: object, byte_limit: int = MAX_CLI_OUTPUT_BYTES) -> None:
    encoded = _encoded_payload(payload)
    if len(encoded) > byte_limit:
        raise CacheIntegrityError(f"CLI output exceeds {byte_limit} bytes")
    sys.stdout.buffer.write(encoded)


def main(argv: list[str] | None = None) -> int:
    args = _parser().parse_args(argv)
    repo = args.repo.resolve()
    cache = args.cache if args.cache.is_absolute() else repo / args.cache
    try:
        if args.command == "check":
            summary = check_catalog(repo)
            _emit(
                {
                    "status": "ok",
                    "records": summary.record_count,
                    "missing_status": summary.missing_status,
                    "conflicts": summary.conflicts,
                }
            )
        else:
            build_cache(repo, cache)
            if args.command == "show":
                _emit(show(cache, args.adr_id))
            elif args.command == "search":
                _emit(search(cache, args.query, args.limit, args.offset))
            else:
                raise QueryError(f"unsupported command: {args.command}")
    except (AdrCatalogError, OSError, sqlite3.Error) as error:
        print(f"adr-catalog: {error}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
