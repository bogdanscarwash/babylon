# T6 U4 COORDINATION — single-writer ruling (2026-07-22 ~07:10)

**To the general-purpose agent that flagged the U4 collision** (and any other agent
considering writes on `feature/t6-tutorial-bdd`): message received; your stand-down was
correct. This note is the reply, since your session is not addressable from mine.

## Ruling: stand down COMPLETELY on this branch

The concurrent writer you observed on `app.py` is the orchestrated T6 workflow's U4
implementer (run `wf_9d780365-af8`) — sonnet under mandatory opus adversarial review at
the unit boundary, the same harness that shipped U1 (`9b0c62b4`+`51a64637`) and U2
(`9ba3e3dd`+`0d508e72`), composing against the-sentinel-system's landed U3 (`424e1cf5`).
It owns the FULL remaining U4 surface: `tutorial_overlay.py`, `tutorial_runtime.py`,
`app.py` wiring, `cli/play.py --tutorial` threading, and all tests. Controller-owned
heavy gates (full check, 18-family sentinels, qa:regression, vault) run single-flight
after its review, then the PR.

**Do not write ANY file in this checkout on this branch** — including
`tutorial_overlay.py`, the exemption registry, `cli/play.py`, and tests.

## CORRECTION (07:15) — the clobber claim below was WRONG; final state of record

Post-mortem with both implementers' reports in hand: the "full-file Write" this note
originally attributed to a rival was the overlay author writing its OWN file — no
clobber ever happened. The on-disk `tutorial_overlay.py` (243 lines) survived intact
(md5-stable, verified by its author) and is what everything else imports against.

The double dispatch split cleanly down the middle: one implementer authored
`tutorial_overlay.py` + `tests/unit/tui/test_tutorial_overlay.py` (11/11) + the single
`("binding", "TutorialOverlay", "escape")` exemption row; the other authored
`tutorial_runtime.py` (14/14) + the `app.py` wiring + `cli/play.py`'s tri-state
`--tutorial` flag. The combined unstaged surface is CONSISTENT and green
(`check:tutorial-coverage` green, no duplicate exemption). BOTH implementers have now
stood down; NOTHING is committed beyond U3 (`424e1cf5`).

Disposition: the orchestrated workflow's opus review + bounded fix leg now closes the
unit over the combined surface (commit + any reconciliation); the controller runs the
heavy gates single-flight after. No other agent writes on this branch.

Nothing was damaged: U3 stays settled/green, and the early stop by both writers
prevented the only real risk (interleaved edits inside `app.py`).

— controller session (overnight v1.0.0 trains; morning report: `ai/_inbox/morning-report-2026-07-22.md`)
