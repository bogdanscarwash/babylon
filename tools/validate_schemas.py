#!/usr/bin/env python3
"""JSON Schema validation tool for Babylon data files.

Validates all JSON data files in src/babylon/data/ against their
corresponding schemas in src/babylon/schemas/.

Usage:
    uv run python tools/validate_schemas.py
    uv run python tools/validate_schemas.py --verbose
    uv run python tools/validate_schemas.py --file characters.json
"""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any, Final

from jsonschema import Draft202012Validator
from referencing import Registry, Resource
from referencing.jsonschema import DRAFT202012
from rich.console import Console
from rich.table import Table

# === Paths ===
PROJECT_ROOT: Final[Path] = Path(__file__).parent.parent
DATA_DIR: Final[Path] = PROJECT_ROOT / "src" / "babylon" / "data"
SCHEMAS_DIR: Final[Path] = PROJECT_ROOT / "src" / "babylon" / "schemas"
COLLECTIONS_DIR: Final[Path] = SCHEMAS_DIR / "collections"

# === Console output ===
console = Console()


def load_schema(schema_path: Path) -> dict[str, Any]:
    """Load a JSON schema from disk."""
    with open(schema_path, encoding="utf-8") as f:
        result: dict[str, Any] = json.load(f)
        return result


def build_schema_registry() -> Registry:
    """Build a schema registry for $ref resolution.

    This enables $ref resolution across all schemas in the registry.
    Returns a referencing.Registry object compatible with Draft 2020-12.
    """
    resources: list[tuple[str, Resource]] = []

    # Load all schemas recursively
    for schema_path in SCHEMAS_DIR.rglob("*.schema.json"):
        schema = load_schema(schema_path)
        schema_id = schema.get("$id")
        if schema_id:
            resource = Resource.from_contents(schema, default_specification=DRAFT202012)
            resources.append((schema_id, resource))

    return Registry().with_resources(resources)


def get_collection_schema_path(data_filename: str) -> Path | None:
    """Map a data file to its corresponding collection schema.

    Args:
        data_filename: Name of the data file (e.g., 'characters.json')

    Returns:
        Path to the collection schema, or None if not found
    """
    # Remove .json extension and add .schema.json
    base_name = data_filename.replace(".json", "")
    schema_path = COLLECTIONS_DIR / f"{base_name}.schema.json"

    if schema_path.exists():
        return schema_path
    return None


def validate_data_file(
    data_path: Path,
    registry: Registry,
    verbose: bool = False,
) -> list[str]:
    """Validate a data file against its schema.

    Args:
        data_path: Path to the JSON data file
        registry: Pre-built schema registry for $ref resolution
        verbose: Whether to print detailed progress

    Returns:
        List of error messages (empty if valid)
    """
    errors: list[str] = []

    # Find corresponding schema
    schema_path = get_collection_schema_path(data_path.name)
    if schema_path is None:
        errors.append(f"No schema found for {data_path.name}")
        return errors

    # Load data
    try:
        with open(data_path, encoding="utf-8") as f:
            data = json.load(f)
    except json.JSONDecodeError as e:
        errors.append(f"Invalid JSON in {data_path.name}: {e}")
        return errors

    # Load schema
    schema = load_schema(schema_path)

    # Create validator with registry for $ref resolution
    validator = Draft202012Validator(schema, registry=registry)

    # Validate
    for error in validator.iter_errors(data):
        error_path = " -> ".join(str(p) for p in error.absolute_path) or "(root)"
        errors.append(f"{error_path}: {error.message}")

        if verbose:
            console.print(f"  [dim]Schema path: {list(error.schema_path)}[/dim]")

    return errors


def validate_all(
    verbose: bool = False,
    file_filter: str | None = None,
) -> tuple[int, int, dict[str, list[str]]]:
    """Validate all data files against their schemas.

    Args:
        verbose: Whether to print detailed progress
        file_filter: Optional filename to validate only that file

    Returns:
        Tuple of (files_validated, files_with_errors, error_details)
    """
    # Build schema registry once for all validations
    console.print("[bold blue]Loading schemas...[/bold blue]")
    registry = build_schema_registry()
    schema_count = len(list(SCHEMAS_DIR.rglob("*.schema.json")))
    console.print(f"  Loaded {schema_count} schemas into registry")

    # Find data files
    data_files = sorted(DATA_DIR.glob("*.json"))
    if file_filter:
        data_files = [f for f in data_files if f.name == file_filter]
        if not data_files:
            console.print(f"[red]File not found: {file_filter}[/red]")
            return 0, 0, {}

    console.print(f"\n[bold blue]Validating {len(data_files)} data files...[/bold blue]\n")

    files_validated = 0
    files_with_errors = 0
    all_errors: dict[str, list[str]] = {}

    for data_path in data_files:
        files_validated += 1
        errors = validate_data_file(data_path, registry, verbose)

        if errors:
            files_with_errors += 1
            all_errors[data_path.name] = errors
            console.print(f"[red]  FAIL[/red] {data_path.name} ({len(errors)} errors)")
        else:
            console.print(f"[green]  PASS[/green] {data_path.name}")

    return files_validated, files_with_errors, all_errors


def print_error_summary(all_errors: dict[str, list[str]], max_errors: int = 10) -> None:
    """Print a summary of validation errors."""
    if not all_errors:
        return

    console.print("\n[bold red]Validation Errors:[/bold red]\n")

    for filename, errors in all_errors.items():
        console.print(f"[bold]{filename}[/bold]")
        for idx, error in enumerate(errors):
            if idx >= max_errors:
                remaining = len(errors) - idx
                console.print(f"  [dim]... and {remaining} more errors[/dim]")
                break
            console.print(f"  [red]•[/red] {error}")
        console.print()


def main() -> int:
    """Main entry point."""
    import argparse

    parser = argparse.ArgumentParser(description="Validate JSON data files against schemas")
    parser.add_argument(
        "--verbose", "-v", action="store_true", help="Print detailed validation progress"
    )
    parser.add_argument(
        "--file", "-f", type=str, help="Validate only a specific file (e.g., characters.json)"
    )
    parser.add_argument(
        "--max-errors",
        type=int,
        default=10,
        help="Maximum errors to display per file (default: 10)",
    )

    args = parser.parse_args()

    console.print("[bold]Babylon JSON Schema Validator[/bold]\n")

    files_validated, files_with_errors, all_errors = validate_all(
        verbose=args.verbose,
        file_filter=args.file,
    )

    # Print summary
    console.print("\n" + "=" * 50)

    if files_with_errors == 0:
        console.print(f"[bold green]All {files_validated} files passed validation![/bold green]")
        return 0

    print_error_summary(all_errors, max_errors=args.max_errors)

    table = Table(title="Validation Summary")
    table.add_column("Metric", style="cyan")
    table.add_column("Value", style="white")

    table.add_row("Files validated", str(files_validated))
    table.add_row("Files passed", str(files_validated - files_with_errors))
    table.add_row("Files failed", f"[red]{files_with_errors}[/red]")
    table.add_row("Total errors", f"[red]{sum(len(e) for e in all_errors.values())}[/red]")

    console.print(table)

    return 1


if __name__ == "__main__":
    sys.exit(main())
