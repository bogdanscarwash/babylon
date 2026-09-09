# Native cohort observer check — 5 September 2026

The operator tested clean head `22427dbb6c586b8002ae7713fa404cdeeb2f98a8` in a real Bevy window.
This is an assistant-operated development check. Director comprehension and
complete G4 acceptance remain open.

## Campaign and control behavior

- The existing county-baseline campaign reopened at durable week 13.
- A separate campaign used the new county-sector revision. It completed two
  monthly planning steps and paused at their boundaries.
- Pause let the in-flight week finish at week 12 and stopped further advances.
- History selected week 5 without changing the durable tail or losing the selected production subject.
- The new campaign reopened in a fresh native process at week 12.
- Its selected county dossier refreshed with content from week 1, verified through week 12.
- Switching to player knowledge cleared the production scene, detailed readings and event log.
- All three sessions closed normally through the game menu. The operator
  restored HDMI-0 to 1366 × 768 at 59.79 Hz.

The native inspector showed the completed week-5 work account as 16 used plus
zero unused hours out of 16 available. The next opening budget appeared
separately. The observed Wayne manufacturing context showed 1,710 establishments
and 89,659 annual-average jobs. It showed USD 8,169,721,127 annual payroll and
USD 1,752 mean weekly wage. Its citation and both processes sharing that context were visible.
It assigned neither workers nor output to either process.

## Frame measurements

Both historical surfaces used committed week 5, durable week 12, the same Wayne
vehicle-parts selection and full-observer perspective. Both used 100% UI scale,
expanded readings, ordinary motion and enabled audio. The operator closed the
History and Archive panels. Each measurement contains 300 frames. After each
configuration change, the operator discarded the first ready-only measurement,
then kept the next three before the next display resize. Raw logs keep every measurement.

| Surface | Window | FPS | Highest p95 |
| --- | --- | --- | --- |
| 3D production | 1366x768 | 59.79, 59.80, 59.79 | 17.955 ms |
| Compact 2D | 1366x768 | 59.79, 59.78, 59.77 | 18.029 ms |
| 3D production | 1920x1080 | 59.96, 59.92, 59.95 | 17.796 ms |
| Compact 2D | 1920x1080 | 59.94, 59.91, 59.96 | 17.933 ms |

Rendering also remained responsive while the expanded campaign processed ticks:

| Window | Complete 300-frame intervals spent advancing | FPS range | Highest p95 |
| --- | --- | --- | --- |
| 1366x768 | 7 | 59.78–59.80 | 17.822 ms |
| 1920x1080 | 8 | 59.93–59.96 | 18.330 ms |

These results concern the bounded five-process scene with the expanded observed
sector context. They do not prove performance for a larger physical network.

## Presentation findings

Exact readings remain difficult to discover: the button sits below the visible
part of the short context panel at both resolutions. Expanded details occupy
that same narrow strip and need extensive scrolling. The side log also
repeats arrival, delivery and quantity-realization evidence as separate cards.
These findings need further presentation work. General keyboard access also
remains incomplete, and historical Archive pages still report unavailability.

## Evidence

- Client SHA-256: `8dcb66ff99d5aefa304ffc00de5dd4a4a9fb234f8c77e18ecb0deb5cd21a7a7b`
- Runtime SHA-256: `e78ae56a4562ea5691af8123324b9f207f98719ba10d05b1cf725b41c58ae9ea`
- Existing campaign: `db91fd38-87b6-43c3-87f1-b001c4682ced`
- New campaign: `359bee47-dce5-4c8b-939b-0e137acb1f98`
- Build: `/tmp/babylon-g4-native-cohorts-build-20260905.log`
- Existing-save session: `/tmp/babylon-g4-native-cohorts-v1-20260905.log`
- New campaign: `/tmp/babylon-g4-native-cohorts-v2-20260905.log`
- Reopen: `/tmp/babylon-g4-native-cohorts-v2-reopen-20260905.log`
- Structured frame receipt: `/tmp/babylon-g4-native-cohorts-receipt-20260905.json`
- Baseline screenshot: `/tmp/babylon-g4-native-v2-observed-baseline-1920-20260905.png`
- Historical chart: `/tmp/babylon-g4-native-v2-history-1920-20260905.png`
- Preview clearing: `/tmp/babylon-g4-native-v1-preview-20260905.png`

The three session logs contain no error-level entries, panics or failed session
states. Rendering continued after window-resize swap-chain warnings.
This check does not independently prove audible sound quality.
