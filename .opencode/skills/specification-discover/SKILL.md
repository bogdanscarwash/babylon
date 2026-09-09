---
name: specification-discover
description: Test scope with Constitution v4 before you make a plan for Babylon.
---

# Specification Scope

Use this phase before you make a plan. Test that the task belongs in Babylon.
Test that one causal slice can use it.

Read `CONSTITUTION.md` v4.0.0 and `NORTH_STAR.md` first. Babylon is an
entertainment-first game. The North Star gives the gate order.

Do not report these planned parts as complete:

- `player actions`
- `Archive materialization`
- `Postgres consolidation`

## Laws

- Connect the task to a player choice or one causal slice.
- Do not make all system ports a condition for play.
- Mark each value `Observed`, `Derived`, `Calibrated`, or `Designed`.
- Keep the data resolution equal to the source resolution.
- Let a shock add pressure. Do not let it write a result.
- A game display must help the player read, choose, or understand an effect.
- Keep geography fixed. Put political claims in overlays.
- Put data storage after tick judgment.

## Steps

1. Restate the user goal. Ask the user to check it.
2. Name the data change and the game rule that will use it.
3. Name the player choice or causal slice that will use the result.
4. Name the formal operator and the behavioral test.
5. Compare the requested resolution with the source resolution.
6. Check that the shock does not write a downstream result.
7. List each section of Constitution v4 that applies.
8. Use ADR221 to translate an old citation.

Give one verdict: `PASS`, `DEFER`, or `ESCALATE`.

For `DEFER`, name the source, game rule, or player choice that the task needs.
For `ESCALATE`, give the law that stops the task.

Use `.specify` as the parent directory. Use the phase name as its child
directory. Use `{YYYYMMDD}-{topic}.md` for the document name. Include the goal,
parts, verdict, applicable laws, blockers, and recommended phase.

## Escalation

Load `specification-govern` if the task needs one of these changes:

- a new mathematical primitive
- a weaker prohibition
- a change to the reserved theory line
- new content vocabulary without its ceremony

Do not continue past such a change without authority.
