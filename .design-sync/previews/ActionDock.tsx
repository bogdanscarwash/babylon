/**
 * ActionDock preview — bottom-center dock (architecture §1.2's `RightDock`
 * disperse row, Actions tab). Two pieces: the always-visible compact bar of
 * the first three engine-wired verbs (`SUPPORTED_VERBS`), and the
 * `ActionComposer` `FloatingPanel` every bar button opens
 * (`ui.chrome.composerOpen`). Same `world.snapshot.organizations` +
 * `actions.*` seeding as RightDock.tsx's `ActionsPopulated` cell —
 * `ActionComposer`'s acting-org selector reads only player-controlled orgs.
 *
 * Card shows the primary story only (needs cfg.overrides.ActionDock =
 * {cardMode:"single", primaryStory:"ComposerOpen"}) — the singleton store
 * makes multi-cell cards lie.
 */
import { ActionDock, useStore } from "babylon-cockpit";

const PLAYER_ORG = {
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
};

function seedActionDock(composerOpen: boolean, actionsPatch: Record<string, unknown> = {}) {
  useStore.setState((s: any) => ({
    world: { ...s.world, snapshot: { tick: 104, organizations: [PLAYER_ORG], events: [] } },
    ui: { ...s.ui, chrome: { ...s.ui.chrome, composerOpen } },
    actions: {
      ...s.actions,
      pending: [],
      submitting: false,
      error: null,
      ...actionsPatch,
    },
  }));
}

function Frame({ children }: { children?: unknown }) {
  return (
    <div className="flex items-end justify-center bg-void p-4" style={{ width: 640, height: 340 }}>
      {children as never}
    </div>
  );
}

export function ComposerOpen() {
  seedActionDock(true, {
    pending: [
      {
        id: "pending-1",
        verb: "educate",
        orgId: "org-uaw-local-600",
        targetId: null,
        submittedAtTick: 104,
      },
    ],
  });
  return (
    <Frame>
      <ActionDock gameId="wayne-county-001" />
    </Frame>
  );
}

export function BarOnlyCollapsed() {
  seedActionDock(false);
  return (
    <Frame>
      <ActionDock gameId="wayne-county-001" />
    </Frame>
  );
}

export function SubmittingLoudFailure() {
  seedActionDock(true, { submitting: true, error: "Action rejected: insufficient Cadre Labor" });
  return (
    <Frame>
      <ActionDock gameId="wayne-county-001" />
    </Frame>
  );
}
