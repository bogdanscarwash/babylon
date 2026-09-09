#!/bin/sh
# R1.4 fence, workflow-visible half (Director ruling 2026-08-18, BSL
# refactor program plan §14: "bevy/dynamic_linking — RULED: APPROVED WITH
# FENCE"). bevy's dynamic_linking feature is an OPT-IN, LOCAL-ITERATION-ONLY
# convenience for babylon-client (`mise run rust:client-dev-dylib`) — faster
# incremental Bevy rebuilds via libbevy_dylib.so. It must NEVER ride a CI,
# release, or pin-ceremony build: the .so is not meant to ship, and every
# gate build must prove the ordinary (statically-linked, default-features)
# path. This script is the static/textual half of the fence; rust/crates/
# babylon-client/tests/dynamic_linking_fence.rs is the compiled half
# (asserts the feature is off in whatever build ran the test suite).
#
# Matches ACTIVATION syntax only (`--features ... dynamic_linking` /
# `--all-features`), never the bare word — this file, ci.yml's own step
# name, and every task description below are free to keep SAYING
# "dynamic_linking" in prose without tripping the gate; only something that
# would actually turn the feature ON does.
#
# Round 1 (2026-08-18, task-fix1-report.md) fixed the original quote-blind
# regex with a character-class allowlist that deliberately excluded `-`.
# Round 2 (2026-08-18, task-fix1-review.md) found that exclusion breaks on
# a LEGAL cargo feature name containing a hyphen sitting between
# `--features` and `dynamic_linking` in the same list —
# `--features "some-feature,dynamic_linking"` evaded round 1's regex
# entirely, and the unquoted `--features=another-thing,dynamic_linking`
# form evaded it too even though the ORIGINAL pre-round-1 regex caught
# that one (a real regression, introduced by the hyphen exclusion). The
# review also flagged a pre-existing, unrelated gap: a shell
# line-continuation split (`--features \` / newline / `dynamic_linking`)
# evades any line-oriented grep no matter what the character class allows,
# since neither physical line alone contains both the flag and the word.
#
# Both are fixed by abandoning the single-regex approach for invariants 1
# and 2 in favor of a small awk program (embedded below via a heredoc into
# a temp file, `$AWK_JOIN_MATCH`) that:
#   1. Joins backslash-continuation lines into one logical line first, so
#      a split activation can no longer hide between two physical lines.
#   2. Scans each logical line for `--all-features`, or for `--features`
#      (`=`-joined or space-joined) followed by its ACTUAL argument —
#      bounded by the real quote characters (`"`, `'`, or TOML's
#      `\"`-escaped form, which this repo's own .mise.toml already uses)
#      when quoted, or by the next whitespace when not — never by a
#      character-class guess. Cargo feature names may legally contain
#      hyphens (also `_`, `.`, `+`); bounding the argument by its real
#      shell/TOML delimiters instead of an allowlisted character class
#      means a hyphenated sibling feature name no longer breaks the scan,
#      with no need to special-case `-` (or any other legal feature-name
#      character) at all.
#   3. Only checks the EXTRACTED ARGUMENT for `dynamic_linking` — so the
#      bare-word non-match property still holds: `--features foo`
#      followed later on the same line by an unrelated comment mentioning
#      "dynamic_linking" still never trips it, because the argument
#      boundary correctly stops at the whitespace after `foo`, before the
#      comment is ever reached.
# Verified directly against /usr/bin/grep and real awk (GNU Awk 5.2.1),
# never the interactive shell's grep-as-ugrep wrapper — see
# task-fix1-report.md's round 2 section for the full drill matrix.
#
# Checks, offline and in-repo:
#   1. No GitHub Actions workflow ever activates dynamic_linking (an
#      explicit `--features ... dynamic_linking` or a blanket
#      `--all-features`) — a workflow IS a CI/release/pin-ceremony build.
#   2. .mise.toml activates dynamic_linking ONLY inside the sanctioned
#      local-dev task's own `run` line (rust:client-dev-dylib) — any other
#      task's `run` doing so (rust:check included) is a leak into the gate.
#   3. No rust/**/Cargo.toml lists dynamic_linking inside a `default = [...]`
#      features array — default-on would silently activate it for every
#      plain `cargo build`/`test`, including CI's.
# Exit 0 clean / 1 violation found / 2 error.
set -eu

# Shared join+extract program for invariants 1 and 2 (design rationale in
# the header above). Written to a temp file via a QUOTED heredoc
# (<<'AWKSCRIPT') so the shell performs zero expansion/escaping on its
# body — the program needs literal ", ', \, and $ characters for its own
# quote/whitespace-boundary detection and awk's own field syntax, and a
# quoted heredoc is the only embedding form that avoids a three-way fight
# between shell single-quotes, awk string-literal escaping, and POSIX ERE
# escaping (a real problem: round 1's ACTIVATION_RE lived entirely inside
# shell single-quotes and never needed a literal `'`; this program does,
# to detect `'...'`-quoted arguments).
# MODE="line" (invariant 2, single file) prints "startline:logical",
# matching plain `grep -n`'s single-file output format, so invariant 2's
# existing sanctioned-line diff logic below needs no other change. Any
# other MODE (invariant 1, multi-file via find -exec) prints
# "FILENAME:startline:logical", matching `grep -rn`'s multi-file format.
AWK_JOIN_MATCH=$(mktemp)
trap 'rm -f "$AWK_JOIN_MATCH"' EXIT
cat <<'AWKSCRIPT' > "$AWK_JOIN_MATCH"
FNR == 1 { buf = ""; joinstart = 0 }
{
    if (joinstart == 0) joinstart = FNR
    line = $0
    if (line ~ /\\$/) {
        sub(/\\$/, "", line)
        buf = buf line
        next
    }
    buf = buf line
    logical = buf
    buf = ""
    start = joinstart
    joinstart = 0

    # This exact, single physical YAML run command only audits dependencies.
    # Anchor the complete command: shell operators, substitutions, comments,
    # extra arguments and continued lines must still pass the activation scan.
    # Never exempt .mise task lines or a substring inside another command.
    if (MODE != "line" && start == FNR && logical ~ /^[[:space:]]*run:[[:space:]]+cargo-deny --log-level warn --manifest-path rust\/Cargo\.toml --all-features check advisories bans licenses sources[[:space:]]*$/) next

    hit = 0
    if (index(logical, "--all-features") > 0) hit = 1

    if (!hit) {
        n = length(logical)
        i = 1
        while (i <= n) {
            p = index(substr(logical, i), "--features")
            if (p == 0) break
            p = i + p - 1
            j = p + 10
            sep_ok = 0
            if (substr(logical, j, 1) == "=") { j++; sep_ok = 1 }
            else {
                while (substr(logical, j, 1) ~ /[[:space:]]/) { j++; sep_ok = 1 }
            }
            if (!sep_ok) { i = p + 1; continue }

            c1 = substr(logical, j, 1)
            c2 = substr(logical, j, 2)
            if (c2 == "\\\"") { closer = "\\\""; astart = j + 2 }
            else if (c1 == "\"") { closer = "\""; astart = j + 1 }
            else if (c1 == "'") { closer = "'"; astart = j + 1 }
            else { closer = ""; astart = j }

            if (closer != "") {
                k = index(substr(logical, astart), closer)
                if (k == 0) arg = substr(logical, astart)
                else arg = substr(logical, astart, k - 1)
            } else {
                rest = substr(logical, astart)
                sp = match(rest, /[[:space:]]/)
                if (sp == 0) arg = rest
                else arg = substr(rest, 1, sp - 1)
            }

            if (index(arg, "dynamic_linking") > 0) { hit = 1; break }
            adv = j + length(arg)
            if (closer != "") adv += length(closer)
            if (adv <= i) adv = i + 1
            i = adv
        }
    }

    if (hit) {
        if (MODE == "line") print start ":" logical
        else print FILENAME ":" start ":" logical
    }
}
AWKSCRIPT

FAIL=0

# --- 1. workflows: no step may actually turn the feature on ---
hits=$(find .github/workflows -type f -exec awk -v MODE=file -f "$AWK_JOIN_MATCH" {} + 2>/dev/null || true)
if [ -n "$hits" ]; then
    echo "check_dynamic_linking_fence: REFUSE — a GitHub Actions workflow activates dynamic_linking:" >&2
    printf '%s\n' "$hits" >&2
    FAIL=1
fi

# --- 2. .mise.toml: only the sanctioned task's run line may activate it ---
[ -f .mise.toml ] || { echo "check_dynamic_linking_fence: FATAL no .mise.toml" >&2; exit 2; }
SANCTIONED_HEADER='[tasks."rust:client-dev-dylib"]'
all_activations=$(awk -v MODE=line -f "$AWK_JOIN_MATCH" .mise.toml || true)
if [ -n "$all_activations" ]; then
    sanctioned_line=$(awk -v start="$SANCTIONED_HEADER" '
        $0 == start { inblock=1; next }
        inblock && /^\[tasks\./ { exit }
        inblock && /^run[[:space:]]*=/ { print NR; exit }
    ' .mise.toml)
    leaked=$(printf '%s\n' "$all_activations" | awk -F: -v ok="${sanctioned_line:-0}" '$1 != ok')
    if [ -n "$leaked" ]; then
        echo "check_dynamic_linking_fence: REFUSE — .mise.toml activates dynamic_linking outside the sanctioned rust:client-dev-dylib task's run line:" >&2
        printf '%s\n' "$leaked" >&2
        FAIL=1
    fi
fi

# --- 3. no Cargo.toml makes dynamic_linking a DEFAULT feature ---
# A crate's [features] `default = [...]` array can span multiple lines
# (this workspace already has multi-line-array precedent: rust/Cargo.toml's
# own [workspace] `members = [` list) — a same-line-only regex would miss
# a `default = [` / `"dynamic_linking",` / `]` split across three lines.
# One awk invocation scans every rust/**/Cargo.toml (find -exec ... {} +
# batches all matches into that single call, no shell loop over find
# output needed): FNR==1 resets state per file; infeatures tracks the
# [features] table specifically (default arrays elsewhere are out of
# scope by design); capturing spans from the `default =` line to the line
# holding its closing `]`, same-line or not; only the captured span is
# checked for dynamic_linking — never the bare word anywhere else in the
# file (a crate's own `dynamic_linking = [...]` feature *definition* line
# must never trip this).
default_hits=$(find rust -name 'Cargo.toml' -exec awk '
    FNR == 1 { infeatures = 0; capturing = 0 }
    /^\[features\]/ { infeatures = 1; next }
    /^\[/            { infeatures = 0; capturing = 0 }
    infeatures && !capturing && /^default[[:space:]]*=/ {
        start = FNR
        buf = $0
        if ($0 ~ /\]/) { if (buf ~ /dynamic_linking/) print FILENAME ":" start ": " buf; next }
        capturing = 1
        next
    }
    capturing {
        buf = buf "\n" $0
        if ($0 ~ /\]/) {
            if (buf ~ /dynamic_linking/) print FILENAME ":" start "-" FNR ":\n" buf
            capturing = 0
        }
    }
' {} + 2>/dev/null || true)
if [ -n "$default_hits" ]; then
    echo "check_dynamic_linking_fence: REFUSE — dynamic_linking is a DEFAULT feature (activates on every plain cargo build/test):" >&2
    printf '%s\n' "$default_hits" >&2
    FAIL=1
fi

if [ "$FAIL" -ne 0 ]; then
    exit 1
fi

printf 'check_dynamic_linking_fence: OK — dynamic_linking activation confined to rust:client-dev-dylib, absent from every workflow, never a default feature.\n'
exit 0
