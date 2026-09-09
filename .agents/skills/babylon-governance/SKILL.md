---
name: babylon-governance
description: Query or draft Babylon ADRs and assess actual game-law changes. Use for ADR status, what governs a topic, a requested architecture ruling, new primitives, weakened prohibitions, reserved theory, or amendments. Do not use for ordinary architecture explanation, ADR-tool implementation, exploratory brainstorming, or implementation lookup.
---

<!-- vale off -->
<!-- Machine instructions preserve exact CLI and governance terms. -->

# Babylon governance

Start with computation, not the corpus.

## ADR lookup

1. Run `mise run adr -- search "<topic>"` or `mise run adr -- show ADRNNN`.
2. Search returns newest ADR matches first. If `results_truncated` is true,
   use `--offset <next_offset>` only when the first page lacks the relevant ADR.
3. Treat structured `status`, index metadata, and diagnostics as recorded facts,
   not proof of live behavior.
4. Use the source path and selector from `show`. Read only that file and the
   relevant field when exact rationale is needed.
5. Verify a live-behavior claim against architecture, source, executable tests,
   and Linear. Never read the full ADR index for lookup.

## Game-law assessment

Read `CONSTITUTION.md`, ADR221, and the relevant returned ADR source only when
the proposal changes game law. Stop for the Director if it adds a primitive,
weakens a prohibition, changes the reserved theory line, or authors governed
political content.

For a formal construct, ask whether it yields a law, prediction, or running
computation. Move deterministic enforcement into a check while retaining the
smallest normative statement that explains what the check protects.

## ADR drafting

Draft only when the user explicitly requests a new decision record and an
architecture decision actually exists.

- Keep Git YAML authoritative; the SQLite catalog never writes back.
- Preserve old ADRs. Express reversal through explicit supersession.
- State decision, alternatives rejected, consequences, evidence class, and
  scoped supersession. Keep ordinary rulings concise.
- Default reserved-law drafts to proposed. Never self-ratify reserved changes.
- Add the registry row required by current sentinels, then run
  `mise run check:adr-catalog` and the relevant ADR tests.

Do not turn implementation plans, PR narratives, or speculative essays into
authority.

<!-- vale on -->
