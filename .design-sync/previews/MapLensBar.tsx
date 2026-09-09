/**
 * MapLensBar preview — replaces `MapModeSelector` (spec-113 Lane B,
 * architecture §3.3, DESIGN_BIBLE.md §3.3). Renders grouped, registry-driven
 * lens buttons (Extraction/Struggle/Political/Reproduction) instead of the
 * old flat 5-button row. Controlled component, no store — mirrors
 * MapModeSelector.tsx's pattern exactly (this is its direct successor,
 * per the component's own docstring).
 */
import { MapLensBar } from "babylon-cockpit";

const FACTIONS = [
  { id: "FAC_COMPRADOR_BLOC", colonial_stance: "uphold" },
  { id: "FAC_NEW_AFRIKAN_UNITY", colonial_stance: "abolish" },
  { id: "FAC_LIBERAL_TECHNOCRAT", colonial_stance: "ignore" },
];

// Inline style for width: .design-sync/previews/ isn't in Tailwind's
// content-scan root, so w-[Npx] never compiles (see MapModeSelector.tsx).
function Frame({ children }: { children?: unknown }) {
  return (
    <div className="flex flex-wrap items-center bg-void p-3" style={{ width: 780 }}>
      {children as never}
    </div>
  );
}

export function ImperialRentDefault() {
  return (
    <Frame>
      <MapLensBar lens={{ kind: "metric", metric: "imperial_rent" }} onLensChange={() => {}} />
    </Frame>
  );
}

export function SolidarityActive() {
  return (
    <Frame>
      <MapLensBar lens={{ kind: "metric", metric: "solidarity_index" }} onLensChange={() => {}} />
    </Frame>
  );
}

export function FactionActiveWithPicker() {
  return (
    <Frame>
      <MapLensBar
        lens={{ kind: "faction" }}
        onLensChange={() => {}}
        factionFilter="FAC_NEW_AFRIKAN_UNITY"
        onFactionFilterChange={() => {}}
        factions={FACTIONS}
      />
    </Frame>
  );
}
