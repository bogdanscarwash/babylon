#!/usr/bin/env python3
"""Verify the bounded PER-275 H3 estate and governed artifact contract."""

from __future__ import annotations

import argparse
import ast
import hashlib
import json
import math
import os
import re
import stat
import struct
import sys
from pathlib import Path
from typing import Any, Final

import h3
import yaml
from yaml.constructor import ConstructorError
from yaml.nodes import MappingNode

MAX_U64: Final = (1 << 64) - 1
MAX_EXACT_F64_INT: Final = 1 << 53
MAX_CONTRACT_BYTES: Final = 262_144
MAX_VECTOR_BYTES: Final = 65_536
MAX_VECTOR_LINES: Final = 256
MAX_SOURCE_BYTES: Final = 1_048_576
MAX_ARTIFACT_BYTES: Final = 67_108_864
MAX_ARTIFACT_ROWS: Final = 65_536
MAX_CATALOG_SOURCES: Final = 128
MAX_CONSUMER_SOURCES: Final = 4_096
CANONICAL_H3_TEXT = re.compile(r"^[0-9a-f]{15}$")
LEGACY_H3_FIELDS: Final = {
    "h3_index",
    "home_hex",
    "parent_h3",
    "r7_parent",
    "res5_parent",
    "res6_parent",
    "source_h3",
    "target_h3",
    "workplace_dest",
}
EXPECTED_META: Final = {
    "contract": "H3EstateContractV1",
    "version": 1,
    "issue": "PER-275",
    "parent": "PER-21",
}
EXPECTED_HARD_GAPS: Final = {
    "census_place_identity",
    "census_place_geometry",
    "county_place_h3_overlap",
}
EXPECTED_HISTORICAL_CONTRACT_DIGEST: Final = (
    "b7fcde99da1d68d1f389f3e1f905d8434f97d2a9d620979001ce4d0f793e4db7"
)
TERMINAL_READER_CONTRACT: Final = Path("contracts/h3_reader_cutover_v1.yaml")


class _UniqueKeyLoader(yaml.SafeLoader):
    """Safe YAML loader that refuses duplicate mapping keys."""


def _construct_unique_mapping(
    loader: _UniqueKeyLoader, node: MappingNode, deep: bool = False
) -> dict[Any, Any]:
    mapping: dict[Any, Any] = {}
    for key_node, value_node in node.value:
        key = loader.construct_object(key_node, deep=deep)
        if key in mapping:
            raise ConstructorError(
                "while constructing a mapping",
                node.start_mark,
                f"found duplicate key {key!r}",
                key_node.start_mark,
            )
        mapping[key] = loader.construct_object(value_node, deep=deep)
    return mapping


_UniqueKeyLoader.add_constructor(
    yaml.resolver.BaseResolver.DEFAULT_MAPPING_TAG, _construct_unique_mapping
)


def _safe_load_unique(raw: bytes) -> Any:
    loader = _UniqueKeyLoader(raw)
    try:
        return loader.get_single_data()
    finally:
        loader.dispose()


class H3EstateContractRefusal(ValueError):
    """One typed refusal from the independent PER-275 verifier."""

    def __init__(self, code: str, detail: str) -> None:
        self.code = code
        self.detail = detail
        super().__init__(f"{code}: {detail}")


def _bounded_bytes(path: Path, maximum: int, overflow_code: str) -> bytes:
    try:
        size = path.stat().st_size
    except OSError as error:
        raise H3EstateContractRefusal("file_read", str(path)) from error
    if size > maximum:
        raise H3EstateContractRefusal(overflow_code, str(size))
    try:
        return path.read_bytes()
    except OSError as error:
        raise H3EstateContractRefusal("file_read", str(path)) from error


def _bounded_text(path: Path) -> str:
    raw = _bounded_bytes(path, MAX_SOURCE_BYTES, "source_too_large")
    try:
        return raw.decode("utf-8")
    except UnicodeDecodeError as error:
        raise H3EstateContractRefusal("source_utf8", str(path)) from error


def load_contract(path: Path) -> dict[str, Any]:
    """Load one bounded YAML contract mapping."""
    raw = _bounded_bytes(path, MAX_CONTRACT_BYTES, "contract_too_large")
    try:
        loaded = _safe_load_unique(raw)
    except yaml.YAMLError as error:
        raise H3EstateContractRefusal("invalid_contract", str(path)) from error
    if not isinstance(loaded, dict):
        raise H3EstateContractRefusal("invalid_contract", "root mapping")
    return loaded


def canonical_contract_digest(contract: dict[str, Any]) -> str:
    """Return the mapping-order-independent handoff digest."""
    try:
        payload = json.dumps(
            contract,
            ensure_ascii=True,
            separators=(",", ":"),
            sort_keys=True,
            allow_nan=False,
        ).encode("ascii")
    except (TypeError, ValueError) as error:
        raise H3EstateContractRefusal("contract_canonicalization", str(error)) from error
    return hashlib.sha256(b"babylon.h3-estate-contract.v1\0" + payload).hexdigest()


def checked_count(value: object) -> int:
    """Convert one finite non-negative integral artifact value to u64."""
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise H3EstateContractRefusal("invalid_count", repr(value))
    if isinstance(value, int):
        if not 0 <= value <= MAX_U64:
            raise H3EstateContractRefusal("invalid_count", repr(value))
        return value
    number = float(value)
    if (
        not math.isfinite(number)
        or number < 0
        or number > MAX_EXACT_F64_INT
        or not number.is_integer()
    ):
        raise H3EstateContractRefusal("invalid_count", repr(value))
    return int(number)


def checked_land_fraction(value: object, *, scale: int, tolerance: float = 1e-12) -> int:
    """Convert one finite [0,1] fixed-scale fraction to its scaled integer."""
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise H3EstateContractRefusal("invalid_land_fraction", repr(value))
    number = float(value)
    if not math.isfinite(number) or not 0.0 <= number <= 1.0:
        raise H3EstateContractRefusal("invalid_land_fraction", repr(value))
    if not isinstance(scale, int) or not 0 <= scale <= 12:
        raise H3EstateContractRefusal("invalid_land_scale", str(scale))
    if not math.isfinite(tolerance) or tolerance < 0:
        raise H3EstateContractRefusal("invalid_land_tolerance", str(tolerance))
    factor = 10**scale
    scaled = round(number * factor)
    if not math.isclose(number, scaled / factor, rel_tol=0.0, abs_tol=tolerance):
        raise H3EstateContractRefusal("land_fraction_scale", repr(value))
    return scaled


def _require_list(mapping: dict[str, Any], key: str) -> list[Any]:
    value = mapping.get(key)
    if not isinstance(value, list):
        raise H3EstateContractRefusal("contract_shape", key)
    return value


def _verify_contract_shape(contract: dict[str, Any]) -> None:
    if contract.get("meta") != EXPECTED_META:
        raise H3EstateContractRefusal("contract_shape", "meta")
    bounds = contract.get("bounds")
    if not isinstance(bounds, dict) or bounds != {
        "contract_bytes": MAX_CONTRACT_BYTES,
        "vector_bytes": MAX_VECTOR_BYTES,
        "vector_lines": MAX_VECTOR_LINES,
        "persistent_tables": 15,
        "current_views": 10,
        "temporary_shapes": 2,
        "runtime_consumers": 31,
        "artifacts": 6,
        "artifact_rows": MAX_ARTIFACT_ROWS,
        "artifact_bytes": MAX_ARTIFACT_BYTES,
    }:
        raise H3EstateContractRefusal("contract_shape", "bounds")
    estate = contract.get("estate")
    if not isinstance(estate, dict):
        raise H3EstateContractRefusal("contract_shape", "estate")
    tables = _require_list(estate, "persistent_tables")
    views = _require_list(estate, "current_views")
    temporary = _require_list(estate, "temporary_shapes")
    artifacts = _require_list(contract, "artifacts")
    if (len(tables), len(views), len(temporary), len(artifacts)) != (15, 10, 2, 6):
        raise H3EstateContractRefusal("contract_shape", "closed cardinalities")
    consumers = _require_list(estate, "runtime_consumer_census")
    if any(not isinstance(row, dict) for row in consumers):
        raise H3EstateContractRefusal("contract_shape", "runtime_consumer_census")
    if len(consumers) != 31 or consumers != sorted(
        consumers,
        key=lambda row: (row.get("path"), row.get("relation"), row.get("access")),
    ):
        raise H3EstateContractRefusal("contract_order", "runtime_consumer_census")
    if len({(row.get("path"), row.get("relation"), row.get("access")) for row in consumers}) != 31:
        raise H3EstateContractRefusal("contract_shape", "runtime_consumer_census")
    if any(row.get("access") not in {"read", "write"} for row in consumers):
        raise H3EstateContractRefusal("contract_shape", "runtime consumer access")
    for key, rows in (
        ("persistent_tables", tables),
        ("current_views", views),
        ("temporary_shapes", temporary),
        ("artifacts", artifacts),
    ):
        names = [row.get("name") for row in rows if isinstance(row, dict)]
        if len(names) != len(rows) or names != sorted(names) or len(names) != len(set(names)):
            raise H3EstateContractRefusal("contract_order", key)
    gaps = _require_list(contract, "hard_gaps")
    if len(gaps) != len(EXPECTED_HARD_GAPS) or any(not isinstance(row, dict) for row in gaps):
        raise H3EstateContractRefusal("contract_shape", "hard_gaps")
    gap_kinds = [row.get("kind") for row in gaps]
    if len(set(gap_kinds)) != len(gap_kinds) or set(gap_kinds) != EXPECTED_HARD_GAPS:
        raise H3EstateContractRefusal("contract_shape", "hard_gaps")
    if any(
        row.get("status") != "blocking"
        or not isinstance(row.get("required_authority"), str)
        or not row["required_authority"]
        for row in gaps
    ):
        raise H3EstateContractRefusal("contract_shape", "hard_gap status")
    handoff = contract.get("migration_handoff")
    if not isinstance(handoff, dict) or handoff.get("prohibited_names") != ["weight"]:
        raise H3EstateContractRefusal("generic_weight_prohibited", "migration_handoff")
    if handoff.get("required_measure_relations") != [
        "county_land_area_share",
        "county_population_count",
        "county_workplace_count",
        "place_land_area_share",
        "linear_feature_length_m",
    ]:
        raise H3EstateContractRefusal("contract_shape", "measure relations")
    policies = contract.get("conversion_policies")
    if not isinstance(policies, dict) or set(policies) != {
        "required_h3",
        "nullable_h3",
        "tagged_h3_or_external",
    }:
        raise H3EstateContractRefusal("contract_shape", "conversion policies")
    for table in tables:
        if not isinstance(table, dict):
            raise H3EstateContractRefusal("contract_shape", "table row")
        fields = _require_list(table, "fields")
        if not fields or not isinstance(table.get("retirement_condition"), str):
            raise H3EstateContractRefusal("contract_shape", str(table.get("name")))
        for field in fields:
            if not isinstance(field, dict) or field.get("kind") not in policies:
                raise H3EstateContractRefusal("contract_shape", "field conversion")
            if "compatibility_name" not in field:
                raise H3EstateContractRefusal("contract_shape", "field compatibility")
        tagged_fields = [field for field in fields if field.get("kind") == "tagged_h3_or_external"]
        if tagged_fields and (
            len(tagged_fields) != 1
            or not isinstance(table.get("tag_field"), str)
            or table.get("tag_legacy_type") != "TEXT"
            or table.get("tag_allowed_values") != ["external", "hex"]
            or table.get("external_literals") != ["canada", "rest_of_usa"]
        ):
            raise H3EstateContractRefusal("contract_shape", "tagged destination")
    for artifact in artifacts:
        if not isinstance(artifact, dict):
            raise H3EstateContractRefusal("contract_shape", "artifact row")
        for key in (
            "evidence_class",
            "vintage",
            "unit",
            "denominator",
            "coverage",
            "absence_zero",
            "canonical_order",
            "digest_framing",
            "semantic_digest",
        ):
            if key not in artifact:
                raise H3EstateContractRefusal("contract_shape", f"{artifact.get('name')}.{key}")


def _python_string_constants(path: Path) -> list[str]:
    text = _bounded_text(path)
    try:
        tree = ast.parse(text, filename=str(path))
    except SyntaxError as error:
        raise H3EstateContractRefusal("source_parse", str(path)) from error
    constants = [
        (node.lineno, node.col_offset, node.value)
        for node in ast.walk(tree)
        if isinstance(node, ast.Constant) and isinstance(node.value, str)
    ]
    return [value for _line, _column, value in sorted(constants)]


def _ddl_texts(path: Path) -> list[str]:
    if path.suffix == ".py":
        return [_without_sql_comments(text) for text in _python_string_constants(path)]
    return [_without_sql_comments(_bounded_text(path))]


def _without_sql_comments(text: str) -> str:
    def blank(match: re.Match[str]) -> str:
        return "".join("\n" if character == "\n" else " " for character in match.group())

    without_blocks = re.sub(r"/\*.*?\*/", blank, text, flags=re.DOTALL)
    return re.sub(r"--[^\n]*", blank, without_blocks)


CREATE_TABLE = re.compile(
    r"CREATE\s+TABLE(?:\s+IF\s+NOT\s+EXISTS)?\s+"
    r"(?:public\.)?([a-z_][a-z0-9_]*)\s*\((.*?)\n\s*\)\s*;?",
    re.IGNORECASE | re.DOTALL,
)
DROP_TABLE = re.compile(
    r"DROP\s+TABLE(?:\s+IF\s+EXISTS)?\s+(?:public\.)?([a-z_][a-z0-9_]*)\b",
    re.IGNORECASE,
)
ALTER_COLUMN_TYPE = re.compile(
    r"ALTER\s+TABLE(?:\s+IF\s+EXISTS)?\s+(?:public\.)?([a-z_][a-z0-9_]*)\s+"
    r"ALTER\s+(?:COLUMN\s+)?([a-z_][a-z0-9_]*)\s+(?:SET\s+DATA\s+)?TYPE\s+"
    r"([A-Z]+(?:\(\d+\))?)",
    re.IGNORECASE,
)
DROP_COLUMN = re.compile(
    r"ALTER\s+TABLE(?:\s+IF\s+EXISTS)?\s+(?:public\.)?([a-z_][a-z0-9_]*)\s+"
    r"DROP\s+(?:COLUMN\s+)?(?:IF\s+EXISTS\s+)?(?!CONSTRAINT\b)([a-z_][a-z0-9_]*)\b",
    re.IGNORECASE,
)
ADD_COLUMN = re.compile(
    r"ALTER\s+TABLE(?:\s+IF\s+EXISTS)?\s+(?:public\.)?([a-z_][a-z0-9_]*)\s+"
    r"ADD\s+(?:COLUMN\s+)?(?:IF\s+NOT\s+EXISTS\s+)?([a-z_][a-z0-9_]*)\s+"
    r"([A-Z]+(?:\(\d+\))?)",
    re.IGNORECASE,
)
DROP_CONSTRAINT = re.compile(
    r"ALTER\s+TABLE(?:\s+IF\s+EXISTS)?\s+(?:public\.)?([a-z_][a-z0-9_]*)\s+"
    r"DROP\s+CONSTRAINT(?:\s+IF\s+EXISTS)?\s+([a-z_][a-z0-9_]*)\b",
    re.IGNORECASE,
)
ADD_TAG_CHECK = re.compile(
    r"ALTER\s+TABLE(?:\s+IF\s+EXISTS)?\s+(?:public\.)?([a-z_][a-z0-9_]*)\s+"
    r"ADD\s+(?:CONSTRAINT\s+[a-z_][a-z0-9_]*\s+)?CHECK\s*\(\s*"
    r"([a-z_][a-z0-9_]*)\s+IN\s*\((.*?)\)\s*\)",
    re.IGNORECASE | re.DOTALL,
)


def _identity_columns(block: str) -> dict[str, str]:
    result: dict[str, str] = {}
    for line in re.split(r"[\n,]", block):
        match = re.match(
            r"\s*([a-z_][a-z0-9_]*)\s+([A-Z]+(?:\(\d+\))?)(?=\s|,|$)",
            line,
            re.IGNORECASE,
        )
        if match is None:
            continue
        name = match.group(1).lower()
        if name in LEGACY_H3_FIELDS:
            result[name] = match.group(2).upper()
    return result


def _tagged_discriminators(block: str) -> dict[str, dict[str, Any]]:
    columns = {
        match.group(1).lower(): match.group(2).upper()
        for match in re.finditer(
            r"(?m)^\s*([a-z_][a-z0-9_]*)\s+([A-Z]+(?:\(\d+\))?)(?=\s|,|$)",
            block,
            re.IGNORECASE,
        )
    }
    result: dict[str, dict[str, Any]] = {}
    for name, legacy_type in columns.items():
        if not name.endswith("_kind"):
            continue
        check = re.search(
            rf"CHECK\s*\(\s*{re.escape(name)}\s+IN\s*\((.*?)\)\s*\)",
            block,
            re.IGNORECASE | re.DOTALL,
        )
        values = None
        if check is not None:
            values = _quoted_allowlist(check.group(1))
        result[name] = {"legacy_type": legacy_type, "allowed_values": values}
    return result


def _quoted_allowlist(raw: str) -> list[str]:
    return sorted(value.replace("''", "'") for value in re.findall(r"'((?:''|[^'])*)'", raw))


def _table_shape(block: str) -> dict[str, Any]:
    return {
        "identity_fields": _identity_columns(block),
        "tagged_discriminators": _tagged_discriminators(block),
    }


def _table_events(text: str) -> list[tuple[int, str, tuple[Any, ...]]]:
    events: list[tuple[int, str, tuple[Any, ...]]] = []
    for match in CREATE_TABLE.finditer(text):
        events.append(
            (match.start(), "create", (match.group(1).lower(), _table_shape(match.group(2))))
        )
    for pattern, kind in (
        (DROP_TABLE, "drop_table"),
        (ALTER_COLUMN_TYPE, "alter_type"),
        (DROP_COLUMN, "drop_column"),
        (ADD_COLUMN, "add_column"),
        (DROP_CONSTRAINT, "drop_constraint"),
    ):
        for match in pattern.finditer(text):
            values = tuple(
                value.upper()
                if index == len(match.groups()) and kind in {"alter_type", "add_column"}
                else value.lower()
                for index, value in enumerate(match.groups(), start=1)
            )
            events.append((match.start(), kind, values))
    for match in ADD_TAG_CHECK.finditer(text):
        events.append(
            (
                match.start(),
                "add_tag_check",
                (match.group(1).lower(), match.group(2).lower(), _quoted_allowlist(match.group(3))),
            )
        )
    return sorted(events, key=lambda row: row[0])


def _catalog_source_paths(repo_root: Path) -> list[Path]:
    persistence = repo_root / "src" / "babylon" / "persistence"
    source_files = [
        persistence / "postgres_schema.py",
        *sorted((persistence / "migrations").glob("*.sql")),
    ]
    if len(source_files) > MAX_CATALOG_SOURCES:
        raise H3EstateContractRefusal("catalog_scan_bound", str(len(source_files)))
    return source_files


def discover_persistent_table_census(repo_root: Path) -> dict[str, dict[str, Any]]:
    """Discover the current H3 table shape after ordered catalog migrations."""
    discovered: dict[str, dict[str, Any]] = {}
    for path in _catalog_source_paths(repo_root):
        for text in _ddl_texts(path):
            for _position, kind, values in _table_events(text):
                name = values[0]
                if kind == "create":
                    shape = values[1]
                    if not shape["identity_fields"]:
                        continue
                    if name in discovered:
                        raise H3EstateContractRefusal("persistent_table_duplicate", name)
                    discovered[name] = shape
                elif kind == "drop_table":
                    discovered.pop(name, None)
                elif kind == "alter_type":
                    field, legacy_type = values[1:]
                    shape = discovered.get(name)
                    if shape is None:
                        continue
                    if field in shape["identity_fields"]:
                        shape["identity_fields"][field] = legacy_type
                    if field in shape["tagged_discriminators"]:
                        shape["tagged_discriminators"][field]["legacy_type"] = legacy_type
                elif kind == "drop_column":
                    field = values[1]
                    shape = discovered.get(name)
                    if shape is None:
                        continue
                    shape["identity_fields"].pop(field, None)
                    shape["tagged_discriminators"].pop(field, None)
                    if not shape["identity_fields"]:
                        discovered.pop(name)
                elif kind == "add_column":
                    field, legacy_type = values[1:]
                    if field in LEGACY_H3_FIELDS:
                        shape = discovered.setdefault(
                            name, {"identity_fields": {}, "tagged_discriminators": {}}
                        )
                        shape["identity_fields"][field] = legacy_type
                elif kind == "drop_constraint":
                    shape = discovered.get(name)
                    if shape is not None:
                        for discriminator in shape["tagged_discriminators"].values():
                            discriminator["allowed_values"] = None
                elif kind == "add_tag_check":
                    field, allowed_values = values[1:]
                    shape = discovered.get(name)
                    if shape is not None and field in shape["tagged_discriminators"]:
                        shape["tagged_discriminators"][field]["allowed_values"] = allowed_values
    return discovered


CREATE_VIEW = re.compile(
    r"CREATE(?:\s+OR\s+REPLACE)?\s+VIEW\s+(?:public\.)?([a-z_][a-z0-9_]*)\s+AS\s+"
    r"(.*?)(?=;\s*(?:\n|$)|\Z)",
    re.IGNORECASE | re.DOTALL,
)
DROP_VIEW = re.compile(
    r"DROP\s+VIEW(?:\s+IF\s+EXISTS)?\s+(?:public\.)?([a-z_][a-z0-9_]*)\b",
    re.IGNORECASE,
)


def discover_current_view_census(contract: dict[str, Any], repo_root: Path) -> set[str]:
    """Discover current H3-related views after ordered catalog migrations."""
    table_names = {row["name"] for row in contract["estate"]["persistent_tables"]}
    discovered: set[str] = set()
    for path in _catalog_source_paths(repo_root):
        for text in _ddl_texts(path):
            events = [
                (match.start(), "create", match.group(1).lower(), match.group(2))
                for match in CREATE_VIEW.finditer(text)
            ]
            events.extend(
                (match.start(), "drop", match.group(1).lower(), "")
                for match in DROP_VIEW.finditer(text)
            )
            for _position, kind, name, body in sorted(events, key=lambda row: row[0]):
                if kind == "drop":
                    discovered.discard(name)
                    continue
                lowered = body.lower()
                related = (
                    "h3_index" in lowered
                    or "cell_id" in lowered
                    or any(re.search(rf"\b{re.escape(table)}\b", lowered) for table in table_names)
                )
                if related:
                    discovered.add(name)
                else:
                    discovered.discard(name)
    return discovered


def _discover_temporary_shapes(
    contract: dict[str, Any], repo_root: Path
) -> dict[str, dict[str, str]]:
    relative = contract["estate"]["source_files"]["temporary_shapes"]
    path = repo_root / relative
    discovered: dict[str, dict[str, str]] = {}
    pattern = re.compile(
        r"CREATE\s+TEMP\s+TABLE\s+([a-z_][a-z0-9_]*)\s*\((.*?)\)\s+ON\s+COMMIT\s+DROP",
        re.IGNORECASE | re.DOTALL,
    )
    for text in _python_string_constants(path):
        for match in pattern.finditer(text):
            fields = _identity_columns(match.group(2))
            if fields:
                discovered[match.group(1).lower()] = fields
    return discovered


def _declared_fields(rows: list[dict[str, Any]]) -> dict[str, dict[str, str]]:
    return {
        row["name"]: {field["legacy_name"]: field["legacy_type"].upper() for field in row["fields"]}
        for row in rows
    }


def _declared_table_shapes(rows: list[dict[str, Any]]) -> dict[str, dict[str, Any]]:
    result: dict[str, dict[str, Any]] = {}
    for row in rows:
        tagged: dict[str, dict[str, Any]] = {}
        tag_field = row.get("tag_field")
        if tag_field is not None:
            tagged[tag_field] = {
                "legacy_type": row.get("tag_legacy_type"),
                "allowed_values": row.get("tag_allowed_values"),
            }
        result[row["name"]] = {
            "identity_fields": {
                field["legacy_name"]: field["legacy_type"].upper() for field in row["fields"]
            },
            "tagged_discriminators": tagged,
        }
    return result


def _verify_reader_owners(contract: dict[str, Any], repo_root: Path) -> None:
    rows = [
        *contract["estate"]["persistent_tables"],
        *contract["estate"]["current_views"],
    ]
    for row in rows:
        relation = row["name"]
        owners = row.get("reader_owners")
        if not isinstance(owners, list):
            raise H3EstateContractRefusal("reader_owner_shape", relation)
        for relative in owners:
            path = repo_root / relative
            text = _bounded_text(path)
            if relation not in text:
                raise H3EstateContractRefusal("reader_owner_drift", f"{relation}:{relative}")


def discover_runtime_consumer_census(
    contract: dict[str, Any], repo_root: Path
) -> list[dict[str, str]]:
    """Discover bounded raw-SQL reads and writes of the frozen estate."""
    relation_names = sorted(
        {
            row["name"]
            for row in (
                *contract["estate"]["persistent_tables"],
                *contract["estate"]["current_views"],
            )
        }
        | {
            child
            for child in [contract["estate"].get("partition", {}).get("default_child")]
            if isinstance(child, str)
        },
        key=lambda value: (-len(value), value),
    )
    alternatives = "|".join(re.escape(name) for name in relation_names)
    pattern = re.compile(
        rf"\b(FROM|JOIN|INSERT\s+INTO|UPDATE|DELETE\s+FROM)\s+(?:public\.)?({alternatives})\b",
        re.IGNORECASE,
    )
    excluded = {
        Path("src/babylon/persistence/postgres_schema.py"),
        Path("src/babylon/persistence/migrations/__init__.py"),
    }
    discovered: set[tuple[str, str, str]] = set()
    paths = sorted(
        {
            *repo_root.joinpath("src", "babylon").rglob("*.py"),
            *repo_root.joinpath("tools").rglob("*.py"),
        }
    )
    if len(paths) > MAX_CONSUMER_SOURCES:
        raise H3EstateContractRefusal("consumer_scan_bound", str(len(paths)))
    for path in paths:
        relative = path.relative_to(repo_root)
        if relative in excluded:
            continue
        for literal in _python_string_constants(path):
            for match in pattern.finditer(literal):
                operation = match.group(1).upper()
                access = "read" if operation in {"FROM", "JOIN"} else "write"
                discovered.add((relative.as_posix(), match.group(2).lower(), access))
    return [
        {"path": path, "relation": relation, "access": access}
        for path, relation, access in sorted(discovered)
    ]


def verify_source_inventory(contract: dict[str, Any], repo_root: Path) -> None:
    """Verify the frozen estate or its exact terminal reader handoff."""
    estate = contract.get("estate")
    if not isinstance(estate, dict):
        raise H3EstateContractRefusal("contract_shape", "estate")
    terminal_reader = repo_root / TERMINAL_READER_CONTRACT
    if terminal_reader.is_file():
        tables = _require_list(estate, "persistent_tables")
        views = _require_list(estate, "current_views")
        consumers = _require_list(estate, "runtime_consumer_census")
        if len(tables) != 15:
            raise H3EstateContractRefusal("persistent_table_census", "historical epoch-7 snapshot")
        if len(views) != 10:
            raise H3EstateContractRefusal("view_census", "historical epoch-7 snapshot")
        if len(consumers) != 31:
            raise H3EstateContractRefusal("runtime_consumer_census", "historical epoch-7 snapshot")
        if canonical_contract_digest(contract) != EXPECTED_HISTORICAL_CONTRACT_DIGEST:
            raise H3EstateContractRefusal(
                "historical_inventory_digest", EXPECTED_HISTORICAL_CONTRACT_DIGEST
            )
        from tools.verify_h3_reader_cutover_v1 import (
            H3ReaderCutoverRefusal,
            load_reader_cutover_contract,
            verify_reader_cutover_contract,
        )

        try:
            reader_contract = load_reader_cutover_contract(terminal_reader)
            verify_reader_cutover_contract(reader_contract, repo_root)
        except H3ReaderCutoverRefusal as error:
            raise H3EstateContractRefusal("terminal_reader_contract", str(error)) from error
        return

    declared_tables = _declared_table_shapes(_require_list(estate, "persistent_tables"))
    discovered_tables = discover_persistent_table_census(repo_root)
    if set(declared_tables) != set(discovered_tables):
        raise H3EstateContractRefusal(
            "persistent_table_census",
            f"declared={sorted(declared_tables)} discovered={sorted(discovered_tables)}",
        )
    for name in sorted(declared_tables):
        if declared_tables[name] != discovered_tables[name]:
            raise H3EstateContractRefusal(
                "persistent_field_census",
                f"{name}: declared={declared_tables[name]} discovered={discovered_tables[name]}",
            )
    declared_views = {row["name"] for row in _require_list(estate, "current_views")}
    discovered_views = discover_current_view_census(contract, repo_root)
    if declared_views != discovered_views:
        raise H3EstateContractRefusal(
            "view_census",
            f"declared={sorted(declared_views)} discovered={sorted(discovered_views)}",
        )
    declared_temporary = _declared_fields(_require_list(estate, "temporary_shapes"))
    discovered_temporary = _discover_temporary_shapes(contract, repo_root)
    if declared_temporary != discovered_temporary:
        raise H3EstateContractRefusal(
            "temporary_shape_census",
            f"declared={declared_temporary} discovered={discovered_temporary}",
        )
    domain = estate.get("unused_domain")
    if not isinstance(domain, dict):
        raise H3EstateContractRefusal("contract_shape", "unused_domain")
    domain_source = _bounded_text(repo_root / domain["source"])
    if re.search(r"CREATE\s+DOMAIN\s+h3index\b", domain_source, re.IGNORECASE) is None:
        raise H3EstateContractRefusal("domain_census", "h3index")
    if any(
        field["legacy_type"].lower() == "h3index"
        for table in estate["persistent_tables"]
        for field in table["fields"]
    ):
        raise H3EstateContractRefusal("domain_usage_drift", "h3index")
    partition = estate.get("partition")
    if (
        not isinstance(partition, dict)
        or partition.get("parent") != "dynamic_hex_state"
        or partition.get("default_child") != "dynamic_hex_state_default"
    ):
        raise H3EstateContractRefusal("partition_census", "contract")
    partition_source = _bounded_text(repo_root / estate["source_files"]["partition_registry"])
    if '"dynamic_hex_state"' not in partition_source or "session_id.hex" not in partition_source:
        raise H3EstateContractRefusal("partition_census", "source")
    _verify_reader_owners(contract, repo_root)
    declared_consumers = estate.get("runtime_consumer_census")
    if not isinstance(declared_consumers, list):
        raise H3EstateContractRefusal("contract_shape", "runtime_consumer_census")
    discovered_consumers = discover_runtime_consumer_census(contract, repo_root)
    if declared_consumers != discovered_consumers:
        raise H3EstateContractRefusal(
            "runtime_consumer_census",
            f"declared={declared_consumers} discovered={discovered_consumers}",
        )


def _load_yaml_mapping(path: Path, maximum: int) -> dict[str, Any]:
    raw = _bounded_bytes(path, maximum, "manifest_too_large")
    try:
        loaded = _safe_load_unique(raw)
    except yaml.YAMLError as error:
        raise H3EstateContractRefusal("manifest_parse", str(path)) from error
    if not isinstance(loaded, dict):
        raise H3EstateContractRefusal("manifest_shape", str(path))
    return loaded


def verify_artifact_manifest(contract: dict[str, Any], manifest_path: Path) -> None:
    """Prove every governed artifact pin against data-artifacts.yaml."""
    manifest = _load_yaml_mapping(manifest_path, MAX_SOURCE_BYTES)
    rows = manifest.get("artifacts")
    if manifest.get("version") != "2.0.0" or not isinstance(rows, list):
        raise H3EstateContractRefusal("manifest_shape", str(manifest_path))
    indexed = {
        row.get("name"): row
        for row in rows
        if isinstance(row, dict) and isinstance(row.get("name"), str)
    }
    for spec in contract["artifacts"]:
        row = indexed.get(spec["name"])
        expected = {
            "format": "parquet",
            "rows": spec["rows"],
            "sha256": spec["sha256"],
            "home": f"dist/data-artifacts/{spec['file']}",
        }
        if not isinstance(row, dict) or any(
            row.get(key) != value for key, value in expected.items()
        ):
            raise H3EstateContractRefusal("artifact_manifest_drift", spec["name"])
        if spec.get("manifest_version") != manifest["version"]:
            raise H3EstateContractRefusal("artifact_manifest_drift", f"{spec['name']}:version")


def _canonical_h3(text: object) -> bool:
    if not isinstance(text, str) or CANONICAL_H3_TEXT.fullmatch(text) is None:
        return False
    try:
        return bool(h3.is_valid_cell(text)) and h3.int_to_str(h3.str_to_int(text)) == text
    except (TypeError, ValueError, h3.H3BaseException):
        return False


def verify_h3_vectors(contract: dict[str, Any], vector_path: Path) -> dict[str, int]:
    """Execute the shared Rust/SQL H3 vector bytes independently in Python."""
    spec = contract.get("vectors")
    if not isinstance(spec, dict):
        raise H3EstateContractRefusal("vector_contract", "missing")
    raw = _bounded_bytes(vector_path, MAX_VECTOR_BYTES, "vector_too_large")
    if len(raw) != spec.get("bytes") or hashlib.sha256(raw).hexdigest() != spec.get("sha256"):
        raise H3EstateContractRefusal("vector_bytes", str(vector_path))
    try:
        lines = raw.decode("utf-8").splitlines()
    except UnicodeDecodeError as error:
        raise H3EstateContractRefusal("vector_utf8", str(vector_path)) from error
    if len(lines) > MAX_VECTOR_LINES:
        raise H3EstateContractRefusal("vector_lines", str(len(lines)))
    counts = {
        "valid": 0,
        "pentagons": 0,
        "invalid_raw": 0,
        "invalid_sql": 0,
        "invalid_text": 0,
        "invalid_ancestor": 0,
    }
    valid_by_resolution: dict[int, list[str]] = {resolution: [] for resolution in range(16)}
    pentagons_by_resolution: dict[int, set[str]] = {resolution: set() for resolution in range(16)}
    for line_number, line in enumerate(lines, start=1):
        if not line or line.startswith("#"):
            continue
        fields = line.split("|")
        kind = fields[0]
        try:
            if kind == "valid":
                if len(fields) != 9:
                    raise H3EstateContractRefusal("vector_shape", str(line_number))
                (
                    _,
                    label,
                    resolution_text,
                    text,
                    raw_hex,
                    sql_text,
                    bytes_hex,
                    parent,
                    chain_text,
                ) = fields
                resolution = int(resolution_text)
                raw_u64 = int(raw_hex, 16)
                sql_i64 = int(sql_text)
                chain = chain_text.split(",")
                if not _canonical_h3(text):
                    raise H3EstateContractRefusal("vector_validity", label)
                if not 0 <= resolution <= 15 or h3.get_resolution(text) != resolution:
                    raise H3EstateContractRefusal("vector_resolution", label)
                if h3.str_to_int(text) != raw_u64 or h3.int_to_str(raw_u64) != text:
                    raise H3EstateContractRefusal("vector_identity", label)
                if raw_u64 != sql_i64 or raw_u64 > (1 << 63) - 1:
                    raise H3EstateContractRefusal("vector_sql", label)
                if raw_u64.to_bytes(8, "big").hex() != bytes_hex:
                    raise H3EstateContractRefusal("vector_bytes_be", label)
                if len(chain) != resolution + 1 or chain[-1] != text:
                    raise H3EstateContractRefusal("vector_ancestor_chain", label)
                for ancestor_resolution, expected in enumerate(chain):
                    if h3.cell_to_parent(text, ancestor_resolution) != expected:
                        raise H3EstateContractRefusal("vector_ancestor", label)
                expected_parent = "" if resolution == 0 else h3.cell_to_parent(text, resolution - 1)
                if parent != expected_parent:
                    raise H3EstateContractRefusal("vector_parent", label)
                valid_by_resolution[resolution].append(text)
                counts["valid"] += 1
                if h3.is_pentagon(text):
                    counts["pentagons"] += 1
                    pentagons_by_resolution[resolution].add(text)
            elif kind == "invalid_raw":
                if len(fields) != 3:
                    raise H3EstateContractRefusal("vector_shape", str(line_number))
                raw_u64 = int(fields[2], 16)
                if _canonical_h3(format(raw_u64, "x")):
                    raise H3EstateContractRefusal("invalid_raw_accepted", fields[1])
                counts["invalid_raw"] += 1
            elif kind == "invalid_sql":
                if len(fields) != 3 or int(fields[2]) >= 0:
                    raise H3EstateContractRefusal("invalid_sql_accepted", fields[1])
                counts["invalid_sql"] += 1
            elif kind == "invalid_text":
                if len(fields) != 3 or _canonical_h3(fields[2]):
                    raise H3EstateContractRefusal("invalid_text_accepted", fields[1])
                counts["invalid_text"] += 1
            elif kind == "invalid_ancestor":
                if len(fields) != 4 or not _canonical_h3(fields[2]):
                    raise H3EstateContractRefusal("vector_shape", str(line_number))
                requested = int(fields[3])
                if requested <= h3.get_resolution(fields[2]) and requested <= 15:
                    raise H3EstateContractRefusal("invalid_ancestor_accepted", fields[1])
                counts["invalid_ancestor"] += 1
            else:
                raise H3EstateContractRefusal("vector_kind", kind)
        except (IndexError, TypeError, ValueError, h3.H3BaseException) as error:
            if isinstance(error, H3EstateContractRefusal):
                raise
            raise H3EstateContractRefusal("vector_parse", str(line_number)) from error
    expected_counts = spec.get("counts")
    if counts != expected_counts:
        raise H3EstateContractRefusal("vector_counts", f"{counts} != {expected_counts}")
    for resolution in range(16):
        if len(valid_by_resolution[resolution]) != 13:
            raise H3EstateContractRefusal("vector_resolution_count", str(resolution))
        if pentagons_by_resolution[resolution] != set(h3.get_pentagons(resolution)):
            raise H3EstateContractRefusal("vector_pentagons", str(resolution))
    return counts


def verified_artifact_bytes(path: Path, expected_size: int, expected_sha: str) -> bytes:
    """Read one regular artifact without following symlinks and verify its bytes."""
    if expected_size > MAX_ARTIFACT_BYTES:
        raise H3EstateContractRefusal("artifact_bound", str(expected_size))
    try:
        descriptor = os.open(path, os.O_RDONLY | os.O_CLOEXEC | os.O_NOFOLLOW)
    except OSError as error:
        raise H3EstateContractRefusal("artifact_open", str(path)) from error
    try:
        metadata = os.fstat(descriptor)
        if not stat.S_ISREG(metadata.st_mode) or metadata.st_size != expected_size:
            raise H3EstateContractRefusal("artifact_bytes", str(path))
        with os.fdopen(descriptor, "rb", closefd=True) as source:
            descriptor = -1
            payload = source.read(expected_size + 1)
    except OSError as error:
        raise H3EstateContractRefusal("artifact_read", str(path)) from error
    finally:
        if descriptor >= 0:
            os.close(descriptor)
    if len(payload) != expected_size or hashlib.sha256(payload).hexdigest() != expected_sha:
        raise H3EstateContractRefusal("artifact_bytes", str(path))
    return payload


def _semantic_digest(domain: bytes, rows: list[bytes]) -> str:
    digest = hashlib.sha256()
    digest.update(domain)
    digest.update(struct.pack(">Q", len(rows)))
    for row in rows:
        digest.update(row)
    return digest.hexdigest()


def verify_artifact_bytes(contract: dict[str, Any], artifact_root: Path) -> dict[str, Any]:
    """Hash-prove, then decode and validate the six governed Parquet artifacts."""
    pinned: dict[str, tuple[dict[str, Any], bytes]] = {}
    for spec in contract["artifacts"]:
        payload = verified_artifact_bytes(
            artifact_root / spec["file"], spec["bytes"], spec["sha256"]
        )
        pinned[spec["name"]] = (spec, payload)

    import pyarrow as pa  # type: ignore[import-untyped]  # hash-before-decode boundary
    import pyarrow.parquet as pq  # type: ignore[import-untyped]

    tables: dict[str, Any] = {}
    for name in sorted(pinned):
        spec, payload = pinned[name]
        try:
            parquet = pq.ParquetFile(pa.BufferReader(payload))
            table = parquet.read()
        except (OSError, pa.ArrowException) as error:
            raise H3EstateContractRefusal("artifact_decode", name) from error
        if parquet.num_row_groups != 1 or table.num_rows != spec["rows"]:
            raise H3EstateContractRefusal("artifact_rows", name)
        if table.num_rows > MAX_ARTIFACT_ROWS:
            raise H3EstateContractRefusal("artifact_bound", name)
        actual_schema = [
            {"name": field.name, "type": str(field.type), "nullable": field.nullable}
            for field in table.schema
        ]
        if actual_schema != spec["schema"]:
            raise H3EstateContractRefusal("artifact_schema", name)
        tables[name] = table

    bridge_rows = tables["bridge_county_h3"].to_pylist()
    if any(not _canonical_h3(row["h3_index"]) for row in bridge_rows):
        raise H3EstateContractRefusal("artifact_h3", "bridge_county_h3")
    bridge_order = [(row["resolution"], h3.str_to_int(row["h3_index"])) for row in bridge_rows]
    if bridge_order != sorted(bridge_order) or len({cell for _, cell in bridge_order}) != 48_764:
        raise H3EstateContractRefusal("artifact_order", "bridge_county_h3")
    resolution_counts = {
        resolution: sum(row["resolution"] == resolution for row in bridge_rows)
        for resolution in (5, 7)
    }
    if resolution_counts != {5: 3_192, 7: 45_572}:
        raise H3EstateContractRefusal("artifact_resolution", "bridge_county_h3")
    if any(
        (row["resolution"] == 5 and row["coverage_pct"] is not None)
        or (row["resolution"] == 7 and row["coverage_pct"] != 100.0)
        for row in bridge_rows
    ):
        raise H3EstateContractRefusal("legacy_coverage_drift", "bridge_county_h3")
    bridge_digest = _semantic_digest(
        b"babylon.h3.reference-source.v1\0",
        [struct.pack(">Q", cell) for _, cell in sorted(bridge_order)],
    )
    if bridge_digest != pinned["bridge_county_h3"][0]["semantic_digest"]:
        raise H3EstateContractRefusal("artifact_semantic_digest", "bridge_county_h3")

    county_rows = tables["dim_county"].to_pylist()
    geometry_rows = tables["dim_county_geometry"].to_pylist()
    county_ids = {row["county_id"] for row in county_rows}
    geometry_ids = {row["county_id"] for row in geometry_rows}
    county_fips = {row["fips"] for row in county_rows}
    bridge_county_ids = {row["county_id"] for row in bridge_rows}
    bridge_r7_county_ids = {row["county_id"] for row in bridge_rows if row["resolution"] == 7}
    if (
        len(county_ids) != 3_285
        or len(county_fips) != 3_285
        or any(not isinstance(value, int) for value in county_ids)
        or any(
            not isinstance(value, str) or re.fullmatch(r"\d{5}", value) is None
            for value in county_fips
        )
        or any(row["h3_res4"] is not None for row in county_rows)
        or len(geometry_ids) != 3_222
        or any(not isinstance(value, int) for value in geometry_ids)
        or not geometry_ids < county_ids
        or len(county_ids - geometry_ids) != 63
        or not bridge_county_ids <= county_ids
        or len(bridge_r7_county_ids) != 83
    ):
        raise H3EstateContractRefusal("county_extent", "dim_county_geometry")

    land_rows = tables["h3_res7_land_mask"].to_pylist()
    population_rows = tables["h3_res7_population"].to_pylist()
    workplace_rows = tables["h3_res7_workplace"].to_pylist()
    bridge_r7 = {row["h3_index"] for row in bridge_rows if row["resolution"] == 7}
    land_cells = {row["h3_index"] for row in land_rows}
    population_cells = {row["h3_index"] for row in population_rows}
    workplace_cells = {row["h3_index"] for row in workplace_rows}
    land_county_fips = {row["county_fips"] for row in land_rows}
    if (
        land_cells != bridge_r7
        or not population_cells <= land_cells
        or not workplace_cells <= land_cells
        or not land_county_fips <= county_fips
        or len(land_county_fips) != 83
    ):
        raise H3EstateContractRefusal("artifact_membership", "measure cohorts")
    for name, rows in (
        ("h3_res7_land_mask", land_rows),
        ("h3_res7_population", population_rows),
        ("h3_res7_workplace", workplace_rows),
    ):
        if any(
            not _canonical_h3(row["h3_index"]) or h3.get_resolution(row["h3_index"]) != 7
            for row in rows
        ):
            raise H3EstateContractRefusal("artifact_h3", name)
        ordered = [h3.str_to_int(row["h3_index"]) for row in rows]
        if ordered != sorted(ordered) or len(ordered) != len(set(ordered)):
            raise H3EstateContractRefusal("artifact_order", name)

    land_spec = pinned["h3_res7_land_mask"][0]
    land_frames: list[bytes] = []
    for row in land_rows:
        scaled = checked_land_fraction(
            row["land_fraction"],
            scale=land_spec["measure"]["scale"],
            tolerance=land_spec["measure"]["tolerance"],
        )
        county_fips = row["county_fips"]
        if not isinstance(county_fips, str) or re.fullmatch(r"\d{5}", county_fips) is None:
            raise H3EstateContractRefusal("county_fips", repr(county_fips))
        land_frames.append(
            struct.pack(">Q", h3.str_to_int(row["h3_index"]))
            + county_fips.encode("ascii")
            + struct.pack(">I", scaled)
        )
    land_digest = _semantic_digest(b"babylon.h3-estate.land-fraction.v1\0", land_frames)
    if land_digest != land_spec["semantic_digest"]:
        raise H3EstateContractRefusal("artifact_semantic_digest", "h3_res7_land_mask")

    receipts: dict[str, int] = {}
    for name, column, domain in (
        ("h3_res7_population", "population", b"babylon.h3-estate.population-count.v1\0"),
        ("h3_res7_workplace", "jobs", b"babylon.h3-estate.workplace-count.v1\0"),
    ):
        spec = pinned[name][0]
        frames: list[bytes] = []
        total = 0
        values: list[int] = []
        for row in tables[name].to_pylist():
            count = checked_count(row[column])
            if count == 0:
                raise H3EstateContractRefusal("zero_row", name)
            values.append(count)
            total += count
            frames.append(struct.pack(">QQ", h3.str_to_int(row["h3_index"]), count))
        measure = spec["measure"]
        if (
            total != measure["total"]
            or min(values) != measure["minimum"]
            or max(values) != measure["maximum"]
        ):
            raise H3EstateContractRefusal("artifact_measure", name)
        if _semantic_digest(domain, frames) != spec["semantic_digest"]:
            raise H3EstateContractRefusal("artifact_semantic_digest", name)
        receipts[name] = total
    return {
        "artifacts": len(pinned),
        "bridge_cells": len(bridge_rows),
        "land_cells": len(land_rows),
        "population": receipts["h3_res7_population"],
        "jobs": receipts["h3_res7_workplace"],
        "county_geometry_absences": 63,
    }


def verify_contract(contract: dict[str, Any], repo_root: Path) -> list[str]:
    """Verify the contract, source inventory, shared vectors, and artifact pins."""
    _verify_contract_shape(contract)
    verify_source_inventory(contract, repo_root)
    verify_artifact_manifest(contract, repo_root / "data-artifacts.yaml")
    vectors = contract["vectors"]
    verify_h3_vectors(contract, repo_root / vectors["path"])
    for artifact in contract["artifacts"]:
        source_manifest = artifact.get("source_manifest")
        if source_manifest is None:
            continue
        path = repo_root / source_manifest
        actual = hashlib.sha256(
            _bounded_bytes(path, MAX_SOURCE_BYTES, "source_too_large")
        ).hexdigest()
        if actual != artifact.get("source_manifest_sha256"):
            raise H3EstateContractRefusal("source_manifest_digest", artifact["name"])
    canonical_contract_digest(contract)
    return []


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--contract", type=Path, default=Path("contracts/h3_estate_contract_v1.yaml")
    )
    parser.add_argument("--repo-root", type=Path, default=Path("."))
    parser.add_argument("--artifact-root", type=Path)
    args = parser.parse_args()
    try:
        contract = load_contract(args.contract)
        verify_contract(contract, args.repo_root)
        digest = canonical_contract_digest(contract)
        print(f"H3EstateContractV1 verified: {digest}")
        if args.artifact_root is None:
            print("artifact bytes: not requested")
        else:
            receipt = verify_artifact_bytes(contract, args.artifact_root)
            print(f"artifact bytes: verified {json.dumps(receipt, sort_keys=True)}")
    except H3EstateContractRefusal as error:
        print(f"{error.code}: {error.detail}")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
