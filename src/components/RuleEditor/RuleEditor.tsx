import { LockedPanel } from "../Locked";

export function RuleEditor() {
  return (
    <LockedPanel
      title="Rules"
      phase="Phase 2"
      blurb="Auto-organize folders by extension, name patterns, size, age, or AI labels. Rules run on a watcher so new files get sorted the moment they land."
      previewItems={[
        "Move PDFs in Downloads to Documents/{year}/{month}",
        "Tag any image >5MB with #raw",
        'Notify me when ".pkg" appears outside Trash',
        "Run a shell script on every new screenshot",
      ]}
    />
  );
}
