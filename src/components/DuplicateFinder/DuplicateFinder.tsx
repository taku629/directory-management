import { LockedPanel } from "../Locked";

export function DuplicateFinder() {
  return (
    <LockedPanel
      title="Duplicate Finder"
      phase="Phase 2"
      blurb="Find exact duplicates by SHA-256 and visually similar images by perceptual hash. Group results, preview, and bulk-move to Trash."
      previewItems={[
        "Exact match (SHA-256)",
        "Similar image (perceptual hash, Hamming distance ≤ 5)",
        "Similar document (MinHash) — Phase 3",
      ]}
    />
  );
}
