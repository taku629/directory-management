# 設計メモ

個人プロジェクトの設計メモ。後で見返す用。

## なぜ Tauri にしたか

| | Tauri | Electron |
|-|-------|----------|
| 配布サイズ | 5–15 MB | 100–200 MB |
| メモリ | 30–80 MB | 200–500 MB |
| ファイル操作 | Rust ネイティブ | Node.js |
| 署名 / バンドル | `tauri build` 内蔵 | electron-builder 別途 |
| アップデータ | プラグインあり | サードパーティ |

「軽い」が今回大事。10 万ファイル単位のスキャンを Node でやりたくない。
Rust の練習にもなる。

## なぜ SQLite + FTS5

- 組み込みでサーバいらない
- FTS5 が rusqlite に同梱される
- WAL モードでスキャン中もブラウズできる

## なぜ React (Svelte じゃなく)

- 慣れてる
- ライブラリが多い (将来 treemap / 仮想スクロール / DnD で困りたくない)
- バンドルサイズの差は WebView 上ではあまり気にならない

## プロセスモデル

```
┌──────────────────────────────────────────────────┐
│              OS WebView                          │
│  React UI ←→ invoke('command', args)             │
└────────────────────┬─────────────────────────────┘
                     │  IPC (typed JSON)
┌────────────────────▼─────────────────────────────┐
│  Tauri Rust process                              │
│   ├─ AppState (DB pool, watcher)                 │
│   ├─ commands::*                                 │
│   ├─ fs::scanner / watcher / thumbnail           │
│   ├─ search::indexer                             │
│   └─ rusqlite + r2d2  →  SQLite                  │
└──────────────────────────────────────────────────┘
                     │
                     ▼
                <data_dir>/sift.db
```

## モジュール分け

| モジュール | 役割 | 状態 |
|--|--|--|
| `commands::files` | ブラウズ/インデックス/メタデータ/操作 | ✅ |
| `commands::tags` | タグ CRUD | ✅ |
| `commands::search` | 検索 | ✅ |
| `commands::settings` | KV 設定 | ✅ |
| `commands::rules` | ルール CRUD (実行は未実装) | 🚧 |
| `commands::duplicates` | 重複検出 | ◻ |
| `commands::disk` | ディスク使用量 | ◻ |
| `commands::history` | 履歴/Undo | 🚧 |
| `commands::ai` | AI 系 (Pro ゲート済) | ◻ |
| `commands::cloud` | クラウド同期 | ◻ |
| `commands::license` | ライセンス | 🚧 |

`commands::*` は IPC と DB のグルーだけ。ロジックは `<area>::` 側に置く方針。
コマンドを薄く保つことで単体テストしやすくしたい。

## 並行性

Tauri コマンドは async がデフォだけど、Phase 1 は短時間 (≤10ms) で済むので
同期 rusqlite で問題ない。長時間の処理 (スキャン, ハッシュ) は
`tokio::task::spawn_blocking` に逃がして、進捗は Tauri イベントで返す予定。

## インデックスの流れ

1. UI が `index_directory(path)` 呼ぶ
2. `walkdir` で再帰、トランザクションで `files` を upsert
3. `files_fts` に同期
4. レポート (件数) を返す
5. (Phase 2) watcher 登録 → 差分 upsert + ルール評価

## 機能を追加するときの手順

1. `src-tauri/src/<area>/mod.rs` でドメイン型/ロジック
2. `src-tauri/src/commands/<area>.rs` で `#[tauri::command]`
3. `lib.rs` の `invoke_handler!` に登録
4. `src/lib/tauri.ts` に typed なラッパ
5. `src/components/<Area>/` に UI
6. (任意) `Sidebar.tsx` にナビ追加
