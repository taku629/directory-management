// Build the Tauri auto-updater manifest (latest.json) from the assets
// attached to the GitHub Release named `tag`. Pulls release-asset URLs
// and `.sig` signatures so the updater can verify before installing.
//
// Usage: node scripts/make-update-manifest.mjs v0.1.0

import { writeFile } from "node:fs/promises";

const tag = process.argv[2];
if (!tag) {
  console.error("usage: make-update-manifest.mjs <tag>");
  process.exit(2);
}

const repo = process.env.GITHUB_REPOSITORY ?? "taku629/directory-management";
const baseUrl = `https://github.com/${repo}/releases/download/${tag}`;

// Asset names tauri-action produces, per platform.
// (Full list at https://tauri.app/v2/distribute/updater/)
const platforms = {
  "darwin-aarch64": {
    url: `${baseUrl}/Sift_${tag.replace(/^v/, "")}_aarch64.app.tar.gz`,
    sigUrl: `${baseUrl}/Sift_${tag.replace(/^v/, "")}_aarch64.app.tar.gz.sig`,
  },
  "darwin-x86_64": {
    url: `${baseUrl}/Sift_${tag.replace(/^v/, "")}_x64.app.tar.gz`,
    sigUrl: `${baseUrl}/Sift_${tag.replace(/^v/, "")}_x64.app.tar.gz.sig`,
  },
  "linux-x86_64": {
    url: `${baseUrl}/sift_${tag.replace(/^v/, "")}_amd64.AppImage`,
    sigUrl: `${baseUrl}/sift_${tag.replace(/^v/, "")}_amd64.AppImage.sig`,
  },
  "windows-x86_64": {
    url: `${baseUrl}/Sift_${tag.replace(/^v/, "")}_x64-setup.nsis.zip`,
    sigUrl: `${baseUrl}/Sift_${tag.replace(/^v/, "")}_x64-setup.nsis.zip.sig`,
  },
};

async function fetchSig(url) {
  try {
    const r = await fetch(url);
    if (!r.ok) return "";
    return (await r.text()).trim();
  } catch {
    return "";
  }
}

const manifest = {
  version: tag.replace(/^v/, ""),
  notes: "See release notes on GitHub.",
  pub_date: new Date().toISOString(),
  platforms: {},
};

for (const [p, a] of Object.entries(platforms)) {
  manifest.platforms[p] = {
    url: a.url,
    signature: await fetchSig(a.sigUrl),
  };
}

await writeFile("latest.json", JSON.stringify(manifest, null, 2));
console.log("wrote latest.json for", tag);
