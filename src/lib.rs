use worker::*;

mod api;
mod guard;
mod metrics;
mod models;
mod print;
mod recording;
mod reports;
mod service;
mod store;

const INDEX_HTML: &str = include_str!("index.html");
const STYLES_CSS: &str = include_str!("styles.css");
const LEDGER_CSS: &str = include_str!("ledger.css");
const APP_JS: &str = include_str!("app.js");
const RECORDER_CSS: &str = include_str!("recorder.css");
const RECORDER_JS: &str = include_str!("recorder.js");
const REPLAY_HTML: &str = include_str!("replay.html");
const REPLAY_JS: &str = include_str!("replay.js");
const REPLAY_CSS: &str = include_str!("replay.css");
const RRWEB_JS: &str = include_str!("vendor/rrweb.min.js");
const RRWEB_PLAYER_JS: &str = include_str!("vendor/rrweb-player.min.js");
const RRWEB_PLAYER_CSS: &str = include_str!("vendor/rrweb-player.css");

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let started = metrics::now_ms();
    let mem_start = metrics::wasm_memory_bytes();
    let method = req.method().to_string();
    let path = req.path();

    let router = Router::new()
        .get("/", |_, _| Response::from_html(INDEX_HTML))
        .get("/styles.css", |_, _| {
            static_asset(STYLES_CSS, "text/css; charset=utf-8")
        })
        .get("/ledger.css", |_, _| {
            static_asset(LEDGER_CSS, "text/css; charset=utf-8")
        })
        .get("/app.js", |_, _| {
            static_asset(APP_JS, "application/javascript; charset=utf-8")
        })
        // rrweb 本体を recorder.js に同梱する。別 URL のスクリプト読み込みを
        // ブロックするブラウザ拡張やキャッシュの影響を受けないようにするため。
        .get("/recorder.css", |_, _| {
            static_asset(RECORDER_CSS, "text/css; charset=utf-8")
        })
        .get("/recorder.js", |_, _| {
            combined_asset(
                &[RRWEB_JS, RECORDER_JS],
                "application/javascript; charset=utf-8",
            )
        })
        .get("/rrweb.js", |_, _| {
            static_asset(RRWEB_JS, "application/javascript; charset=utf-8")
        })
        .get("/rrweb-player.js", |_, _| {
            static_asset(RRWEB_PLAYER_JS, "application/javascript; charset=utf-8")
        })
        .get("/rrweb-player.css", |_, _| {
            static_asset(RRWEB_PLAYER_CSS, "text/css; charset=utf-8")
        })
        // 再生ページもプレイヤーを replay.js に同梱する。
        .get("/replay.js", |_, _| {
            combined_asset(
                &[RRWEB_PLAYER_JS, REPLAY_JS],
                "application/javascript; charset=utf-8",
            )
        })
        // 拡張機能の URL フィルタを避けるための別名。
        .get("/player.css", |_, _| {
            static_asset(RRWEB_PLAYER_CSS, "text/css; charset=utf-8")
        })
        .get("/replay.css", |_, _| {
            static_asset(REPLAY_CSS, "text/css; charset=utf-8")
        })
        // 操作記録の再生ページ（rrweb Replayer）
        .get("/replay/:token", |_, _| Response::from_html(REPLAY_HTML))
        .get("/api/health", |_, _| {
            let mut res = Response::from_json(&serde_json::json!({
                "status": "ok",
                "runtime": "cloudflare-workers",
                "language": "rust",
            }))?;
            res.headers_mut().set("cache-control", "no-store")?;
            Ok(res)
        })
        // 無料枠の確認用（メモリ・レイテンシ・CPU 制限シグナル）
        .get("/api/diagnostics", |_, ctx| diagnostics(&ctx.env))
        // トップページのダッシュボード集計
        .get("/api/dashboard", api::dashboard)
        // マスタ（得意先・商品・自社情報）
        .get("/api/meta", api::meta)
        // 見積
        .get("/api/quotes", api::list_quotes)
        .post_async("/api/quotes", api::create_quote)
        .get("/api/quotes/:id", api::get_quote)
        .put_async("/api/quotes/:id", api::update_quote)
        .delete_async("/api/quotes/:id", api::delete_quote)
        .post_async("/api/quotes/:id/convert", api::convert_quote)
        .get("/quotes/:id/print", api::print_quote)
        // 受注
        .get("/api/orders", api::list_orders)
        .post_async("/api/orders", api::create_order)
        .get("/api/orders/:id", api::get_order)
        .put_async("/api/orders/:id", api::update_order)
        .delete_async("/api/orders/:id", api::delete_order)
        .get("/orders/:id/print", api::print_order)
        // 売上
        .post_async("/api/orders/:id/sales", api::create_sale)
        .get("/api/sales", api::list_sales)
        .delete_async("/api/sales/:id", api::delete_sale)
        .get("/sales/:id/print", api::print_sale)
        // 発注・仕入
        .post_async("/api/orders/:id/purchase", api::create_purchase_order)
        .get("/api/purchase-orders", api::list_purchase_orders)
        .post_async(
            "/api/purchase-orders/:id/receive",
            api::receive_purchase_order,
        )
        .delete_async("/api/purchase-orders/:id", api::delete_purchase_order)
        .get("/purchase-orders/:id/print", api::print_purchase_order)
        .get("/api/purchases", api::list_purchases)
        // 入金・支払
        .get("/api/receipts", api::list_receipts)
        .post_async("/api/receipts", api::create_receipt)
        .get("/api/supplier-payments", api::list_supplier_payments)
        .post_async("/api/supplier-payments", api::create_supplier_payment)
        // 締処理
        .get("/api/closings/preview", api::preview_closing)
        .get("/api/closings", api::list_closings)
        .post_async("/api/closings", api::run_closing)
        // 操作記録・お問い合わせ（rrweb）
        .get("/api/recordings", recording::list)
        .post_async("/api/recordings", recording::create)
        .post_async("/api/recordings/:id/events", recording::append_events)
        .post_async("/api/recordings/:id/finish", recording::finish)
        .delete("/api/recordings/:id", recording::delete)
        // 共有トークンで取得（再生ページ用）
        .get("/api/replay/:token", recording::get_by_token)
        // 集計
        .get("/api/reports/summary", api::report_summary)
        .get("/api/reports/order-remaining", api::report_order_remaining)
        .get(
            "/api/reports/purchase-remaining",
            api::report_purchase_remaining,
        )
        .get("/api/reports/balances", api::report_balances)
        .get("/reports/summary/print", api::print_summary);

    let mutation = matches!(method.as_str(), "POST" | "PUT" | "PATCH" | "DELETE");
    let result = if mutation && path.starts_with("/api/") {
        match guard::check_mutation(&req, &env) {
            Ok(()) => router.run(req, env).await,
            Err((status, message)) => api::api_error(status, &message),
        }
    } else {
        router.run(req, env).await
    };

    let wall_ms = metrics::now_ms() - started;
    let mem_end = metrics::wasm_memory_bytes();

    let mut res = match result {
        Ok(res) => res,
        Err(err) => {
            console_error!("[error] {} {}: {}", method, path, err);
            Response::error("Internal Server Error", 500)?
        }
    };

    apply_common_headers(&path, &mut res);
    if res.status_code() == 429 {
        let _ = res.headers_mut().set("retry-after", "60");
    }
    metrics::annotate(&mut res, wall_ms, mem_start, mem_end);
    metrics::log_request(
        &method,
        &path,
        res.status_code(),
        wall_ms,
        mem_start,
        mem_end,
    );

    Ok(res)
}

/// 静的アセットを適切な Content-Type で返す。
fn static_asset(body: &'static str, content_type: &str) -> Result<Response> {
    let mut res = Response::from_body(ResponseBody::Body(body.as_bytes().to_vec()))?;
    res.headers_mut().set("content-type", content_type)?;
    res.headers_mut().set("cache-control", "no-cache")?;
    Ok(res)
}

/// 複数のスクリプトを 1 レスポンスに連結して返す。
/// rrweb を別 URL で読み込まずに済ませるための同梱用。
fn combined_asset(parts: &[&'static str], content_type: &str) -> Result<Response> {
    let mut body = Vec::new();
    for (index, part) in parts.iter().enumerate() {
        // UMD の末尾が `}))` で終わるため、次の IIFE と連結されないよう
        // 区切りにセミコロンを入れる。
        if index > 0 {
            body.extend_from_slice(b"\n;\n");
        }
        body.extend_from_slice(part.as_bytes());
        body.push(b'\n');
    }
    let mut res = Response::from_body(ResponseBody::Body(body))?;
    res.headers_mut().set("content-type", content_type)?;
    res.headers_mut().set("cache-control", "no-cache")?;
    Ok(res)
}

/// API / HTML / アセットに共通のキャッシュ・セキュリティヘッダーを付与する。
fn apply_common_headers(path: &str, res: &mut Response) {
    let headers = res.headers_mut();
    let _ = headers.set("x-content-type-options", "nosniff");
    let _ = headers.set("referrer-policy", "no-referrer");
    let _ = headers.set("x-frame-options", "SAMEORIGIN");

    if path.starts_with("/api/") || path.ends_with("/print") {
        let _ = headers.set("cache-control", "no-store");
        return;
    }

    if path.starts_with("/replay/") {
        let _ = headers.set("cache-control", "no-store");
        let _ = headers.set("x-robots-tag", "noindex");
        let _ = headers.set(
            "content-security-policy",
            "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; font-src 'self' data:; connect-src 'self'; frame-src 'self' data: blob:; base-uri 'none'; frame-ancestors 'self'",
        );
        return;
    }

    if matches!(
        path,
        "/" | "/styles.css"
            | "/ledger.css"
            | "/app.js"
            | "/recorder.css"
            | "/recorder.js"
            | "/rrweb.js"
            | "/rrweb-player.js"
            | "/rrweb-player.css"
            | "/player.css"
            | "/replay.js"
            | "/replay.css"
    ) {
        let _ = headers.set("cache-control", "no-cache");
        let _ = headers.set(
            "content-security-policy",
            "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'; base-uri 'none'; frame-ancestors 'self'",
        );
    }

    // rrweb の配布ファイルはデプロイ直後の 404 をキャッシュしないよう no-store にする。
    if matches!(
        path,
        "/rrweb.js" | "/rrweb-player.js" | "/rrweb-player.css" | "/player.css"
    ) {
        let _ = headers.set("cache-control", "no-store");
    }
}

/// 無料枠の上限と、このリクエストで観測した値を返す。
///
/// `DIAGNOSTICS_ENABLED=false` が設定されている場合は 404 を返す。
/// 本番では secret で無効化し、ローカル / CI では既定で有効にする。
fn diagnostics(env: &Env) -> Result<Response> {
    let enabled = env
        .var("DIAGNOSTICS_ENABLED")
        .ok()
        .map(|value| !value.to_string().eq_ignore_ascii_case("false"))
        .unwrap_or(true);
    if !enabled {
        return Response::error("Not Found", 404);
    }

    let started = metrics::now_ms();
    let memory = metrics::wasm_memory_bytes();
    let handler_wall_ms = metrics::round3(metrics::now_ms() - started);
    let memory_mib = metrics::round3(metrics::mib(memory));
    let memory_percent =
        metrics::round3(memory as f64 / metrics::FREE_PLAN_MEMORY_BYTES as f64 * 100.0);

    let mut res = Response::from_json(&serde_json::json!({
        "free_plan": {
            "cpu_ms_per_request": metrics::FREE_PLAN_CPU_MS,
            "memory_mib_per_isolate": metrics::FREE_PLAN_MEMORY_MIB,
            "requests_per_day": metrics::FREE_PLAN_REQUESTS_PER_DAY,
        },
        "measured": {
            "wasm_memory_bytes": memory,
            "wasm_memory_mib": memory_mib,
            "wasm_memory_percent_of_limit": memory_percent,
            "handler_wall_time_ms": handler_wall_ms,
        },
        "cpu_limit_signal": {
            "supported": worker::signal::is_registered(),
            "near_limit": worker::signal::is_near_cpu_limit(),
        },
        "notes": [
            "wasm_memory は WebAssembly のリニアメモリのみで、isolate 全体（JS ヒープを含む）のメモリではありません。",
            "handler_wall_time_ms は CPU 時間ではなく経過時間です。正確な CPU 時間は Workers Logs で確認してください。",
            "ローカルの workerd は本番の CPU 時間・メモリ上限を強制しません。"
        ],
    }))?;
    res.headers_mut().set("cache-control", "no-store")?;
    Ok(res)
}
