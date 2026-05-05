// Reference implementation — DO NOT use in production as-is.
//
// Run:
//   SIFT_PRIVATE_KEY_PEM="$(cat license-priv.pem)" cargo run -p licenser
//
// Endpoints:
//   POST /issue   { "tier": "pro", "email": "...", "machine_id": "...", "expires_at": null }
//   POST /verify  { "token": "SIFT-PRO-..." }
//
// Persistence is just a JSON file on disk so the example stays self-contained.

use std::sync::Arc;

use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use base64::engine::{general_purpose::URL_SAFE_NO_PAD, Engine};
use ed25519_dalek::{pkcs8::DecodePrivateKey, Signer, SigningKey};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
struct AppState {
    signing: Arc<SigningKey>,
    store: Arc<Mutex<Vec<License>>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct License {
    id: String,
    tier: String,
    email: String,
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
}

#[derive(Debug, Serialize)]
struct VerifyResponse {
    valid: bool,
    reason: Option<String>,
}

#[tokio::main]
async fn main() {
    let pem = std::env::var("SIFT_PRIVATE_KEY_PEM")
        .expect("SIFT_PRIVATE_KEY_PEM env var required");
    let signing = SigningKey::from_pkcs8_pem(&pem).expect("bad private key");
    let state = AppState {
        signing: Arc::new(signing),
        store: Arc::new(Mutex::new(load_or_init())),
    };

    let app = Router::new()
        .route("/issue", post(issue))
        .route("/verify", post(verify))
        .with_state(state);

    let addr: std::net::SocketAddr = "0.0.0.0:8080".parse().unwrap();
    println!("listening on {addr}");
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}

async fn issue(
    State(state): State<AppState>,
    Json(req): Json<IssueRequest>,
) -> Result<Json<IssueResponse>, StatusCode> {
    let id = format!("SIFT-{}-{}", req.tier.to_uppercase(), random_id(8));
    let now = now_ts();
    let payload = serde_json::json!({
        "tier": req.tier,
        "machine_id": req.machine_id.clone().unwrap_or_default(),
        "email": req.email,
        "issued_at": now,
        "expires_at": req.expires_at,
    });
    let payload_bytes = serde_json::to_vec(&payload).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let sig = state.signing.sign(&payload_bytes);
    let token = format!(
        "{}.{}.{}",
        id,
        URL_SAFE_NO_PAD.encode(&payload_bytes),
        URL_SAFE_NO_PAD.encode(sig.to_bytes()),
    );

    let lic = License {
        id: id.clone(),
        tier: req.tier,
        email: req.email,
        machine_ids: req.machine_id.into_iter().collect(),
        issued_at: now,
        expires_at: req.expires_at,
        revoked: false,
    };
    {
        let mut store = state.store.lock();
        store.push(lic);
        save(&store);
    }
    Ok(Json(IssueResponse { token }))
}

async fn verify(
    State(state): State<AppState>,
    Json(req): Json<VerifyRequest>,
) -> Json<VerifyResponse> {
    // Quick lookup by id prefix.
    let id = req.token.split('.').next().unwrap_or("").to_string();
    let store = state.store.lock();
    let Some(lic) = store.iter().find(|l| l.id == id) else {
        return Json(VerifyResponse {
            valid: false,
            reason: Some("unknown".into()),
        });
    };
    if lic.revoked {
        return Json(VerifyResponse {
            valid: false,
            reason: Some("revoked".into()),
        });
    }
    if let Some(exp) = lic.expires_at {
        if exp < now_ts() {
            return Json(VerifyResponse {
                valid: false,
                reason: Some("expired".into()),
            });
        }
    }
    Json(VerifyResponse {
        valid: true,
        reason: None,
    })
}

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
