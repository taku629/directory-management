# sift

ローカルのファイル/フォルダをいい感じに整理したいので、ちまちま作っているデスクトップアプリ。

> 個人プロジェクト。実装途中。動くけど荒い。

## 何これ

Finder/Explorer は便利なんだけど、

- ダウンロードフォルダがすぐ散らかる
- 写真とかスクショが何万枚あって全然探せない
- タグみたいな自分用メタデータを付けたい
- AI で自動分類とか自動タグ付けできたら楽だなあ

…と前から思ってて、市販アプリ (Hazel + Eagle + DaisyDisk + Czkawka) を全部足したのが欲しいので、自分で作ってみることにした。

将来もし他の人にも便利そうなら課金にするかも、ぐらいの温度感。

## スタック

- Tauri 2 (Rust) + React + TypeScript + Vite
- SQLite (FTS5) でメタデータ
- 軽量 (10MB 弱) で動かしたいので Electron じゃなく Tauri

選定理由のメモは [docs/architecture.md](docs/architecture.md) に。

## いま動くもの

- 指定したフォルダの一覧表示 (グリッド / リスト)
- フォルダの再帰インデックス (SQLite に放り込む)
- タグ付け (複数タグ + 色 + 階層)
- 星評価 + カラーラベル + メモ
- 全文検索 (ファイル名 + メモ、FTS5)
- お気に入り、スマートフォルダ (保存検索)
- ファイル操作 (move / rename / delete) と操作ログ

## まだ動かないもの (TODO 多め)

- ルールで自動整理
- フォルダ監視 (notify)
- 重複検出
- ディスク使用量の可視化
- AI タグ付け / OCR / 自然言語検索
- クラウド同期

進捗は [docs/roadmap.md](docs/roadmap.md)、メモは [NOTES.md](NOTES.md) に書いてる。

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

## ライセンス

個人プロジェクトなのでまだ決めてない。とりあえず無断利用禁止で
([LICENSE](LICENSE))、その辺は気が向いたら考える。
