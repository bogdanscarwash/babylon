/**
 * NarrationBlock preview — the one reusable AI-narration slot (Program 16
 * Lane N). Props-driven, no store: callers pass a `beat` (or `null`) plus
 * the narrator's overall `state`, rendering either the beat or one of
 * three honest degradation states (Constitution III.11: absent narration
 * is labeled, never silently blank). Two voice registers: `"wire"` (terse
 * newspaper declaratives) and `"analysis"` (longer theory register).
 */
import { NarrationBlock } from "babylon-cockpit";

function Frame({ children }: { children?: unknown }) {
  return (
    <div style={{ width: 320 }} className="bg-void p-3">
      {children as never}
    </div>
  );
}

export function WireBeat() {
  const beat = {
    id: "beat-104-wclf-raid",
    tick: 104,
    scope: "event" as const,
    subjectRef: "ev-excessive-force-104",
    headline: "Detroit PD raids the WCLF hall.",
    body: "Fourteen cadre detained. Solidarity edges hold; the local votes to escalate.",
    register: "wire" as const,
  };
  return (
    <Frame>
      <NarrationBlock beat={beat} state="ready" />
    </Frame>
  );
}

export function AnalysisBeat() {
  const beat = {
    id: "beat-104-county-analysis",
    tick: 104,
    scope: "county" as const,
    subjectRef: "26163",
    headline: "Wayne County crosses the bifurcation threshold.",
    body: "With core wages falling below value produced for the third consecutive tick, agitation among the industrial proletariat has begun routing toward organization rather than fascism — the SOLIDARITY edge between UAW Local 600 and the tenants union is the load-bearing fact here, not the raid itself.",
    register: "analysis" as const,
  };
  return (
    <Frame>
      <NarrationBlock beat={beat} state="ready" />
    </Frame>
  );
}

export function OfflineHonestEmpty() {
  return (
    <Frame>
      <NarrationBlock beat={null} state="offline" />
    </Frame>
  );
}

export function PendingHonestEmpty() {
  return (
    <Frame>
      <NarrationBlock beat={null} state="pending" />
    </Frame>
  );
}

export function ReadyButQuiet() {
  return (
    <Frame>
      <NarrationBlock beat={null} state="ready" />
    </Frame>
  );
}
