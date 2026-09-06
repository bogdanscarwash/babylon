#!/usr/bin/env python3
"""Verify the closed PER-280 canonical H3 reader inventory."""

from __future__ import annotations

import argparse
import ast
import hashlib
import json
import math
import re
import struct
import sys
from pathlib import Path
from typing import Any, Final
from uuid import UUID

import yaml
from yaml.constructor import ConstructorError
from yaml.nodes import MappingNode

MAX_CONTRACT_BYTES: Final = 65_536
MAX_SOURCE_BYTES: Final = 1_048_576
MAX_VECTOR_BYTES: Final = 131_072
MAX_VECTOR_ROWS: Final = 9
MAX_VECTOR_LINE_BYTES: Final = 32_768
MAX_VECTOR_TEXT_BYTES: Final = 4_096
MAX_I64: Final = (1 << 63) - 1
MIN_I64: Final = -(1 << 63)
EXPECTED_META: Final = {
    "contract": "H3ReaderCutoverV1",
    "version": 1,
    "issue": "PER-280",
    "parent": "PER-21",
}
EXPECTED_BOUNDS: Final = {
    "contract_bytes": MAX_CONTRACT_BYTES,
    "read_edges": 0,
    "source_files": 0,
    "runtime_consumer_edges": 0,
    "bootstrap_definition_authorities": 0,
    "dynamic_authorities": 0,
    "epoch7_read_edges": 13,
    "epoch7_source_files": 5,
    "epoch7_dynamic_authorities": 1,
    "current_views": 10,
    "persistent_tables": 15,
    "identity_fields": 21,
    "vector_bytes": MAX_VECTOR_BYTES,
    "vector_rows": MAX_VECTOR_ROWS,
    "vector_line_bytes": MAX_VECTOR_LINE_BYTES,
}
EXPECTED_PARITY_CASES: Final = (
    "sparse_current_and_asof_fill_forward",
    "county_state_national_aggregates",
    "runtime_trace_queries",
    "session_and_partition_isolation",
    "resolution_8_parent_reference_identity",
    "nullable_org_and_event_locations",
    "tagged_hex_and_external_lodes_destinations",
    "stable_ordering_and_pagination",
    "archive_export_and_query",
)
EXPECTED_PARITY_OPERATIONS: Final = (
    ("sparse_current_and_asof_fill_forward", "sparse_fill_forward"),
    ("county_state_national_aggregates", "value_aggregates"),
    ("runtime_trace_queries", "runtime_trace"),
    ("session_and_partition_isolation", "session_partition_isolation"),
    ("resolution_8_parent_reference_identity", "r8_parent_reference_identity"),
    ("nullable_org_and_event_locations", "nullable_locations"),
    ("tagged_hex_and_external_lodes_destinations", "tagged_destinations"),
    ("stable_ordering_and_pagination", "stable_pagination"),
    ("archive_export_and_query", "archive_round_trip"),
)
EXPECTED_ATOM_TYPES: Final = (
    "bool",
    "f64_bits",
    "h3_cell_id",
    "i64",
    "nullable_h3_cell_id",
    "nullable_text",
    "text",
    "uuid",
)
EXPECTED_VECTOR_PATH: Final = "contracts/h3_reader_cutover_v1_vectors.jsonl"
EXPECTED_VECTOR_BYTES: Final = 9_192
EXPECTED_VECTOR_SHA256: Final = "a3f4c00c69ab2b0e0a088ada32f7e6832ad948ec7119adc2042ac46df0a3320e"
EXPECTED_VECTOR_SEMANTIC_SHA256: Final = (
    "96e9d07f4a3882dea306758621d183211efd5470435a9768e695320b179002b6"
)
EXPECTED_FOUNDATION_BINDING: Final = {
    "contract": "contracts/michigan_dynamic_hex_foundation_v1.yaml",
    "contract_sha256": "83f4ced209fb361c48ce7500e6cf9d60d39a688f92b34d2ef48414ea396c996c",
    "artifact_sha256": "81ee8f8abbee6727655d52c6d56a6f2967af9dfdf01da53dd593da8339d650a4",
    "source_r7_digest": "7f8d126ee81356a60605013b4b1c23942a77a4b2d6f890125d6c938dae70228b",
    "base_reference_cohort_digest": (
        "92b21ff325bde67f26565f52882d3664daacd6d51423f2a588344da012fd4161"
    ),
    "reference_bundle_digest": ("84bbffa9b2388aa168c065e710a61313fbd46522d2022b628f0919ecffec9831"),
    "r8_child_parent": {
        "type": "MichiganH3R8ChildParentV1",
        "rows": 319_004,
        "child_field": "child_cell_id",
        "parent_field": "parent_r7_cell_id",
        "domain_utf8": "babylon.h3.reference-r8-child-parent.v1\0",
        "digest": "b5ebf405140f6f79ddbc44fa1005b195bed0bc28e0eacf2d8e1697cd9c839491",
    },
}
RESULT_DOMAIN: Final = b"babylon.h3-reader-parity-result.v1\0"
CORPUS_DOMAIN: Final = b"babylon.h3-reader-parity-vectors.v1\0"
# ADR251's economic observations do not expose the retired H3 estate. Keep
# this exact ownership census separate from the default-deny adapter prefix.
NON_H3_OBSERVER_SURFACES: Final = {
    "v_observer_economy_foundation_v1": (
        "observer_economy_v1.sql",
        frozenset({"src/observer_reader.rs", "src/reader.rs"}),
    ),
    "v_observer_county_economy_v1": (
        "observer_economy_v1.sql",
        frozenset({"src/observer_reader.rs", "src/reader.rs"}),
    ),
    "v_observer_material_state_v1": (
        "observer_material_v1.sql",
        frozenset(
            {
                "src/observer_material.rs",
                "src/observer_reader.rs",
                "src/reader.rs",
                "tests/observer_material_live.rs",  # proves raw-view denial for known preview
            }
        ),
    ),
}
LEGACY_IDENTITY_FIELDS: Final = {
    "h3_index",
    "home_hex",
    "parent_h3",
    "r7_parent",
    "res5_parent",
    "res6_parent",
    "source_h3",
    "target_h3",
}
SQL_START = re.compile(r"\b(?:SELECT|WITH)\b", re.IGNORECASE)


class _UniqueKeyLoader(yaml.SafeLoader):
    """YAML loader that refuses duplicate mapping keys."""


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


class H3ReaderCutoverRefusal(ValueError):
    """One closed refusal from the PER-280 source verifier."""

    def __init__(self, code: str, detail: str) -> None:
        self.code = code
        self.detail = detail
        super().__init__(f"{code}: {detail}")


def _bounded_bytes(path: Path, maximum: int) -> bytes:
    try:
        size = path.stat().st_size
    except OSError as error:
        raise H3ReaderCutoverRefusal("file_read", str(path)) from error
    if size > maximum:
        raise H3ReaderCutoverRefusal("file_too_large", f"{path}: {size}")
    try:
        return path.read_bytes()
    except OSError as error:
        raise H3ReaderCutoverRefusal("file_read", str(path)) from error


def _bounded_text(path: Path) -> str:
    try:
        return _bounded_bytes(path, MAX_SOURCE_BYTES).decode("utf-8")
    except UnicodeDecodeError as error:
        raise H3ReaderCutoverRefusal("source_utf8", str(path)) from error


def load_reader_cutover_contract(path: Path) -> dict[str, Any]:
    """Load one bounded, duplicate-key-free contract mapping."""
    raw = _bounded_bytes(path, MAX_CONTRACT_BYTES)
    loader = _UniqueKeyLoader(raw)
    try:
        loaded = loader.get_single_data()
    except yaml.YAMLError as error:
        raise H3ReaderCutoverRefusal("invalid_contract", str(path)) from error
    finally:
        loader.dispose()
    if not isinstance(loaded, dict):
        raise H3ReaderCutoverRefusal("invalid_contract", "root mapping")
    return loaded


def load_reader_parity_vectors(path: Path) -> list[dict[str, Any]]:
    """Load the exact bounded JSONL parity corpus without an unbounded read."""
    try:
        size = path.stat().st_size
    except OSError as error:
        raise H3ReaderCutoverRefusal("file_read", str(path)) from error
    if size > MAX_VECTOR_BYTES:
        raise H3ReaderCutoverRefusal("vector_file_too_large", f"{path}: {size}")

    rows: list[dict[str, Any]] = []
    try:
        with path.open("rb") as stream:
            for index in range(MAX_VECTOR_ROWS + 1):
                raw = stream.readline(MAX_VECTOR_LINE_BYTES + 2)
                if raw == b"":
                    break
                if index == MAX_VECTOR_ROWS:
                    raise H3ReaderCutoverRefusal("too_many_vectors", str(index + 1))
                content = raw.removesuffix(b"\n").removesuffix(b"\r")
                if not content or len(content) > MAX_VECTOR_LINE_BYTES:
                    raise H3ReaderCutoverRefusal(
                        "invalid_vector_line", f"{index + 1}: {len(content)}"
                    )
                try:
                    value = json.loads(content)
                except (UnicodeDecodeError, json.JSONDecodeError) as error:
                    raise H3ReaderCutoverRefusal("invalid_vector_json", str(index + 1)) from error
                if not isinstance(value, dict):
                    raise H3ReaderCutoverRefusal("vector_shape", str(index + 1))
                rows.append(value)
    except OSError as error:
        raise H3ReaderCutoverRefusal("file_read", str(path)) from error
    return rows


def _canonical_json(value: object) -> bytes:
    try:
        return json.dumps(
            value,
            ensure_ascii=True,
            separators=(",", ":"),
            sort_keys=True,
            allow_nan=False,
        ).encode("ascii")
    except (TypeError, ValueError) as error:
        raise H3ReaderCutoverRefusal("vector_canonicalization", str(error)) from error


def _canonical_decimal(value: object, *, nullable: bool) -> str | None:
    if nullable and value is None:
        return None
    if (
        not isinstance(value, str)
        or not value.isascii()
        or not value.isdecimal()
        or value.startswith("0")
    ):
        raise H3ReaderCutoverRefusal("vector_atom", "h3_cell_id")
    parsed = int(value)
    if parsed <= 0 or parsed > MAX_I64 or str(parsed) != value:
        raise H3ReaderCutoverRefusal("vector_atom", "h3_cell_id")
    return value


def _canonical_atom(atom: object) -> dict[str, Any]:
    if not isinstance(atom, dict) or set(atom) != {"type", "value"}:
        raise H3ReaderCutoverRefusal("vector_atom", "shape")
    atom_type = atom.get("type")
    value = atom.get("value")
    if atom_type not in EXPECTED_ATOM_TYPES:
        raise H3ReaderCutoverRefusal("vector_atom", f"type={atom_type!r}")
    if atom_type == "bool":
        if not isinstance(value, bool):
            raise H3ReaderCutoverRefusal("vector_atom", "bool")
    elif atom_type == "i64":
        if isinstance(value, bool) or not isinstance(value, int) or not MIN_I64 <= value <= MAX_I64:
            raise H3ReaderCutoverRefusal("vector_atom", "i64")
    elif atom_type == "h3_cell_id":
        value = _canonical_decimal(value, nullable=False)
    elif atom_type == "nullable_h3_cell_id":
        value = _canonical_decimal(value, nullable=True)
    elif atom_type == "f64_bits":
        if (
            not isinstance(value, str)
            or len(value) != 16
            or value.lower() != value
            or re.fullmatch(r"[0-9a-f]{16}", value) is None
        ):
            raise H3ReaderCutoverRefusal("vector_atom", "f64_bits")
        decoded = struct.unpack(">d", bytes.fromhex(value))[0]
        if not math.isfinite(decoded) or value == "8000000000000000":
            raise H3ReaderCutoverRefusal("vector_atom", "f64_bits")
    elif atom_type == "uuid":
        if not isinstance(value, str):
            raise H3ReaderCutoverRefusal("vector_atom", "uuid")
        try:
            parsed_uuid = UUID(value)
        except ValueError as error:
            raise H3ReaderCutoverRefusal("vector_atom", "uuid") from error
        if str(parsed_uuid) != value:
            raise H3ReaderCutoverRefusal("vector_atom", "uuid")
    elif atom_type in {"text", "nullable_text"}:
        if value is None and atom_type == "nullable_text":
            pass
        elif (
            not isinstance(value, str)
            or "\x00" in value
            or len(value.encode("utf-8")) > MAX_VECTOR_TEXT_BYTES
        ):
            raise H3ReaderCutoverRefusal("vector_atom", atom_type)
    return {"type": atom_type, "value": value}


def _canonical_expected_sets(value: object) -> dict[str, list[dict[str, dict[str, Any]]]]:
    if not isinstance(value, dict) or not value:
        raise H3ReaderCutoverRefusal("vector_result_shape", "sets")
    normalized: dict[str, list[dict[str, dict[str, Any]]]] = {}
    for set_name, rows in value.items():
        if (
            not isinstance(set_name, str)
            or not set_name
            or "\x00" in set_name
            or not isinstance(rows, list)
        ):
            raise H3ReaderCutoverRefusal("vector_result_shape", "result set")
        normalized_rows: list[dict[str, dict[str, Any]]] = []
        for row in rows:
            if not isinstance(row, dict) or not row:
                raise H3ReaderCutoverRefusal("vector_result_shape", f"{set_name}: row")
            if any(not isinstance(name, str) or not name or "\x00" in name for name in row):
                raise H3ReaderCutoverRefusal("vector_result_shape", f"{set_name}: column")
            normalized_rows.append({name: _canonical_atom(atom) for name, atom in row.items()})
        normalized[set_name] = normalized_rows
    return normalized


def _canonical_contract_digest(contract: dict[str, Any]) -> str:
    try:
        payload = json.dumps(
            contract,
            ensure_ascii=True,
            separators=(",", ":"),
            sort_keys=True,
            allow_nan=False,
        ).encode("ascii")
    except (TypeError, ValueError) as error:
        raise H3ReaderCutoverRefusal("contract_canonicalization", str(error)) from error
    return hashlib.sha256(b"babylon.h3-estate-contract.v1\0" + payload).hexdigest()


def _contract_relations(contract: dict[str, Any]) -> set[str]:
    edges = contract.get("read_edges")
    epoch7_edges = contract.get("epoch7_read_edges")
    if not isinstance(edges, list) or not isinstance(epoch7_edges, list):
        raise H3ReaderCutoverRefusal("contract_shape", "reader edges")
    relations = {
        str(edge["relation"])
        for edge in [*edges, *epoch7_edges]
        if isinstance(edge, dict) and isinstance(edge.get("relation"), str)
    }
    bootstrap = contract.get("bootstrap_definition_authorities", [])
    if not isinstance(bootstrap, list):
        raise H3ReaderCutoverRefusal("contract_shape", "bootstrap_definition_authorities")
    relations.update(
        str(authority["relation"])
        for authority in bootstrap
        if isinstance(authority, dict) and isinstance(authority.get("relation"), str)
    )
    return relations


def _relation_matches(sql: str, relation: str) -> list[tuple[str | None, str]]:
    pattern = re.compile(
        rf"\b(?:FROM|JOIN)\s+(?:(?P<schema>[A-Za-z_][A-Za-z0-9_]*)\s*\.\s*)?"
        rf"(?P<relation>{re.escape(relation)})\b",
        re.IGNORECASE,
    )
    return [(match.group("schema"), match.group("relation")) for match in pattern.finditer(sql)]


def inspect_sql_literal(contract: dict[str, Any], path: Path, sql: str) -> list[tuple[str, str]]:
    """Inspect one SQL literal and return its canonical governed reads."""
    relative = path.as_posix()
    historical_roots = contract.get("scan", {}).get("historical_migration_roots", [])
    if any(
        relative == str(prefix) or relative.startswith(f"{prefix}/") for prefix in historical_roots
    ):
        return []
    lowered = sql.lower()
    observer_names = re.findall(r"\bv_observer_[a-z0-9_]+\b", lowered)
    for name in observer_names:
        entry = NON_H3_OBSERVER_SURFACES.get(name)
        if entry is None or relative not in {
            f"rust/crates/babylon-persistence/{owner}" for owner in entry[1]
        }:
            raise H3ReaderCutoverRefusal("compatibility_read", str(path))
        if any(schema != "public" for schema, _ in _relation_matches(lowered, name)):
            raise H3ReaderCutoverRefusal("compatibility_read", str(path))
    remaining = lowered
    for name in observer_names:
        remaining = re.sub(rf"\b{re.escape(name)}\b", "", remaining)
    if "v_compat_" in lowered or "v_observer_" in remaining:
        raise H3ReaderCutoverRefusal("compatibility_read", str(path))

    reads: list[tuple[str, str]] = []
    for relation in sorted(_contract_relations(contract)):
        matches = _relation_matches(sql, relation)
        for schema, _ in matches:
            if schema is None or schema.lower() != "public":
                raise H3ReaderCutoverRefusal("unqualified_relation", f"{path}: {relation}")
            reads.append((relation, f"public.{relation}"))
    if not reads:
        return []

    if re.search(r"\bSELECT\s+(?:DISTINCT\s+)?(?:[A-Za-z_]\w*\s*\.\s*)?\*", sql, re.I):
        raise H3ReaderCutoverRefusal("wildcard_identity_read", str(path))

    without_presentation_aliases = re.sub(
        r"\bAS\s+(?:h3_index|home_hex|parent_h3|r7_parent|res5_parent|res6_parent|source_h3|target_h3)\b",
        "",
        sql,
        flags=re.IGNORECASE,
    )
    for field in sorted(LEGACY_IDENTITY_FIELDS):
        if re.search(rf"\b{re.escape(field)}\b", without_presentation_aliases, re.I):
            raise H3ReaderCutoverRefusal("legacy_identity_read", f"{path}: {field}")

    if re.search(r"\bworkplace_dest\b", lowered):
        required = ("workplace_cell_id", "workplace_dest_kind")
        if any(not re.search(rf"\b{name}\b", lowered) for name in required):
            raise H3ReaderCutoverRefusal("legacy_identity_read", f"{path}: workplace_dest")

    return list(dict.fromkeys(reads))


def _python_sql_literals(path: Path, source: str) -> list[str]:
    try:
        tree = ast.parse(source, filename=str(path))
    except SyntaxError as error:
        raise H3ReaderCutoverRefusal("source_parse", str(path)) from error
    docstrings: set[int] = set()
    for node in ast.walk(tree):
        body = getattr(node, "body", None)
        if (
            isinstance(body, list)
            and body
            and isinstance(body[0], ast.Expr)
            and isinstance(body[0].value, ast.Constant)
            and isinstance(body[0].value.value, str)
        ):
            docstrings.add(id(body[0].value))
    literals: list[str] = []
    for node in ast.walk(tree):
        if isinstance(node, ast.Constant) and isinstance(node.value, str):
            if id(node) not in docstrings and SQL_START.search(node.value):
                literals.append(node.value)
        elif isinstance(node, ast.JoinedStr):
            text = "".join(
                part.value
                for part in node.values
                if isinstance(part, ast.Constant) and isinstance(part.value, str)
            )
            if SQL_START.search(text):
                literals.append(text)
    return literals


def _rust_sql_literals(source: str) -> list[str]:
    strings = re.findall(r'r#*"(.*?)"#*|"((?:\\.|[^"\\])*)"', source, re.DOTALL)
    literals = [first or second for first, second in strings]
    return [literal for literal in literals if SQL_START.search(literal)]


def _source_sql_literals(path: Path, source: str) -> list[str]:
    if path.suffix == ".py":
        return _python_sql_literals(path, source)
    if path.suffix == ".rs":
        return _rust_sql_literals(source)
    return []


def _python_tree(path: Path, source: str) -> ast.Module:
    try:
        return ast.parse(source, filename=str(path))
    except SyntaxError as error:
        raise H3ReaderCutoverRefusal("source_parse", str(path)) from error


def _module_string_collection(tree: ast.Module, symbol: str) -> tuple[str, ...]:
    value_node: ast.expr | None = None
    for node in tree.body:
        if isinstance(node, ast.AnnAssign) and isinstance(node.target, ast.Name):
            if node.target.id == symbol:
                value_node = node.value
                break
        elif isinstance(node, ast.Assign) and any(
            isinstance(target, ast.Name) and target.id == symbol for target in node.targets
        ):
            value_node = node.value
            break
    if value_node is None:
        raise H3ReaderCutoverRefusal("dynamic_authority_drift", f"missing {symbol}")
    try:
        value = ast.literal_eval(value_node)
    except (TypeError, ValueError) as error:
        raise H3ReaderCutoverRefusal("dynamic_authority_drift", f"nonliteral {symbol}") from error
    if not isinstance(value, tuple | list) or not all(isinstance(item, str) for item in value):
        raise H3ReaderCutoverRefusal("dynamic_authority_drift", f"non-string {symbol}")
    return tuple(value)


def _render_joined_string(node: ast.JoinedStr) -> str:
    parts: list[str] = []
    for part in node.values:
        if isinstance(part, ast.Constant) and isinstance(part.value, str):
            parts.append(part.value)
        elif isinstance(part, ast.FormattedValue) and isinstance(part.value, ast.Name):
            parts.append(f"{{{part.value.id}}}")
        else:
            parts.append("{expression}")
    return " ".join("".join(parts).split()).lower()


def _dynamic_select_templates(tree: ast.AST, variable: str) -> list[str]:
    marker = f"{{{variable}}}"
    return [
        rendered
        for node in ast.walk(tree)
        if isinstance(node, ast.JoinedStr)
        for rendered in [_render_joined_string(node)]
        if marker in rendered and SQL_START.search(rendered) and "from" in rendered
    ]


def _function(tree: ast.Module, name: str) -> ast.FunctionDef | ast.AsyncFunctionDef:
    for node in tree.body:
        if isinstance(node, ast.FunctionDef | ast.AsyncFunctionDef) and node.name == name:
            return node
    raise H3ReaderCutoverRefusal("dynamic_authority_drift", f"missing function {name}")


def _function_sql(tree: ast.Module, name: str) -> str:
    node = _function(tree, name)
    literals = [
        literal
        for child in node.body
        for literal in _source_sql_literals(Path("authority.py"), ast.unparse(child))
    ]
    return " ".join(" ".join(literal.split()) for literal in literals).lower()


def _function_call_names(tree: ast.Module, name: str) -> set[str]:
    node = _function(tree, name)
    return {
        call.func.id
        for call in ast.walk(node)
        if isinstance(call, ast.Call) and isinstance(call.func, ast.Name)
    }


def _parent_relation_names(parent: dict[str, Any]) -> set[str]:
    estate = parent.get("estate")
    if not isinstance(estate, dict):
        raise H3ReaderCutoverRefusal("parent_contract_shape", "estate")
    rows = [*estate.get("persistent_tables", []), *estate.get("current_views", [])]
    names = {
        str(row["name"])
        for row in rows
        if isinstance(row, dict) and isinstance(row.get("name"), str)
    }
    partition = estate.get("partition")
    if isinstance(partition, dict) and isinstance(partition.get("default_child"), str):
        names.add(str(partition["default_child"]))
    return names


def _parent_session_relations(parent: dict[str, Any]) -> set[str]:
    estate = parent.get("estate")
    if not isinstance(estate, dict):
        raise H3ReaderCutoverRefusal("parent_contract_shape", "estate")
    tables = estate.get("persistent_tables")
    if not isinstance(tables, list):
        raise H3ReaderCutoverRefusal("parent_contract_shape", "persistent_tables")
    return {
        str(row["name"])
        for row in tables
        if isinstance(row, dict)
        and isinstance(row.get("name"), str)
        and isinstance(row.get("scope"), str)
        and str(row["scope"]).startswith("session")
    }


def _parent_persistent_relation_names(parent: dict[str, Any]) -> set[str]:
    estate = parent.get("estate")
    if not isinstance(estate, dict):
        raise H3ReaderCutoverRefusal("parent_contract_shape", "estate")
    tables = estate.get("persistent_tables")
    if not isinstance(tables, list):
        raise H3ReaderCutoverRefusal("parent_contract_shape", "persistent_tables")
    names = {
        str(row["name"])
        for row in tables
        if isinstance(row, dict) and isinstance(row.get("name"), str)
    }
    if len(names) != EXPECTED_BOUNDS["persistent_tables"]:
        raise H3ReaderCutoverRefusal("parent_contract_shape", "persistent relation count")
    return names


def _load_parent_contract(contract: dict[str, Any], root: Path) -> dict[str, Any]:
    authority = contract.get("authority")
    if not isinstance(authority, dict):
        raise H3ReaderCutoverRefusal("contract_shape", "authority")
    parent_path = root / str(authority.get("parent_contract", ""))
    parent = load_reader_cutover_contract(parent_path)
    if _canonical_contract_digest(parent) != authority.get("parent_contract_sha256"):
        raise H3ReaderCutoverRefusal("parent_digest", str(parent_path))
    return parent


def _verify_foundation_binding(contract: dict[str, Any], root: Path) -> tuple[list[str], str, str]:
    authority = contract.get("authority")
    if not isinstance(authority, dict):
        raise H3ReaderCutoverRefusal("contract_shape", "authority")
    binding = authority.get("foundation_binding")
    if binding != EXPECTED_FOUNDATION_BINDING:
        raise H3ReaderCutoverRefusal("foundation_binding", "reader binding")

    foundation_path = root / EXPECTED_FOUNDATION_BINDING["contract"]
    raw = _bounded_bytes(foundation_path, MAX_CONTRACT_BYTES)
    if hashlib.sha256(raw).hexdigest() != binding["contract_sha256"]:
        raise H3ReaderCutoverRefusal("foundation_binding", "contract digest")
    foundation = load_reader_cutover_contract(foundation_path)
    try:
        wire = foundation["wire"]
        dynamic_r7 = foundation["dynamic_r7"]
        reference_bundle = foundation["reference_bundle"]
        r8_manifest = reference_bundle["r8_child_parent"]
        artifact = foundation["artifact"]
        audited = foundation["audited_identities"]
        rust_binding = foundation["rust_binding"]
        r8_binding = binding["r8_child_parent"]
        manifest_values = {
            "artifact_sha256": artifact["sha256"],
            "source_r7_digest": dynamic_r7["source_digest"],
            "base_reference_cohort_digest": reference_bundle["base_cohort_digest"],
            "reference_bundle_digest": reference_bundle["composite_digest"],
        }
        bound_values = {
            name: binding[name]
            for name in (
                "artifact_sha256",
                "source_r7_digest",
                "base_reference_cohort_digest",
                "reference_bundle_digest",
            )
        }
        manifest_r8 = {
            "type": rust_binding["r8_row_type"],
            "rows": r8_manifest["row_count"],
            "child_field": rust_binding["child_accessor"],
            "parent_field": rust_binding["parent_accessor"],
            "domain_utf8": wire["r8_section_domain_utf8"],
            "digest": r8_manifest["section_digest"],
        }
        r7_cells = audited["r7_cells"]
        r8_child = audited["r8_child"]
        r8_parent = audited["r8_parent"]
    except (KeyError, TypeError) as error:
        raise H3ReaderCutoverRefusal("foundation_binding", "manifest shape") from error
    if bound_values != manifest_values or r8_binding != manifest_r8:
        raise H3ReaderCutoverRefusal("foundation_binding", "manifest values")
    if (
        r7_cells != ["872664800ffffff", "872664801ffffff", "872664802ffffff"]
        or r8_child != "8826648001fffff"
        or r8_parent != "872664800ffffff"
    ):
        raise H3ReaderCutoverRefusal("foundation_binding", "audited identities")
    return (
        [str(int(cell, 16)) for cell in r7_cells],
        str(int(r8_child, 16)),
        str(int(r8_parent, 16)),
    )


def _verify_parity_contract_metadata(contract: dict[str, Any], root: Path) -> None:
    if tuple(contract.get("parity_cases", ())) != EXPECTED_PARITY_CASES:
        raise H3ReaderCutoverRefusal("contract_shape", "parity_cases")
    parity = contract.get("parity_vectors")
    if not isinstance(parity, dict):
        raise H3ReaderCutoverRefusal("contract_shape", "parity_vectors")
    expected_operations = [
        {"case": case_id, "operation": operation}
        for case_id, operation in EXPECTED_PARITY_OPERATIONS
    ]
    expected_coverage = {
        "current_views": "exact 10-name union",
        "persistent_relations": "exact 15-name parent-estate union",
        "runtime_reader_edges": "exact 13-edge union",
    }
    expected_execution = {
        "backend_call": "execute_reader_case(operation, decoded_inputs)",
        "result_shape": "named ordered row sets with closed typed atoms",
        "authority": "read-only production readers or projection-only SQL",
        "prohibited": [
            "Formula reconstruction",
            "Join reconstruction",
            "Game-state writes",
            "Compatibility or fallback reads",
        ],
    }
    if set(parity) != {
        "path",
        "bytes",
        "sha256",
        "semantic_sha256",
        "result_domain_ascii_nul",
        "corpus_domain_ascii_nul",
        "operations",
        "atom_types",
        "coverage",
        "execution",
    }:
        raise H3ReaderCutoverRefusal("contract_shape", "parity vector keys")
    if (
        parity["path"] != EXPECTED_VECTOR_PATH
        or parity["bytes"] != EXPECTED_VECTOR_BYTES
        or parity["sha256"] != EXPECTED_VECTOR_SHA256
        or parity["semantic_sha256"] != EXPECTED_VECTOR_SEMANTIC_SHA256
        or parity["result_domain_ascii_nul"] != "babylon.h3-reader-parity-result.v1"
        or parity["corpus_domain_ascii_nul"] != "babylon.h3-reader-parity-vectors.v1"
        or parity["operations"] != expected_operations
        or parity["atom_types"] != list(EXPECTED_ATOM_TYPES)
        or parity["coverage"] != expected_coverage
        or parity["execution"] != expected_execution
    ):
        raise H3ReaderCutoverRefusal("contract_shape", "parity vector authority")

    vector_path = root / EXPECTED_VECTOR_PATH
    raw = _bounded_bytes(vector_path, MAX_VECTOR_BYTES)
    if len(raw) != EXPECTED_VECTOR_BYTES or hashlib.sha256(raw).hexdigest() != parity["sha256"]:
        raise H3ReaderCutoverRefusal("vector_file_digest", str(vector_path))


def _ordered_unique_strings(value: object, detail: str) -> list[str]:
    if (
        not isinstance(value, list)
        or any(not isinstance(item, str) or not item for item in value)
        or value != sorted(value)
        or len(value) != len(set(value))
    ):
        raise H3ReaderCutoverRefusal("vector_coverage_shape", detail)
    return value


def _reader_edges(value: object, detail: str) -> list[tuple[str, str]]:
    if not isinstance(value, list):
        raise H3ReaderCutoverRefusal("vector_coverage_shape", detail)
    edges: list[tuple[str, str]] = []
    for edge in value:
        if (
            not isinstance(edge, dict)
            or set(edge) != {"path", "relation"}
            or not isinstance(edge.get("path"), str)
            or not isinstance(edge.get("relation"), str)
        ):
            raise H3ReaderCutoverRefusal("vector_coverage_shape", detail)
        edges.append((edge["path"], edge["relation"]))
    if edges != sorted(edges) or len(edges) != len(set(edges)):
        raise H3ReaderCutoverRefusal("vector_coverage_shape", detail)
    return edges


def verify_reader_parity_vectors(
    contract: dict[str, Any], vectors: list[dict[str, Any]], root: Path
) -> list[str]:
    """Verify the nine language-neutral cases and their closed estate coverage."""
    _verify_contract_shape(contract)
    _verify_parity_contract_metadata(contract, root)
    foundation_r7, foundation_r8_child, foundation_r8_parent = _verify_foundation_binding(
        contract, root
    )
    parent = _load_parent_contract(contract, root)

    if len(vectors) != MAX_VECTOR_ROWS:
        raise H3ReaderCutoverRefusal("parity_case_coverage", str(len(vectors)))
    ids = [row.get("id") for row in vectors]
    if any(not isinstance(row_id, str) or not row_id for row_id in ids):
        raise H3ReaderCutoverRefusal("vector_shape", "id")
    if len(set(ids)) != len(ids):
        raise H3ReaderCutoverRefusal("duplicate_vector_id", repr(ids))
    if tuple(ids) != EXPECTED_PARITY_CASES:
        raise H3ReaderCutoverRefusal("parity_case_coverage", repr(ids))
    rows_by_id = {str(row["id"]): row for row in vectors}
    pagination_sets = rows_by_id["stable_ordering_and_pagination"]["expected"]["sets"]
    pagination_cells = [
        row["cell_id"]["value"]
        for row in [*pagination_sets["page_one"], *pagination_sets["page_two"]]
    ]
    r8_row = rows_by_id["resolution_8_parent_reference_identity"]["expected"]["sets"][
        "parent_rows"
    ][0]
    if pagination_cells != foundation_r7 or (
        r8_row["cell_id"]["value"],
        r8_row["parent_cell_id"]["value"],
    ) != (foundation_r8_child, foundation_r8_parent):
        raise H3ReaderCutoverRefusal("foundation_binding", "parity identities")

    operations = dict(EXPECTED_PARITY_OPERATIONS)
    covered_views: set[str] = set()
    covered_relations: set[str] = set()
    covered_edges: set[tuple[str, str]] = set()
    covered_atom_types: set[str] = set()
    normalized_vectors: list[dict[str, Any]] = []
    for row in vectors:
        row_id = row["id"]
        if set(row) != {"id", "operation", "inputs", "coverage", "expected"}:
            raise H3ReaderCutoverRefusal("vector_shape", row_id)
        operation = row.get("operation")
        if operation != operations[row_id]:
            raise H3ReaderCutoverRefusal("vector_operation", row_id)
        inputs = row.get("inputs")
        if not isinstance(inputs, dict) or not inputs:
            raise H3ReaderCutoverRefusal("vector_shape", f"{row_id}: inputs")
        normalized_inputs = {
            name: _canonical_atom(atom)
            for name, atom in inputs.items()
            if isinstance(name, str) and name and "\x00" not in name
        }
        if len(normalized_inputs) != len(inputs):
            raise H3ReaderCutoverRefusal("vector_shape", f"{row_id}: input name")
        covered_atom_types.update(atom["type"] for atom in normalized_inputs.values())

        coverage = row.get("coverage")
        if not isinstance(coverage, dict) or set(coverage) != {
            "views",
            "relations",
            "reader_edges",
        }:
            raise H3ReaderCutoverRefusal("vector_coverage_shape", row_id)
        views = _ordered_unique_strings(coverage["views"], f"{row_id}: views")
        relations = _ordered_unique_strings(coverage["relations"], f"{row_id}: relations")
        edges = _reader_edges(coverage["reader_edges"], f"{row_id}: reader_edges")
        covered_views.update(views)
        covered_relations.update(relations)
        covered_edges.update(edges)

        expected = row.get("expected")
        if not isinstance(expected, dict) or set(expected) != {"sets", "sha256"}:
            raise H3ReaderCutoverRefusal("vector_result_shape", row_id)
        normalized_sets = _canonical_expected_sets(expected["sets"])
        covered_atom_types.update(
            atom["type"]
            for rows in normalized_sets.values()
            for result_row in rows
            for atom in result_row.values()
        )
        digest = hashlib.sha256(RESULT_DOMAIN + _canonical_json(normalized_sets)).hexdigest()
        if expected.get("sha256") != digest:
            raise H3ReaderCutoverRefusal("vector_digest", row_id)
        normalized_vectors.append(
            {
                "id": row_id,
                "operation": operation,
                "inputs": normalized_inputs,
                "coverage": coverage,
                "expected": {"sets": normalized_sets, "sha256": digest},
            }
        )

    expected_atom_types = set(EXPECTED_ATOM_TYPES)
    if covered_atom_types != expected_atom_types:
        raise H3ReaderCutoverRefusal(
            "vector_atom_coverage",
            f"missing={sorted(expected_atom_types - covered_atom_types)} "
            f"extra={sorted(covered_atom_types - expected_atom_types)}",
        )
    expected_views = {str(row["name"]) for row in contract["current_views"]}
    if covered_views != expected_views:
        raise H3ReaderCutoverRefusal(
            "current_view_coverage",
            f"missing={sorted(expected_views - covered_views)} extra={sorted(covered_views - expected_views)}",
        )
    expected_relations = _parent_persistent_relation_names(parent)
    if covered_relations != expected_relations:
        raise H3ReaderCutoverRefusal(
            "persistent_relation_coverage",
            f"missing={sorted(expected_relations - covered_relations)} extra={sorted(covered_relations - expected_relations)}",
        )
    read_edges = {(str(row["path"]), str(row["relation"])) for row in contract["epoch7_read_edges"]}
    bootstrap_edges = {
        (str(row["path"]), str(row["relation"]))
        for row in contract["bootstrap_definition_authorities"]
    }
    expected_edges = read_edges - bootstrap_edges
    if covered_edges != expected_edges:
        raise H3ReaderCutoverRefusal(
            "runtime_reader_edge_coverage",
            f"missing={sorted(expected_edges - covered_edges)} extra={sorted(covered_edges - expected_edges)}",
        )
    semantic_digest = hashlib.sha256(
        CORPUS_DOMAIN + _canonical_json(normalized_vectors)
    ).hexdigest()
    if semantic_digest != EXPECTED_VECTOR_SEMANTIC_SHA256:
        raise H3ReaderCutoverRefusal("vector_semantic_digest", semantic_digest)
    return []


def _discover_dynamic_reader_inventory(
    contract: dict[str, Any], parent: dict[str, Any], root: Path
) -> set[tuple[str, str]]:
    authorities = contract.get("dynamic_read_authorities")
    if not isinstance(authorities, list):
        raise H3ReaderCutoverRefusal("contract_shape", "dynamic_read_authorities")
    governed = _parent_relation_names(parent)
    discovered: set[tuple[str, str]] = set()
    for authority in authorities:
        if not isinstance(authority, dict):
            raise H3ReaderCutoverRefusal("contract_shape", "dynamic read authority")
        relative = str(authority["path"])
        path = root / relative
        source = _bounded_text(path)
        tree = _python_tree(path, source)
        declared = {str(value) for value in authority["governed_relations"]}
        kind = authority["kind"]
        if kind == "fixed_table_loop":
            selector = str(authority["selector"])
            actual = set(_module_string_collection(tree, selector)) & governed
            loop_templates = [
                template
                for node in ast.walk(tree)
                if isinstance(node, ast.For)
                and isinstance(node.target, ast.Name)
                and isinstance(node.iter, ast.Name)
                and node.iter.id == selector
                for template in _dynamic_select_templates(node, node.target.id)
            ]
            if not loop_templates:
                raise H3ReaderCutoverRefusal(
                    "dynamic_authority_drift", f"{relative}: {selector} has no SELECT loop"
                )
        elif kind == "public_session_catalog":
            selectors = authority["selectors"]
            if selectors != ["_session_keyed_tables", "_session_reference_tables"]:
                raise H3ReaderCutoverRefusal("contract_shape", f"{relative}: selectors")
            actual = _parent_session_relations(parent)
            keyed_sql = _function_sql(tree, "_session_keyed_tables")
            reference_sql = _function_sql(tree, "_session_reference_tables")
            keyed_fragments = (
                "from pg_class",
                "join pg_namespace",
                "join information_schema.columns",
                "n.nspname = 'public'",
                "not c.relispartition",
                "ic.column_name = 'session_id'",
                "c.relname not like 'immutable_reference_%'",
            )
            reference_fragments = (
                "from information_schema.columns",
                "table_name like 'immutable_reference_%'",
                "column_name = 'session_id'",
            )
            calls = _function_call_names(tree, "export_session_to_parquet")
            if (
                any(fragment not in keyed_sql for fragment in keyed_fragments)
                or any(fragment not in reference_sql for fragment in reference_fragments)
                or not {
                    "_session_keyed_tables",
                    "_session_reference_tables",
                    "_export_table",
                }.issubset(calls)
            ):
                raise H3ReaderCutoverRefusal(
                    "dynamic_authority_drift", f"{relative}: session catalog"
                )
        else:
            raise H3ReaderCutoverRefusal("contract_shape", f"dynamic kind {kind!r}")
        if actual != declared:
            raise H3ReaderCutoverRefusal(
                "dynamic_authority_drift",
                f"{relative}: declared={sorted(declared)} discovered={sorted(actual)}",
            )
        discovered.update((relative, relation) for relation in actual)
    return discovered


def discover_dynamic_reader_inventory(contract: dict[str, Any], root: Path) -> set[tuple[str, str]]:
    """Discover the closed dynamic-reader census from executable authority."""
    return _discover_dynamic_reader_inventory(contract, _load_parent_contract(contract, root), root)


def _verify_dynamic_reader_qualification(contract: dict[str, Any], root: Path) -> None:
    for authority in contract["dynamic_read_authorities"]:
        relative = str(authority["path"])
        path = root / relative
        tree = _python_tree(path, _bounded_text(path))
        templates = _dynamic_select_templates(tree, "table")
        if not templates or any("from public.{table}" not in template for template in templates):
            raise H3ReaderCutoverRefusal("unqualified_relation", f"{relative}: dynamic table")


def require_exact_reader_inventory(
    expected: set[tuple[str, str]], observed: set[tuple[str, str]]
) -> None:
    """Refuse a missing governed edge before diagnosing unexpected readers."""
    missing = expected - observed
    if missing:
        raise H3ReaderCutoverRefusal("missing_reader", repr(sorted(missing)))
    extra = observed - expected
    if extra:
        raise H3ReaderCutoverRefusal("unexpected_reader", repr(sorted(extra)))


def _verify_contract_shape(contract: dict[str, Any]) -> None:
    if contract.get("meta") != EXPECTED_META or contract.get("bounds") != EXPECTED_BOUNDS:
        raise H3ReaderCutoverRefusal("contract_shape", "meta or bounds")
    edges = contract.get("read_edges")
    views = contract.get("current_views")
    if not isinstance(edges, list) or len(edges) != EXPECTED_BOUNDS["read_edges"]:
        raise H3ReaderCutoverRefusal("contract_shape", "read_edges")
    if not isinstance(views, list) or len(views) != EXPECTED_BOUNDS["current_views"]:
        raise H3ReaderCutoverRefusal("contract_shape", "current_views")
    triples = []
    for edge in edges:
        if not isinstance(edge, dict) or set(edge) != {
            "path",
            "relation",
            "canonical_relation",
        }:
            raise H3ReaderCutoverRefusal("contract_shape", "read edge")
        triple = (str(edge["path"]), str(edge["relation"]), str(edge["canonical_relation"]))
        if triple[2] != f"public.{triple[1]}":
            raise H3ReaderCutoverRefusal("contract_shape", "canonical relation")
        triples.append(triple)
    if triples != sorted(triples) or len(triples) != len(set(triples)):
        raise H3ReaderCutoverRefusal("contract_order", "read_edges")
    if len({path for path, _, _ in triples}) != EXPECTED_BOUNDS["source_files"]:
        raise H3ReaderCutoverRefusal("contract_shape", "source_files")
    epoch7_edges = contract.get("epoch7_read_edges")
    if (
        not isinstance(epoch7_edges, list)
        or len(epoch7_edges) != EXPECTED_BOUNDS["epoch7_read_edges"]
    ):
        raise H3ReaderCutoverRefusal("contract_shape", "epoch7_read_edges")
    epoch7_triples: list[tuple[str, str, str]] = []
    for edge in epoch7_edges:
        if not isinstance(edge, dict) or set(edge) != {
            "path",
            "relation",
            "canonical_relation",
        }:
            raise H3ReaderCutoverRefusal("contract_shape", "epoch7 read edge")
        triple = (str(edge["path"]), str(edge["relation"]), str(edge["canonical_relation"]))
        if triple[2] != f"public.{triple[1]}":
            raise H3ReaderCutoverRefusal("contract_shape", "epoch7 canonical relation")
        epoch7_triples.append(triple)
    if epoch7_triples != sorted(epoch7_triples) or len(epoch7_triples) != len(set(epoch7_triples)):
        raise H3ReaderCutoverRefusal("contract_order", "epoch7_read_edges")
    if len({path for path, _, _ in epoch7_triples}) != EXPECTED_BOUNDS["epoch7_source_files"]:
        raise H3ReaderCutoverRefusal("contract_shape", "epoch7_source_files")
    bootstrap = contract.get("bootstrap_definition_authorities")
    if (
        not isinstance(bootstrap, list)
        or len(bootstrap) != EXPECTED_BOUNDS["bootstrap_definition_authorities"]
    ):
        raise H3ReaderCutoverRefusal("contract_shape", "bootstrap_definition_authorities")
    expected_pairs = {(path, relation) for path, relation, _ in triples}
    bootstrap_pairs: set[tuple[str, str]] = set()
    for authority in bootstrap:
        if not isinstance(authority, dict) or set(authority) != {
            "path",
            "relation",
            "canonical_relation",
            "kind",
            "lifecycle",
            "superseded_by_epoch",
            "retirement_issue",
        }:
            raise H3ReaderCutoverRefusal("contract_shape", "bootstrap definition authority")
        path = str(authority["path"])
        relation = str(authority["relation"])
        pair = (path, relation)
        if (
            authority["canonical_relation"] != f"public.{relation}"
            or authority["kind"] != "bootstrap_view_definition_predecessor"
            or authority["superseded_by_epoch"] != 7
            or authority["retirement_issue"] != "PER-281"
            or authority["lifecycle"] not in {"reachable_pre_cutover", "retired"}
            or pair in bootstrap_pairs
        ):
            raise H3ReaderCutoverRefusal("contract_shape", "bootstrap definition authority")
        if (authority["lifecycle"] == "reachable_pre_cutover") != (pair in expected_pairs):
            raise H3ReaderCutoverRefusal("contract_shape", "bootstrap lifecycle")
        bootstrap_pairs.add(pair)
    if len(expected_pairs - bootstrap_pairs) != EXPECTED_BOUNDS["runtime_consumer_edges"]:
        raise H3ReaderCutoverRefusal("contract_shape", "runtime_consumer_edges")
    authorities = contract.get("dynamic_read_authorities")
    if (
        not isinstance(authorities, list)
        or len(authorities) != EXPECTED_BOUNDS["dynamic_authorities"]
    ):
        raise H3ReaderCutoverRefusal("contract_shape", "dynamic_read_authorities")
    authority_paths: list[str] = []
    for authority in authorities:
        if not isinstance(authority, dict):
            raise H3ReaderCutoverRefusal("contract_shape", "dynamic read authority")
        kind = authority.get("kind")
        required = {"path", "kind", "governed_relations"}
        required.add("selector" if kind == "fixed_table_loop" else "selectors")
        if set(authority) != required:
            raise H3ReaderCutoverRefusal("contract_shape", "dynamic read authority")
        path = str(authority["path"])
        relations = authority["governed_relations"]
        if (
            not isinstance(relations, list)
            or relations != sorted(relations)
            or len(relations) != len(set(relations))
            or any((path, str(relation)) not in expected_pairs for relation in relations)
        ):
            raise H3ReaderCutoverRefusal("contract_shape", f"dynamic relations: {path}")
        authority_paths.append(path)
    if authority_paths != sorted(authority_paths) or len(authority_paths) != len(
        set(authority_paths)
    ):
        raise H3ReaderCutoverRefusal("contract_order", "dynamic_read_authorities")
    epoch7_authorities = contract.get("epoch7_dynamic_read_authorities")
    if (
        not isinstance(epoch7_authorities, list)
        or len(epoch7_authorities) != EXPECTED_BOUNDS["epoch7_dynamic_authorities"]
    ):
        raise H3ReaderCutoverRefusal("contract_shape", "epoch7_dynamic_read_authorities")
    epoch7_pairs = {(path, relation) for path, relation, _ in epoch7_triples}
    for authority in epoch7_authorities:
        if not isinstance(authority, dict) or set(authority) != {
            "path",
            "kind",
            "selectors",
            "governed_relations",
        }:
            raise H3ReaderCutoverRefusal("contract_shape", "epoch7 dynamic authority")
        path = str(authority["path"])
        relations = authority["governed_relations"]
        if (
            authority["kind"] != "public_session_catalog"
            or authority["selectors"] != ["_session_keyed_tables", "_session_reference_tables"]
            or not isinstance(relations, list)
            or relations != sorted(relations)
            or any((path, str(relation)) not in epoch7_pairs for relation in relations)
        ):
            raise H3ReaderCutoverRefusal("contract_shape", "epoch7 dynamic authority")
    terminal = contract.get("terminal_disposition")
    if (
        not isinstance(terminal, dict)
        or terminal.get("authority_state") != "rust_active"
        or terminal.get("schema_epoch") != 9
        or terminal.get("python_game_state_reader_edges") != 0
    ):
        raise H3ReaderCutoverRefusal("contract_shape", "terminal_disposition")


def verify_non_h3_observer_surfaces(parent: dict[str, Any], root: Path) -> None:
    """Prove the exact economic view declarations cannot read the retired H3 estate."""
    migration_root = root / "rust/crates/babylon-persistence/migrations"
    retired_relations = _parent_relation_names(parent)
    for migration in sorted({entry[0] for entry in NON_H3_OBSERVER_SURFACES.values()}):
        source = _bounded_text(migration_root / migration)
        expected = {
            name for name, entry in NON_H3_OBSERVER_SURFACES.items() if entry[0] == migration
        }
        declarations = re.findall(
            r"\bCREATE\s+VIEW\s+public\.(v_observer_[a-z0-9_]+)\b", source, re.IGNORECASE
        )
        if len(declarations) != len(expected) or set(declarations) != expected:
            raise H3ReaderCutoverRefusal("observer_declaration_drift", migration)
        for relation in sorted(retired_relations):
            if re.search(rf"\b{re.escape(relation)}\b", source, re.IGNORECASE):
                raise H3ReaderCutoverRefusal("compatibility_read", f"{migration}: {relation}")


def verify_reader_cutover_contract(contract: dict[str, Any], root: Path) -> list[tuple[str, str]]:
    """Verify the parent digest, exact inventory, and every production SQL read."""
    _verify_contract_shape(contract)
    verify_reader_parity_vectors(
        contract,
        load_reader_parity_vectors(root / EXPECTED_VECTOR_PATH),
        root,
    )
    parent = _load_parent_contract(contract, root)
    verify_non_h3_observer_surfaces(parent, root)

    expected = {(str(row["path"]), str(row["relation"])) for row in contract["read_edges"]}
    observed = _discover_dynamic_reader_inventory(contract, parent, root)
    _verify_dynamic_reader_qualification(contract, root)
    excluded_roots = {
        str(path) for path in contract.get("scan", {}).get("historical_migration_roots", [])
    }
    excluded_files = {
        str(path) for path in contract.get("scan", {}).get("closed_migration_sources", [])
    }
    for root_name in contract.get("scan", {}).get("production_roots", []):
        base = root / str(root_name)
        for path in sorted(base.rglob("*")):
            if not path.is_file() or path.suffix not in {".py", ".rs"}:
                continue
            relative = path.relative_to(root).as_posix()
            if relative == "tools/verify_h3_reader_cutover_v1.py":
                continue
            if relative in excluded_files or any(
                relative == prefix or relative.startswith(prefix + "/") for prefix in excluded_roots
            ):
                continue
            source = _bounded_text(path)
            for sql in _source_sql_literals(path, source):
                for relation, _ in inspect_sql_literal(contract, Path(relative), sql):
                    edge = (relative, relation)
                    if edge not in expected:
                        raise H3ReaderCutoverRefusal("unexpected_reader", f"{relative}: {relation}")
                    observed.add(edge)
    require_exact_reader_inventory(expected, observed)
    return []


def main(argv: list[str] | None = None) -> int:
    """CLI entry point used by the repository sentinel gate."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--contract",
        type=Path,
        default=Path("contracts/h3_reader_cutover_v1.yaml"),
    )
    parser.add_argument("--root", type=Path, default=Path("."))
    args = parser.parse_args(argv)
    try:
        contract = load_reader_cutover_contract(args.contract)
        verify_reader_cutover_contract(contract, args.root.resolve())
    except H3ReaderCutoverRefusal as error:
        print(f"H3 reader cutover REFUSED: {error}", file=sys.stderr)
        return 1
    print("H3 reader cutover contract: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
