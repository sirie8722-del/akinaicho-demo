use worker::*;

mod api;
mod metrics;
mod models;
mod print;
mod reports;
mod store;

const INDEX_HTML: &str = include_str!("index.html");

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let started = metrics::now_ms();
    let mem_start = metrics::wasm_memory_bytes();
    let method = req.method().to_string();
    let path = req.path();

    let router = Router::new();

    let result = router
        .get("/", |_, _| Response::from_html(INDEX_HTML))
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
        .get("/api/diagnostics", |_, _| diagnostics())
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
        // 集計
        .get("/api/reports/summary", api::report_summary)
        .get("/api/reports/order-remaining", api::report_order_remaining)
        .get(
            "/api/reports/purchase-remaining",
            api::report_purchase_remaining,
        )
        .get("/api/reports/balances", api::report_balances)
        .get("/reports/summary/print", api::print_summary)
        .run(req, env)
        .await;

    let wall_ms = metrics::now_ms() - started;
    let mem_end = metrics::wasm_memory_bytes();

    let mut res = match result {
        Ok(res) => res,
        Err(err) => {
            console_error!("[error] {} {}: {}", method, path, err);
            Response::error("Internal Server Error", 500)?
        }
    };

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

/// 無料枠の上限と、このリクエストで観測した値を返す。
///
/// ここでの wall time / memory はローカルでも取得できる目安であり、
/// 本番の CPU 時間や isolate 全体のメモリを保証するものではない。
fn diagnostics() -> Result<Response> {
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
