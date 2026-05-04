import { useState } from "react";
import { useAppStore } from "../../lib/store";

export function SearchBar({ onStatus }: { onStatus: (s: string) => void }) {
  const setView = useAppStore((s) => s.setView);
  const [text, setText] = useState("");

  return (
    <input
      type="search"
      placeholder="Search indexed files…"
      style={{ width: 320 }}
      value={text}
      onChange={(e) => setText(e.target.value)}
      onKeyDown={(e) => {
        if (e.key === "Enter") {
          setView({ kind: "search", text });
          onStatus(`searching: "${text}"`);
        }
      }}
    />
  );
}
