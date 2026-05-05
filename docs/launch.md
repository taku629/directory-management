# ローンチ Runbook

「ちゃんと動くアプリができた」から「お金が振り込まれる」までのチェックリスト。

---

## T-30 日: 準備フェーズ

### 法務 / 会社
- [ ] 個人事業の開業届 (or 法人)
- [ ] 銀行口座 (個人事業用)
- [ ] 屋号の登記 (任意)
- [ ] インボイス番号取得 (T+...) — 法人/年間1000万超なら

### ブランド
- [ ] 商品名最終決定 (Sift で行く / 他で行く)
- [ ] ドメイン取得 (`sift.app`, `getsift.com` 等。`.app` は HTTPS 必須)
- [ ] アイコン/ロゴ作成 (Figma → 1024x1024 PNG → `sips` で各サイズ生成)
- [ ] スクリーンショット 5 枚撮影 (機能ごと)
- [ ] スクリーンキャスト GIF 1 本 (タグ→検索のフロー)
- [ ] OG 画像 (1200x630 PNG)

### 課金
- [ ] **Lemon Squeezy アカウント作成** (Stripe より楽、税金もやってくれる)
- [ ] 商品設定 (Pro 買い切り ¥4,800 / Pro サブスク ¥500/月 / Team)
- [ ] License key 機能を有効化
- [ ] テスト購入 → メール → アプリで activate まで通す
- [ ] 返金ポリシーをサイトに明記 (14 日)

### 配布インフラ
- [ ] Apple Developer Program 加入 ($99)
- [ ] Windows コードサイン契約 (Azure Trusted Signing 推奨)
- [ ] GitHub Releases に上げる準備 (`docs/distribution.md`)
- [ ] Cloudflare Pages にマーケサイトデプロイ

### サポート
- [ ] サポートメールアドレス (`hello@sift.app`)
- [ ] FAQ 準備
- [ ] サポート用 Notion or Linear

---

## T-7 日: ベータ

- [ ] 信頼できる人 5〜10 人にβ版配布
- [ ] フィードバック収集 (Tally とか Google Forms)
- [ ] クラッシュ・データ損失系を最優先で潰す
- [ ] テスト購入を友人にやってもらう
- [ ] Privacy / Terms を弁護士に見てもらう (5 万円〜)

---

## T-1 日: 仕込み

- [ ] [Product Hunt 投稿準備](https://www.producthunt.com/posts/new)
  - タグライン (60 字以内)
  - 説明 (260 字以内)
  - 画像 5 枚 + GIF 1 枚
  - 「ローンチ日 0:00 PST」を狙う
- [ ] HN Show HN 用テキスト
- [ ] Twitter/X / Bluesky スレッド (技術ノート + GIF)
- [ ] 個人ブログ記事 (作った経緯 / なぜ Tauri / コスト感)
- [ ] [Hacker News に Show HN](https://news.ycombinator.com/submit) (太平洋時間 08:00〜10:00 がベスト)

---

## ローンチ当日

- [ ] 0:00 PST に Product Hunt 投稿
- [ ] 朝にスレッド・ブログ・HN に同時投下
- [ ] 全日コメント返信に張り付く (2-3 時間ごと)
- [ ] Discord/Slack コミュニティで宣伝 (押し売りにならない範囲)
- [ ] r/tauri, r/sideproject, r/macapps でシェア
- [ ] フィードバックがあれば即対応リリース

---

## ローンチ +7 日

- [ ] 売上集計 (LS ダッシュボード)
- [ ] フィードバック整理 → ロードマップ更新
- [ ] サポート問い合わせ全部返信
- [ ] バグレポート対応リリース v0.1.1
- [ ] 振り返り記事 (Indie Hackers / 個人ブログ)

---

## マイルストーン

- 🎯 **Day 1**: ProductHunt 100 upvote、HN 50 point
- 🎯 **Week 1**: 100 ダウンロード、10 Pro 購入 (= ¥48,000)
- 🎯 **Month 1**: 1,000 ダウンロード、50 Pro
- 🎯 **Month 3**: MRR ¥30,000 (買い切りメイン)
- 🎯 **Month 6**: MRR ¥100,000 → 副業として継続するか本気でやるか判断

数字いかなくても、**自分が毎日使うアプリができた** だけで成功。

---

## やってはいけないこと

- 機能多すぎて伝わらない LP → 「3 つの動詞」に絞る
- AI ガチ推し → 「AI で〜」を一行目に書かない (食傷気味)
- 値段が一番上 → ベネフィットを先に
- 「世界初」「革命的」 → 信用なくす
- 早すぎる localization (英語 + 日本語で十分)
- いきなり Team プラン売る → 個人売りで土台固めてから

---

## 役に立つリンク

- [Indie Hackers](https://www.indiehackers.com/products/) — 似た規模の事例
- [Tauri Showcase](https://tauri.app/showcase/) — 競合と差別化のヒント
- [Lemon Squeezy License Keys](https://docs.lemonsqueezy.com/help/license-keys/license-key-management) — 連携ガイド
- [PaddleSimulator](https://www.indiestack.com/simulators/) — 価格シミュレーション
