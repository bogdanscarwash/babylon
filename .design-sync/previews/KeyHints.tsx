/**
 * KeyHints preview — the keyboard-hint footer row (DESIGN_BIBLE.md §9b).
 * Pure props, no store: `hints` defaults to the full global set
 * (`DEFAULT_KEY_HINTS`); hosts pass a scoped subset when only some
 * shortcuts apply in that dialog (e.g. a modal that only supports Esc).
 */
import { KeyHints } from "babylon-cockpit";

function Frame({ children }: { children?: unknown }) {
  return (
    <div style={{ width: 420 }} className="bg-plate p-1">
      {children as never}
    </div>
  );
}

export function DefaultGlobalHints() {
  return (
    <Frame>
      <KeyHints />
    </Frame>
  );
}

export function ScopedToModal() {
  return (
    <Frame>
      <KeyHints hints={[{ keys: "Esc", label: "close" }]} />
    </Frame>
  );
}

export function ScopedToActionDock() {
  return (
    <Frame>
      <KeyHints
        hints={[
          { keys: "1/2/3", label: "speed" },
          { keys: "Esc", label: "close composer" },
        ]}
      />
    </Frame>
  );
}
