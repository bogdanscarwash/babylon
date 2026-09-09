/**
 * FormulaCard preview — the one generic section/row renderer backing
 * EVERY resolved InspectionStack frame (architecture.md §2.1, DESIGN_BIBLE
 * §4's "one fixed card layout shell across all depth levels"). Pure props
 * (`node`/`canDrill`/`onDrill`), no store — each cell just varies the
 * `InspectionNode` payload.
 */
import { FormulaCard } from "babylon-cockpit";

const IMPERIAL_RENT_NODE = {
  ref: { kind: "formula" as const, id: "imperial_rent", scope: "county:26163" },
  title: "Imperial Rent Φ — Wayne County",
  sections: [
    {
      label: "Expression",
      rows: [{ label: "Formula", value: "Φ = (W_c − V_c) × population", format: "raw" as const }],
    },
    {
      label: "Inputs",
      rows: [
        {
          label: "Core Wages (W_c)",
          value: 0.62,
          format: "decimal2" as const,
          ref: { kind: "metric" as const, id: "core_wages", scope: "county:26163" },
          history: [0.58, 0.59, 0.6, 0.61, 0.62],
        },
        {
          label: "Value Produced (V_c)",
          value: 0.41,
          format: "decimal2" as const,
          ref: { kind: "metric" as const, id: "value_produced", scope: "county:26163" },
        },
        { label: "Population", value: 639111, format: "integer" as const },
      ],
    },
    {
      label: "Consciousness Composition",
      rows: [
        {
          label: "Revolutionary / Liberal / Fascist",
          value: 0.85,
          format: "decimal2" as const,
          composition: [
            { key: "revolutionary", value: 0.85, color: "text-rupture" },
            { key: "liberal", value: 0.12, color: "text-cadre" },
            { key: "fascist", value: 0.03, color: "text-laser" },
          ],
        },
      ],
    },
    {
      label: "Constants",
      rows: [{ label: "Exchange Rate Constant", value: 1.0, format: "decimal3" as const }],
    },
  ],
};

function Frame({ children }: { children?: unknown }) {
  return (
    <div style={{ width: 320 }} className="bg-void p-2">
      {children as never}
    </div>
  );
}

export function Populated() {
  return (
    <Frame>
      <FormulaCard node={IMPERIAL_RENT_NODE} canDrill={true} onDrill={() => {}} />
    </Frame>
  );
}

export function DepthLimitBlocksExplain() {
  return (
    <Frame>
      <FormulaCard node={IMPERIAL_RENT_NODE} canDrill={false} onDrill={() => {}} />
    </Frame>
  );
}

export function UngroupedSectionAndNoData() {
  const node = {
    ref: { kind: "metric" as const, id: "solidarity_index", scope: "county:26163" },
    title: "Solidarity Index",
    sections: [
      {
        rows: [
          { label: "SOLIDARITY-edge density", value: null, format: "decimal2" as const },
          { label: "Sample size", value: 0, format: "integer" as const },
        ],
      },
    ],
  };
  return (
    <Frame>
      <FormulaCard node={node} canDrill={true} onDrill={() => {}} />
    </Frame>
  );
}
