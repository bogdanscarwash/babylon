#!/usr/bin/env python3
"""Static, standalone re-implementation of the E-TYPE-040 kind-mixing arm
(rust/crates/babylon-bsl/src/typecheck.rs's expr_kind/add_sub_kind/
mul_div_kind/if_kind/fold_kind) for auditing every <arith>/if site across
ALL committed BSL rule packs at once, without a cargo build — used once as
a complete pre-landing sweep before D183's intensive-div licensing (#491
T1, 2026-08-18: the controller's explicit "no more one-layer-per-round-trip"
instruction), validated against a known-clean file and a pre-repair
historical commit's own known finds before being trusted for that verdict.
Kept here so a future kind-arm change can re-run the same audit instead of
re-deriving it; NOT wired into CI or any gate — the Rust arm is the
enforced source of truth, this is a cheap advisory cross-check. Keep the
kind tables below in sync with typecheck.rs's mul_div_kind/add_sub_kind/
if_kind by hand; a drift between the two only means this script's verdict
goes stale, it cannot mask a real Rust-side regression.

Usage:
    python3 tools/kind_straddle_static_sweep.py [rule_file scenario_file]

With no arguments, sweeps all rule/scenario pairs under
rust/crates/babylon-tick/content/{rules,scenarios}/ known at authoring
time (see RULE_SCENARIO_PAIRS below — add a pair by hand when a new pack
lands). With two path arguments, sweeps just that one pair.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
CONTENT_RULES = REPO_ROOT / "rust/crates/babylon-tick/content/rules"
CONTENT_SCENARIOS = REPO_ROOT / "rust/crates/babylon-tick/content/scenarios"

# (rule file, its primary conformance/foundation scenario) — the same 13
# packs #491 T1's D183 sweep walked. Add a row by hand when a new pack lands.
RULE_SCENARIO_PAIRS = [
    ("lifecycle.bsl", "lifecycle-conformance.bscn"),
    ("decomposition.bsl", "decomposition-conformance.bscn"),
    ("control-ratio.bsl", "control-ratio-conformance.bscn"),
    ("dispossession.bsl", "dispossession-conformance.bscn"),
    ("metabolism.bsl", "metabolism-conformance.bscn"),
    ("territory.bsl", "territory-conformance.bscn"),
    ("production.bsl", "production-conformance.bscn"),
    ("fundamental-theorem.bsl", "two-classes.bscn"),
    ("vitality.bsl", "vitality-conformance.bscn"),
    ("organization.bsl", "organization-foundation.bscn"),
    ("worldview.bsl", "worldview-foundation.bscn"),
    ("consciousness.bsl", "consciousness-ternary-conformance.bscn"),
    ("solidarity.bsl", "solidarity-conformance.bscn"),
]


def tokenize_with_lines(text: str) -> list[tuple[str, int]]:
    """Tokenize BSL source into (token, line) pairs. Handles ``;`` comments
    (to EOL), ``"..."`` string literals (atomic, no internal comment/paren
    interpretation), parens as their own tokens, everything else
    whitespace-split atoms."""
    tokens: list[tuple[str, int]] = []
    i = 0
    line = 1
    n = len(text)
    while i < n:
        c = text[i]
        if c == "\n":
            line += 1
            i += 1
            continue
        if c in " \t\r":
            i += 1
            continue
        if c == ";":
            j = text.find("\n", i)
            if j == -1:
                j = n
            i = j
            continue
        if c == '"':
            j = i + 1
            while j < n and text[j] != '"':
                if text[j] == "\\":
                    j += 2
                else:
                    j += 1
            j = min(j + 1, n)
            tokens.append(("STRLIT", line))
            line += text[i:j].count("\n")
            i = j
            continue
        if c in "()":
            tokens.append((c, line))
            i += 1
            continue
        j = i
        while j < n and text[j] not in " \t\r\n()" and text[j] != ";":
            j += 1
        tokens.append((text[i:j], line))
        i = j
    return tokens


def parse_forms(tokens: list[tuple[str, int]]):
    """Parse a flat (token, line) stream into nested lists. Returns a list
    of top-level forms; each form is ``('LIST', items, line)`` or
    ``('ATOM', text, line)``."""
    pos = [0]

    def read():
        tok, ln = tokens[pos[0]]
        if tok == "(":
            pos[0] += 1
            lst = []
            startln = ln
            while tokens[pos[0]][0] != ")":
                lst.append(read())
            pos[0] += 1
            return ("LIST", lst, startln)
        pos[0] += 1
        return ("ATOM", tok, ln)

    forms = []
    while pos[0] < len(tokens):
        forms.append(read())
    return forms


def load_scenario_kinds(path: Path):
    text = path.read_text()
    field_kind: dict[str, str] = {}
    const_names: set[str] = set()
    for m in re.finditer(r"\(deffield\s+([^\s()]+)\s+([^\s()]+)\s+(intensive|extensive)\)", text):
        qname, _ty, kind = m.groups()
        field_kind[qname] = "I" if kind == "intensive" else "E"
    for m in re.finditer(r"\(deffield\s+([^\s()]+)\s+enum\s+", text):
        field_kind[m.group(1)] = "NA"
    for m in re.finditer(r"\(defconst\s+([^\s()]+)", text):
        const_names.add(m.group(1))
    return field_kind, const_names


def atom_text(node):
    return node[1] if node[0] == "ATOM" else None


class Sweeper:
    """One sweep pass over a single rule file. Violations accumulate in
    ``self.violations`` as (rule_name, line, op, left_kind, right_kind)."""

    def __init__(self, field_kind: dict[str, str]):
        self.field_kind = field_kind
        self.violations: list[tuple[str, int, str, str, str]] = []

    def field_lookup(self, qname: str):
        k = self.field_kind.get(qname)
        return k if k in ("E", "I") else None  # NA or unknown -> None

    def find_bindings(self, rule_items):
        """Returns dict name -> ('field', qname) | ('const',) |
        ('expr', node) | ('none',) | ('unknown',)."""
        bindings = {}
        for item in rule_items:
            if item[0] == "LIST" and item[1] and atom_text(item[1][0]) == "bindings":
                for b in item[1][1:]:
                    if b[0] != "LIST":
                        continue
                    bl = b[1]
                    if len(bl) < 3 or atom_text(bl[0]) != "binding":
                        continue
                    name = atom_text(bl[1])
                    src_kw = atom_text(bl[2])
                    if src_kw == ":field" and len(bl) > 3:
                        bindings[name] = ("field", atom_text(bl[3]))
                    elif src_kw == ":const":
                        bindings[name] = ("const",)
                    elif src_kw == ":expr" and len(bl) > 3:
                        bindings[name] = ("expr", bl[3])
                    elif src_kw in (
                        ":tick",
                        ":year",
                        ":tick-of-year",
                        ":tick-in-cycle",
                        ":metric",
                    ):
                        bindings[name] = ("none",)
                    else:
                        bindings[name] = ("unknown",)
        return bindings

    def symbol_kind(self, name, bindings, rule_name):
        src = bindings.get(name)
        if src is None:
            return None
        kind = src[0]
        if kind == "field":
            return self.field_lookup(src[1])
        if kind == "const":
            return "N"
        if kind == "expr":
            return self.expr_kind(src[1], bindings, rule_name)
        return None  # none/unknown/metric/tick

    def add_sub_kind(self, op, lk, r, rule_name, line):
        if lk == "N" and r == "N":
            return "N"
        if lk == "N":
            return r
        if r == "N":
            return lk
        if lk == r:
            return lk
        self.violations.append((rule_name, line, op, lk, r))
        return None

    def mul_div_kind(self, op, lk, r, rule_name, line):
        """Mirrors typecheck.rs's mul_div_kind AS OF D183 (2026-08-18): `*`
        and `/` agree on the (Intensive, Intensive) cell — no `op` guard —
        because D183 widened D182's `*`-only licensing to cover both
        operators. Extensive-mixed-with-intensive under `/` and
        extensive x extensive under `*` both stay refused (D181)."""
        if lk == "N" and r == "N":
            return "N"
        if lk == "N":
            return r
        if r == "N":
            return lk
        if lk == "E" and r == "E" and op == "/":
            return "I"
        if {lk, r} == {"E", "I"} and op == "*":
            return "E"
        if lk == "I" and r == "I":  # D183: both `*` (D182) and `/` licensed
            return "I"
        self.violations.append((rule_name, line, op, lk, r))
        return None

    def field_of_kind(self, items):
        if len(items) >= 3:
            qn = atom_text(items[2])
            if qn:
                return self.field_lookup(qn)
        return None

    def fold_kind(self, items, bindings, rule_name):
        if len(items) < 2:
            return None
        op = atom_text(items[1])
        if op in ("count", "sum"):
            return "E"
        if op in ("mean", "min", "max"):
            body_idx = 3
            if body_idx < len(items) and atom_text(items[body_idx]) == ":as":
                body_idx = 5
            if body_idx < len(items):
                return self.expr_kind(items[body_idx], bindings, rule_name)
        return None

    def if_kind(self, items, bindings, rule_name, line):
        if len(items) < 4:
            return None
        then_k = self.expr_kind(items[2], bindings, rule_name)
        else_k = self.expr_kind(items[3], bindings, rule_name)
        if then_k == "N" and else_k is not None:
            return else_k
        if else_k == "N" and then_k is not None:
            return then_k
        if then_k is not None and else_k is not None:
            if then_k == else_k:
                return then_k
            self.violations.append((rule_name, line, "if", then_k, else_k))
            return None
        return then_k if then_k is not None else else_k

    def expr_kind(self, node, bindings, rule_name):
        if node[0] == "ATOM":
            tok = node[1]
            if tok in ("#t", "#f"):
                return "N"
            if re.match(r"^-?[0-9]", tok):
                return "N"
            if re.match(r"^[a-zA-Z_][a-zA-Z0-9_-]*$", tok):
                return self.symbol_kind(tok, bindings, rule_name)
            return None
        items = node[1]
        line = node[2]
        if not items:
            return None
        head = items[0]
        if head[0] == "ATOM" and head[1] in ("+", "-", "*", "/"):
            op = head[1]
            if len(items) < 3:
                return None
            lk = self.expr_kind(items[1], bindings, rule_name)
            r = self.expr_kind(items[2], bindings, rule_name)
            if lk is None or r is None:
                return None
            if op in ("+", "-"):
                return self.add_sub_kind(op, lk, r, rule_name, line)
            return self.mul_div_kind(op, lk, r, rule_name, line)
        if head[0] == "ATOM" and head[1] == "if":
            return self.if_kind(items, bindings, rule_name, line)
        if head[0] == "ATOM" and head[1] == "field-of":
            return self.field_of_kind(items)
        if head[0] == "ATOM" and head[1] == "fold":
            return self.fold_kind(items, bindings, rule_name)
        return None

    def walk_for_violations(self, node, bindings, rule_name):
        if node[0] != "LIST":
            return
        items = node[1]
        if items:
            head = items[0]
            is_kinded = (head[0] == "ATOM" and head[1] in ("+", "-", "*", "/")) or (
                head[0] == "ATOM" and head[1] == "if"
            )
            if is_kinded:
                self.expr_kind(node, bindings, rule_name)
        for child in items:
            self.walk_for_violations(child, bindings, rule_name)

    def sweep_rule_file(self, rules_text: str) -> int:
        """Returns the number of `rule` forms walked."""
        tokens = tokenize_with_lines(rules_text)
        top_forms = parse_forms(tokens)
        rule_count = 0
        for form in top_forms:
            if form[0] != "LIST":
                continue
            items = form[1]
            if not items or atom_text(items[0]) != "rule":
                continue
            rule_name = atom_text(items[1]) if len(items) > 1 else "?"
            rule_count += 1
            bindings = self.find_bindings(items)
            for item in items:
                if (
                    item[0] == "LIST"
                    and item[1]
                    and atom_text(item[1][0])
                    in (
                        "when",
                        "effects",
                    )
                ):
                    self.walk_for_violations(item, bindings, rule_name)
                # :expr bindings never referenced by when/effects would
                # otherwise be missed — walk their trees explicitly too.
                if item[0] == "LIST" and item[1] and atom_text(item[1][0]) == "bindings":
                    for b in item[1][1:]:
                        if b[0] != "LIST":
                            continue
                        bl = b[1]
                        if len(bl) > 3 and atom_text(bl[2]) == ":expr":
                            self.walk_for_violations(bl[3], bindings, rule_name)
        return rule_count


def sweep_one(rules_path: Path, scenario_path: Path) -> list[tuple[str, int, str, str, str]]:
    field_kind, _const_names = load_scenario_kinds(scenario_path)
    sweeper = Sweeper(field_kind)
    rule_count = sweeper.sweep_rule_file(rules_path.read_text())
    seen: set[tuple[str, int, str, str, str]] = set()
    ordered: list[tuple[str, int, str, str, str]] = []
    for v in sweeper.violations:
        if v not in seen:
            seen.add(v)
            ordered.append(v)
    print(
        f"  {rules_path.name}: {rule_count} rules, {len(ordered)} violation site(s)",
        file=sys.stderr,
    )
    return ordered


def main(argv: list[str]) -> int:
    if len(argv) == 3:
        pairs = [(Path(argv[1]), Path(argv[2]))]
    elif len(argv) == 1:
        pairs = [
            (CONTENT_RULES / rule_file, CONTENT_SCENARIOS / scen_file)
            for rule_file, scen_file in RULE_SCENARIO_PAIRS
        ]
    else:
        print(__doc__)
        return 2

    total_violations = 0
    for rules_path, scenario_path in pairs:
        if not rules_path.exists() or not scenario_path.exists():
            print(f"SKIP (missing): {rules_path} / {scenario_path}", file=sys.stderr)
            continue
        violations = sweep_one(rules_path, scenario_path)
        total_violations += len(violations)
        for rule_name, line, op, lk, r in violations:
            print(f"  {rules_path.name}:{line}  rule={rule_name} op={op} left={lk} right={r}")

    print(f"\nTotal distinct violation sites across all swept files: {total_violations}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
