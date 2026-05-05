# Marketing landing page

純粋な静的 HTML/CSS。フレームワーク不使用。

## 編集

```bash
# ローカルプレビュー
python3 -m http.server -d landing 8000
```

## デプロイ

無料の選択肢:
- **Cloudflare Pages**: `wrangler pages deploy landing` または GitHub 連携
- **Vercel**: `vercel deploy landing`
- **Netlify**: ドラッグ&ドロップ or `netlify deploy --dir landing`
- **GitHub Pages**: `landing/` を `gh-pages` ブランチに push

ドメイン (例: `sift.app`) を取って Cloudflare に向けるのがおすすめ。
DNS は Cloudflare、ホスティングは Pages という組み合わせが運用コスト 0 円。

## やることリスト

- [ ] スクリーンショットを撮って `/screenshot.png` 配置
- [ ] アイコン・ロゴ
- [ ] OG 画像 (`og.png`, 1200x630)
- [ ] favicon
- [ ] 購入リンク (`#buy`) を Lemon Squeezy / Stripe Checkout に向ける
- [ ] アナリティクス (Plausible / Fathom 推奨、GA は重い)
- [ ] 法務テキスト (privacy.html / terms.html) を弁護士チェック
