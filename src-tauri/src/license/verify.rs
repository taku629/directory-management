//! License signature verification.
//!
//! The licensing service signs a JSON payload with Ed25519 and concatenates:
//! `<key_id>.<base64url(payload_json)>.<base64url(signature)>`
//!
//! The client bundles the service's Ed25519 *public* key at compile time.
//! `SIFT_PUBLIC_KEY_PEM` is read from the environment at build time so we
//! can keep the actual key out of source control. A development key is
//! shipped as a fallback so devs can run the app without setup.

use base64::engine::{general_purpose::URL_SAFE_NO_PAD, Engine};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};

use crate::error::{AppError, AppResult};
use crate::license::LicensePayload;

/// Embedded at build time via `option_env!("SIFT_PUBLIC_KEY_PEM")`.
/// In dev we fall back to a known dev key so the activation pathway works.
const DEV_PUBLIC_KEY_PEM: &str = "-----BEGIN PUBLIC KEY-----
MCowBQYDK2VwAyEA0000000000000000000000000000000000000000000=
-----END PUBLIC KEY-----";

fn public_key() -> AppResult<VerifyingKey> {
    let pem = option_env!("SIFT_PUBLIC_KEY_PEM").unwrap_or(DEV_PUBLIC_KEY_PEM);
    use ed25519_dalek::pkcs8::DecodePublicKey;
    VerifyingKey::from_public_key_pem(pem)
        .map_err(|e| AppError::Other(anyhow::anyhow!("bad public key: {e}")))
}

/// Verify a signed license string and return the decoded payload.
/// Format: `<key_id>.<base64url(payload)>.<base64url(sig)>`
pub fn verify_license(token: &str) -> AppResult<LicensePayload> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(AppError::Invalid("license: malformed".into()));
    }
    let payload_b64 = parts[1];
    let sig_b64 = parts[2];

    let payload_bytes = URL_SAFE_NO_PAD
        .decode(payload_b64)
        .map_err(|_| AppError::Invalid("license: bad payload b64".into()))?;
    let sig_bytes = URL_SAFE_NO_PAD
        .decode(sig_b64)
        .map_err(|_| AppError::Invalid("license: bad sig b64".into()))?;

    let key = public_key()?;
    let sig: Signature = sig_bytes
        .as_slice()
        .try_into()
        .map_err(|_| AppError::Invalid("license: sig length".into()))?;

    key.verify(&payload_bytes, &sig)
        .map_err(|_| AppError::Invalid("license: signature mismatch".into()))?;

    let payload: LicensePayload = serde_json::from_slice(&payload_bytes)
        .map_err(|e| AppError::Invalid(format!("license: payload json: {e}")))?;
    Ok(payload)
}
