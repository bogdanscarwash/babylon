/**
 * InspectionStack preview — the recursive drill-down surface
 * (architecture.md §1.1/§2.1, DESIGN_BIBLE.md §4). A compact `>`-separated
 * breadcrumb trail above the top frame's full `InspectionCard`. Seeds
 * `inspect.stack` directly (bypassing `push()`'s real fetch — same
 * technique InspectorPanel.tsx used for the retired `panels.inspector`)
 * so every frame in a story is already resolved, no network race.
 *
 * `position:absolute bottom-2 right-[300px] top-14` needs the same
 * transformed, definitely-sized ancestor TakeoverOverlay.tsx's preview
 * documents.
 *
 * Card shows the primary story only (needs cfg.overrides.InspectionStack =
 * {cardMode:"single", primaryStory:"MultiFrameBreadcrumb"}) — the
 * singleton store makes multi-cell cards lie.
 */
import { InspectionStack, useStore } from "babylon-cockpit";

function frame(
  ref: { kind: string; id: string; label?: string },
  title: string,
  sections: unknown[],
  extra: Record<string, unknown> = {},
) {
  return {
    ref,
    data: { ref, title, sections },
    loading: false,
    error: null,
    pinned: false,
    fetchedAtTick: 104,
    ...extra,
  };
}

const ORG_FRAME = frame(
  { kind: "org", id: "org-uaw-local-600" },
  "UAW Local 600",
  [
    {
      label: "Organization",
      rows: [
        {
          label: "Imperial Rent Φ",
          value: 84213907.42,
          format: "decimal2",
          ref: { kind: "metric", id: "imperial_rent", scope: "org:org-uaw-local-600", label: "Imperial Rent Φ" },
        },
        { label: "Cohesion", value: 0.68, format: "decimal2" },
      ],
    },
  ],
);

const METRIC_FRAME = frame(
  { kind: "metric", id: "imperial_rent", scope: "org:org-uaw-local-600", label: "Imperial Rent Φ" },
  "Imperial Rent Φ",
  [
    {
      label: "Inputs",
      rows: [
        {
          label: "Core Wages (W_c)",
          value: 0.62,
          format: "decimal2",
          ref: { kind: "formula", id: "core_wages_formula", label: "Core Wages (W_c)" },
        },
        { label: "Value Produced (V_c)", value: 0.41, format: "decimal2" },
      ],
    },
  ],
);

const FORMULA_FRAME = frame(
  { kind: "formula", id: "core_wages_formula", label: "Core Wages (W_c)" },
  "Core Wages (W_c)",
  [{ label: "Expression", rows: [{ label: "Formula", value: "W_c = wage_bill / hours", format: "raw" }] }],
);

// Same `transform` + `h-screen` containing-block trick TakeoverOverlay.tsx
// uses — InspectionStack's outer region is `position:absolute`.
function Frame({ children }: { children?: unknown }) {
  return (
    <div className="h-screen w-full bg-void" style={{ transform: "translateZ(0)" }}>
      {children as never}
    </div>
  );
}

export function SingleFrame() {
  useStore.setState((s: any) => ({ inspect: { ...s.inspect, stack: [ORG_FRAME] } }));
  return (
    <Frame>
      <InspectionStack gameId="wayne-county-001" />
    </Frame>
  );
}

export function MultiFrameBreadcrumb() {
  useStore.setState((s: any) => ({
    inspect: { ...s.inspect, stack: [ORG_FRAME, METRIC_FRAME, FORMULA_FRAME] },
  }));
  return (
    <Frame>
      <InspectionStack gameId="wayne-county-001" />
    </Frame>
  );
}

export function DepthLimitReached() {
  // MAX_INSPECTION_DEPTH = 6 — six frames drilled all the way down means
  // the top frame's rows render dimmed with "Depth limit reached" instead
  // of an "explain" click target.
  const stack = [
    ORG_FRAME,
    METRIC_FRAME,
    FORMULA_FRAME,
    frame({ kind: "metric", id: "wage_bill" }, "Wage Bill", [
      { rows: [{ label: "Total", value: 41200000, format: "integer" }] },
    ]),
    frame({ kind: "metric", id: "hours_worked" }, "Hours Worked", [
      { rows: [{ label: "Total", value: 66452800, format: "integer" }] },
    ]),
    frame({ kind: "formula", id: "hours_per_year" }, "HOURS_PER_YEAR Constant", [
      { rows: [{ label: "Value", value: 2080, format: "integer" }] },
    ]),
  ];
  useStore.setState((s: any) => ({ inspect: { ...s.inspect, stack } }));
  return (
    <Frame>
      <InspectionStack gameId="wayne-county-001" />
    </Frame>
  );
}

/**
 * Honest-empty: an empty stack renders `null` — the map is never occluded
 * when there's nothing to inspect. This annotation is this preview file's
 * own text, documenting the blank space is the correct designed render.
 */
export function EmptyStackRendersNothing() {
  useStore.setState((s: any) => ({ inspect: { ...s.inspect, stack: [] } }));
  return (
    <Frame>
      <span className="absolute left-2 top-2 text-[10px] italic text-shroud">
        (InspectionStack renders null when inspect.stack is [])
      </span>
      <InspectionStack gameId="wayne-county-001" />
    </Frame>
  );
}
