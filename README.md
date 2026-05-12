# Sift

ローカルのファイル/フォルダを「いい感じ」に整理するデスクトップアプリ。
タグ・全文検索・自動整理ルール・重複検出・ディスク可視化・AI 自動分類を、
Electron じゃなく Tauri で **10MB 弱** に収めた "全部入り" 版。

> **開発版 (v0.1.0 / pre-release)**。コア機能は一通り動く。公開リリース前の残作業は
> [`CHANGELOG.md`](CHANGELOG.md) の "Pending before first public release" を参照。

## 何これ

Finder/Explorer は便利なんだけど、

- ダウンロードフォルダがすぐ散らかる
- 写真とかスクショが何万枚あって全然探せない
- タグみたいな自分用メタデータを付けたい
- AI で自動分類とか自動タグ付けできたら楽だなあ

…という不満を、市販アプリ (Hazel + Eagle + DaisyDisk + Czkawka) を全部足したような
1 本のアプリで解決する。最初は無料で配って、ルール/AI/クラウド同期を Pro tier
(¥4,800 買い切り or ¥500/月) にする想定。価格・配布まわりは [`docs/commerce.md`](docs/commerce.md)。

## スタック

- Tauri 2 (Rust) + React + TypeScript + Vite
- SQLite (FTS5) でメタデータ
- 軽量 (10MB 弱) で動かしたいので Electron じゃなく Tauri

選定理由のメモは [docs/architecture.md](docs/architecture.md) に。

## いま動くもの

- フォルダの一覧表示 (グリッド / リスト) + 再帰インデックス (SQLite)
- タグ付け (複数 + 色 + 階層)、星評価、カラーラベル、メモ
- 全文検索 (ファイル名 + メモ + OCR テキスト、FTS5)
- お気に入り、スマートフォルダ (保存検索 + ビルトインプリセット)
- ルールで自動整理 (条件/アクションの構造化エディタ) + フォルダ監視 (notify)
- 重複検出 (SHA-256 完全一致 + perceptual hash の類似画像)
- ディスク使用量の treemap
- AI (Pro): 画像自動タグ付け / 自然言語検索 / 要約 / OCR (Claude API)
- Undo (move / rename / tag)、delete は OS ゴミ箱経由、操作ログ
- i18n (ja/en)、オンボーディング、ライセンス + 14 日トライアル、自動アップデート、opt-in テレメトリ

## まだ (ロードマップ)

- クラウド同期 (S3 / Dropbox / iCloud) — スキーマだけ用意済み
- PDF 要約 (pdfium バンドルが必要)
- squarified treemap、ルール UI のフルビジュアル化

進捗は [docs/roadmap.md](docs/roadmap.md)、開発メモは [NOTES.md](NOTES.md)、変更履歴は [CHANGELOG.md](CHANGELOG.md)。

## 動かし方

```bash
# Rust toolchain (rustup) と Node 20+ が前提
npm install
npm run tauri:dev
```

初回は

1. 上の **Open Folder…** で適当なフォルダを選ぶ
2. **Index this folder** でインデックス
3. 右の inspector でタグや評価を付けて遊ぶ

## ディレクトリ

```
src/                  React 側
src-tauri/            Rust 側
  src/commands/       Tauri IPC 入口
  src/db/             SQLite 関連
  src/fs/             scanner / watcher / thumbnail
  src/search/         検索クエリ
  src/{rules,duplicates,ai,cloud,license}/
docs/                 設計メモ
NOTES.md              開発メモ
```

## 商品化まわり

このリポジトリは「ちまちま開発」と「商品化準備」の両方を含んでる。

| | どこ |
|--|--|
| マーケサイト (静的 HTML) | [`landing/`](landing/) |
| 課金フロー設計 | [`docs/commerce.md`](docs/commerce.md) |
| ローンチ runbook | [`docs/launch.md`](docs/launch.md) |
| 配布手順 (公証 / 署名 / 鍵) | [`docs/distribution.md`](docs/distribution.md) |
| ライセンスサーバ参考実装 | [`licenser/`](licenser/) |
| リリース CI | [`.github/workflows/release.yml`](.github/workflows/release.yml) |
| 法務テンプレ (要弁護士レビュー) | [`landing/privacy.html`](landing/privacy.html), [`landing/terms.html`](landing/terms.html) |

3 行まとめ:

1. **Lemon Squeezy で買ってもらう** → メールでライセンスキー届く
2. **アプリの設定画面でキー貼る** → Ed25519 署名検証 → Pro 機能解放
3. **タグ push で CI が回る** → 各 OS 向けに署名済みインストーラ + 自動アップデート manifest 生成 → GitHub Releases に公開

詳細は [`docs/commerce.md`](docs/commerce.md) と [`docs/launch.md`](docs/launch.md)。

## ライセンス

プロプライエタリ (ソース閲覧・個人評価は可、再配布・商用利用は要ライセンス)。
詳細は [LICENSE](LICENSE)。Pro 機能は購入したライセンスキーで解放する。
