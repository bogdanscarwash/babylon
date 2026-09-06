#!/usr/bin/env python3
"""Repo-hygiene gate: allowlisted root, no tracked-ignored files, no fat blobs.

Program 14 (Correspondence) Phase 0.5. Enforces three invariants, loudly
(Constitution III.11 — no silent degradation):

a. **Root allowlist** — every *tracked* top-level entry must appear in
   ``ALLOWED_TOP_LEVEL_DIRS`` or ``ALLOWED_TOP_LEVEL_FILES``. Untracked local
   state (``.venv``, ``data/`` mount symlinks, caches) is ``.gitignore``'s
   jurisdiction, not this gate's.
b. **No tracked-but-ignored files** — the failure mode that let 70 MB of
   ``reports/`` artifacts ride in git (ignore rules added after commit, index
   never purged). The ``.gitkeep`` convention (tracked keeper inside an
   ignored directory) is exempt.
c. **No tracked blob over 1 MiB at HEAD** — LFS pointers are ~130-byte blobs,
   so blob size alone separates pointers from real heavyweights; a >1 MiB
   blob is either missing an LFS attribute or missing a renormalize. Named
   exemptions live in ``LARGE_BLOB_EXEMPTIONS`` (grown only with a per-entry
   owner-visible justification comment).

Run: ``uv run python tools/check_repo_hygiene.py`` (wired into
``mise run check`` as ``check:hygiene`` and into CI). Exit 0 = clean,
1 = violations (printed), 2 = git itself failed.
"""

from __future__ import annotations

import subprocess
import sys

#: Sanctioned tracked top-level directories (Program 14 root architecture).
ALLOWED_TOP_LEVEL_DIRS: frozenset[str] = frozenset(
    {
        ".agents",  # AGENTS.md cross-tool standard config
        ".claude",  # Claude Code project settings/agents
        ".codex",  # Codex worktree setup and shared Rust host policy (PER-286)
        ".design-sync",  # claude.ai/design converter durable inputs
        ".github",
        ".mise",  # split Mise task modules loaded from the root config
        ".opencode",
        ".serena",
        ".specify",  # spec-kit remnant; teardown is a separate deferred item
        ".understand-anything",
        "ai",  # Claude's owned tree: context yamls, decisions/ ADRs, scratch/
        "assets",
        "contracts",
        "config-examples",  # §A3/§A8 player config templates (ADR096); never real secrets
        "design",
        "docker",
        "docs",
        "infra",  # babylon-infra git submodule (gitlink), separate infrastructure scope
        "openwiki",  # generated wiki estate + engine instructions (ADR181 Train F); daily workflow regenerates
        "output",  # tracked delivery evidence: demo screenshots (spec-113 Living Map, ADR066)
        "project",  # long-horizon governance: programs/owner/execution/notes
        "reports",  # tracked audit evidence; run artifacts are gitignored
        "results",  # gitignored output dir; tracked .gitkeep only
        "rust",  # in-tree Rust/Ratatui client workspace (raster cutover, Amendment AC/ADR150)
        "security",  # pip-audit expiring-ignores policy (program 15)
        "sources",  # Percy's theory texts (LFS)
        "specs",
        "src",
        "tests",
        "tools",
    }
)

#: Sanctioned tracked top-level files (root canon + tool dot-configs).
ALLOWED_TOP_LEVEL_FILES: frozenset[str] = frozenset(
    {
        ".actrc",  # act (gh act) local-runner defaults — ci:local tasks (uv train, 2026-07-22)
        ".env.example",
        ".gitattributes",
        ".gitignore",
        ".gitleaks.toml",  # secret-scan policy shared by CI + pre-commit (program 15)
        ".gitmodules",  # submodule pointer — program 18: src/frontend → babylon-cockpit subrepo
        ".markdownlint.yaml",
        ".markdownlintignore",
        ".mdformat.toml",
        ".mise.toml",
        ".pre-commit-config.yaml",
        ".python-version",  # uv-facing interpreter-minor pin (3.12); guard test in tests/unit/cli/test_uv_migration.py
        ".semgrep-tests.yml",  # test-estate process rules (ADR181 R9a — wall-clock ban)
        ".semgrep.yml",
        ".semgrepignore",  # replaces semgrep's default ignore (which silently excludes tests/)
        ".trivyignore",  # curated IaC-scan ignores, every entry evidenced (program 15)
        ".vale.ini",  # repo-specific exceptions merged with the global Vale configuration
        ".yamllint.yaml",
        "AGENTS.md",
        "babylon.code-workspace",
        "docker-compose.yml",
        "docker-compose.ci.yml",  # CI override: tmpfs datadir + runner-sized conf (ADR176 r.33)
        "CHANGELOG.md",
        "CLAUDE.md",
        "CONSTITUTION.md",
        "CONTRIBUTORS.md",
        "LICENSE",
        "LICENSE-ASSETS",  # #650 Director ruling: CC0-1.0 for assets (AGPL/CC0 split)
        "LICENSING.md",  # #650: the code/assets license split + per-dir inventory
        "NORTH_STAR.md",  # BD-blessed orientation doc (2026-07-21); cited by CLAUDE.md as repo-root
        "README.md",
        "SETUP_GUIDE.md",
        "data-artifacts.yaml",  # ADR076 successor registry for demoted reference tables
        "data-catalog.yaml",
        "logging.yaml",  # runtime logging config (src/babylon/config/logging_config.py)
        "mise.lock",  # ADR252: exact native tool download URLs and checksums
        "uv.lock",  # ADR095 D3a: uv replaces Poetry as the dependency toolchain
        "pyproject.toml",
        "setup.cfg",  # doc8 config; doc8 cannot read pyproject (documented upstream issue)
    }
)

#: Tracked blobs allowed to exceed MAX_BLOB_BYTES. Grow it only with an
#: owner-visible justification comment per entry.
LARGE_BLOB_EXEMPTIONS: frozenset[str] = frozenset(
    {
        # Runtime seed read by the engine bridge and CI unit tests
        # (tests/unit/balkanization/) — LFS would force lfs:true + quota spend
        # on every CI checkout for one 1.6MB file. Tolerated in-pack.
        "src/babylon/data/game/balkanization/seed_influences.json",
        # ADR095 D3: uv single-lock replaces poetry.lock (825 KB) with uv.lock
        # (~1.05 MB) — PEP 735 dependency groups resolve to a denser lockfile.
        # Read by every checkout/CI run via `uv sync`/`uv run`; an LFS pointer
        # would force lfs:true + quota spend on every clone for a build-critical
        # file. Same tolerated-in-pack reasoning as the two entries above.
        "uv.lock",
        # Program 28 B1: the county map the Bevy client renders. Amendment AF
        # ships the game as a pure Rust binary, so the geometry can no longer
        # arrive over an FFI seam — it has to be a committed asset the client
        # include_bytes!s, and an LFS pointer would hand the reader 130 bytes
        # of text where it expects 3,222 quantized TIGER 2024 counties. Budget
        # (1.6 MB target, 3 MB hard stop) and regeneration live in
        # tools/build_county_atlas.py; `mise run data:county-atlas` rebuilds
        # it. Same tolerated-in-pack reasoning as the entries above.
        "assets/map/county_atlas.bin",
    }
)

#: 1 MiB — anything larger in plain git belongs in LFS (or out of the repo).
MAX_BLOB_BYTES: int = 1_048_576

# These two embedded, quality-5 Vorbis themes need 1.5 and 1.8 MiB. Keep a
# finite per-file budget: assets/audio-renders.json pins their MIDI sources,
# soundfont, render recipe, and exact output hashes. Licensing is unchanged.
RUNTIME_THEME_BLOB_LIMITS: dict[str, int] = {
    "assets/music/babylon_theme_phi.ogg": 2_097_152,
    "assets/music/babylon_theme_panopticon.ogg": 2_097_152,
}

#: Fixed upper bound on git output lines (Power-of-10 rule 2). The repo
#: tracks ~7k files; hitting this bound means something is deeply wrong.
MAX_GIT_OUTPUT_LINES: int = 100_000

#: Symlink mode in git tree entries — their blobs are target paths, not data.
_SYMLINK_MODE: str = "120000"


def _git_lines(args: list[str], *, nul_separated: bool = False) -> list[str]:
    """Run a git subcommand and return its stdout entries (bounded).

    :param args: Arguments after ``git`` (e.g. ``["ls-files", "-z"]``).
    :param nul_separated: Split on NUL instead of newlines. Callers listing
        paths MUST pass ``-z`` in ``args`` and set this — without it git
        C-quotes non-ASCII paths (``"ai/_inbox/Theory \\342\\200\\242.md"``),
        and the quoted string's first segment masquerades as a bogus
        top-level entry (broke the Fast Gate on 2026-07-15).
    :returns: Non-empty stdout entries.
    :raises RuntimeError: If git exits non-zero or output exceeds the fixed
        line bound — both are loud infrastructure failures, never ignored.
    """
    try:
        proc = subprocess.run(
            ["git", *args],
            capture_output=True,
            text=True,
            check=True,
            timeout=120,
        )
    except subprocess.CalledProcessError as exc:
        raise RuntimeError(f"git {args[0]} failed: {exc.stderr.strip()}") from exc
    except subprocess.TimeoutExpired as exc:
        raise RuntimeError(f"git {args[0]} timed out after 120s") from exc
    raw = proc.stdout.split("\0") if nul_separated else proc.stdout.splitlines()
    lines = [line for line in raw if line]
    if len(lines) > MAX_GIT_OUTPUT_LINES:
        raise RuntimeError(
            f"git {args[0]} returned {len(lines)} lines (bound {MAX_GIT_OUTPUT_LINES})"
        )
    return lines


def check_top_level_allowlist(tracked_paths: list[str]) -> list[str]:
    """Return tracked top-level entries that are not on the allowlist.

    :param tracked_paths: Repo-relative tracked paths (``git ls-files``).
    :returns: Sorted offending top-level entry names.
    """
    allowed = ALLOWED_TOP_LEVEL_DIRS | ALLOWED_TOP_LEVEL_FILES
    top_level = {path.split("/", 1)[0] for path in tracked_paths[:MAX_GIT_OUTPUT_LINES]}
    return sorted(entry for entry in top_level if entry not in allowed)


def check_tracked_but_ignored(ignored_tracked_paths: list[str]) -> list[str]:
    """Return tracked files matching ignore rules, minus the .gitkeep convention.

    :param ignored_tracked_paths: Output of
        ``git ls-files -i -c --exclude-standard``.
    :returns: Sorted offending paths.
    """
    return sorted(
        path
        for path in ignored_tracked_paths[:MAX_GIT_OUTPUT_LINES]
        if path.rsplit("/", 1)[-1] != ".gitkeep"
    )


def check_large_non_lfs_blobs(ls_tree_lines: list[str]) -> list[str]:
    """Return HEAD blobs exceeding the default or named budget (LFS pointers are tiny).

    :param ls_tree_lines: Output of ``git ls-tree -r -l HEAD`` — each line is
        ``<mode> <type> <oid> <size>\\t<path>`` (size is ``-`` for non-blobs).
    :returns: Sorted ``"path (size bytes)"`` strings for offending blobs.
    """
    violations: list[str] = []
    for line in ls_tree_lines[:MAX_GIT_OUTPUT_LINES]:
        meta, _, path = line.partition("\t")
        if not path or path in LARGE_BLOB_EXEMPTIONS:
            continue
        fields = meta.split()
        if len(fields) != 4 or fields[0] == _SYMLINK_MODE or fields[1] != "blob":
            continue
        size_field = fields[3]
        if not size_field.isdigit():
            continue
        if int(size_field) > RUNTIME_THEME_BLOB_LIMITS.get(path, MAX_BLOB_BYTES):
            violations.append(f"{path} ({size_field} bytes)")
    return sorted(violations)


def main() -> int:
    """Run all three hygiene checks against the repository; print violations.

    :returns: 0 clean, 1 violations found, 2 git infrastructure failure.
    """
    try:
        tracked = _git_lines(["ls-files", "-z"], nul_separated=True)
        ignored_tracked = _git_lines(
            ["ls-files", "-z", "-i", "-c", "--exclude-standard"], nul_separated=True
        )
        tree_lines = _git_lines(["ls-tree", "-r", "-l", "-z", "HEAD"], nul_separated=True)
    except RuntimeError as exc:
        print(f"HYGIENE GATE ERROR: {exc}", file=sys.stderr)
        return 2

    failures: list[tuple[str, list[str]]] = [
        ("top-level entry not on root allowlist", check_top_level_allowlist(tracked)),
        ("tracked file matches .gitignore", check_tracked_but_ignored(ignored_tracked)),
        (
            f"tracked blob exceeds {MAX_BLOB_BYTES} bytes and is not LFS",
            check_large_non_lfs_blobs(tree_lines),
        ),
    ]

    exit_code = 0
    for label, violations in failures:
        for violation in violations:
            print(f"HYGIENE VIOLATION [{label}]: {violation}", file=sys.stderr)
            exit_code = 1
    if exit_code == 0:
        print("Repo hygiene: clean (allowlist, ignore-consistency, blob sizes).")
    return exit_code


if __name__ == "__main__":
    sys.exit(main())
