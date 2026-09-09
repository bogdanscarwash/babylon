/**
 * FloatingPanel preview — the one primitive every chrome overlay instances
 * (architecture §0/§1.3, DESIGN_BIBLE.md §9b's Installer/newt dialog
 * anatomy). Pure props, no store: `anchor`/`collapsed`/`onToggle`/`width`/
 * `testId`/`children` is the whole frozen contract, so every cell just
 * varies those.
 *
 * `top`/`bottom`/`left`/`right` anchors are `position:absolute` against
 * the nearest positioned ancestor — same "transformed, definitely-sized
 * ancestor" requirement TakeoverOverlay's preview documents. `anchor="free"`
 * is plain `position:relative` and needs no such wrapper.
 */
import { FloatingPanel } from "babylon-cockpit";

// Same trick as TakeoverOverlay.tsx: `transform` establishes a containing
// block for the panel's `position:absolute` anchors, and `h-screen`/`w-full`
// (real Tailwind utilities, not arbitrary values, so they compile here) give
// it a definite size to anchor against.
function PositionedFrame({ children }: { children?: unknown }) {
  return (
    <div className="h-screen w-full bg-void" style={{ transform: "translateZ(0)" }}>
      {children as never}
    </div>
  );
}

export function FreeAnchorWithHeader() {
  return (
    <div style={{ width: 320 }} className="bg-void p-4">
      <FloatingPanel
        anchor="free"
        title="Territory Brief"
        collapsed={false}
        onToggle={() => {}}
        testId="floating-panel-preview-free"
      >
        <div className="flex flex-col gap-1 p-2 text-[11px]">
          <span className="text-ash">Detroit Downtown — FIPS 26163</span>
          <span className="text-bone">Heat 0.71 · Habitability 0.24</span>
        </div>
      </FloatingPanel>
    </div>
  );
}

export function TopAnchorNoHeader() {
  return (
    <PositionedFrame>
      <FloatingPanel anchor="top" testId="floating-panel-preview-top">
        <div className="flex items-center gap-4 px-4 py-2 text-[11px] text-bone">
          <span>BABYLON COCKPIT</span>
          <span className="text-ash">Tick 104</span>
        </div>
      </FloatingPanel>
    </PositionedFrame>
  );
}

export function LeftAnchorCollapsedRail() {
  return (
    <PositionedFrame>
      <FloatingPanel
        anchor="left"
        title="☰"
        collapsed={true}
        onToggle={() => {}}
        width={44}
        testId="floating-panel-preview-left"
      >
        <div className="p-2 text-[10px] text-ash">Outliner</div>
      </FloatingPanel>
    </PositionedFrame>
  );
}

export function BottomAnchorExpanded() {
  return (
    <PositionedFrame>
      <FloatingPanel
        anchor="bottom"
        title="Trends"
        collapsed={false}
        onToggle={() => {}}
        testId="floating-panel-preview-bottom"
      >
        <div className="p-3 text-[11px] text-bone">
          Imperial Rent Φ climbing tick over tick — 78.0M → 84.2M since tick 100.
        </div>
      </FloatingPanel>
    </PositionedFrame>
  );
}
