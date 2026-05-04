# Sift — Monetisation

## Tiers

| Tier | Price (target) | Includes |
|------|---------------|----------|
| **Free** | $0 | Browse, tag, search, smart folders, favourites, single watched root |
| **Pro**  | $49 buyout *or* $5/mo | Everything in Free, plus rules, watcher, dedup, disk usage, undo, AI, OCR, cloud sync, multiple watched roots |
| **Team** | $12 / user / mo | Pro + shared tag dictionaries, multi-workspace, SSO, priority support |

Strategy: free is **genuinely useful** (Eagle / Hazel / DaisyDisk demos are
not — they nag). The upgrade prompt appears *in context* the first time you
hit a Pro feature, so the value is concrete.

### Why one-time buy + subscription

A buy-once option ($49) is essential for the desktop-app crowd that hates
SaaS. It gets the current major version + 1 year of updates; year 2 onward
needs renewal at 50% off ($25/yr) for continued updates.

A subscription ($5/mo, $50/yr) suits people who want continuous updates and
cloud sync (which has real backend cost).

## Why these prices

| Competitor | Price | What you get |
|-----------|-------|--------------|
| Hazel | $42 buyout | Rules only, Mac only |
| DaisyDisk | $10 buyout | Disk usage only, Mac only |
| Eagle | $30 buyout | Asset manager, no automation/AI |
| Czkawka | Free | Dedup CLI, no UX |
| Forklift | $30/yr | File manager, no AI |
| Adobe Bridge | bundled | Heavy, requires Adobe subscription |

Sift bundles all of those + AI for $49. Comfortably under Adobe; on par with
Eagle for the asset-manager crowd; cheaper than Hazel + DaisyDisk combined.

## Distribution

- **Direct**: marketing site (Astro/Next), Stripe checkout, download .dmg/.msi
- **Mac App Store**: later — sandboxing limits some FS features, so direct
  distribution comes first
- **Microsoft Store**: same calculus
- **Setapp**: consider once we have feature parity and traction

## Telemetry

**Opt-in only**, off by default. If on, we send:

- App version, OS, locale
- Anonymised feature usage counts (e.g. "rules:run" +1)
- Crash reports

We do **not** send filenames, paths, content, tags, or any user data. This is
a contract on the website + visible in Settings.

## Marketing positioning

Headline: **"Finally, a folder you can actually find things in."**

Three landing-page sections:
1. **Browse like Eagle, organise like Hazel, search like Spotlight on
   steroids.** GIF showing tagging + search.
2. **AI that doesn't suck.** GIF: drag in 500 photos, watch them get tagged
   and grouped.
3. **Yours forever.** Local-first, your files never leave your machine
   without consent.

## First 90 days post-launch

| Week | Goal |
|------|------|
| 1-2  | ProductHunt + HN launch, free tier only, gather signal |
| 3-4  | Open Pro signups; price = $39 launch discount |
| 5-8  | Iterate based on feedback; ship dedup + disk usage |
| 9-12 | AI tier rollout to existing Pro users; evaluate annual plan uptake |

Success metric for going full-time: **$3k MRR by day 90** (~600 Pro users at
$5/mo, or 60 buyouts at $49). Stretch: **$10k MRR by day 180**.
