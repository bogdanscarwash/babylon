/**
 * FramingSelector preview — toolbar for switching admin framing levels
 * (spec-112 C5, spec-113 Lane B, DESIGN_BIBLE.md §9.2/§2.2 Carto
 * addendum: county/state are now the PRIMARY framings, hex is demoted to a
 * deep-zoom entry). Controlled component (props: `framing`,
 * `onFramingChange`), no store — mirrors MapModeSelector.tsx's pattern,
 * the component this one visually pairs with in `MapControls`.
 */
import { FramingSelector } from "babylon-cockpit";

// Inline style for width: .design-sync/previews/ isn't in Tailwind's
// content-scan root, so w-[Npx] never compiles (see MapModeSelector.tsx).
function Frame({ children }: { children?: unknown }) {
  return (
    <div className="flex items-center bg-void p-3" style={{ width: 460 }}>
      {children as never}
    </div>
  );
}

export function CountyActive() {
  return (
    <Frame>
      <FramingSelector framing="county" onFramingChange={() => {}} />
    </Frame>
  );
}

export function StateActive() {
  return (
    <Frame>
      <FramingSelector framing="state" onFramingChange={() => {}} />
    </Frame>
  );
}

export function HexActiveDeepZoom() {
  return (
    <Frame>
      <FramingSelector framing="hex" onFramingChange={() => {}} />
    </Frame>
  );
}
