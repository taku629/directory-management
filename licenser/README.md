# licenser — reference license server for Sift

> ⚠️ Reference implementation. For production, add auth, rate limiting, an audit
> log, backups, and a real database. If you'd rather not run anything, Lemon
> Squeezy's built-in license-key feature covers most of this — see "Lemon
> Squeezy" below.

## What it does

1. Receives Lemon Squeezy purchase webhooks (`POST /webhook/lemonsqueezy`),
   verifies the HMAC-SHA256 signature, and on `order_created` /
   `subscription_created` issues an Ed25519-signed license token.
2. Emails the token to the buyer (via Resend if configured, otherwise logs it).
3. Answers re-verification requests from the app (`POST /verify`), tracking
   device activations and enforcing a per-license device limit.
4. Revokes licenses on refund/cancel webhooks, or manually via `POST /revoke`.

The Ed25519 public key that matches this server's private key must be bundled
into the app at build time as the `SIFT_PUBLIC_KEY_PEM` secret (see
`../docs/distribution.md`); the app verifies tokens fully offline and only calls
`/verify` for revocation checks.

## Token format

```
SIFT-PRO-9F2K7Q4R.<base64url(payload_json)>.<base64url(ed25519_signature)>
```

`payload_json`:

```json
{ "tier": "pro", "machine_id": "", "email": "user@example.com", "issued_at": 1735689600, "expires_at": null }
```

One-off purchases get `expires_at: null` (perpetual for that major version);
subscriptions get a ~32-day expiry that the next renewal webhook refreshes.

## Run it

```bash
# generate the keypair once (see ../docs/distribution.md)
openssl genpkey -algorithm ED25519 -out license-priv.pem
openssl pkey -in license-priv.pem -pubout -out license-pub.pem   # -> SIFT_PUBLIC_KEY_PEM

SIFT_PRIVATE_KEY_PEM="$(cat license-priv.pem)" \
LS_WEBHOOK_SECRET="whsec_..." \
RESEND_API_KEY="re_..." \
LICENSE_FROM_EMAIL="Sift <licenses@yourdomain.com>" \
cargo run -p licenser
```

| Env var | Required | Default | Notes |
|---|---|---|---|
| `SIFT_PRIVATE_KEY_PEM` | yes | — | Ed25519 private key, PKCS#8 PEM |
| `LS_WEBHOOK_SECRET` | for webhooks | — | Lemon Squeezy "Signing secret"; without it the webhook rejects everything |
| `RESEND_API_KEY` | no | — | if unset, license keys are printed to stdout instead of emailed |
| `LICENSE_FROM_EMAIL` | no | `Sift <licenses@sift.app>` | From: header |
| `DEVICE_LIMIT` | no | `3` | max activated machines per license |
| `BIND_ADDR` | no | `0.0.0.0:8080` | listen address |

State is persisted to `licenses.json` in the working directory.

## Endpoints

| Method | Path | Body | Response |
|---|---|---|---|
| POST | `/issue` | `{ "tier", "email", "machine_id"?, "expires_at"? }` | `{ "token" }` |
| POST | `/verify` | `{ "token", "machine_id"? }` | `{ "valid", "reason"? }` — `reason` ∈ `unknown` `revoked` `expired` `device_limit` |
| POST | `/revoke` | `{ "id" }` | `{ "ok" }` |
| POST | `/webhook/lemonsqueezy` | Lemon Squeezy event payload | `200` (or `401` on bad signature) |

## DB schema (when you outgrow the JSON file)

```sql
CREATE TABLE licenses (
  id            TEXT PRIMARY KEY,   -- SIFT-PRO-9F2K7Q4R
  tier          TEXT NOT NULL,
  email         TEXT NOT NULL,
  payment_id    TEXT,               -- Lemon Squeezy order/subscription id
  issued_at     INTEGER NOT NULL,
  expires_at    INTEGER,
  revoked       INTEGER NOT NULL DEFAULT 0,
  machine_ids   TEXT NOT NULL DEFAULT '[]'  -- JSON array of activated machines
);
CREATE INDEX idx_licenses_email      ON licenses(email);
CREATE INDEX idx_licenses_payment_id ON licenses(payment_id);
```

## Lemon Squeezy

1. Create the Pro / Team products; under **Settings → Webhooks** add a webhook
   pointing at `https://your-host/webhook/lemonsqueezy`, subscribed to
   `order_created`, `order_refunded`, `subscription_created`,
   `subscription_cancelled`, `subscription_expired`. Copy the signing secret
   into `LS_WEBHOOK_SECRET`.
2. Deploy this server somewhere with a stable URL (Fly.io / a small VM /
   Cloudflare Workers if you port it). It needs outbound HTTPS for Resend.
3. Test with a real purchase in test mode → confirm the email arrives → paste
   the key into Sift → Settings.

Alternatively, skip this server entirely: enable Lemon Squeezy's own license
keys, and have a tiny webhook (one Vercel/Workers route) fetch the LS-generated
key and re-sign it in the `SIFT-...` format so the app's offline verification
stays the same even if you later move off Lemon Squeezy.

## Deploy options

| Option | Cost | Notes |
|---|---|---|
| Lemon Squeezy license keys (+ a tiny re-signing webhook) | 5% of revenue | least to run; start here |
| Fly.io + a volume for `licenses.json` (or Postgres) | $0–few $/mo | this server as-is |
| Cloudflare Workers + D1 | ~$0 | needs a port off axum/tokio |
| Vercel + Neon | ~$0 | rewrite as a Next.js route |

## Checklist

- [ ] Generate the production Ed25519 keypair (`openssl genpkey -algorithm ED25519`)
- [ ] Register the public key as the `SIFT_PUBLIC_KEY_PEM` build secret
- [ ] Give the private key to this server via `SIFT_PRIVATE_KEY_PEM`
- [ ] Create a Lemon Squeezy account + products + webhook; set `LS_WEBHOOK_SECRET`
- [ ] Set up Resend (or another provider) and a verified sending domain
- [ ] Deploy with a stable HTTPS URL and persistent storage for `licenses.json`
- [ ] Test-purchase → email → activate end to end
