# 配布の Runbook

Sift をリリースするまでの手順メモ。一度通せば後はタグ push だけで回る。

## 0. 必要なアカウント

| | コスト |
|--|--|
| GitHub | 0円 (releases 無料、updater のホストもこれでいい) |
| Apple Developer Program | 年 $99 (Mac 公証必須) |
| Windows コードサイニング証明書 (Azure Trusted Signing / SSL.com など) | 月 $10 〜 年 $300 |
| Lemon Squeezy or Stripe | 売上に対する手数料のみ |
| ドメイン (`sift.app` など) | 年 $10〜 |
| Cloudflare Pages or Vercel | 0円 (静的サイト) |

## 1. 鍵を作る (一回だけ)

### Tauri 自動アップデート用 (Tauri 専用フォーマット)

```bash
npx @tauri-apps/cli signer generate -w ~/.tauri/sift-update.key
# プロンプトでパスワードを設定
# 公開鍵が標準出力に出るので tauri.conf.json の plugins.updater.pubkey に貼る
```

GitHub Secrets に登録:
- `TAURI_SIGNING_PRIVATE_KEY` ← 上で生成した秘密鍵の中身
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` ← パスワード

### ライセンス署名用 (Ed25519, JWT風)

```bash
# 秘密鍵 (これはライセンスサーバが持つ)
openssl genpkey -algorithm ED25519 -out license-priv.pem
# 対応する公開鍵 (アプリにバンドル)
openssl pkey -in license-priv.pem -pubout -out license-pub.pem
```

GitHub Secrets:
- `SIFT_PUBLIC_KEY_PEM` ← `license-pub.pem` の中身

ライセンスサーバ側 (`licenser/` 参照) に `license-priv.pem` を秘密で持たせる。

## 2. macOS 公証セットアップ

1. Apple Developer Program に加入
2. 証明書 "Developer ID Application" を Keychain で生成 → エクスポート (.p12)
3. App-specific password を appleid.apple.com で発行 (公証用)
4. GitHub Secrets:
   - `APPLE_CERTIFICATE` ← .p12 を base64 エンコードした文字列
   - `APPLE_CERTIFICATE_PASSWORD` ← .p12 のパスワード
   - `APPLE_SIGNING_IDENTITY` ← `Developer ID Application: Your Name (TEAMID)`
   - `APPLE_ID` ← Apple ID メアド
   - `APPLE_PASSWORD` ← App-specific password
   - `APPLE_TEAM_ID` ← Team ID (10桁)

## 3. Windows コードサイン

OV/EV 証明書か Azure Trusted Signing を使う。クラウド HSM が今は楽:

1. Azure Trusted Signing アカウント作成
2. 認証情報を GitHub Secrets に:
   - `WINDOWS_CERTIFICATE`, `WINDOWS_CERTIFICATE_PASSWORD`

(あるいは `signtool.exe` を直接 CI から呼ぶ。.tauri/tauri.conf.json の `bundle.windows.signCommand` を設定。)

## 4. リリースの作り方

```bash
# CHANGELOG.md を更新
git tag v0.1.0
git push origin v0.1.0
```

これで `.github/workflows/release.yml` が走って:

1. mac (arm64/x64) / linux (x64) / windows (x64) で `tauri build`
2. それぞれ署名 & 公証
3. GitHub Releases に draft で attach
4. `latest.json` (auto-updater 用 manifest) を生成して同じ release に attach

draft を確認 → "Publish release" で本番化。
旧バージョンを使ってる人の手元には数時間以内にアップデート通知が出る。

## 5. ベータチャンネル (任意)

タグを `v0.2.0-beta.1` のように `-` を含めると prerelease 扱い。
manifest を分けたければ `tauri.conf.json` の updater endpoint を環境変数で切り替えて
beta ビルドだけ別 endpoint にする。

## 6. ロールバック手順

ヤバいバージョンが出てしまったら:

1. GitHub Releases で問題のリリースを "Pre-release" に切り替え (latest 扱いから外す)
2. ひとつ前のリリースを "Set as latest" に戻す
3. `latest.json` がそっちを指すようになる
4. 新しいインストールはひとつ前に戻る (既にインストールした人は手動ロールバック必要)

## 7. クラッシュレポート (任意)

Sentry を使うなら:

```toml
# Cargo.toml
sentry = "0.34"
```

`main.rs` 冒頭で `_guard = sentry::init(("https://...", ...));`、
ユーザーが telemetry をオフにしてたら一切送らない、というロジックを入れる。

## 8. 通常運用のチェックリスト

- [ ] CHANGELOG.md 更新した?
- [ ] バージョンを `package.json` / `tauri.conf.json` / `Cargo.toml` の 3 箇所揃えた?
- [ ] 手元で `npm run tauri build` が通る?
- [ ] テストした (主要 Smart Folder / ルール 1 つ / AI タグ)?
- [ ] タグ push した?
- [ ] CI 緑になった?
- [ ] Release notes に大事な変更を書いた?
- [ ] Publish した?
- [ ] X / Bluesky に告知?
