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

1. **重複検出** ← 一番欲しい。SHA-256 でいけそう
2. **ディスク使用量 treemap** ← Treemap 描くの面白そう
3. **ルールエンジン** ← Hazel 相当。一番楽しい
4. **Watcher** ← notify クレートで。debouncer も必要
5. **Undo** ← 操作ログから逆操作生成。地味だけどあると安心

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
