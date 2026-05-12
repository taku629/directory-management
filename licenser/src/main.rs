// Reference license server for Sift — DO NOT run in production as-is.
//
// It signs license tokens with Ed25519 (the matching public key is bundled in
// the app via SIFT_PUBLIC_KEY_PEM), receives Lemon Squeezy purchase webhooks,
// and answers re-verification requests. Persistence is a flat JSON file so the
// example stays self-contained — swap it for Postgres/D1 for real use.
//
// Env vars:
//   SIFT_PRIVATE_KEY_PEM   (required)  Ed25519 private key, PKCS#8 PEM
//   LS_WEBHOOK_SECRET      (optional)  Lemon Squeezy "Signing secret"; if set,
//                                      /webhook/lemonsqueezy verifies X-Signature
//   RESEND_API_KEY         (optional)  if set, license emails are sent via Resend
//   LICENSE_FROM_EMAIL     (optional)  From: address for emails (default below)
//   DEVICE_LIMIT           (optional)  max activated machines per license (default 3)
//   BIND_ADDR              (optional)  default 0.0.0.0:8080
//
// Endpoints:
//   POST /issue                  { "tier", "email", "machine_id"?, "expires_at"? } -> { "token" }
//   POST /verify                 { "token" }                       -> { "valid", "reason"? }
//   POST /revoke                 { "id" }                          -> { "ok" }
//   POST /webhook/lemonsqueezy   (Lemon Squeezy event payload)     -> 200

use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::post,
    Json, Router,
};
use base64::engine::{general_purpose::URL_SAFE_NO_PAD, Engine};
use ed25519_dalek::{pkcs8::DecodePrivateKey, Signer, SigningKey};
use hmac::{Hmac, Mac};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
struct AppState {
    signing: Arc<SigningKey>,
    store: Arc<Mutex<Vec<License>>>,
    cfg: Arc<Config>,
    http: reqwest::Client,
}

struct Config {
    ls_webhook_secret: Option<String>,
    resend_api_key: Option<String>,
    from_email: String,
    device_limit: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct License {
    id: String,
    tier: String,
    email: String,
    payment_id: Option<String>,
    machine_ids: Vec<String>,
    issued_at: i64,
    expires_at: Option<i64>,
    revoked: bool,
}

#[derive(Debug, Deserialize)]
struct IssueRequest {
    tier: String,
    email: String,
    machine_id: Option<String>,
    expires_at: Option<i64>,
}

#[derive(Debug, Serialize)]
struct IssueResponse {
    token: String,
}

#[derive(Debug, Deserialize)]
struct VerifyRequest {
    token: String,
    machine_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct VerifyResponse {
    valid: bool,
    reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RevokeRequest {
    id: String,
}

#[derive(Debug, Serialize)]
struct OkResponse {
    ok: bool,
}

#[tokio::main]
async fn main() {
    let pem = std::env::var("SIFT_PRIVATE_KEY_PEM").expect("SIFT_PRIVATE_KEY_PEM env var required");
    let signing = SigningKey::from_pkcs8_pem(&pem).expect("bad Ed25519 private key (PKCS#8 PEM)");

    let cfg = Config {
        ls_webhook_secret: std::env::var("LS_WEBHOOK_SECRET").ok().filter(|s| !s.is_empty()),
        resend_api_key: std::env::var("RESEND_API_KEY").ok().filter(|s| !s.is_empty()),
        from_email: std::env::var("LICENSE_FROM_EMAIL")
            .unwrap_or_else(|_| "Sift <licenses@sift.app>".to_string()),
        device_limit: std::env::var("DEVICE_LIMIT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3),
    };
    if cfg.ls_webhook_secret.is_none() {
        eprintln!("warning: LS_WEBHOOK_SECRET not set — /webhook/lemonsqueezy will reject everything");
    }
    if cfg.resend_api_key.is_none() {
        eprintln!("note: RESEND_API_KEY not set — license emails will be logged, not sent");
    }

    let state = AppState {
        signing: Arc::new(signing),
        store: Arc::new(Mutex::new(load_or_init())),
        cfg: Arc::new(cfg),
        http: reqwest::Client::new(),
    };

    let app = Router::new()
        .route("/issue", post(issue))
        .route("/verify", post(verify))
        .route("/revoke", post(revoke))
        .route("/webhook/lemonsqueezy", post(ls_webhook))
        .with_state(state);

    let addr: std::net::SocketAddr = std::env::var("BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
        .parse()
        .expect("bad BIND_ADDR");
    println!("listening on {addr}");
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}

// --- token issuance ---------------------------------------------------------

/// Build and persist a license, return the signed token string.
fn issue_license(
    state: &AppState,
    tier: &str,
    email: &str,
    machine_id: Option<String>,
    expires_at: Option<i64>,
    payment_id: Option<String>,
) -> Result<String, serde_json::Error> {
    let id = format!("SIFT-{}-{}", tier.to_uppercase(), random_id(8));
    let now = now_ts();
    let payload = serde_json::json!({
        "tier": tier,
        "machine_id": machine_id.clone().unwrap_or_default(),
        "email": email,
        "issued_at": now,
        "expires_at": expires_at,
    });
    let payload_bytes = serde_json::to_vec(&payload)?;
    let sig = state.signing.sign(&payload_bytes);
    let token = format!(
        "{}.{}.{}",
        id,
        URL_SAFE_NO_PAD.encode(&payload_bytes),
        URL_SAFE_NO_PAD.encode(sig.to_bytes()),
    );

    let lic = License {
        id,
        tier: tier.to_string(),
        email: email.to_string(),
        payment_id,
        machine_ids: machine_id.into_iter().collect(),
        issued_at: now,
        expires_at,
        revoked: false,
    };
    let mut store = state.store.lock();
    store.push(lic);
    save(&store);
    Ok(token)
}

async fn issue(
    State(state): State<AppState>,
    Json(req): Json<IssueRequest>,
) -> Result<Json<IssueResponse>, StatusCode> {
    let token = issue_license(&state, &req.tier, &req.email, req.machine_id, req.expires_at, None)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(IssueResponse { token }))
}

async fn verify(
    State(state): State<AppState>,
    Json(req): Json<VerifyRequest>,
) -> Json<VerifyResponse> {
    let id = req.token.split('.').next().unwrap_or("").to_string();
    let bad = |reason: &str| {
        Json(VerifyResponse {
            valid: false,
            reason: Some(reason.to_string()),
        })
    };

    let mut store = state.store.lock();
    let Some(lic) = store.iter_mut().find(|l| l.id == id) else {
        return bad("unknown");
    };
    if lic.revoked {
        return bad("revoked");
    }
    if let Some(exp) = lic.expires_at {
        if exp < now_ts() {
            return bad("expired");
        }
    }
    // Track device activations and enforce the per-license limit.
    if let Some(mid) = req.machine_id.filter(|m| !m.is_empty()) {
        if !lic.machine_ids.contains(&mid) {
            if lic.machine_ids.len() >= state.cfg.device_limit {
                return bad("device_limit");
            }
            lic.machine_ids.push(mid);
            save(&store);
        }
    }
    Json(VerifyResponse { valid: true, reason: None })
}

async fn revoke(
    State(state): State<AppState>,
    Json(req): Json<RevokeRequest>,
) -> Json<OkResponse> {
    let mut store = state.store.lock();
    let ok = match store.iter_mut().find(|l| l.id == req.id) {
        Some(lic) => {
            lic.revoked = true;
            save(&store);
            true
        }
        None => false,
    };
    Json(OkResponse { ok })
}

// --- Lemon Squeezy webhook --------------------------------------------------

async fn ls_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> StatusCode {
    // 1. Verify the HMAC-SHA256 signature (hex) in the X-Signature header.
    let Some(secret) = state.cfg.ls_webhook_secret.as_ref() else {
        return StatusCode::SERVICE_UNAVAILABLE;
    };
    let Some(sig_hex) = headers.get("x-signature").and_then(|v| v.to_str().ok()) else {
        return StatusCode::UNAUTHORIZED;
    };
    let Ok(sig_bytes) = hex::decode(sig_hex.trim()) else {
        return StatusCode::UNAUTHORIZED;
    };
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("hmac key");
    mac.update(&body);
    if mac.verify_slice(&sig_bytes).is_err() {
        return StatusCode::UNAUTHORIZED;
    }

    // 2. Parse the event.
    let Ok(event): Result<serde_json::Value, _> = serde_json::from_slice(&body) else {
        return StatusCode::BAD_REQUEST;
    };
    let event_name = event
        .get("meta")
        .and_then(|m| m.get("event_name"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let attrs = event
        .get("data")
        .and_then(|d| d.get("attributes"))
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    let order_id = event
        .get("data")
        .and_then(|d| d.get("id"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    match event_name {
        // New purchase / subscription start -> issue a Pro license.
        "order_created" | "subscription_created" => {
            let email = attrs
                .get("user_email")
                .or_else(|| attrs.get("customer_email"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if email.is_empty() {
                return StatusCode::BAD_REQUEST;
            }
            // Subscriptions expire ~32 days out and get renewed by the next event;
            // one-off orders never expire here (the buy is perpetual for that major).
            let expires_at = if event_name == "subscription_created" {
                Some(now_ts() + 32 * 24 * 3600)
            } else {
                None
            };
            match issue_license(&state, "pro", &email, None, expires_at, order_id) {
                Ok(token) => {
                    deliver_license(&state, &email, &token).await;
                    StatusCode::OK
                }
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
            }
        }
        // Refund / expiry / cancellation -> revoke any license tied to this order.
        "order_refunded" | "subscription_expired" | "subscription_cancelled" => {
            if let Some(oid) = order_id {
                let mut store = state.store.lock();
                let mut changed = false;
                for lic in store.iter_mut().filter(|l| l.payment_id.as_deref() == Some(oid.as_str())) {
                    lic.revoked = true;
                    changed = true;
                }
                if changed {
                    save(&store);
                }
            }
            StatusCode::OK
        }
        // Everything else: acknowledge so Lemon Squeezy stops retrying.
        _ => StatusCode::OK,
    }
}

// --- email ------------------------------------------------------------------

async fn deliver_license(state: &AppState, to: &str, token: &str) {
    let html = format!(
        "<p>Thanks for buying Sift Pro!</p>\
         <p>Your license key:</p>\
         <pre style=\"font-size:14px;background:#f4f4f5;padding:12px;border-radius:6px\">{token}</pre>\
         <p>Open Sift → Settings → paste the key to unlock Pro features.</p>"
    );
    let Some(api_key) = state.cfg.resend_api_key.as_ref() else {
        println!("[email:stub] to={to}\n  key={token}");
        return;
    };
    let res = state
        .http
        .post("https://api.resend.com/emails")
        .bearer_auth(api_key)
        .json(&serde_json::json!({
            "from": state.cfg.from_email,
            "to": [to],
            "subject": "Your Sift Pro license key",
            "html": html,
        }))
        .send()
        .await;
    match res {
        Ok(r) if r.status().is_success() => println!("[email] sent license to {to}"),
        Ok(r) => eprintln!("[email] Resend returned {} for {to}", r.status()),
        Err(e) => eprintln!("[email] failed to send to {to}: {e}"),
    }
}

// --- misc -------------------------------------------------------------------

fn random_id(n: usize) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let mut s = format!("{seed:x}");
    s.truncate(n);
    s.to_uppercase()
}

fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

fn load_or_init() -> Vec<License> {
    std::fs::read_to_string("licenses.json")
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save(licenses: &[License]) {
    if let Ok(s) = serde_json::to_string_pretty(licenses) {
        let _ = std::fs::write("licenses.json", s);
    }
}
