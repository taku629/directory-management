# やりたいことリスト

進捗管理用。✅ できた / 🚧 やってる / ◻ まだ。

> このリストは生きた TODO で、適当に追加・削除する。

---

## Phase 1 — まず動くやつ

### 裏側
- ✅ SQLite スキーマ (全 Phase 分)
- ✅ コネクションプール (r2d2 + WAL)
- ✅ ディレクトリ再帰スキャン
- ✅ FTS5 インデックス
- ✅ タグ + 関連付け
- ✅ 検索 (名前/パス/拡張子/タグ/サイズ/日付)
- ✅ お気に入り / スマートフォルダ / Watched root
- ✅ ファイル操作 (move/rename/delete) + 操作ログ
- ◻ サムネイルキャッシュ (画像系を JPG にして保存)
- ◻ Tokio で重い処理を非同期化 (今は同期スキャン)
- ◻ ゴミ箱に送る (今は完全削除)

### 表側
- ✅ アプリシェル
- ✅ ファイルブラウザ (grid / list、grid は画像サムネ)
- ✅ インスペクタ (タグ/評価/色/メモ + 画像/動画/音声プレビュー)
- ✅ サイドバー (favorites / smart / tags)
- ✅ 検索バー
- ✅ 設定画面
- ✅ キーボードショートカット (j/k, 矢印, Enter, Esc)
- ✅ 複数選択 (⌘/Ctrl/Shift クリック) + 一括削除
- ◻ PDF プレビュー (pdfium 入れる)
- ◻ ドラッグドロップでフォルダ移動
- ◻ 初回オンボーディング

---

## Phase 2 — 自動化と掃除

### ルールエンジン
- ✅ 条件評価器 (NameMatches/ExtensionIn/MimeStartsWith/SizeBetween/ModifiedWithinDays/PathIsIn/HasTag)
- ✅ アクション実行器 (MoveTo/RenameTo/AddTag/SetColor/SetRating/Notify) + テンプレート
- ✅ notify-debouncer-mini でフォルダ監視 → ルール評価
- ✅ Dry-run ボタン
- ✅ 即実行ボタン
- ◻ 定期実行 (cron 的な)
- ◻ ルールを JSON じゃなく UI で組み立てる (フルビジュアル)
- ◻ Run script アクション

### Undo
- ✅ move/rename/tag_add の逆操作実行
- ✅ 履歴 UI (直近 20 件)
- ◻ delete のゴミ箱対応 + undo
- ◻ set_color/set_rating の前値保存

### 重複検出
- ✅ SHA-256 でグルーピング + 不要ファイル削除 UI
- ◻ バックグラウンドジョブ化 (今は同期)
- ◻ 知覚ハッシュ (image_hasher) で類似画像

### ディスク使用量
- ✅ 再帰サイズ集計
- ✅ Treemap 描画 (slice-and-dice)
- ◻ Squarified treemap に置き換え
- ◻ 「でかいファイル Top N」/「冷たいファイル」スマートフォルダ

---

## Phase 3 — AI (Pro)

- ◻ Claude Vision で画像タグ付け
- ◻ ローカル CLIP もオプション (オフライン用)
- ◻ ユーザ定義カテゴリで分類
- ◻ 自然言語 → SearchQuery 変換
- ◻ ドキュメント要約 (PDF/docx/md)
- ◻ OCR (Tesseract or Apple Vision)
- ◻ 埋め込み + 類似検索

---

## Phase 4 — 配布と課金

- ◻ ライセンス検証サーバ (Lemon Squeezy か自前)
- ◻ JWT 検証
- ◻ S3 / B2 / Dropbox / Google Drive 同期
- ◻ ワークスペース (Team プラン)
- ◻ 自動アップデート
- ◻ Mac 公証 + Windows 署名
- ◻ ランディング + Stripe
- ◻ 多言語化 (EN/JA)

---

## どこかで考えたい
- 命名 (sift で行くかどうか)
- アイコン (今プレースホルダ)
- 「ホットキーで Spotlight 風に呼び出し」
- VS Code 拡張的な連携 (パスをコピーとか)
