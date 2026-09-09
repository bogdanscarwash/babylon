/**
 * ValueRow preview — one labeled value in an InspectionCard
 * (architecture.md §2.1). Constitution III.11 null-honesty: `value ===
 * null` always renders "no data". Rows carrying a `ref` render the
 * "explain" affordance unless `canDrill` is `false` (dimmed, "Depth limit
 * reached" tooltip). Composition rows delegate to `BreakdownBar`;
 * `row.history` renders a `Sparkline` with realized min/max. Pure props,
 * no store.
 */
import { ValueRow } from "babylon-cockpit";

function Frame({ children }: { children?: unknown }) {
  return (
    <div style={{ width: 300 }} className="flex flex-col gap-2 bg-void p-2">
      {children as never}
    </div>
  );
}

export function ExplainableWithHistory() {
  const row = {
    label: "Core Wages (W_c)",
    value: 0.62,
    format: "decimal2" as const,
    ref: { kind: "metric" as const, id: "core_wages", scope: "county:26163" },
    history: [0.58, 0.59, 0.6, 0.61, 0.62],
  };
  return (
    <Frame>
      <ValueRow row={row} canDrill={true} onDrill={() => {}} />
    </Frame>
  );
}

export function DepthLimitBlocked() {
  const row = {
    label: "Value Produced (V_c)",
    value: 0.41,
    format: "decimal2" as const,
    ref: { kind: "metric" as const, id: "value_produced", scope: "county:26163" },
  };
  return (
    <Frame>
      <ValueRow row={row} canDrill={false} onDrill={() => {}} />
    </Frame>
  );
}

export function HonestNoData() {
  const row = { label: "Solidarity Index", value: null, format: "decimal2" as const };
  return (
    <Frame>
      <ValueRow row={row} canDrill={true} onDrill={() => {}} />
    </Frame>
  );
}

export function PlainNonRef() {
  const row = { label: "Population", value: 639111, format: "integer" as const };
  return (
    <Frame>
      <ValueRow row={row} canDrill={true} onDrill={() => {}} />
    </Frame>
  );
}

export function CompositionBreakdown() {
  const row = {
    label: "Consciousness",
    value: 0.85,
    format: "decimal2" as const,
    composition: [
      { key: "revolutionary", value: 0.85, color: "text-rupture" },
      { key: "liberal", value: 0.12, color: "text-cadre" },
      { key: "fascist", value: 0.03, color: "text-laser" },
    ],
  };
  return (
    <Frame>
      <ValueRow row={row} canDrill={true} onDrill={() => {}} />
    </Frame>
  );
}
