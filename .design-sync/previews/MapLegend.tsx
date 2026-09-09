/**
 * MapLegend v2 preview (spec-113 Lane B) — the ramp/categorical swatch strip
 * driven by a `LensLegend` spec (`@/lib/lenses/registry`), NOT the pre-113
 * `lens` prop. Props: `legend` (ramp | categorical | none), `label`,
 * `currentValue` (ramp-mode marker, normalized [0,1]), `flash` (one-render
 * domain-rescale highlight). Pure props, no store.
 */
import { MapLegend } from "babylon-cockpit";

// Inline width: .design-sync/previews/ isn't in Tailwind's content-scan root,
// so w-[380px] never compiles (see learnings) — style the frame directly.
function Frame({ children }: { children?: unknown }) {
  return (
    <div className="flex items-center bg-void p-3" style={{ width: 380 }}>
      {children as never}
    </div>
  );
}

// Cold Collapse ramp: near-black → crimson → gold (the imperial-rent/heat idiom).
const HEAT_STOPS = ["#12100e", "#5a1a17", "#a83224", "#d98a2b", "#e8c05a"];

export function HeatRampWithMarker() {
  return (
    <Frame>
      <MapLegend legend={{ kind: "ramp", stops: HEAT_STOPS }} label="Heat" currentValue={0.62} />
    </Frame>
  );
}

export function RampNoMarkerHonestNull() {
  // currentValue omitted → no tick drawn (III.11: no marker beats a fabricated one).
  return (
    <Frame>
      <MapLegend legend={{ kind: "ramp", stops: HEAT_STOPS }} label="Imperial Rent" />
    </Frame>
  );
}

export function RampDomainRescaleFlash() {
  // flash → one-render highlight after the domain memo reports a would-be rescale.
  return (
    <Frame>
      <MapLegend
        legend={{ kind: "ramp", stops: HEAT_STOPS }}
        label="Exploitation Rate"
        currentValue={0.88}
        flash
      />
    </Frame>
  );
}

export function StanceCategorical() {
  return (
    <Frame>
      <MapLegend
        legend={{
          kind: "categorical",
          entries: [
            { label: "Revolutionary", color: [200, 50, 40, 220] },
            { label: "Reactionary", color: [90, 100, 120, 220] },
            { label: "Liberal", color: [216, 160, 60, 220] },
            { label: "Contested", color: [255, 180, 50, 220] },
          ],
        }}
        label="Political Stance"
      />
    </Frame>
  );
}

/**
 * Honest-empty: `legend.kind === "none"` renders null by design (the ramp-less
 * lenses lean on DeckGLMap's own legend-label text chip instead). The caption
 * is this preview file's annotation, outside the component, documenting that
 * the blank below is the correct designed render, not a capture failure.
 */
export function NoneRendersNull() {
  return (
    <Frame>
      <span className="text-[10px] italic text-shroud">
        (MapLegend renders null for legend.kind="none")
      </span>
      <MapLegend legend={{ kind: "none" }} label="Faction" />
    </Frame>
  );
}
