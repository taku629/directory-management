import { LockedPanel } from "../Locked";

export function AIPanel() {
  return (
    <LockedPanel
      title="AI Assistant"
      phase="Phase 3 · Pro"
      blurb="Auto-tag images, summarise documents, search by natural language, and let the assistant propose organization rules from how you actually browse."
      previewItems={[
        '"Show me all screenshots from last month"',
        "Auto-tag photos with subjects (people, places, scenes)",
        "Classify Downloads into Receipts / Statements / Other",
        'OCR all PDFs and search inside their text',
      ]}
    />
  );
}
