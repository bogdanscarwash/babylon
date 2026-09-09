/**
 * TopBar preview — Layer 1 chrome, architecture §1.1/§1.2's `StatusBar` →
 * `TopBar` migrate row (keeps `region-statusbar`/`tick-value` testids —
 * real-loop.spec.ts and friends read them). Real `/summary/` StatChips,
 * alert badges, the three takeover-open buttons, and the embedded
 * `SpeedControls` cluster (itself wrapping `TimeControls`) — so this
 * preview seeds both `panels.summary` and `time` in one place, mirroring
 * StatusBar.tsx's seeding pattern (width is an inline style; `summary.fetch`
 * is overridden to a no-op since `TopBar`'s mount effect always fires
 * `fetchSummary(gameId)` against the capture harness's static file server).
 *
 * Card shows the primary story only (needs cfg.overrides.TopBar =
 * {cardMode:"single", primaryStory:"Populated", viewport:"1220x200"}) —
 * the singleton store makes multi-cell cards lie.
 */
import { TopBar, useStore } from "babylon-cockpit";

function seedTopBar(
  tick: number | null,
  summary: Record<string, unknown> | null,
  timePatch: Record<string, unknown> = {},
) {
  useStore.setState((s: any) => ({
    world: {
      ...s.world,
      snapshot: tick === null ? null : { tick, events: [] },
      lastTick: tick,
    },
    panels: {
      ...s.panels,
      summary: {
        ...s.panels.summary,
        data: summary,
        loading: false,
        error: null,
        fetch: async () => {},
      },
    },
    time: {
      ...s.time,
      status: "paused",
      playIntent: false,
      errorMessage: null,
      autopauseEventIds: [],
      speed: 5,
      ...timePatch,
    },
  }));
}

function Frame({ children }: { children?: unknown }) {
  return (
    <div style={{ width: 1200 }} className="bg-void p-2">
      {children as never}
    </div>
  );
}

export function Populated() {
  seedTopBar(104, {
    tick: 104,
    imperial_rent: 84213907.42,
    avg_consciousness: 0.42,
    population_total: 1793561,
    exploitation_rate: 0.55,
    profit_rate: 0.142,
    org_count: 12,
    class_count: 5,
    event_counts: { critical: 1, warning: 2, informational: 5 },
  });
  return (
    <Frame>
      <TopBar gameId="wayne-county-001" />
    </Frame>
  );
}

export function PlayingAtSpeed2() {
  seedTopBar(
    104,
    {
      tick: 104,
      imperial_rent: 84213907.42,
      avg_consciousness: 0.42,
      population_total: 1793561,
      exploitation_rate: 0.55,
      profit_rate: 0.142,
      org_count: 12,
      class_count: 5,
      event_counts: { critical: 0, warning: 1, informational: 3 },
    },
    { status: "playing", playIntent: true, speed: 2 },
  );
  return (
    <Frame>
      <TopBar gameId="wayne-county-001" />
    </Frame>
  );
}

export function HonestNoData() {
  seedTopBar(null, null);
  return (
    <Frame>
      <TopBar gameId="wayne-county-001" />
    </Frame>
  );
}
