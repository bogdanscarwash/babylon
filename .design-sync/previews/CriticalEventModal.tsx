/**
 * CriticalEventModal preview — Paradox-style modal for
 * `time.status === "autopaused"` (architecture §4.2). Lists the critical
 * events that fired the autopause (`time.autopauseEventIds`, resolved
 * against the current tick's events via `classifyEvents`), with "Open
 * Wire" and "Resume" CTAs. `gameId=""` on every cell — same pattern as
 * DialecticTakeover/TakeoverOverlay's previews: this component's props are
 * unused internally (`_props`), so the value doesn't matter, but omitting
 * a real id keeps intent honest.
 *
 * `position:absolute inset-0` needs the same transformed, definitely-sized
 * ancestor TakeoverOverlay.tsx's preview documents.
 *
 * Card shows the primary story only (needs cfg.overrides.CriticalEventModal
 * = {cardMode:"single", primaryStory:"Autopaused"}).
 */
import { CriticalEventModal, useStore } from "babylon-cockpit";

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
    id: "ev-excessive-force-104",
    type: "excessive_force",
    tick: 104,
    severity: "important" as const,
    title: "Excessive Force at the WCLF Hall",
    body: "Detroit PD raided the West Central Labor Federation hall — 14 cadre detained.",
    data: { org_id: "org-uaw-local-600" },
  },
];

function Frame({ children }: { children?: unknown }) {
  return (
    <div className="h-screen w-full bg-void" style={{ transform: "translateZ(0)" }}>
      {children as never}
    </div>
  );
}

export function Autopaused() {
  useStore.setState((s: any) => ({
    world: { ...s.world, snapshot: { tick: 104, events: TICK_EVENTS } },
    time: {
      ...s.time,
      status: "autopaused",
      autopauseEventIds: ["ev-rupture-26163", "ev-excessive-force-104"],
      errorMessage: null,
    },
  }));
  return (
    <Frame>
      <CriticalEventModal gameId="" />
    </Frame>
  );
}

export function FiringEventsNoLongerOnRecord() {
  useStore.setState((s: any) => ({
    world: { ...s.world, snapshot: { tick: 105, events: [] } },
    time: {
      ...s.time,
      status: "autopaused",
      autopauseEventIds: ["ev-rupture-26163"],
      errorMessage: null,
    },
  }));
  return (
    <Frame>
      <CriticalEventModal gameId="" />
    </Frame>
  );
}

/**
 * Honest-empty: the component returns `null` whenever `time.status !==
 * "autopaused"` — this annotation is this preview file's own text (outside
 * the component), documenting that the blank space below is the correct,
 * designed render, matching MapLegend.tsx's `NoRampForBalkanizationLens`
 * pattern.
 */
export function NotAutopausedRendersNothing() {
  useStore.setState((s: any) => ({
    world: { ...s.world, snapshot: { tick: 104, events: TICK_EVENTS } },
    time: { ...s.time, status: "paused", autopauseEventIds: [], errorMessage: null },
  }));
  return (
    <Frame>
      <span className="absolute left-2 top-2 text-[10px] italic text-shroud">
        (CriticalEventModal renders null when time.status !== "autopaused")
      </span>
      <CriticalEventModal gameId="" />
    </Frame>
  );
}
