import { useEffect } from "react";

/**
 * Global keydown listener that ignores keys typed into form fields.
 * Re-attaches on dep change so closures see fresh state.
 */
export function useKeyboard(
  handler: (e: KeyboardEvent) => void,
  deps: unknown[],
) {
  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      const t = e.target as HTMLElement | null;
      if (
        t &&
        (t.tagName === "INPUT" ||
          t.tagName === "TEXTAREA" ||
          t.isContentEditable)
      ) {
        return;
      }
      handler(e);
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, deps);
}
