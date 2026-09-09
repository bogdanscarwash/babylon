/**
 * BottomDrawer preview — "Trends" drawer hosting `TimeseriesChart`
 * verbatim (architecture §1.2's `BottomStrip` disperse row). Same
 * `panels.timeseries` seeding + no-op `fetch` override as
 * TimeseriesChart.tsx (its mount effect always fires against the capture
 * harness's static file server).
 *
 * KNOWN RENDER RISK (not fixable from this file — `FloatingPanel`'s
 * internals are frozen/out of this lane's ownership): `anchor="bottom"`
 * is `position:absolute inset-x-0 bottom-0` with NO `top` — its box height
 * is content/shrink-to-fit, never stretched to fill an ancestor. The
 * `min-h-0 flex-1` → `h-full` → recharts `ResponsiveContainer
 * height="100%"` chain inside therefore has no definite pixel height
 * anywhere in the tree (percentages against a non-definite ancestor height
 * resolve to `auto`), so the chart is likely to render at ~0 height in
 * this capture regardless of the ancestor Frame given here — the same
 * root cause TimeseriesChart.tsx's own docstring flags for `h-full`
 * chains, just with no fixed-height ancestor available to fix it this
 * time since `AppShell.tsx` mounts `<BottomDrawer>` with no wrapping
 * height constraint either (verified against the real shell). The header
 * strip ("Trends" title tab) still renders correctly regardless.
 *
 * Card shows the primary story only (needs cfg.overrides.BottomDrawer =
 * {cardMode:"single", primaryStory:"TrendsOpen"}).
 */
import { BottomDrawer, useStore } from "babylon-cockpit";

const TIMESERIES = {
  ticks: [100, 101, 102, 103, 104],
  imperial_rent: [78000000, 80100000, 81600000, 83000000, 84213907.42],
  consciousness: [0.33, 0.35, 0.37, 0.39, 0.42],
  solidarity: [0.5, 0.58, 0.62, 0.68, 0.72],
  heat: [0.4, 0.42, 0.45, 0.5, 0.55],
  wealth: [150, 152, 155, 158, 160],
  biocapacity: [0.3, 0.3, 0.29, 0.29, 0.28],
};

function seedBottomDrawer(bottomDrawer: "none" | "trends" | "events") {
  useStore.setState((s: any) => ({
    ui: { ...s.ui, chrome: { ...s.ui.chrome, bottomDrawer } },
    panels: {
      ...s.panels,
      timeseries: { ...s.panels.timeseries, data: TIMESERIES, loading: false, error: null, fetch: async () => {} },
    },
  }));
}

// Same `transform` + `h-screen` containing-block trick TakeoverOverlay.tsx
// uses for its `position:fixed` content — `anchor="bottom"` is
// `position:absolute`, which needs the same kind of positioned ancestor.
function Frame({ children }: { children?: unknown }) {
  return (
    <div className="h-screen w-full bg-void" style={{ transform: "translateZ(0)" }}>
      {children as never}
    </div>
  );
}

export function TrendsOpen() {
  seedBottomDrawer("trends");
  return (
    <Frame>
      <BottomDrawer gameId="wayne-county-001" />
    </Frame>
  );
}

export function EventsPointerShown() {
  seedBottomDrawer("events");
  return (
    <Frame>
      <BottomDrawer gameId="wayne-county-001" />
    </Frame>
  );
}

export function Closed() {
  seedBottomDrawer("none");
  return (
    <Frame>
      <BottomDrawer gameId="wayne-county-001" />
    </Frame>
  );
}
