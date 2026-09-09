/**
 * ObjectivesTray preview — chrome stub hosting `ObjectivesTracker` verbatim
 * (architecture §1.2's `RightDock` disperse row, Objectives tab). Badge =
 * active objective count, baked into `FloatingPanel`'s `title` string.
 * Same `useObjectives` mount-fetch no-op override as ObjectivesTracker.tsx
 * (its mount effect always fires against the capture harness's static file
 * server).
 *
 * `anchor="free"` — no absolute-position ancestor trick needed.
 *
 * Card shows the primary story only (needs cfg.overrides.ObjectivesTray =
 * {cardMode:"single", primaryStory:"Populated"}).
 */
import { ObjectivesTray, useStore } from "babylon-cockpit";

async function noopFetch(): Promise<void> {}
function noopSetMounted(): void {}

const OBJECTIVES = [
  {
    id: "obj-dual-power",
    title: "Establish Dual Power in Wayne County",
    description:
      "Push P(S|R) past P(S|A) county-wide via SOLIDARITY edge growth among the proletariat.",
    progress: 0.64,
    status: "active",
    category: "revolution",
  },
  {
    id: "obj-imperial-rent-collapse",
    title: "Collapse the Imperial Rent Flow",
    description: "Drive Imperial Rent Φ toward zero by severing TRIBUTE edges out of the core.",
    progress: 0.22,
    status: "active",
    category: "collapse",
  },
  {
    id: "obj-fascist-consolidation-averted",
    title: "Prevent Fascist Consolidation",
    description: "Keep the reserve army's fascist consciousness pole below the bifurcation threshold.",
    progress: 1.0,
    status: "complete",
    category: "fascist",
  },
];

function seedObjectivesTray(objectivesOpen: boolean, patch: Record<string, unknown>) {
  useStore.setState((s: any) => ({
    ui: { ...s.ui, chrome: { ...s.ui.chrome, objectivesOpen } },
    panels: {
      ...s.panels,
      objectives: {
        ...s.panels.objectives,
        data: null,
        loading: false,
        error: null,
        fetch: noopFetch,
        setMounted: noopSetMounted,
        ...patch,
      },
    },
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
  seedObjectivesTray(true, { data: { tick: 104, objectives: OBJECTIVES } });
  return (
    <Frame>
      <ObjectivesTray gameId="wayne-county-001" />
    </Frame>
  );
}

export function CollapsedBadgeStillShows() {
  seedObjectivesTray(false, { data: { tick: 104, objectives: OBJECTIVES } });
  return (
    <Frame>
      <ObjectivesTray gameId="wayne-county-001" />
    </Frame>
  );
}

export function EmptyBeforeLoad() {
  seedObjectivesTray(true, { data: { tick: 104, objectives: [] } });
  return (
    <Frame>
      <ObjectivesTray gameId="wayne-county-001" />
    </Frame>
  );
}
