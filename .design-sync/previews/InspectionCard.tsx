/**
 * InspectionCard preview — one resolved InspectionFrame (architecture.md
 * §2.1): title, kind badge, loading/error/no-data states (Constitution
 * III.11), pin toggle, and the "act" read/write symmetry link for
 * `org`/`hex` subjects. Reads one store action (`ui.toggleComposer`) but
 * takes no seeded data from the store — `frame`/`canDrill`/`onDrill`/
 * `onTogglePin` are plain props, matching ValueRow.tsx/FormulaCard.tsx.
 */
import { InspectionCard } from "babylon-cockpit";

const ORG_NODE = {
  ref: { kind: "org" as const, id: "org-uaw-local-600" },
  title: "UAW Local 600",
  sections: [
    {
      label: "Organization",
      rows: [
        { label: "Cohesion", value: 0.68, format: "decimal2" as const },
        { label: "Heat", value: 0.35, format: "decimal2" as const },
        {
          label: "Budget",
          value: 82.0,
          format: "decimal2" as const,
          ref: { kind: "metric" as const, id: "org_budget", scope: "org:org-uaw-local-600" },
        },
      ],
    },
  ],
};

function Frame({ children }: { children?: unknown }) {
  return (
    <div style={{ width: 340 }} className="bg-void p-2">
      {children as never}
    </div>
  );
}

export function OrgPopulatedActionable() {
  const frame = {
    ref: ORG_NODE.ref,
    data: ORG_NODE,
    loading: false,
    error: null,
    pinned: false,
    fetchedAtTick: 104,
  };
  return (
    <Frame>
      <InspectionCard frame={frame} canDrill={true} onDrill={() => {}} onTogglePin={() => {}} />
    </Frame>
  );
}

export function HexPinnedAtDepthLimit() {
  const frame = {
    ref: { kind: "hex" as const, id: "territory-detroit-downtown" },
    data: {
      ref: { kind: "hex" as const, id: "territory-detroit-downtown" },
      title: "Detroit Downtown",
      sections: [
        {
          label: "Territory",
          rows: [
            { label: "Heat", value: 0.71, format: "decimal2" as const },
            { label: "Habitability", value: 0.24, format: "decimal2" as const },
          ],
        },
      ],
    },
    loading: false,
    error: null,
    pinned: true,
    fetchedAtTick: 104,
  };
  return (
    <Frame>
      <InspectionCard frame={frame} canDrill={false} onDrill={() => {}} onTogglePin={() => {}} />
    </Frame>
  );
}

export function NonActionableNodeLoading() {
  const frame = {
    ref: { kind: "node" as const, id: "node-reserve-army-48210" },
    data: null,
    loading: true,
    error: null,
    pinned: false,
    fetchedAtTick: null,
  };
  return (
    <Frame>
      <InspectionCard frame={frame} canDrill={true} onDrill={() => {}} onTogglePin={() => {}} />
    </Frame>
  );
}

export function LoudFailure() {
  const frame = {
    ref: { kind: "edge" as const, id: "edge-solidarity-104" },
    data: null,
    loading: false,
    error: "Failed to load: HTTP 500",
    pinned: false,
    fetchedAtTick: 104,
  };
  return (
    <Frame>
      <InspectionCard frame={frame} canDrill={true} onDrill={() => {}} onTogglePin={() => {}} />
    </Frame>
  );
}
