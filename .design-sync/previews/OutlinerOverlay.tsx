/**
 * OutlinerOverlay preview — chrome stub hosting the unchanged `Outliner`
 * (architecture §1.2 migrate row). Collapses to a narrow icon rail rather
 * than a same-width empty strip. Same seeding as Outliner.tsx's `Populated`
 * cell (organizations / communities panel / map panel's balkanization
 * block), plus `ui.chrome.outlinerOpen`.
 *
 * `anchor="left"` is `position:absolute left-0 top-14 bottom-0` — needs the
 * same transformed, definitely-sized ancestor TakeoverOverlay.tsx's
 * preview documents.
 *
 * Card shows the primary story only (needs cfg.overrides.OutlinerOverlay =
 * {cardMode:"single", primaryStory:"Populated"}).
 */
import { OutlinerOverlay, useStore } from "babylon-cockpit";

const ORGANIZATIONS = [
  {
    id: "org-uaw-local-600",
    name: "UAW Local 600",
    short_name: "UAW Local 600",
    player_controlled: true,
    org_type: "civil_society_org",
    class_character: "proletarian",
    cohesion: 0.68,
    cadre_level: 0.42,
    budget: 82.0,
    heat: 0.35,
    territory_ids: ["territory-detroit-mi"],
    hyperedge_memberships: ["hx-new-afrikan"],
    consciousness: { liberal: 0.12, fascist: 0.03, revolutionary: 0.85 },
    ooda: { observe: 0.6, orient: 0.55, decide: 0.7, act: 0.75, cycle_ticks: 1 },
  },
  {
    id: "org-detroit-pd",
    name: "Detroit Police Department",
    short_name: "Detroit PD",
    player_controlled: false,
    org_type: "state_apparatus",
    class_character: "repressive_apparatus",
    cohesion: 0.8,
    cadre_level: 0.1,
    budget: 340.0,
    heat: 0.6,
    territory_ids: ["territory-detroit-mi"],
    hyperedge_memberships: [],
    consciousness: null,
    ooda: { observe: 0.7, orient: 0.5, decide: 0.6, act: 0.65, cycle_ticks: 1 },
  },
];

const COMMUNITIES = [
  {
    id: "comm-uaw-solidarity",
    member_ids: ["org-uaw-local-600"],
    member_count: 340,
    dominant_role: "proletariat",
    avg_consciousness: 0.62,
    total_solidarity_strength: 4.8,
  },
];

const FACTIONS = [
  { id: "FAC_DECOLONIAL", colonial_stance: "abolish", is_settler_formation: false },
  { id: "FAC_LOYALIST", colonial_stance: "uphold", is_settler_formation: true },
];

function noopPanel(data: unknown) {
  return { data, loading: false, error: null, fetch: async () => {} };
}

function mapPanelWithFactions() {
  return noopPanel({
    type: "FeatureCollection",
    features: [],
    metadata: { balkanization: { factions: FACTIONS, sovereigns: [], territory_influence: [] } },
  });
}

// Same `transform` + `h-screen` containing-block trick TakeoverOverlay.tsx
// uses — `anchor="left"` is `position:absolute`.
function Frame({ children }: { children?: unknown }) {
  return (
    <div className="h-screen w-full bg-void" style={{ transform: "translateZ(0)" }}>
      {children as never}
    </div>
  );
}

export function Populated() {
  useStore.setState((s: any) => ({
    world: { ...s.world, snapshot: { tick: 104, organizations: ORGANIZATIONS, events: [] } },
    panels: {
      ...s.panels,
      communities: { ...s.panels.communities, ...noopPanel({ communities: COMMUNITIES }) },
      map: { ...s.panels.map, ...mapPanelWithFactions() },
    },
    map: { ...s.map, selection: { kind: "org", id: "org-uaw-local-600" }, factionFilter: null },
    ui: { ...s.ui, chrome: { ...s.ui.chrome, outlinerOpen: true } },
  }));
  return (
    <Frame>
      <OutlinerOverlay gameId="wayne-county-001" />
    </Frame>
  );
}

export function CollapsedRail() {
  useStore.setState((s: any) => ({
    world: { ...s.world, snapshot: { tick: 104, organizations: ORGANIZATIONS, events: [] } },
    panels: {
      ...s.panels,
      communities: { ...s.panels.communities, ...noopPanel({ communities: COMMUNITIES }) },
      map: { ...s.panels.map, ...mapPanelWithFactions() },
    },
    map: { ...s.map, selection: null, factionFilter: null },
    ui: { ...s.ui, chrome: { ...s.ui.chrome, outlinerOpen: false } },
  }));
  return (
    <Frame>
      <OutlinerOverlay gameId="wayne-county-001" />
    </Frame>
  );
}
