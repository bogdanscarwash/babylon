<!-- Vale: technical program names, hashes, and census terms are exact. -->
<!-- vale ste.NounClusters = NO -->
<!-- vale ste.UnapprovedWords = NO -->
<!-- vale Vale.Spelling = NO -->

# Currency Value Census — 2026-08-23

This report remeasures the Program 27 section 6.1 value budget after the
hydrated Michigan baseline changed. The original
`currency-magnitude-census-2026-07-29.md` report remains immutable historical
evidence.

## Cause of the stale pin

Commit `7f2cb785` added the original report and test at 11:41 EDT on
2026-07-29. It measured the 36,149,703-byte Michigan LFS object with SHA-256
`3daba23792e1478c385d92125b249b1225a2047a83349928dbcdcab2079ae9a9`.

Commit `6e6f96f5` regenerated that baseline at 15:15 EDT on the same day. The
replacement is the 105,204,141-byte LFS object with SHA-256
`ea498b34913de078ca30b47b6d3f6fc6d4bda03694adb7985c6a396cfdb126eb`.
The test budget did not change with the later baseline. Thus, stale evidence
caused the failure. The T0 theory work did not cause it.

## Method

The unchanged `tools/currency_magnitude_census.py` script scanned:

1. finite numeric leaves in each JSON baseline.
2. numeric cells in each dense CSV golden.
3. numeric columns in the 4,492,881,920-byte reference SQLite build
   product.

This remains the original superset census. It does not narrow the scan to
fields declared as `Currency`, and it does not reinterpret accumulated flow
values as node fields. The run used a fully hydrated checkout and the present
reference database.

## Current result

The command was:

```bash
python tools/currency_magnitude_census.py
```

The script returned these first results:

| Rank | Absolute value | Source |
|---:|---:|---|
| 1 | `5.78329e13` | `michigan-e2e.json.external_node_flows[2].total_phi_inflow` |
| 2 | `1.31305e13` | `michigan-e2e.json.external_node_flows[4].total_phi_inflow` |
| 3 | `8.67365e12` | `michigan-e2e.json.external_node_flows[0].total_phi_inflow` |
| 4 | `2.96949e12` | `michigan-e2e.json.external_node_flows[3].total_phi_inflow` |
| 5 | `1.95317e12` | `michigan-e2e.json.external_node_flows[1].total_phi_inflow` |
| 6 | `1.52437e12` | `michigan-e2e.json.external_node_flows[5].total_phi_inflow` |
| 7 | `1.17954e12` | `michigan-e2e.json.terminal_state.total_k` |

The test read `57832906099703.03` as the exact high value. An upward round to
two significant figures gives the new pin:

```text
MAX_OBSERVED_ABS_VALUE = 5.8e13
```

## Limit check

- Current i64 micro-unit limit: `2**63 / 1e6`, about `9.223372e12` units.
- Pinned current value: `5.8e13`, about 6.29 times that limit.
- Nationwide headroom multiplier: `1,000`.
- Pinned nationwide micro-unit value: `5.8e22`.
- i128 micro-unit limit: `2**127 / 1e6`, about `1.701412e32` units.
- The i128 margin is a factor of about `2.93e9`.

The two prior behavioral claims still hold. The i64 micro-units overflow at
the observed scale. The i128 micro-units keep ample headroom after the
unchanged nationwide multiplier.

<!-- vale Vale.Spelling = YES -->
<!-- vale ste.UnapprovedWords = YES -->
<!-- vale ste.NounClusters = YES -->
