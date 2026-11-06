# コードレビュー（develop ブランチ）

- 対象ブランチ: `develop`
- 対象コミット: `2f4aa5b`（main と同一）
- レビュー日: 2026-11-06
- 対象: `src/*.rs`、`src/index.html`、`.github/workflows/*`、`wrangler.toml`、`README.md`
- 方法: 静的レビュー + ローカル Worker / 本番 Worker での動作確認

## 総評

デモとしての完成度は高く、見積〜締処理までの業務フロー、帳票、集計、レスポンシブ UI、無料枠の計測、CI/CD まで一通り揃っています。HTML/JS/帳票のエスケープも一貫しており、レビュー範囲では XSS は見つかりませんでした。

一方で、**公開デモとしての防御（認証・レート制限）**、**日付まわりの業務的な正しさ**、**自動テスト不足**、**CI/CD のガードレール**に改善余地があります。特に日付は、Workers のタイムゾーンが UTC であるため、日本時間の早朝に「今日」が前日になる問題があります。

## 今回の対応（2026-11-06）

- **H-1**: 更新系 API に IP 単位のレート制限（60 回/分）、コレクション上限（500 件）、`DEMO_READ_ONLY` による読み取り専用切替を追加。
- **H-2**: CI に `wrangler dev` を起動する runtime API 検証を追加。JST 日付・不正日付・不正対象月のチェックも追加。
- **H-3**: `develop` を CI の push トリガーに追加。
- **H-4**: Deploy workflow に fmt / Clippy / ビルドを追加し、main 以外のブランチと main 外のタグを拒否。
- **H-5 / M-1 / M-3**: JST 固定の日付処理、実在日付チェック、対象月の完全検証を追加。
- **M-5**: API は `no-store`、HTML/アセットは `no-cache`、CSP / nosniff / frame-ancestors を追加。
- **M-12**: `index.html` から CSS / JS を分離し、API の業務ロジックを `service.rs` に分離。
- **L-3 / L-10 / L-11 / L-12 / L-13 / L-14**: `unsigned_abs()`、MIT LICENSE、パッケージ名統一、diagnostics の環境無効化、workflow timeout、ログ出力の条件化を実施。


## 指摘一覧

| ID | 重要度 | 分類 | 対象 | 概要 | 状態 |
| --- | --- | --- | --- | --- | --- |
| H-1 | High | セキュリティ | `src/lib.rs`, `src/api.rs` | 更新系 API に認証・レート制限がない | **一部対応**（レート制限・件数上限・読み取り専用スイッチ） |
| H-2 | High | テスト | `.github/workflows/ci.yml` | 自動テストがなく、CI は fmt / clippy / build のみ | **対応**（runtime API 検証 + 入力検証） |
| H-3 | High | CI | `.github/workflows/ci.yml` | `develop` への push で CI が動かない | **対応済み** |
| H-4 | High | CD | `.github/workflows/deploy.yml` | CI 成功を待たずに任意の ref / タグからデプロイできる | **対応**（品質チェック + ref 制限） |
| H-5 | High | 日付 | `src/models.rs` | `today()` が UTC 基準で、JST の早朝に日付が 1 日ずれる | **対応** |
| M-1 | Medium | 入力検証 | `src/models.rs` | 存在しない日付（例: 2026-02-31）を受け付ける | **対応** |
| M-2 | Medium | UI / 整合性 | `src/index.html`, `src/api.rs` | 見積の状態で「受注済」を選べるが、サーバは draft に矯正する | 未対応 |
| M-3 | Medium | 入力検証 | `src/api.rs` | 締処理の対象月で年部分が未検証（例: `abcd-01`） | **対応** |
| M-4 | Medium | 集計 | `src/reports.rs` | 受注残高の消費税を明細単位で丸めており、伝票単位の税額と一致しない可能性 | 未対応 |
| M-5 | Medium | セキュリティ / キャッシュ | `src/lib.rs` | HTML / API に `cache-control` とセキュリティヘッダーがない | **対応** |
| M-6 | Medium | パフォーマンス | `src/index.html` | 1 回の更新で 9 API を並列取得し、約 295 KB を再取得 | 未対応 |
| M-7 | Medium | アクセシビリティ | `src/index.html` | 明細の並べ替えがマウスの HTML5 Drag and Drop のみ | 未対応 |
| M-8 | Medium | 業務ルール | `src/api.rs` | 受注済み見積の削除、下書き見積の受注変換が可能 | 未対応 |
| M-9 | Medium | 整合性 | `src/index.html`, `src/api.rs` | 受注を手動で「完了」にでき、受注残 KPI と矛盾する | 未対応 |
| M-10 | Medium | 入力検証 | `src/api.rs`, `src/store.rs` | 文字列長・入金方法・明細数の上限が不十分 | 未対応 |
| M-11 | Medium | 設計 | `src/store.rs` | インメモリ + isolate ごとの保持で、データがリセット・分岐する | 既知の制約 |
| M-12 | Medium | 保守性 | `src/index.html`, `src/app.js`, `src/styles.css`, `src/api.rs`, `src/service.rs` | 画面と業務ロジックを分割済み（更なる細分化は可能） | **一部対応** |
| M-13 | Medium | サプライチェーン | `.github/workflows/*` | Actions がタグ参照で、`dtolnay/rust-toolchain@master` は可変 | 未対応 |
| M-14 | Medium | 依存関係 | CI | `cargo audit` / `npm audit` が未実行 | 未対応 |
| L-1 | Low | ドキュメント | `README.md` | ローカルの絶対パス（ユーザー名を含む）が記載されていた | **対応済み** |
| L-2 | Low | 帳票 | `src/print.rs` | 帳票が 210mm 固定で、スマートフォンでは横スクロールになる | 未対応 |
| L-3 | Low | 境界値 | `src/models.rs` | `format_yen(i64::MIN)` で `abs()` がオーバーフローし得る | **対応** |
| L-4 | Low | API | `src/api.rs` | 集計の `source` が不正でも受注扱いにフォールバックする | 未対応 |
| L-5 | Low | 入力検証 | `src/api.rs` | 入金・支払の `method` が自由文字列 | 未対応 |
| L-6 | Low | 性能 | 全体 | 一覧にページネーションがない | 未対応 |
| L-7 | Low | UI | `src/index.html` | `refresh()` の abort / 競合制御がない | 未対応 |
| L-8 | Low | UI | `src/index.html` | 作成成功後の `refresh()` 失敗で再送信すると二重作成になり得る | 未対応 |
| L-9 | Low | 整合性 | `src/api.rs` | 売上・発注作成時に受注の `updated_at` を更新していない | 未対応 |
| L-10 | Low | 法務 | リポジトリ | LICENSE がない | **対応**（MIT） |
| L-11 | Low | 設定 | `package.json` | パッケージ名が `rs-wk` のまま | **対応** |
| L-12 | Low | 情報公開 | `src/lib.rs` | `/api/diagnostics` が本番でも公開されている | **対応**（`DIAGNOSTICS_ENABLED=false` で 404） |
| L-13 | Low | CI | `.github/workflows/*` | `timeout-minutes` が未設定 | **対応** |
| L-14 | Low | 運用 | `src/metrics.rs` | 全リクエストをログ出力しており、ログ量が多い | **対応**（エラー / 100ms 以上のみ） |

## 詳細

### H-1 更新系 API に認証・レート制限がない

`src/lib.rs` のルーティングは、見積・受注・売上・入金・発注・仕入・支払・締処理の作成/更新/削除をすべて公開しています。`src/api.rs` にも認証・認可のチェックはありません。

- 影響: URL を知っていれば誰でもデモデータを変更・削除できます。無料枠のリクエスト数や CPU を消費させることも可能です。
- 現状は「インメモリでリセットされるデモ」なので実害は限定的ですが、公開URLとして割り切るなら README に明記が必要です。
- 推奨:
  - デモ公開用に読み取り専用モードを用意する、または
  - Cloudflare Access / Turnstile / 共有トークンで更新系を保護する、または
  - Cloudflare Rate Limiting ルールで `/api/*` の POST/PUT/DELETE を制限する。

### H-2 自動テストがない

`src` 配下にテストコードはなく、CI は `cargo fmt --check`、Clippy、ビルドのみです。今回の開発中にも、見積一覧のルーティング漏れ（`#/quotes` がダッシュボードになる）や、明細金額セルのクラス名変更による計算失敗を手動で発見しています。

- 推奨:
  - `models` / `reports` の純粋関数に対するユニットテスト（日付、税計算、残高、集計）。
  - `wrangler dev` を起動して `scripts/verify-free-tier.mjs` を実行する integration ジョブ。
  - 主要な画面遷移のスモークテスト（Playwright など）。

### H-3 `develop` への push で CI が動かない（対応済み）

`ci.yml` の `push.branches` が `[main]` のみでした。`develop` を追加しました。

```yaml
on:
  push:
    branches: [main, develop]
```

### H-4 Deploy workflow のガードレール

`deploy.yml` は `workflow_dispatch` と `v*` タグで実行できますが、CI の成功を前提にしていません。また `production` environment に保護ルールがありません。

- 推奨:
  - `workflow_run` で CI 成功後にデプロイする、またはデプロイ job 内で fmt / clippy / build を再実行する。
  - `workflow_dispatch` の ref を `main` に限定する、またはタグは `main` 上のコミットのみ許可する。
  - `production` environment に Required reviewers を設定する。

### H-5 `today()` が UTC 基準

`src/models.rs` の `today()` / `date_days_ago()` は `js_sys::Date` のローカル日付 getter を使っています。Cloudflare Workers のタイムゾーンは UTC のため、日本時間の 00:00〜08:59 は「今日」が前日になります。

- 影響: 見積日・受注日・入金日・締処理月・ダッシュボードの当月判定が、JST の早朝だけ 1 日ずれます。
- 推奨: JST 固定の日付ヘルパーを用意する。

```rust
// 例: UTC ミリ秒に +9h して UTC getter でフォーマットする
let jst = js_sys::Date::new(&JsValue::from_f64(now.get_time() + 9.0 * 3_600_000.0));
format!("{:04}-{:02}-{:02}", jst.get_utc_full_year(), jst.get_utc_month() + 1, jst.get_utc_date())
```

### M-1 存在しない日付を受け付ける

`is_valid_date()` は月を 1〜12、日を 1〜31 でしか検証していません。ローカル API で `date: "2026-02-31"` を送信し、`Q-0101` として保存されることを確認しました。

- 影響: 帳票・月別集計・締処理に存在しない日付が入ります。
- 推奨: うるう年を含む月ごとの日数で検証する。`chrono` を追加するか、月日テーブルで実装する。

### M-2 見積の「受注済」選択が無視される

`src/index.html` の見積フォームには `accepted`（受注済）の選択肢がありますが、`src/api.rs` の `create_quote_inner()` / `update_quote_inner()` は `sent` 以外を `draft` に矯正します。

- 影響: ユーザーが「受注済」を選んで保存しても、画面には「下書き」で保存されます。
- 推奨: フォームから `accepted` を削除し、受注済みは読み取り専用のステータス表示にする。API も `accepted` を明示的に 400 にする。

### M-3 締処理の対象月で年が未検証

`validate_closing_month()` は `YYYY-MM` の長さと月だけを見ており、年部分が数字であることを確認していません。`month=abcd-01` でプレビューが 200 と残高行を返すことを確認しました。

- 推奨: `^\d{4}-(0[1-9]|1[0-2])$` で検証する。

### M-4 受注残高の消費税の丸め

`reports::dashboard()` の受注残高は、明細ごとに `amount + round(amount * 0.1)` を合計しています。実際の売上計上時は「伝票の課税対象小計 × 10%」を 1 回だけ丸めるため、明細数が多いと数円差が出る可能性があります。

- 推奨: 受注ごとに残数量の課税対象金額を合計し、その合計に対して 1 回だけ税を計算する。
- なお商品別集計の明細単位税は README に注記済みで、意図的な仕様です。

### M-5 キャッシュ / セキュリティヘッダー

`/`、`/api/quotes`、`/api/dashboard` のレスポンスに `cache-control` がありません。また CSP、`X-Content-Type-Options`、`Referrer-Policy`、`X-Frame-Options` なども未設定です。

- 推奨: `fetch` の最後に共通ヘッダーを付与する。
  - API: `cache-control: no-store`
  - HTML: `cache-control: no-cache`
  - セキュリティ: `x-content-type-options: nosniff`、`referrer-policy: no-referrer`、`x-frame-options: DENY`
  - CSP はインライン script/style があるため、nonce 化するか `'unsafe-inline'` を許容して段階導入する。

### M-6 1 回の更新で 9 API / 約 295 KB

`refresh()` は dashboard、quotes、orders、sales、purchase-orders、purchases、receipts、supplier-payments、closings を毎回並列取得します。ローカル実測では合計約 295 KB（非圧縮）でした。

- 内訳: quotes 92 KB、orders 88 KB、sales 58 KB、purchase-orders 30 KB、purchases 15 KB など。
- 推奨: `/api/bootstrap` で初回のみまとめて取得し、更新後は変更したリソースだけ再取得する。または一覧をページネーションする。

### M-7 並べ替えがマウス専用

HTML5 Drag and Drop はタッチ端末では動作せず、キーボード操作もできません。レスポンシブでカード型に切り替えても、並べ替えだけはデスクトップ専用になっています。

- 推奨: 各行に「上へ / 下へ」ボタンを追加する、または Pointer Events でタッチ対応する。キーボード操作も併用する。

### M-8 受注済み見積の削除 / 下書き見積の変換

- `delete_quote_inner()` は受注済みでも削除でき、受注側の `quote_id` / `quote_number` を消します。監査性が失われます。
- `convert_quote_inner()` は `order_id` の有無しか見ないため、下書き見積も受注に変換できます。
- 推奨: 受注済み見積の削除は 409 で拒否する。変換は `sent` のみ許可する、または明示的な確認を入れる。

### M-9 受注ステータスの手動変更

受注フォームで「完了」を選べるため、未売上の明細が残っていても `completed` にできます。ダッシュボードの受注残はステータスを見ずに残数量を集計するため、「完了なのに受注残がある」状態になります。

- 推奨: ステータスを売上数量から自動導出する、または「完了」を選ぶ際に全明細の売上完了を検証する。

### M-10 入力長・列挙値の上限

- `subject`、`notes`、`customer_order_no`、`delivery_place`、`payment_terms`、明細の `name` / `note` に長さ上限がありません。
- 入金・支払の `method` は任意文字列を受け付けます。
- 明細は 100 行までですが、売上・発注の `lines` には件数上限がありません（受注明細数に依存）。
- 推奨: 項目ごとの最大長、`method` の許可リスト、フロー明細数の上限を設定する。

### M-11 インメモリ + isolate ごとの保持

`thread_local!` の `STORE` は isolate ごとに独立しています。同じ URL でもリクエストが別 isolate に当たると、編集内容が見えたり消えたりします。README には記載がありますが、画面上の注意表示はありません。

- 推奨: デモであることを UI にも表示する。永続化するなら D1 / Durable Objects へ移行する。

### M-12 単一ファイルの肥大化

- `src/index.html`: 約 2,660 行（CSS / HTML / JS / ルーティング / 画面描画）
- `src/api.rs`: 約 1,286 行（全ドメインのハンドラと業務ロジック）

- 推奨: フロントは `app.js` / `styles.css` / 画面モジュールに分割する。API は quotes / orders / billing / reports などに分割する。

### M-13 Actions の固定

`dtolnay/rust-toolchain@master` は可変参照で、`Swatinem/rust-cache@v2` などのメジャータグも付け替え可能です。

- 推奨: コミット SHA でピン留めし、Dependabot / Renovate で更新する。

### M-14 依存関係の監査

`cargo audit` と `npm audit` が CI にありません。Rust / npm の依存脆弱性を継続的に検出するジョブを追加してください。

## 軽微な指摘

- **L-1（対応済み）**: README にローカルの絶対パス（`C:\Users\...`）が含まれていました。公開リポジトリ向けに一般的な表現へ修正しました。
- **L-2**: 帳票の `.paper` は `width: 210mm` 固定です。画面表示でスマートフォンから開くと横スクロールになります。`@media screen and (max-width: 800px)` で幅を `100%` にするなどの対応が可能です。
- **L-3**: `format_yen()` は `value.abs()` を使うため、`i64::MIN` でオーバーフローします。入力上限があるため実運用では到達しませんが、`unsigned_abs()` を使うと安全です。
- **L-4**: 集計の `source` が `sales` 以外だと受注扱いになります。不正値は 400 にする方が明確です。
- **L-5**: 入金・支払の `method` は UI では固定選択ですが、API では自由文字列です。許可リストを推奨します。
- **L-6**: 一覧は 100 件固定ですが、ページネーションや検索 API はありません。
- **L-7**: `refresh()` は abort せず、短時間に連続実行すると古いレスポンスが新しい状態を上書きする可能性があります。
- **L-8**: 作成 POST 成功後に `refresh()` が失敗すると、catch で送信ボタンが再活性化され、再送信で二重作成になり得ます。POST 成功後は画面遷移を優先するなどの分離を推奨します。
- **L-9**: 売上・発注の作成時に受注の `updated_at` を更新していません。
- **L-10**: Public リポジトリですが LICENSE がありません。デモの利用条件を明確にするなら追加を検討してください。
- **L-11**: `package.json` の `name` が `rs-wk`、Worker 名が `akinaicho-demo` です。統一すると分かりやすくなります。
- **L-12**: `/api/diagnostics` はメモリ・wall time・無料枠情報を返します。本番で不要なら無効化するか、管理者向けに制限してください。
- **L-13**: workflow に `timeout-minutes` がありません。ハング時の課金・待機を防ぐため設定を推奨します。
- **L-14**: `metrics::log_request()` が全リクエストをログ出力します。Workers Logs の量が増えるため、サンプリングやデバッグ時のみの出力を検討してください。

## 良い点

- HTML / JS / 帳票のエスケープが一貫しており、ユーザー入力由来の XSS は見つかりませんでした。
- 数量・単価のサーバ側検証、受注残・発注残の超過チェック、重複明細チェックが実装されています。
- 得意先名・仕入先名を伝票にスナップショット保存しており、マスタ変更の影響を受けにくい設計です。
- 仕入計上済み発注の削除禁止、売上削除時の売上済数量の戻しなど、業務整合性への配慮があります。
- モバイル幅でのカード型明細、ナビの折り返し、グラフのラベル間引きなど、レスポンシブ対応が実装されています。
- 無料枠の wall time / WASM メモリ / CPU 制限シグナルを可視化し、本番デプロイと CI/CD まで整備されています。
- README が機能・API・制約・デプロイ手順まで網羅しています。

## 推奨する対応順

1. **H-3（対応済み）**: `develop` の CI を有効化する。
2. **H-5 / M-1**: JST 固定の日付ヘルパーと、実在日付の検証を追加する。
3. **H-2**: 日付・税計算・集計のユニットテストと、`wrangler dev` を使う integration テストを追加する。
4. **H-1 / H-4**: 公開デモの更新系保護（読み取り専用 or トークン / Rate Limiting）と、Deploy の CI 成功条件・environment 保護を設定する。
5. **M-2 / M-8 / M-9**: 見積ステータス、削除・変換ルール、受注ステータスの整合性を直す。
6. **M-5 / M-6**: 共通レスポンスヘッダーと、初回ロード・更新時の API 呼び出し削減。
7. **M-7**: タッチ / キーボードでの明細並べ替え。
8. **M-13 / M-14**: Actions の SHA 固定、依存関係監査、Dependabot の導入。

## 検証ログ

- `mise run verify`: 成功（JST 日付・不正日付 400・不正対象月 400 を含む）。
- `2026-02-31` の見積作成: `400` で拒否。
- `month=abcd-01` の締処理プレビュー: `400` で拒否。
- `DEMO_READ_ONLY=true` で起動した Worker: 見積 POST が `403`。
- `/`、`/styles.css`、`/app.js`: CSP / nosniff / frame-ancestors / no-cache を確認。
- `/api/quotes`、`/api/dashboard`: `cache-control: no-store` を確認。
- ブラウザ確認: 外部 CSS / JS 読み込み、コンソールエラーなし、6 KPI・見積 100 件・明細計算を確認。
- 本番 `/api/health`、`/api/dashboard`、帳票 5 種: `200`（変更前デプロイ時点）。
