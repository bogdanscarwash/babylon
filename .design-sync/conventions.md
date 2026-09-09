# Babylon Cockpit — Cold Collapse conventions

Dark-first Paradox-style simulation cockpit. **Every design sits on the void**:
compose on `bg-void` (the base stylesheet already paints `html` void + `text-bone`);
panels sit on `bg-concrete`, elevated surfaces on `bg-rebar`, borders `border-rebar`
(subtle) or `border-wet-steel` (default). "Light emissions in a concrete bunker."

## Setup — no provider; one global store

Components need **no wrapper**. Interactive/stateful components (EventsFeed,
TimeControls, TopBar, Outliner, takeovers…) read one global zustand store.
Seed it **before** composing them, always spreading the existing slice (its
actions live there):

```tsx
import { EventsFeed, useStore } from "babylon-cockpit";

useStore.setState((s) => ({
  world: { ...s.world, snapshot: { tick: 104, events: [/* GameEvent[] */] } },
}));

export function Feed() {
  return (
    <div className="w-[440px] bg-void p-2">
      <EventsFeed />
    </div>
  );
}
```

Slices: `world` (snapshot), `time` (status: paused|playing|resolving|autopaused|error),
`map` (selection/lens), `ui` (docks, takeovers), `panels` (per-panel data),
`session`, `actions`. Purely prop-driven leaves (StatChip, Sparkline, BblLabel,
ValueRow…) take plain props — see each component's `.d.ts`.

## Styling idiom — Tailwind utilities bound to Cold Collapse tokens

Never invent colors; the palette is closed and each accent **encodes a verb**:

| Utility family        | Names                                                                | Meaning                                                    |
| --------------------- | -------------------------------------------------------------------- | ---------------------------------------------------------- |
| Substrate             | `bg-void` `bg-tar` `bg-concrete` `bg-rebar` `bg-wet-steel` `bg-rust` | page → panel → elevated → borders/hover                    |
| Emissions (text)      | `text-bone` `text-fog` `text-ash` `text-shroud`                      | primary → secondary → muted → disabled/empty               |
| `spire` / `spire-dim` | `text-spire` `bg-spire` `border-spire`                               | PRIMARY cyan — infrastructure online, numeric data, active |
| `laser`               | `text-laser` `border-laser`                                          | THREAT — critical, errors, alerts                          |
| `thermal`             | `text-thermal`                                                       | CRITICAL system stress (deeper red)                        |
| `rupture`             | `text-rupture`                                                       | REVOLUTION bronze-gold — use *rarely*                      |
| `solidarity`          | `text-solidarity`                                                    | success, mass-line green                                   |
| `heat`                | `text-heat`                                                          | warning, surveillance pressure orange                      |
| `rent`                | `text-rent`                                                          | imperial-rent extraction purple                            |
| `cadre`               | `text-cadre`                                                         | info blue                                                  |
| `population`          | `text-population`                                                    | demographic violet                                         |

All of `bg-`/`text-`/`border-` work for every color above.

**Type**: `font-mono` (JetBrains Mono — labels, data, code), `font-sans`
(Space Grotesk — body, headings), `font-display` (Redaction 35 — dramatic
titles), `font-pixel` (Departure Mono — terminal flavor). Sizes use the
**pixel idiom** — arbitrary values from this fixed set only: `text-[8px]`
`text-[9px]` `text-[10px]` `text-[11px]` `text-[12px]` `text-[13px]`
`text-[14px]` `text-[16px]` `text-[18px]` `text-[20px]` `text-[24px]`
`text-[32px]` `text-[48px]` (do NOT use `text-xs`/`text-sm` etc. — those are
not this DS's scale and are deliberately not shipped). Tracking:
`tracking-widest` (0.4em), `tracking-[0.08em]` (labels), `tracking-[0.25em]`
(titles). The house type roles:

- label: `font-mono text-[10px] uppercase tracking-widest text-ash`
- numeric data: `font-mono text-[11px] font-semibold text-spire`

For bespoke values use the shipped CSS vars (`var(--text-base)`,
`var(--space-4)`, `var(--radius-md)`…) in a `style` attribute rather than
inventing new arbitrary classes — only the vocabulary above is guaranteed to
exist in the shipped stylesheet.

**CRT chrome** (frames, headers, empty states ONLY — never inside charts,
maps, sparklines or any data-encoding surface): `crt-scanlines`,
`crt-vignette`, `bloom-spire`, `bloom-laser`, `bloom-rupture`, `bloom-solid`,
`bbl-flicker`, `bbl-cursor`.

## Hard rules

- **Honest empties**: when there is no data, render the designed empty state
  (`italic text-shroud`, e.g. "No events this tick.") — never fabricate values.

- Severity mapping is fixed: critical=`laser`, warning/important=`heat`,
  informational/success=`solidarity`.

- Buttons are bordered chips; active state swaps to `border-spire text-spire`:

  ```text
  rounded border border-wet-steel px-2.5 py-1 font-mono text-[10px] uppercase tracking-widest text-fog hover:border-spire
  ```

## Where the truth lives

- `styles.css` — every token (`--babylon-*`, semantic `--text-*`/`--label-*`
  roles) and every generated utility. Read it before styling.
- `components/<group>/<Name>/<Name>.prompt.md` — per-component usage;
  `<Name>.d.ts` — the props contract.
- Fonts are self-hosted in `fonts/` — never link Google Fonts.
