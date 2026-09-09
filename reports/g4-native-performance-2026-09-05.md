# Native observer performance measurement — 5 September 2026

The development build sustained the display refresh rate in 3D production and
compact 2D. The measurements cover paused historical inspection. They do not
prove performance during advances or complete G4 acceptance.

The source combined `23d912c98e8566769d5f35cda9a50436dc598115` with
uncommitted changes. External dependencies used development `opt-level = 3`.
Workspace crates retained development debug assertions and overflow checks.
The measured binaries predate the new industry-cohort admission and inspector
context work.

| Surface | Window | Three settled FPS readings | Median frame range | Highest p95 |
| --- | --- | --- | --- | --- |
| 3D production | 1366 × 768 | 59.77, 59.79, 59.79 | 16.724–16.753 ms | 17.807 ms |
| Compact 2D | 1366 × 768 | 59.80, 59.79, 59.78 | 16.703–16.766 ms | 18.197 ms |
| 3D production | 1920 × 1080 | 59.95, 59.96, 59.93 | 16.682–16.715 ms | 18.102 ms |
| Compact 2D | 1920 × 1080 | 59.96, 59.93, 59.95 | 16.685–16.725 ms | 17.858 ms |

Each reading contains 300 frames. Both surfaces used the same campaign at
committed week 5, with durable tail 13 and the Wayne vehicle-parts selection.
Both used a 60-event log, full-observer perspective, 100% UI scale, ordinary
motion and enabled audio. History and detailed Archive panels stayed closed.
The session requested no advances.

Display modes were 59.79 Hz and 59.94 Hz
respectively. After measurement, the operator restored the original
1366 × 768 mode. The owned game process closed normally through its menu.

An earlier development measurement without dependency optimization recorded
approximately 29 FPS at 1366 × 692. The different window size prevents a
comparison at the same resolution.

Binary SHA-256 identities:

- Client: `cd659be904eaeff7540fece7ec9c24a0e2ebde06598a469b42464560bd4dec2d`
- Runtime: `59626faf7604bacb98e6b89ac4f6fb5332593483a097ee1a22c093089f1d2bf1`

The local evidence files are
`/tmp/babylon-g4-perf-opt3-receipt-20260905.json` and
`/tmp/babylon-g4-perf-opt3-20260905.log`. This development receipt is separate
from the required exact-head Director comprehension session.
