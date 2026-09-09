/**
 * MapControls preview — composes `MapLensBar` + `MapLegend` +
 * `FramingSelector` into the map's floating control cluster (spec-113 Lane
 * B, architecture §3.3). Pure props, no store — `DeckGLMap`/`MapStage`
 * wire it to `mapSlice`'s lens/framing/selection state in the real shell.
 *
 * Both inner clusters are `position:absolute` (`left-[250px] top-14` /
 * `right-[300px] top-14`), so — like TakeoverOverlay.tsx's fixed overlays —
 * this needs a positioned, definitely-sized ancestor; a plain `relative`
 * div with an explicit width/height (rather than the transform trick) is
 * enough here since nothing inside uses `position:fixed`.
 */
import { MapControls } from "babylon-cockpit";

const FACTIONS = [
  { id: "FAC_COMPRADOR_BLOC", colonial_stance: "uphold" },
  { id: "FAC_NEW_AFRIKAN_UNITY", colonial_stance: "abolish" },
  { id: "FAC_LIBERAL_TECHNOCRAT", colonial_stance: "ignore" },
];

// Inline style: position:relative + explicit width/height give the two
// position:absolute clusters (left-[250px]/right-[300px], both top-14) a
// definite box to anchor against — .design-sync/previews/ is outside
// Tailwind's content-scan root, so arbitrary classes like w-[Npx] never
// compile here (see MapLegend.tsx/DeckGLMap.tsx).
function Frame({ children }: { children?: unknown }) {
  return (
    <div className="bg-void" style={{ position: "relative", width: 1000, height: 220 }}>
      {children as never}
    </div>
  );
}

export function ImperialRentDefaultCountyFraming() {
  return (
    <Frame>
      <MapControls
        lens={{ kind: "metric", metric: "imperial_rent" }}
        onLensChange={() => {}}
        framing="county"
        onFramingChange={() => {}}
        currentValue={0.62}
        legendStatusText="Imperial Rent — county aggregate"
      />
    </Frame>
  );
}

export function FactionLensWithPickerHexFraming() {
  return (
    <Frame>
      <MapControls
        lens={{ kind: "faction" }}
        onLensChange={() => {}}
        factionFilter="FAC_NEW_AFRIKAN_UNITY"
        onFactionFilterChange={() => {}}
        factions={FACTIONS}
        framing="hex"
        onFramingChange={() => {}}
        legendStatusText="Faction Filter — county aggregate — no data"
      />
    </Frame>
  );
}

export function SolidarityRampFlashingRescale() {
  return (
    <Frame>
      <MapControls
        lens={{ kind: "metric", metric: "solidarity_index" }}
        onLensChange={() => {}}
        framing="state"
        onFramingChange={() => {}}
        currentValue={0.72}
        flash={true}
      />
    </Frame>
  );
}
