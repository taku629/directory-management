# licenser — 参考実装のライセンス発行サーバ

> ⚠️ これはリファレンス実装です。本番運用するなら認証・レート制限・監査ログ・
> バックアップを足してください。Lemon Squeezy / Paddle のライセンス機能を使えば
> 自前運用しなくて済みます (推奨)。

## 何をするか

1. 決済プロバイダ (Lemon Squeezy / Stripe) からの購入 webhook を受ける
2. Ed25519 で署名したライセンストークンを生成してメール送付
3. アプリからの再検証 (`/verify`) リクエストに応答
4. 失効・払い戻しを管理

## ライセンストークンのフォーマット

```
SIFT-PRO-9F2K7Q4R.<base64url(payload_json)>.<base64url(signature)>
```

`payload_json`:

```json
{
  "tier": "pro",
  "machine_id": "ABCD...",
  "email": "user@example.com",
  "issued_at": 1735689600,
  "expires_at": null
}
```

クライアントは `tauri::license::verify` で `expires_at` チェック + 機械 ID 一致を検証。

## 最小実装 (Rust + axum)

```rust
// licenser/src/main.rs (擬似コード)
use axum::{Json, Router, routing::post};
use ed25519_dalek::{SigningKey, Signer};

async fn issue(Json(req): Json<IssueRequest>) -> Json<IssueResponse> {
    let key = SigningKey::from_bytes(&load_private_key());
    let payload = serde_json::json!({
        "tier": req.tier,
        "machine_id": req.machine_id,
        "email": req.email,
        "issued_at": now_unix(),
        "expires_at": req.expires_at,
    });
    let payload_bytes = serde_json::to_vec(&payload).unwrap();
    let sig = key.sign(&payload_bytes);
    let token = format!(
        "SIFT-{}-{}.{}.{}",
        req.tier.to_uppercase(),
        random_id(8),
        b64url(&payload_bytes),
        b64url(&sig.to_bytes()),
    );
    save_to_db(&req.email, &token);
    send_email(&req.email, &token);
    Json(IssueResponse { token })
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/webhook/lemonsqueezy", post(handle_purchase))
        .route("/issue", post(issue))
        .route("/verify", post(verify))
        .route("/revoke", post(revoke));
    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

## DB スキーマ

```sql
CREATE TABLE licenses (
  id            TEXT PRIMARY KEY,            -- SIFT-PRO-9F2K7Q4R
  tier          TEXT NOT NULL,
  email         TEXT NOT NULL,
  payment_id    TEXT NOT NULL,                -- LS / Stripe order id
  issued_at     INTEGER NOT NULL,
  expires_at    INTEGER,
  revoked       INTEGER NOT NULL DEFAULT 0,
  machine_ids   TEXT,                          -- JSON array of activated machines
  created_at    INTEGER NOT NULL
);
CREATE INDEX idx_licenses_email ON licenses(email);
```

`machine_ids` を見て同時アクティベート数を制限 (例: Pro は 3 デバイスまで)。

## デプロイ案

| 候補 | コスト | メモ |
|------|-------|------|
| **Lemon Squeezy ライセンス機能** | 売上 5% | コード不要、決済+ライセンス両方やってくれる。**最初これでいい** |
| Fly.io + Postgres | 月 $0〜 | 自前運用、無料枠で足りる |
| Cloudflare Workers + D1 | 月 $0 | エッジで動くので速い、設計しやすい |
| Vercel + Neon | 月 $0 | Next.js API Route で書ける |

個人開発なら **Lemon Squeezy のライセンス機能から始めるのが圧倒的に楽**。
売上規模が出てきて手数料が辛くなったら自前に移行する流れ。

## Lemon Squeezy 連携の最小手順

1. [LS で商品作成](https://lemonsqueezy.com) → "License keys" を有効化
2. 購入完了 webhook を受ける (Vercel API Route 1 個でいい)
3. webhook で LS API を叩いて生成された license key を取得
4. それを **Sift 用に Ed25519 で再署名** して `SIFT-PRO-...` フォーマットでメール送信

LS が生成するキーは LS でしか検証できないので、自分で再署名フォーマットを
被せると後で LS 以外に乗り換えても手元の検証ロジックは変わらない。

## やることリスト

- [ ] 鍵を本番用に生成 (`openssl genpkey -algorithm ED25519`)
- [ ] 公開鍵を `tauri-action` の Secret に登録 (`SIFT_PUBLIC_KEY_PEM`)
- [ ] 秘密鍵をライセンスサーバに環境変数で渡す
- [ ] LS or Stripe アカウント作る
- [ ] webhook 受信エンドポイント作る
- [ ] メール送信 (Resend / Postmark) 連携
- [ ] 失効処理 (`/revoke`) と払い戻し連動
