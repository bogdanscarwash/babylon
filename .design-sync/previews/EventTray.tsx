/**
 * EventTray preview — persistent right rail hosting `EventsFeed` verbatim
 * (architecture §1.2's `BottomStrip` disperse row; §4.2; DESIGN_BIBLE §5.2).
 * Adds badge counts (`summary.event_counts`), per-category mute toggles,
 * the recoverable dismissed-toast tray, and mounts `NarrationBlock` as the
 * canonical always-warm host for the cumulative beat feed (via
 * `useNarration`, which fetches on mount — same no-op `fetch`/`setMounted`
 * override technique as ObjectivesTracker.tsx/TimeseriesChart.tsx).
 *
 * `anchor="free"` — no absolute-position ancestor trick needed, unlike the
 * `top`/`bottom`/`left` anchors.
 *
 * Card shows the primary story only (needs cfg.overrides.EventTray =
 * {cardMode:"single", primaryStory:"Populated"}) — the singleton store
 * makes multi-cell cards lie.
 */
import { EventTray, useStore } from "babylon-cockpit";

const TICK_EVENTS = [
  {
    id: "ev-rupture-26163",
    type: "rupture",
    tick: 104,
    severity: "critical" as const,
    title: "Rupture in Wayne County",
    body: "P(S|R) exceeded P(S|A) for the industrial proletariat.",
    data: { territory_id: "26163" },
  },
  {
    id: "ev-solidarity-104",
    type: "solidarity_awakening",
    tick: 104,
    severity: "warning" as const,
    title: "Solidarity Awakening",
    body: "New SOLIDARITY edge: auto workers ↔ tenants union.",
    data: { org_id: "org-uaw-local-600" },
  },
];

function noopNarration(patch: Record<string, unknown>) {
  return {
    status: "ready",
    beats: [],
    loading: false,
    error: null,
    mounted: true,
    fetch: async () => {},
    setMounted: () => {},
    ...patch,
  };
}

function seedEventTray(
  eventTrayOpen: boolean,
  narrationPatch: Record<string, unknown>,
  extra: Record<string, unknown> = {},
) {
  useStore.setState((s: any) => ({
    world: { ...s.world, snapshot: { tick: 104, events: TICK_EVENTS } },
    ui: { ...s.ui, chrome: { ...s.ui.chrome, eventTrayOpen } },
    panels: {
      ...s.panels,
      summary: {
        ...s.panels.summary,
        data: { ...s.panels.summary.data, event_counts: { critical: 1, warning: 1, informational: 3 } },
      },
      narration: { ...s.panels.narration, ...noopNarration(narrationPatch) },
    },
    events: { ...s.events, mutedCategories: [], tray: [] },
    ...extra,
  }));
}

function Frame({ children }: { children?: unknown }) {
  return (
    <div style={{ width: 300 }} className="bg-void p-2">
      {children as never}
    </div>
  );
}

export function Populated() {
  seedEventTray(true, {
    beats: [
      {
        id: "beat-104-tick",
        tick: 104,
        scope: "tick",
        subjectRef: null,
        headline: "Detroit PD raids the WCLF hall.",
        body: "Fourteen cadre detained. Solidarity edges hold; the local votes to escalate.",
        register: "wire",
      },
    ],
  });
  return (
    <Frame>
      <EventTray gameId="wayne-county-001" />
    </Frame>
  );
}

export function DismissedToastsInTray() {
  seedEventTray(
    true,
    { beats: [] },
    {
      events: {
        mutedCategories: [],
        tray: [
          {
            id: "ev-rupture-26163",
            tick: 104,
            severity: "critical",
            lifetime: "persistent",
            events: [
              {
                id: "ev-rupture-26163",
                event: TICK_EVENTS[0],
                tick: 104,
                severity: "critical",
                category: "struggle",
                stream: "urgent",
                linkedEntityId: "26163",
                linkedEntityType: "territory",
              },
            ],
          },
        ],
      },
    },
  );
  return (
    <Frame>
      <EventTray gameId="wayne-county-001" />
    </Frame>
  );
}

export function NarratorOffline() {
  seedEventTray(true, { status: "offline", beats: [] });
  return (
    <Frame>
      <EventTray gameId="wayne-county-001" />
    </Frame>
  );
}
