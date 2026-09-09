/**
 * SpeedControls preview — architecture §1.2's `TimeControls` →
 * `SpeedControls` rewrite row (architecture §4.1, DESIGN_BIBLE §5.1's
 * identity/date/speed cluster). Wraps `TimeControls` verbatim (keeps
 * `time-controls`/`time-status`/`time-resume` testids untouched) and adds
 * the ⏸ ▶1 ▶▶2 ▶▶▶5 speed row + the tick-pulse blip. Same `time` seeding
 * pattern as TimeControls.tsx, plus `world.snapshot.tick` for the pulse dot.
 *
 * Card shows the primary story only (needs cfg.overrides.SpeedControls =
 * {cardMode:"single", primaryStory:"Speed1Active"}) — the singleton store
 * makes multi-cell cards lie; per-story captures grade each speed state
 * truly.
 */
import { SpeedControls, useStore } from "babylon-cockpit";

function seedSpeed(patch: Record<string, unknown>, tick: number | null = 104) {
  useStore.setState((s: any) => ({
    world: { ...s.world, snapshot: tick === null ? null : { tick, events: [] } },
    time: {
      ...s.time,
      status: "paused",
      playIntent: false,
      errorMessage: null,
      autopauseEventIds: [],
      speed: 5,
      ...patch,
    },
  }));
}

function Frame({ children }: { children?: unknown }) {
  return <div className="flex items-center bg-void p-4">{children as never}</div>;
}

export function Speed1Active() {
  seedSpeed({ speed: 1, status: "paused" });
  return (
    <Frame>
      <SpeedControls gameId="wayne-county-001" />
    </Frame>
  );
}

export function Speed5PlayingWithPulse() {
  seedSpeed({ speed: 5, status: "playing", playIntent: true });
  return (
    <Frame>
      <SpeedControls gameId="wayne-county-001" />
    </Frame>
  );
}

export function NoTickYet() {
  seedSpeed({ speed: 2, status: "paused" }, null);
  return (
    <Frame>
      <SpeedControls gameId="wayne-county-001" />
    </Frame>
  );
}
