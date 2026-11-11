# 商い帖（あきないちょう） — 販売管理システムデモ（Rust / Cloudflare Workers）

[![CI](https://github.com/sirie8722-del/akinaicho-demo/actions/workflows/ci.yml/badge.svg)](https://github.com/sirie8722-del/akinaicho-demo/actions/workflows/ci.yml)
[![Deploy](https://github.com/sirie8722-del/akinaicho-demo/actions/workflows/deploy.yml/badge.svg)](https://github.com/sirie8722-del/akinaicho-demo/actions/workflows/deploy.yml)

「商い帖」は、見積から受注・売上・入金、発注・仕入・支払、締処理までを一冊の帖（帳簿）にまとめるイメージで名付けた販売管理アプリのデモです。

Django で作った元 PoC から、**見積・受注・売上・入金・発注・仕入・支払・締処理・帳票・集計**を Rust（[workers-rs](https://github.com/cloudflare/workers-rs)）+ Cloudflare Workers に移植したデモです。ランタイムとツールのバージョンは [mise](https://mise.jdx.dev/) で固定しています。

Cloudflare Workers の無料プランで動作する構成です（WASM モジュール）。

デモ公開 URL: **https://akinaicho-demo.s-irie8722.workers.dev**

> インメモリストアのため、表示中のデータは Worker の isolate が入れ替わると初期デモデータに戻ります。デモ操作中の変更が全員に永続的に共有されるものではありません。

## 実装範囲

- **ダッシュボード**: トップページに KPI（当月売上・入金・売掛残高・受注残高・当月仕入・買掛残高）、売上/仕入の 12 か月推移、得意先別/商品別 TOP5、対応待ち、伝票ステータス、最近の動きを表示
- **見積**: 一覧・検索、作成、編集、削除、見積書（A4）の印刷
- **受注**: 一覧・検索、作成、編集、削除、注文請書（A4）の印刷
- **見積 → 受注変換**: 見積の明細を引き継いで受注を作成
- **売上**: 受注の一部または全部を売上計上、納品書（A4）の印刷
- **入金**: 得意先ごとの入金登録
- **発注・仕入**: 受注の一部または全部を仕入先へ発注、発注の仕入計上
- **支払**: 仕入先ごとの支払登録
- **締処理**: 得意先締・仕入先締の月次プレビューと確定、締処理履歴
- **集計**: 売上・受注の得意先別/商品別/月別集計、受注残、発注残、売掛・買掛残高
- **金額計算**: 数量 × 単価、課税 / 非課税、消費税 10%、小計・税・合計の自動計算
- **マスタ**: 得意先 5 件・仕入先 3 件・商品 8 件・自社情報をシードデータとして保持
- **デモデータ**: 見積 100 件・受注 100 件、売上 70 件、発注 40 件、仕入 20 件、入金 45 件、支払 12 件
- **UI**: Michelin のデータビズ原則を参考にしたグレー背景＋白カードの台帳 UI、KPI ストリップ / 台帳パネル、明細行のドラッグ&ドロップ並べ替え、一覧のソート・検索・絞り込み、モバイル幅までのレスポンシブ表示（明細行はカード型に切替）
- **操作記録・お問い合わせ**: [rrweb](https://www.rrweb.io/) で画面操作を記録し、入力値をマスクしたままお問い合わせに添付。共有トークン付きの再生ページで、担当者が同じ画面と操作手順を確認できる
- **公開デモの保護**: 更新系 API の IP レート制限（60 回/分）、コレクションごとの登録上限（500 件）、`DEMO_READ_ONLY` による読み取り専用切替
- **無料枠の計測**: リクエストごとの wall time / WASM メモリ / CPU 制限シグナル

### 元 PoC から省いているもの

在庫、請求書、マスタ編集、認証、印刷レイアウトの種類切替（タイプ 01〜08 など）は対象外です。帳票は元 PoC のタイプ 01 相当を HTML / A4 印刷用に簡略化しています。

> **データの保持について**
> この PoC は `thread_local!` + `RefCell` によるインメモリストアです。データは isolate ごとに保持され、Worker の再起動やファイル変更で初期データに戻ります。永続化する場合は D1 / KV / Durable Objects に置き換えてください。集計も現在はインメモリの受注・売上データを Rust で集計しており、データ量が増える場合は D1 の SQL 集計へ移す前提です。

UI の方向性・デザイントークン・スコアは [docs/ui-direction.md](docs/ui-direction.md) にまとめています。

## 必要なもの

- [mise](https://mise.jdx.dev/getting-started.html)
- Cloudflare アカウント（デプロイ時のみ）

## セットアップ

```bash
# 1. mise が管理するツール（Rust, Node.js）をインストール
mise install

# 2. wasm32 ターゲット、worker-build、npm 依存関係をセットアップ
mise run setup
```

シェル連携（`mise activate`）を使っていない場合は `mise exec --` を付けます。

```bash
mise exec -- npm run dev
```

## 開発

```bash
mise run dev
```

[http://localhost:8787](http://localhost:8787) で起動します。`src/` を変更すると自動でリビルドされます。

```bash
mise run check    # cargo check（wasm32）
mise run fmt      # cargo fmt
mise run clippy   # cargo clippy
mise run build    # リリースビルド（dist/ に出力、デプロイはしない）
mise run verify   # 無料枠の目安 + 見積/受注フローの確認
```

## 画面

| パス | 内容 |
| --- | --- |
| `/` | ダッシュボード（タブで各機能に切替） |
| `#/dashboard` | ダッシュボード |
| `#/quotes` | 見積一覧 |
| `#/quotes/new` | 見積の新規作成 |
| `#/quotes/:id/edit` | 見積の編集（受注済みは閲覧のみ） |
| `#/orders/new` | 受注の新規作成 |
| `#/orders/:id/edit` | 受注の編集（売上・発注済みは閲覧のみ） |
| `#/orders/:id/sales` | 受注残からの売上計上 |
| `#/orders/:id/purchase` | 受注残からの発注 |
| `#/sales` | 売上一覧 |
| `#/purchase-orders` | 発注一覧（仕入計上） |
| `#/purchases` | 仕入一覧 |
| `#/payments` | 入金・支払の登録と一覧 |
| `#/closings` | 得意先締・仕入先締のプレビューと実行 |
| `#/summary` | 集計表（売上/受注、受注残、発注残、売掛・買掛） |
| `#/recordings` | 操作記録の一覧・お問い合わせ履歴 |
| `/replay/:token` | 操作記録の再生（rrweb Replayer） |
| `/quotes/:id/print` | 見積書（A4・印刷 / PDF 保存） |
| `/orders/:id/print` | 注文請書（A4・印刷 / PDF 保存） |
| `/purchase-orders/:id/print` | 発注書（A4・印刷 / PDF 保存） |
| `/sales/:id/print` | 納品書（A4・印刷 / PDF 保存） |
| `/reports/summary/print` | 集計表（A4・印刷 / PDF 保存） |

帳票ページはブラウザの印刷機能で PDF に保存できます。`@page { size: A4; }` を指定済みです。

## API

| メソッド | パス | 内容 |
| --- | --- | --- |
| GET | `/api/health` | ヘルスチェック |
| GET | `/api/diagnostics` | 無料枠の上限・メモリ・CPU 制限シグナル |
| GET | `/api/dashboard` | トップページ用の KPI・推移・ランキング・最近の動き |
| GET | `/api/meta` | 得意先・商品・自社情報・税率・当日 |
| GET | `/api/quotes` | 見積一覧 |
| POST | `/api/quotes` | 見積作成 → `201` |
| GET | `/api/quotes/:id` | 見積取得 |
| PUT | `/api/quotes/:id` | 見積更新（受注済みは `409`） |
| DELETE | `/api/quotes/:id` | 見積削除 → `204` |
| POST | `/api/quotes/:id/convert` | 受注に変換 → `201` |
| GET | `/api/orders` | 受注一覧 |
| POST | `/api/orders` | 受注作成 → `201` |
| GET | `/api/orders/:id` | 受注取得 |
| PUT | `/api/orders/:id` | 受注更新 |
| DELETE | `/api/orders/:id` | 受注削除 → `204` |
| POST | `/api/orders/:id/sales` | 受注残から売上計上 |
| POST | `/api/orders/:id/purchase` | 受注残から発注 |
| GET | `/api/sales` | 売上一覧 |
| DELETE | `/api/sales/:id` | 売上削除（受注数量を戻す） |
| GET | `/api/purchase-orders` | 発注一覧 |
| POST | `/api/purchase-orders/:id/receive` | 仕入計上 |
| DELETE | `/api/purchase-orders/:id` | 未仕入の発注削除 |
| GET | `/purchase-orders/:id/print` | 発注書（A4） |
| GET | `/api/purchases` | 仕入一覧 |
| GET/POST | `/api/receipts` | 入金一覧 / 登録 |
| GET/POST | `/api/supplier-payments` | 支払一覧 / 登録 |
| GET | `/api/closings/preview` | 締処理プレビュー |
| GET/POST | `/api/closings` | 締処理履歴 / 実行 |
| GET | `/api/reports/summary` | 売上・受注集計（`source` / `group` / `from` / `to`） |
| GET | `/api/reports/order-remaining` | 受注残一覧 |
| GET | `/api/reports/purchase-remaining` | 発注残一覧 |
| GET | `/api/reports/balances` | 売掛・買掛残高 |
| GET | `/api/recordings` | 操作記録の一覧（イベント本体を含まない） |
| POST | `/api/recordings` | 操作記録の開始 → `201`（`id` と共有 `token`） |
| POST | `/api/recordings/:id/events` | rrweb イベントを追記（1 回 100 件まで） |
| POST | `/api/recordings/:id/finish` | お問い合わせ内容を添えて送信済みにする |
| DELETE | `/api/recordings/:id` | 操作記録の削除 |
| GET | `/api/replay/:token` | 共有トークンで再生用のイベントとお問い合わせ内容を取得 |

エラーは `{ "error": "..." }` の JSON（`400` / `404` / `409` / `413`）で返します。金額はすべてサーバ側で再計算します。

### 見積の作成例

```bash
curl -X POST http://localhost:8787/api/quotes \
  -H 'content-type: application/json' \
  -d '{
    "customer_code": "C001",
    "date": "2026-11-05",
    "subject": "オフィスPC環境更新",
    "valid_until": "2026-12-05",
    "delivery_date": "2026-12-20",
    "delivery_place": "貴社 本社",
    "payment_terms": "納品後 30日以内",
    "notes": "納期は発注後 2 週間を予定しています。",
    "status": "sent",
    "lines": [
      { "product_code": "P001", "quantity": 2, "unit_price": 128000, "taxable": true },
      { "product_code": "P002", "quantity": 3, "unit_price": 3500, "taxable": true }
    ]
  }'
```

### 受注への変換

```bash
curl -X POST http://localhost:8787/api/quotes/3/convert
```

変換後の見積は `status: "accepted"`、`order_id` が設定されます。受注を削除すると見積は「提出済み」に戻ります。

### 集計

```bash
# 受注を得意先別（全期間）
curl 'http://localhost:8787/api/reports/summary?source=orders&group=customer'

# 売上を月別・期間指定
curl 'http://localhost:8787/api/reports/summary?source=sales&from=2026-09-01&to=2026-10-31&group=month'

# 受注残 / 発注残 / 売掛・買掛残高
curl 'http://localhost:8787/api/reports/order-remaining'
curl 'http://localhost:8787/api/reports/purchase-remaining'
curl 'http://localhost:8787/api/reports/balances'
```

`source` は `orders` / `sales`、`group` は `customer` / `product` / `month` です。レスポンスは `rows`（区分・件数・数量・小計・消費税・合計）と `totals` を返します。商品別の消費税は明細単位で計算するため、伝票単位の税額合計と数円異なる場合があります。

集計表は `GET /reports/summary/print?source=...&group=...` で A4 印刷できます。

### 販売・調達の流れ

1. 見積を作成し、「受注へ」で受注に変換
2. 受注一覧の「売上へ」で、受注残から一部または全部を売上計上
3. 入出金タブで得意先の入金を登録
4. 受注一覧の「発注へ」で、受注残から仕入先へ発注
5. 発注一覧の「仕入計上」で仕入を登録
6. 入出金タブで仕入先への支払を登録
7. 締処理タブで得意先締・仕入先締をプレビューして実行
8. 集計タブで売上・受注・残高を確認

### 操作記録とお問い合わせ

画面右下の「操作記録」から、操作手順をそのままお問い合わせに添付できます。

1. 「記録を開始」を押し、お問い合わせしたい操作を行う
2. 「記録を停止」を押し、件名・お問い合わせ内容（任意で名前・メール）を入力して送信
3. 発行された `/replay/:token` の共有リンクを担当者に渡す
4. 担当者は再生ページで画面を巻き戻し・早送りしながら、主なクリック操作を一覧で確認できる

記録は `#/recordings` の一覧からも再生・削除できます。入力欄の値は rrweb の `maskAllInputs` で伏せ字にし、記録ウィジェット自身は `.rr-block` で録画対象から除外しています。それでもデモデータや画面上の文字は記録されるため、公開デモでは機密情報を入力しない前提で利用してください。名前・メールアドレスは一覧 API には含めず、共有トークンの再生ページでのみ表示します。認証のない公開デモのため、記録一覧自体は誰でも閲覧できます。

保存は isolate 内のインメモリで、Worker の再起動時に消えます。上限は 10 記録・1 記録あたり 5,000 イベント / 約 3 MB です。共有 URL は連番ではなく推測されにくいトークンですが、認証ではありません。rrweb の配布ファイルは `src/vendor/` に固定バージョンで同梱しています。rrweb 本体は `/recorder.js`、プレイヤーは `/replay.js` にサーバ側で連結して返すため、セッション記録系の URL をブロックするブラウザ拡張の影響を受けにくい構成です。

## 帳票

- 見積書: 御見積書 / 見積番号 / 見積日 / 得意先・ご担当 / 件名 / 有効期限 / 受渡期日・場所 / 取引方法 / 明細 / 小計・消費税・合計 / 備考 / 自社情報 / 振込先
- 注文請書: 注文請書 / 受注番号 / 受注日 / 得意先 / 注文番号 / 納期 / 見積番号 / 明細 / 小計・消費税・合計 / 備考 / 自社情報
- 発注書: 発注書 / 発注番号 / 発注日 / 仕入先 / 受注番号 / 状態 / 明細 / 小計・消費税・合計 / 備考 / 自社情報
- 納品書: 納品書 / 売上番号 / 売上日 / 得意先 / 受注番号 / 見積番号 / 明細 / 小計・消費税・合計 / 自社情報
- 集計表: 売上/受注の集計区分・対象期間・件数・数量・小計・消費税・合計 / 自社情報

元 PoC の ReportLab レイアウトを参考に、A4 印刷向けの HTML + CSS で実装しています。

## 無料枠の確認

```bash
# ターミナル 1
mise run dev

# ターミナル 2
mise run verify
```

`mise run verify` は以下を確認します。

- 100 リクエストの wall time（p50 / p95 / max）
- WASM リニアメモリと 128 MiB に対する使用率
- 見積 100 件・受注 100 件・売上 70 件・発注 40 件・仕入 20 件などのデモデータ
- 受注 → 売上 → 入金、受注 → 発注 → 仕入 → 支払のフロー
- 得意先締のプレビューと実行
- 売上 / 受注 / 月別 / 受注残 / 発注残 / 売掛・買掛の集計
- 見積作成 → 更新 → 受注変換 → 帳票出力 → 削除の一連の流れ
- 見積 CRUD 20 回前後のメモリ増加
- JST の今日と `meta.today` の一致
- 存在しない日付（`2026-02-31`）と不正な対象月（`abcd-01`）が 400 で拒否されること
- 128 MiB 超過時は終了コード `1`

### リクエストごとのログ

通常の 200 応答はログ量を抑えるため出力せず、エラー（400 以上）または 100ms 以上かかったリクエストだけを出力します。

```
[metrics] GET /api/quotes -> 500 | wall=120.000ms | wasm_mem=1.188MiB (start=1.188MiB) | cpu_limit=10ms | near_cpu_limit=false
```

| ヘッダ / エンドポイント | 内容 |
| --- | --- |
| `server-timing: app;dur=...` | Worker ハンドラの wall time（ミリ秒） |
| `x-worker-wall-time-ms` | 同上（数値のみ） |
| `x-wasm-memory-kib` | WASM リニアメモリ（KiB） |
| `GET /api/diagnostics` | 上限値、メモリ使用率、CPU 制限シグナル（本番では `DIAGNOSTICS_ENABLED=false` で 404） |

### ローカル検証の限界

- `wall` は **CPU 時間ではなく経過時間**です。
- `wasm_mem` は **WebAssembly のリニアメモリのみ**で、isolate 全体（JS ヒープを含む）のメモリではありません。
- ローカルの workerd は、本番の 10 ms CPU 制限や 128 MiB メモリ制限を強制しません。
- 正確な CPU 時間は、デプロイ後に **Workers Logs**（`[observability]` 有効化済み）で確認できます。
- 1 日 100,000 リクエストの上限はアカウント単位です。
- workerd の `performance.now()` は約 1 ms 分解能です。

## 構成

```
.
├── mise.toml              # ツールのバージョンとタスク定義
├── Cargo.toml             # Rust クレート設定
├── wrangler.toml          # Cloudflare Workers の設定
├── package.json           # wrangler と npm スクリプト
├── scripts/
│   └── verify-free-tier.mjs  # 無料枠 + 見積/受注フローの検証
├── src/
│   ├── lib.rs             # エントリポイント / ルーティング / 計測 / 共通ヘッダー
│   ├── api.rs             # JSON API ハンドラ
│   ├── service.rs         # 見積・受注・売上・発注・締処理の業務ロジック
│   ├── guard.rs           # 公開デモ向けの読み取り専用 / レート制限
│   ├── models.rs          # ドメインモデル・金額計算・日付処理
│   ├── store.rs           # インメモリストアとシードデータ（見積/受注 100 件＋売上・発注・仕入・入出金）
│   ├── reports.rs         # 集計ロジック（ダッシュボード / 得意先別・商品別・月別）
│   ├── print.rs           # 見積書 / 注文請書 / 納品書 / 発注書 / 集計表の HTML 帳票
│   ├── metrics.rs         # wall time / WASM メモリの計測
│   ├── index.html         # SPA のシェル
│   ├── app.js             # 画面描画・ルーティング・API 呼び出し
│   ├── styles.css         # 基本スタイル
│   └── ledger.css         # 紙とインクの台帳テーマ
├── docs/
│   ├── code-review.md     # コードレビュー記録
│   └── ui-direction.md    # UI 方向性・デザイントークン・スコア
├── LICENSE                # MIT
└── .gitignore
```

## 元 PoC との対応

| Django PoC | この実装 |
| --- | --- |
| `T_見積伝票` / `T_見積明細` | `Quote` / `LineItem` |
| `T_受注伝票` / `T_受注明細` | `Order` / `LineItem` |
| `T_売上伝票` / `T_売上明細` | `Sale` / `LineItem` |
| `T_発注伝票` / `T_仕入伝票` | `PurchaseOrder` / `Purchase` |
| `T_入金伝票` / `T_支払伝票` | `Receipt` / `SupplierPayment` |
| `T_得意先締` / `T_仕入先締` | `ClosingRecord` |
| `M_得意先` / `M_商品` / `M_会社情報` | `Customer` / `Product` / `Company`（シード） |
| `slip.py` の明細・合計計算 | `models::totals` / 数量 × 単価 |
| `conversion.adopt_quotation_to_order` | `POST /api/quotes/:id/convert` |
| `reports` の見積書 / 受注書 / 納品書 | `/quotes/:id/print` / `/orders/:id/print` / `/sales/:id/print` |
| `reports` の売上集計・受注明細表 | `/api/reports/summary` / `/reports/summary/print` |

簡略化のため、伝票番号は `Q-0001` / `O-0001` / `S-0001` などの連番、税区分は「伝票毎・消費税 10%・四捨五入」に固定しています。締処理は月単位の累計残高で計算し、締処理後の当月データ変更までは考慮しません。

## デプロイ

```bash
mise exec -- npx wrangler login   # 初回のみ（ブラウザで OAuth 認証）
mise run deploy
```

`wrangler.toml` の `name = "akinaicho-demo"` が Worker 名です。デプロイ後の URL は `https://<Worker 名>.<アカウントのサブドメイン>.workers.dev` になります。

- 本番 URL: https://akinaicho-demo.s-irie8722.workers.dev
- アカウントの確認: `mise exec -- npx wrangler whoami`
- ログの確認: Cloudflare ダッシュボード → Workers & Pages → `akinaicho-demo` → Logs（`[observability]` 有効化済み）

永続化を入れる場合は D1 の作成とバインディング追加が必要です。現在は `thread_local!` のインメモリ実装のため、isolate の入れ替わりでデータがリセットされます。

> **無料プランについて**
> - リクエスト数: 1 日 100,000 件まで
> - CPU 時間: 1 リクエストあたり 10 ms まで
> - メモリ: isolate あたり 128 MiB まで
> - Worker サイズ: 64 MiB まで（非圧縮）
>
> 最新の[料金ページ](https://developers.cloudflare.com/workers/platform/pricing/)と[Limits](https://developers.cloudflare.com/workers/platform/limits/)を確認してください。

## 公開デモの保護

このデモには認証がありません。公開 URL で誰でも操作できるため、次の保護を入れています。

- 更新系 API（POST / PUT / DELETE）は `cf-connecting-ip` ごとに **60 回/分**まで（isolate 単位のベストエフォート）
- 見積・受注・売上・発注・仕入・入金・支払・締処理はコレクションごとに **500 件**まで
- 読み取り専用にしたい場合は Cloudflare の secret で切り替え

```bash
# 更新系をすべて 403 にする
mise exec -- npx wrangler secret put DEMO_READ_ONLY
# 値: true

# 本番で /api/diagnostics を 404 にする
mise exec -- npx wrangler secret put DIAGNOSTICS_ENABLED
# 値: false
```

レート制限は isolate ごとのメモリで保持する簡易実装です。本格的な防御には Cloudflare の Rate Limiting ルール、Turnstile、Cloudflare Access などを併用してください。

## GitHub Actions

`.github/workflows/ci.yml` と `.github/workflows/deploy.yml` を用意しています。

| Workflow | トリガー | 内容 |
| --- | --- | --- |
| CI | `main` / `develop` への push、Pull Request、手動 | `cargo fmt --check`、Clippy、Worker ビルド、`dist/` artifact、`wrangler dev` を使った runtime API 検証 |
| Deploy | 手動実行（main）、`v*` タグ（main 上のコミットのみ） | fmt / Clippy / ビルド → Cloudflare Workers へデプロイ → `/api/health` スモークテスト |

### Deploy workflow に必要な Secrets

GitHub の **Settings → Secrets and variables → Actions** に次の 2 つを登録してください。

| Secret | 内容 |
| --- | --- |
| `CLOUDFLARE_API_TOKEN` | Workers の編集権限を持つ API トークン（Cloudflare ダッシュボード → My Profile → API Tokens → 「Edit Cloudflare Workers」テンプレート） |
| `CLOUDFLARE_ACCOUNT_ID` | Cloudflare のアカウント ID（`wrangler whoami` でも確認可能） |

`gh` CLI を使う場合は次のように登録できます。

```bash
gh secret set CLOUDFLARE_API_TOKEN
gh secret set CLOUDFLARE_ACCOUNT_ID
```

Deploy workflow は `production` environment を使うため、GitHub 側で承認ルールを設定すると本番デプロイ前に確認を挟めます。

## メモ

- ビルドは `worker-build` が行い、`wrangler.toml` の `[build].command` から呼び出されます。
- `worker-build` は初回ビルド時に `wasm-bindgen` / `wasm-opt` を自動でダウンロードします。
- 秘密情報は `.dev.vars`（ローカル）と `wrangler secret put`（本番）で管理してください。`.dev.vars` は Git 管理対象外です。
- `build/` と `dist/` は生成物のため Git 管理対象外です。
- `src/index.html` / `styles.css` / `app.js` は `include_str!` で Worker に埋め込まれます。

## ライセンス

MIT License（`LICENSE` を参照）。
