/**
 * EventToasts preview — the toast queue (architecture §4.2, DESIGN_BIBLE
 * §5.2). Renders `events.toasts` (urgent-stream only). `critical` events
 * are individually persistent (one toast each); `notable` events from the
 * same tick batch into one expandable ephemeral toast.
 *
 * `position:absolute right-3 top-14` needs the same transformed,
 * definitely-sized ancestor TakeoverOverlay.tsx's preview documents.
 *
 * Card shows the primary story only (needs cfg.overrides.EventToasts =
 * {cardMode:"single", primaryStory:"Populated"}).
 */
import { EventToasts, useStore } from "babylon-cockpit";

const CRITICAL_TOAST = {
  id: "ev-rupture-26163",
  tick: 104,
  severity: "critical" as const,
  lifetime: "persistent" as const,
  events: [
    {
      id: "ev-rupture-26163",
      event: {
        id: "ev-rupture-26163",
        type: "rupture",
        tick: 104,
        title: "Rupture in Wayne County",
        body: "P(S|R) exceeded P(S|A) for the industrial proletariat.",
        data: { territory_id: "26163" },
      },
      tick: 104,
      severity: "critical" as const,
      category: "struggle" as const,
      stream: "urgent" as const,
      linkedEntityId: "26163",
      linkedEntityType: "territory" as const,
    },
  ],
};

const NOTABLE_BATCH_TOAST = {
  id: "batch-104",
  tick: 104,
  severity: "notable" as const,
  lifetime: "ephemeral" as const,
  events: [
    {
      id: "104-1",
      event: {
        id: "104-1",
        type: "solidarity_awakening",
        tick: 104,
        title: "Solidarity Awakening",
        body: "New SOLIDARITY edge: auto workers ↔ tenants union.",
        data: { org_id: "org-uaw-local-600" },
      },
      tick: 104,
      severity: "notable" as const,
      category: "solidarity" as const,
      stream: "urgent" as const,
      linkedEntityId: "org-uaw-local-600",
      linkedEntityType: "organization" as const,
    },
    {
      id: "104-2",
      event: {
        id: "104-2",
        type: "bifurcation_threshold",
        tick: 104,
        title: "Bifurcation Threshold Crossed",
        body: "Wage collapse routes agitation toward organization, not fascism.",
        data: { territory_id: "26163" },
      },
      tick: 104,
      severity: "notable" as const,
      category: "system" as const,
      stream: "urgent" as const,
      linkedEntityId: "26163",
      linkedEntityType: "territory" as const,
    },
  ],
};

function Frame({ children }: { children?: unknown }) {
  return (
    <div className="h-screen w-full bg-void" style={{ transform: "translateZ(0)" }}>
      {children as never}
    </div>
  );
}

export function Populated() {
  useStore.setState((s: any) => ({
    events: { ...s.events, toasts: [CRITICAL_TOAST, NOTABLE_BATCH_TOAST] },
  }));
  return (
    <Frame>
      <EventToasts gameId="wayne-county-001" />
    </Frame>
  );
}

export function SingleCriticalOnly() {
  useStore.setState((s: any) => ({ events: { ...s.events, toasts: [CRITICAL_TOAST] } }));
  return (
    <Frame>
      <EventToasts gameId="wayne-county-001" />
    </Frame>
  );
}

/**
 * Honest-empty: `toasts: []` renders the wrapper `<div>` with no toast
 * cards inside — this annotation is this preview file's own text,
 * documenting the blank space is the correct designed render.
 */
export function EmptyQueue() {
  useStore.setState((s: any) => ({ events: { ...s.events, toasts: [] } }));
  return (
    <Frame>
      <span className="absolute left-2 top-2 text-[10px] italic text-shroud">
        (EventToasts renders an empty wrapper when events.toasts is [])
      </span>
      <EventToasts gameId="wayne-county-001" />
    </Frame>
  );
}
