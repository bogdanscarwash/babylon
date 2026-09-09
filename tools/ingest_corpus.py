#!/usr/bin/env python3
"""Corpus ingestion tool for Babylon's RAG pipeline.

This tool prepares texts from the durable OCR/extraction tree
(``~/Documents/ocr/`` by default) as chunked Markdown files for RAG
ingestion. Which works are eligible, and where their extracted text lives,
is declared entirely in the corpus manifest
(``src/babylon/data/corpus/manifest.yaml`` +
:mod:`babylon.intelligence.corpus_manifest`, ADR107) — this tool no longer
hardcodes a corpus list or fuzzy-matches filenames against an arbitrary
external mirror (the retired ``MVP_CORPUS`` tuple / ``fuzzy_match_score``).

NOTE: ChromaDB has been removed. Vector storage is now handled by
pgvector via PgVectorStore (Feature 037). This tool can still import
and prepare corpus files; ingestion into the vector store requires
a running PostgreSQL instance with pgvector and is NOT performed here
(no embedding calls, no DB writes — file preparation only).

Usage:
    # Prepare corpus files from the OCR extraction tree
    uv run python tools/ingest_corpus.py --prepare

    # List corpus status
    uv run python tools/ingest_corpus.py --list
"""

from __future__ import annotations

import argparse
import logging
import re
import sys
from pathlib import Path

from babylon.config.logging_config import setup_logging
from babylon.intelligence.corpus_manifest import (
    CorpusFormat,
    CorpusManifest,
    CorpusRow,
    ManifestTarget,
    load_bundled_manifest,
)

setup_logging(default_level="INFO")
logger = logging.getLogger(__name__)

#: Default durable extraction tree root (ADR107 "step zero"). Manifest
#: path_globs resolve against this; overridable via --corpus-root for
#: alternate boxes (and always overridden in tests, which never touch it).
DEFAULT_CORPUS_ROOT: Path = Path.home() / "Documents" / "ocr"


# =============================================================================
# FILE DISCOVERY AND IMPORT
# =============================================================================


def convert_html_to_markdown(html_content: str) -> str:
    """Convert HTML content to Markdown format.

    Args:
        html_content: Raw HTML content

    Returns:
        Converted Markdown content
    """
    from markdownify import markdownify

    markdown = markdownify(
        html_content,
        heading_style="atx",
        bullets="-",
        strip=["script", "style", "nav", "footer", "header"],
    )

    # Clean up excessive whitespace
    markdown = re.sub(r"\n{3,}", "\n\n", markdown)
    markdown = re.sub(r" +", " ", markdown)

    return markdown.strip()


def import_manifest_work(files: tuple[Path, ...], row: CorpusRow, dest_path: Path) -> bool:
    """Concatenate one manifest row's matched files into a single Markdown file.

    HTML-format rows are converted via :func:`convert_html_to_markdown`
    (REUSED unchanged); every other format (the ~/Documents/ocr/ convention:
    plain ``.txt`` extraction output) is read as-is. Multiple matched files
    (chapter splits) are joined in the deterministic order the manifest
    already sorted them in.

    Args:
        files: This row's existing, deny-subtracted matched files (already
            sorted deterministically by :meth:`CorpusManifest.ingest_targets`).
        row: The manifest row these files belong to.
        dest_path: Destination path for the combined Markdown file.

    Returns:
        True if at least one file yielded non-empty content, False otherwise.
    """
    if not files:
        return False

    chunks: list[str] = []
    for file_path in files:
        try:
            with open(file_path, encoding="utf-8", errors="replace") as f:
                raw = f.read()
        except OSError as e:
            logger.warning(f"Failed to read {file_path.name}: {e}")
            continue

        text = convert_html_to_markdown(raw) if row.format is CorpusFormat.HTML else raw.strip()
        if text:
            chunks.append(text)

    if not chunks:
        return False

    full_content = f"# {row.work}\n\n**Author:** {row.author}\n\n---\n\n"
    full_content += "\n\n---\n\n".join(chunks)

    with open(dest_path, "w", encoding="utf-8") as f:
        f.write(full_content)

    logger.info(f"Imported {len(files)} file(s) for '{row.work}' -> {dest_path.name}")
    return True


def _dest_filename(row: CorpusRow) -> str:
    """Deterministic ``<author>_<work>.md`` destination filename for a row."""
    safe_title = re.sub(r"[^\w\s-]", "", row.work).replace(" ", "_").lower()
    safe_author = re.sub(r"[^\w\s-]", "", row.author).replace(" ", "_").lower()
    return f"{safe_author}_{safe_title}.md"


def import_corpus_files(
    corpus_root: Path,
    corpus_dir: Path,
    manifest: CorpusManifest,
) -> tuple[int, list[str]]:
    """Prepare every allow-listed, existing manifest work as a Markdown file.

    Enumeration is entirely manifest-driven: allow minus deny, existing-files
    -only, in the manifest's declared (deterministic) row order — see
    :meth:`CorpusManifest.ingest_targets`. A row whose work has not been
    extracted onto this box yet is a MANIFEST fact, reported as missing,
    never raised as an error.

    Args:
        corpus_root: Durable extraction tree root manifest globs resolve
            against (e.g. ``~/Documents/ocr``).
        corpus_dir: Local corpus directory to write prepared Markdown into.
        manifest: The loaded, validated corpus manifest.

    Returns:
        Tuple of (number imported, list of missing "work (author)" strings).
    """
    corpus_dir.mkdir(parents=True, exist_ok=True)

    imported = 0
    missing: list[str] = []

    target: ManifestTarget
    for target in manifest.ingest_targets(corpus_root):
        row = target.row
        if not target.present:
            missing.append(f"{row.work} ({row.author})")
            continue

        dest_path = corpus_dir / _dest_filename(row)
        if import_manifest_work(target.files, row, dest_path):
            imported += 1
        else:
            missing.append(f"{row.work} ({row.author})")

    return imported, missing


def list_corpus(corpus_dir: Path) -> None:
    """List corpus files and their status.

    Args:
        corpus_dir: Directory containing corpus files
    """
    if not corpus_dir.exists():
        logger.info(f"Corpus directory does not exist: {corpus_dir}")
        return

    extensions = (".txt", ".md", ".htm", ".html")
    corpus_files = sorted(f for ext in extensions for f in corpus_dir.glob(f"*{ext}"))

    if not corpus_files:
        logger.info("No corpus files found.")
        return

    logger.info(f"Corpus directory: {corpus_dir}")
    logger.info(f"Files found: {len(corpus_files)}")
    for f in corpus_files:
        size_kb = f.stat().st_size / 1024
        logger.info(f"  {f.name} ({size_kb:.1f} KB)")


# =============================================================================
# CLI
# =============================================================================


def main() -> int:
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="Prepare manifest-declared corpus texts for Babylon's RAG pipeline",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
    # Prepare from the default OCR extraction tree (~/Documents/ocr)
    %(prog)s --prepare

    # Prepare from an alternate extraction tree
    %(prog)s --prepare --corpus-root /path/to/ocr

    # List corpus status
    %(prog)s --list

Note: Vector ingestion requires pgvector (Feature 037) and is a separate step.
        """,
    )

    parser.add_argument(
        "--prepare",
        action="store_true",
        help="Prepare manifest-declared, allow-listed works as Markdown from --corpus-root",
    )

    parser.add_argument(
        "--list",
        action="store_true",
        help="List corpus files and their status",
    )

    parser.add_argument(
        "--corpus-root",
        type=Path,
        default=DEFAULT_CORPUS_ROOT,
        metavar="PATH",
        help=f"Durable OCR extraction tree manifest globs resolve against (default: {DEFAULT_CORPUS_ROOT})",
    )

    parser.add_argument(
        "--corpus-dir",
        type=Path,
        default=Path(__file__).parent.parent / "data" / "corpus",
        metavar="PATH",
        help="Local corpus directory prepared Markdown is written to (default: data/corpus/)",
    )

    parser.add_argument(
        "-v",
        "--verbose",
        action="store_true",
        help="Enable verbose logging",
    )

    args = parser.parse_args()

    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)

    manifest = load_bundled_manifest()

    logger.info("=" * 60)
    logger.info("Babylon Corpus Tool")
    logger.info("=" * 60)
    logger.info(f"\nManifest allow-listed works ({len(manifest.allow_rows())}):")
    for i, row in enumerate(manifest.allow_rows(), 1):
        logger.info(f"  {i}. {row.work} ({row.author})")
    logger.info("")

    if args.list:
        list_corpus(args.corpus_dir)
        return 0

    if args.prepare:
        logger.info(f"\nPreparing from corpus root: {args.corpus_root}")
        imported, missing = import_corpus_files(args.corpus_root, args.corpus_dir, manifest)

        logger.info(f"\nPrepare summary: {imported}/{len(manifest.allow_rows())} works found")
        if missing:
            logger.warning(f"Missing works (not yet extracted): {', '.join(missing)}")
        return 0

    # No action specified
    logger.info("No action specified. Use --prepare to prepare files, or --list to view status.")
    logger.info("Vector ingestion requires pgvector backend (Feature 037).")
    return 0


if __name__ == "__main__":
    sys.exit(main())
