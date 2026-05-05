# 開発メモ

ちまちま作業した記録。日付は適当。

---

## 最初

- 「ローカルファイル管理アプリ作りたい」から開始
- 市販品調べた:
  - Hazel ($42) → ルールベース自動整理。Mac 専用。
  - Eagle ($30) → 画像/動画ライブラリ。重い (Electron)。
  - DaisyDisk ($10) → ディスク使用量可視化のみ。
  - Czkawka → 重複検出 CLI、UI はおまけ。
  - **欲しいのは全部入りで軽いやつ**。なさそうなので作る。

## スタック決め

- Electron か Tauri で迷った
- Electron: エコシステム◎、配布サイズ△ (200MB)、メモリ多め
- Tauri: 配布サイズ◎ (10MB)、Rust の練習にもなる、商用配布も楽
- → Tauri にした
- フロントは React (慣れてるから)
- DB は SQLite + FTS5 (組み込み + 全文検索が一発)

## DB 設計

- 全フェーズ分のスキーマ最初に書いた → 後からマイグレーションで困らない
- `files`, `tags`, `file_tags`, `watched_roots`, `favorites`, `smart_folders` あたりが Phase 1
- `rules`, `operations` が Phase 2
- `ai_tags`, `ai_embeddings`, `ai_summaries`, `ocr_text` が Phase 3
- `license`, `cloud_sync`, `workspaces` が Phase 4
- 詳細は [docs/data-model.md](docs/data-model.md)

## Phase 1 進捗

### できた
- スキャナ (walkdir で再帰)
- FTS5 インデックス
- タグ + メタデータ CRUD
- 検索 (動的 WHERE + バインド)
- React 側 UI 一通り

### 詰まったとこ
- `tauri::Manager` を import し忘れて `app.manage(...)` がコンパイル通らなかった
- Linux で `cargo check` するのに `libwebkit2gtk-4.1-dev` 等必要
- FTS5 のクエリエスケープ、`*` を後ろに付けて prefix match に。記号入りのファイル名で詰まりそう

## Phase 2 やりたい順

1. ~~**重複検出**~~ ✅ SHA-256 でいけた。サイズバケットでフィルタ→必要なやつだけハッシュ。意外と速い
2. ~~**ディスク使用量 treemap**~~ ✅ flexbox の slice-and-dice で済ませた。見た目はそれっぽい
3. ~~**ルールエンジン**~~ ✅ 条件/アクションを JSON シリアライズで保存、glob と template で十分動く
4. ~~**Watcher**~~ ✅ notify-debouncer-mini で 1 秒デバウンス。テストはまだ
5. ~~**Undo**~~ ✅ move/rename/tag_add だけ。delete はゴミ箱対応してから

### Phase 2 やってみての感想

- ルール書くのが「JSON 直書き」なのは UI として弱い。いずれフルビジュアル化したい
  (例: 条件をボタンで追加していく、アクションの template フィールドが UI で組み立てられる、等)
- Watcher、ファイル丸ごと書かれる前にイベント来る場合あり。サイズ 0 だと skip するロジック要るかも
- 重複検出のハッシュをバックグラウンドジョブにすべき。今は同期でフリーズの可能性
- treemap、flexbox 版は計算は楽だけど比率が小さい子が薄くなりすぎる。
  squarified treemap にすると見やすい。後回し

## Phase 3 やってみた

✅ Claude API クライアント (テキスト/Vision/tool-use) を自前で書いた
- SDK 使わず reqwest で直接 → 依存軽い
- API キーは SQLite に保存、リクエストは直接 anthropic に飛ぶ
- ai_tags / ai_summaries / ocr_text にキャッシュ

✅ 画像タグ付け (Vision)
- tool-use で構造化 JSON 返させてる
- confidence ≥ 0.6 のものは自動で user tag にも反映 → サイドバーに即出る

✅ 自然言語検索
- 「先月の犬の写真」みたいなクエリを SearchQuery に変換
- 実ファイルは送らずクエリだけ送る → 安い

✅ ドキュメント要約
- 今のとこテキスト系のみ。PDF は pdfium 入れる必要あり、保留

✅ OCR
- Tesseract サイドカーやめて Vision で済ませた → ビルド軽い
- 抽出テキストを FTS にも混ぜ込むので普通の検索でも引っかかる

### コスト感
- 画像 1 枚 ~$0.005 ぐらい (Sonnet 4.6 vision)
- 1000 枚タグ付けで $5 ≒ 750 円
- 思ったより高いので、バッチ処理で大量にやらせるならキャッシュ徹底

## やりつつ思ったこと

- 個人で使う分にはほぼ十分
- 売るなら課金導線がまだ弱い (Pro tier ゲートは入ってるけど活性化しない)
- アイコンと UI の magnetism が足りない
- 名前 "sift" は地味すぎる気もしてる

## Phase 4 着手 (商品化準備)

✅ **License system 本気版**
- Ed25519 署名 + 公開鍵バンドル (ビルド時 env)
- 14 日トライアル (machine_id ベース)
- machine-uid で機械固有 ID
- アプリ内 Pricing 画面 + LockedPanel リニューアル
- TrialBadge をトップバーに常駐

✅ **自動アップデート**
- tauri-plugin-updater 統合
- 設定画面に「今すぐ確認」/「起動時自動」
- GitHub Releases を更新元に

✅ **opt-in テレメトリ**
- デフォルトオフ
- カウンタだけ送る、ファイル名・パスは絶対送らない契約をコードに埋め込んだ

✅ **CI リリースパイプライン**
- mac (arm64 / x64) / linux / windows でタグ push → ビルド → 署名 → Release
- Apple 公証 / Windows コードサイン scaffolding (Secrets 入れれば動く)
- latest.json 自動生成スクリプト

✅ **マーケサイト**
- landing/index.html — フレームワーク不使用、CSS インライン
- privacy.html / terms.html (テンプレ、要弁護士チェック)

✅ **ライセンスサーバ参考実装** (licenser/)
- axum + Ed25519、JSON ファイル DB
- 本番は Lemon Squeezy のライセンス機能を使う想定

✅ **ドキュメント**
- distribution.md (鍵作成 / Apple 公証 / Win 署名 / ロールバック)
- commerce.md (課金フロー / LS vs Stripe / 価格根拠)
- launch.md (T-30 → 当日 → +7 のチェックリスト)

### まだやってない (要外部作業)
- ◻ Apple Developer Program 加入 ($99/年)
- ◻ Windows コードサイン契約
- ◻ Lemon Squeezy アカウント開設
- ◻ ドメイン取得 + Cloudflare Pages デプロイ
- ◻ 弁護士に privacy/terms レビュー
- ◻ アイコン・ロゴ・スクショ
- ◻ クラウド同期実装
- ◻ Squarified treemap

商品化のコード側はだいたい揃った。あとは外部サービスとの契約と、アイコン作って
Lemon Squeezy で商品設定して LP デプロイすれば売り出せる状態。

## アイデア / 妄想

- Spotlight っぽくグローバルホットキー → 検索
- ドラッグでタグ付け (タグサイドバーに drop)
- 写真の Exif 自動読み取り (撮影日でフォルダ分け、的な)
- 「最近触ってないファイル」スマートフォルダ
- ピン留めしたファイルだけプレビューサムネイル巨大化
- AI 系は Claude API 叩く形で。料金気になるからキャッシュ徹底
- Markdown ファイル選んだら inspector でレンダリング

## 売る話

売れる気はあんまりしてないけど、もしやるなら:

- Free: 単一ルート、基本機能
- Pro: $49 買い切り or $5/月 (ルール / AI / クラウド)
- 個人売りで Lemon Squeezy か Gumroad あたり

最初は無料で出して反応見るのが先かな。

## 雑メモ

- アイコンはちゃんと作りたい (今プレースホルダの青い四角)
- 名前は仮で `sift` だけど、もうちょっといいの考えたい
  - 候補: nest, stash, drawer, perch, …
- ロゴは適当に書いて Figma で詰める
