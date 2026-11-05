# rs-wk

Rust（[workers-rs](https://github.com/cloudflare/workers-rs)）+ Cloudflare Workers で構成した Web アプリケーションの雛形です。ランタイムとツールのバージョンは [mise](https://mise.jdx.dev/) で固定しています。

Cloudflare Workers の無料プランで動作する構成です（WASM モジュールとしてデプロイされます）。

## 必要なもの

- [mise](https://mise.jdx.dev/getting-started.html)
- Cloudflare アカウント（デプロイ時のみ）

Rust / Node.js / wasm ターゲット / `worker-build` / `wrangler` は `mise.toml` とセットアップタスクで導入されます。事前に Rust をインストールしておく必要はありません。

## セットアップ

```bash
# 1. mise が管理するツール（Rust, Node.js）をインストール
mise install

# 2. wasm32 ターゲット、worker-build、npm 依存関係をセットアップ
mise run setup
```

シェル連携（`mise activate`）を設定していない場合は、各コマンドの前に `mise exec --` を付けてください。

```bash
mise exec -- npm run dev
```

## 開発

```bash
mise run dev
```

[http://localhost:8787](http://localhost:8787) で起動します。`src/lib.rs` を変更すると自動でリビルドされます。

```bash
mise run check    # cargo check（wasm32）
mise run fmt      # cargo fmt
mise run clippy   # cargo clippy
mise run build    # リリースビルド（dist/ に出力、デプロイはしない）
```

## デプロイ

初回のみ Cloudflare にログインします。

```bash
mise exec -- npx wrangler login
```

その後、デプロイします。

```bash
mise run deploy
```

`wrangler.toml` の `name` が Worker 名になります。独自ドメインやルートを設定する場合は `wrangler.toml` に追記してください。

> **無料プランについて**
> - リクエスト数: 1 日 100,000 件まで
> - CPU 時間: 1 リクエストあたり 10 ms まで
> - Worker サイズ: 圧縮後 3 MiB まで
>
> 上限や条件は変更される可能性があるため、最新の[料金ページ](https://developers.cloudflare.com/workers/platform/pricing/)を確認してください。

## 構成

```
.
├── mise.toml          # ツールのバージョンとタスク定義
├── Cargo.toml         # Rust クレート設定
├── wrangler.toml      # Cloudflare Workers の設定
├── package.json       # wrangler と npm スクリプト
├── src/
│   ├── lib.rs         # Worker のエントリポイント（ルーティング）
│   └── index.html     # トップページ（include_str! で埋め込み）
└── .gitignore
```

## エンドポイント

| メソッド | パス          | 内容                                   |
| -------- | ------------- | -------------------------------------- |
| GET      | `/`           | トップページ（HTML）                   |
| GET      | `/api/health` | ヘルスチェック（JSON、`no-store`）     |

## ルーティングの追加

`src/lib.rs` の `Router` にハンドラを追加します。

```rust
router
    .get("/hello/:name", |_req, ctx| {
        let name = ctx.param("name").map(String::as_str).unwrap_or("world");
        Response::ok(format!("Hello, {name}!"))
    })
```

## メモ

- ビルドは `worker-build` が行い、`wrangler.toml` の `[build].command` から呼び出されます。
- `worker-build` は初回ビルド時に `wasm-bindgen` / `wasm-opt` を自動でダウンロードします。
- 秘密情報は `.dev.vars`（ローカル）と `wrangler secret put`（本番）で管理してください。`.dev.vars` は Git 管理対象外です。
- `build/` と `dist/` は生成物のため Git 管理対象外です。
