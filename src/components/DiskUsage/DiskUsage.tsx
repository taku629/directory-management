import { LockedPanel } from "../Locked";

export function DiskUsage() {
  return (
    <LockedPanel
      title="Disk Usage"
      phase="Phase 2"
      blurb="Treemap visualisation of where your bytes go. Click in to drill down, sort by size or age, and bulk-archive cold data."
      previewItems={[
        "Treemap (DaisyDisk-style)",
        "Top 100 largest files",
        "Cold data (untouched for >1 year)",
      ]}
    />
  );
}
