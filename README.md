# 商い帖（あきないちょう） — 販売管理システムデモ（Rust / Cloudflare Workers）

「商い帖」は、見積から受注・売上・入金、発注・仕入・支払、締処理までを一冊の帖（帳簿）にまとめるイメージで名付けた販売管理アプリのデモです。

`C:\Users\LIFE-PC110.LIFE\Desktop\WebRakkaru` の Django PoC から、**見積・受注・売上・入金・発注・仕入・支払・締処理・帳票・集計**を Rust（[workers-rs](https://github.com/cloudflare/workers-rs)）+ Cloudflare Workers に移植したデモです。ランタイムとツールのバージョンは [mise](https://mise.jdx.dev/) で固定しています。

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
- **UI**: 明細行のドラッグ&ドロップ並べ替え、一覧のソート・検索・絞り込み、モバイル幅までのレスポンシブ表示（明細行はカード型に切替）
- **無料枠の計測**: リクエストごとの wall time / WASM メモリ / CPU 制限シグナル

### 元 PoC から省いているもの

在庫、請求書、マスタ編集、認証、印刷レイアウトの種類切替（タイプ 01〜08 など）は対象外です。帳票は元 PoC のタイプ 01 相当を HTML / A4 印刷用に簡略化しています。

> **データの保持について**
> この PoC は `thread_local!` + `RefCell` によるインメモリストアです。データは isolate ごとに保持され、Worker の再起動やファイル変更で初期データに戻ります。永続化する場合は D1 / KV / Durable Objects に置き換えてください。集計も現在はインメモリの受注・売上データを Rust で集計しており、データ量が増える場合は D1 の SQL 集計へ移す前提です。

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

エラーは `{ "error": "..." }` の JSON（`400` / `404` / `409`）で返します。金額はすべてサーバ側で再計算します。

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
- 128 MiB 超過時は終了コード `1`

### リクエストごとのログ

```
[metrics] GET /api/quotes -> 200 | wall=1.000ms | wasm_mem=1.188MiB (start=1.188MiB) | cpu_limit=10ms | near_cpu_limit=false
```

| ヘッダ / エンドポイント | 内容 |
| --- | --- |
| `server-timing: app;dur=...` | Worker ハンドラの wall time（ミリ秒） |
| `x-worker-wall-time-ms` | 同上（数値のみ） |
| `x-wasm-memory-kib` | WASM リニアメモリ（KiB） |
| `GET /api/diagnostics` | 上限値、メモリ使用率、CPU 制限シグナル |

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
│   ├── lib.rs             # エントリポイント / ルーティング / 計測
│   ├── api.rs             # JSON API ハンドラ
│   ├── models.rs          # ドメインモデル・金額計算・日付処理
│   ├── store.rs           # インメモリストアとシードデータ（見積/受注 100 件＋売上・発注・仕入・入出金）
│   ├── reports.rs         # 受注の集計ロジック（得意先別/商品別/月別）
│   ├── print.rs           # 見積書 / 注文請書 / 集計表の HTML 帳票
│   ├── metrics.rs         # wall time / WASM メモリの計測
│   └── index.html         # 一覧・入力フォーム
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

## メモ

- ビルドは `worker-build` が行い、`wrangler.toml` の `[build].command` から呼び出されます。
- `worker-build` は初回ビルド時に `wasm-bindgen` / `wasm-opt` を自動でダウンロードします。
- 秘密情報は `.dev.vars`（ローカル）と `wrangler secret put`（本番）で管理してください。`.dev.vars` は Git 管理対象外です。
- `build/` と `dist/` は生成物のため Git 管理対象外です。
