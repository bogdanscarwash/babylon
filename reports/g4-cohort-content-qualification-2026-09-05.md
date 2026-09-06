# G4 cohort content and save-format qualification — 5 September 2026

This development receipt records executable evidence from the observer lane
based on `23d912c98e8566769d5f35cda9a50436dc598115`, with uncommitted changes.
This receipt does not qualify the final head or claim G4 completion.

## Observed content and its consumer

The pinned QCEW source supplies 1,603 nonzero county-sector BUSINESS aggregates
across Michigan's 83 counties. Nineteen native sector hyperedges classify 1,522
of those aggregates. The 81 unclassified cells keep code 99 without an
invented sector membership. Fifty-seven absent county-sector cells create no
nodes. Suppressed cells keep observed establishments while withheld
employment, annual payroll and mean weekly wage remain absent.

The five Designed material processes reference four deduplicated Observed
manufacturing contexts. Wayne's two processes share one county-sector context.
These references assign neither workers nor output. The inspector presents
the 2024 baseline separately from completed modeled labor-hours. Preview receives
no full-observer material or cohort context.

The committed labor-time account reconciles used and unused hours against the
prior opening budget, with the next opening budget displayed separately.
Foundation observations have no invented completed work week. Hours are not
employment, job losses, paid wages or ReserveArmy movement.

## Explicit content format and saved identities

The full cohort scenario contains 493,177 UTF-8 bytes. Its graph loads through
BSL, but the historical V1 foundation format correctly rejects source fields
above 65,535 bytes. The explicit V2 content format permits 1 MiB per source
field under the unchanged 64 MiB complete-content bound. It retains UTF-8 and
NUL validation and binds the selected version into the foundation identity.

Existing V1 constructors, bounds and wire vectors remain intact. A durable
version row selects the format for reconstruction. Neither source length nor
failed hashes select another format. Installation records historical V1
versions once under a lock. Later missing or changed versions refuse.
Both runtime implementations check and hold the required version row before
writes, retry acknowledgements and ambiguous-commit reconciliation.

Three pre-existing native QA saves independently supplied these historical
standard-V1 identity pins, without invoking the new factory:

- Material foundation: `6c24dfb1cdd1ca2b6fe19f99a5c44c8f413043c4ee61dbead816285de84e0695`
- Graph foundation: `a5b141825fa5199eddc27a0f0e4f58a30b11a70facac3544c5d646a01fb319f3`

The retained test asserts both pins. New campaign creation selects the cohort
revision. Existing county-baseline campaigns keep their original revision.

## Executable evidence

| Check | Result | Local log |
| --- | --- | --- |
| Client library | 312 passed | `/tmp/babylon-g4-cohorts-format-inspector-green-20260905.log` |
| Persistence library | 120 passed, 24 live tests ignored | Same combined library log |
| Foundation formats, retained vectors, checkpoints, material replay and causal scenarios | 42 passed | `/tmp/babylon-g4-foundation-material-contracts-20260905.log` |
| Live material/Archive/content and campaign-owner checks | 9 passed | `/tmp/babylon-g4-campaign-owner-live-green-20260905.log` |
| Live restricted-reader checks | 5 passed | Same live log |
| Live graph runtime, restart and retry checks | 16 passed | `/tmp/babylon-g4-graph-ack-live-green-20260905.log` |
| Frozen Python regression scenarios and independent-process determinism | 12 scenarios passed | `/tmp/babylon-g4-regression-20260905.log` |
| Retained golden vault | Both scenarios matched two independent bakes | `/tmp/babylon-g4-vault-regression-20260905.log` |
| Python static checks | Passed | `/tmp/babylon-g4-python-static-20260905.log` |
| Complete canonical Python check | 15,016 passed, 14 skipped, one expected failure. Static checks passed. | `/tmp/babylon-g4-python-check-green-20260905.log` |
| Archive dossier integration | 13 passed, including shared layout at both target resolutions | `/tmp/babylon-g4-dossier-layout-green-20260905.log` |
| Independent comparison and perspective shortcuts | Reproduced the collision, then passed after comparison moved to X | `/tmp/babylon-g4-keyboard-collision-green-20260905.log` |
| Strict Rust gate without documentation builds | 3,154 passed, 69 ignored. Format, Clippy, documentation example tests and BSL checks passed. | `/tmp/babylon-g4-rust-gate-pass5-20260905.log` |

The live tests prove that four admitted content revisions coexist. They resume
the same next ticks and preserve historical reads. Earlier catalog entries
outside admission do not hide them. Tests cover first version assignment,
repeated installation, unknown and mixed versions, and deletion after
installation.

Two live tests first reproduced unsafe acknowledgement after
deleting required version metadata. Both passed after the locking repair.
Each harness verified removal of its owned container, volume and scratch
databases.

Two further live tests reproduced duplicate campaign ownership at creation.
A graph runtime claimed an existing material campaign. Concurrent creation
also attached a material owner to the winning graph campaign. Both tests
passed after the creators shared a lock and the graph runtime checked its
owner.

A third test confirmed that graph campaigns still create, reopen and
commit before the material schema exists. These checks prove admission.
The existing database trigger already refused graph commits to a material
campaign.

The first full Python unit run found one stale artifact-count assertion.
The source hashes already matched. The inventory correction changed 27 to 29
and added the new names. The focused file then passed 26 tests, with three
distribution-artifact skips. The repeated complete canonical check passed all
15,016 tests and its static checks.

The coverage sentinel accepted the estate declaration but reports no declaration
for its own scope. This result does not prove broader game coverage.

## Limits of this receipt

Native inspection of the newly expanded content, final async-frame measurements,
exact-head qualification and Director comprehension are still required. Individual
sector bundles, wider productive coverage and governed person-flow mechanics
remain separate work. The physical scenario still contains five Designed
processes. Neither these tests nor observer controls prove G5 player agency.
