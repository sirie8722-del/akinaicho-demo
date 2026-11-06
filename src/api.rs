use serde::Deserialize;
use worker::*;

use crate::models::*;
use crate::print;
use crate::reports::{self, Summary};
use crate::service;
use crate::store;

pub(crate) fn api_error(status: u16, message: &str) -> Result<Response> {
    Ok(Response::from_json(&serde_json::json!({ "error": message }))?.with_status(status))
}

fn id_from(ctx: &RouteContext<()>) -> Option<u64> {
    ctx.param("id")
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|id| *id > 0)
}

#[derive(Deserialize)]
struct SummaryQuery {
    #[serde(default)]
    from: String,
    #[serde(default)]
    to: String,
    #[serde(default)]
    group: String,
    #[serde(default)]
    source: String,
}

fn run_summary(req: &Request) -> std::result::Result<Summary, (u16, String)> {
    let query: SummaryQuery = req
        .query()
        .map_err(|_| (400, "クエリパラメータが不正です。".to_string()))?;
    let from = normalize_optional_date(Some(&query.from)).map_err(|message| (400, message))?;
    let to = normalize_optional_date(Some(&query.to)).map_err(|message| (400, message))?;
    let group = if query.group.trim().is_empty() {
        "customer"
    } else {
        query.group.trim()
    };
    let source = if query.source == "sales" {
        "sales"
    } else {
        "orders"
    };

    store::with_store(|store| {
        if source == "sales" {
            reports::summarize_sales(&store.sales, &from, &to, group)
        } else {
            reports::summarize_orders(&store.orders, &from, &to, group)
        }
    })
    .map_err(|message| (400, message))
}

pub fn report_order_remaining(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    store::with_store(|store| Response::from_json(&reports::order_remaining(&store.orders)))
}

pub fn report_purchase_remaining(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    store::with_store(|store| {
        Response::from_json(&reports::purchase_remaining(&store.purchase_orders))
    })
}

pub fn report_balances(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    store::with_store(|store| {
        Response::from_json(&reports::balances(
            &store.customers,
            &store.suppliers,
            &store.sales,
            &store.receipts,
            &store.purchases,
            &store.supplier_payments,
        ))
    })
}

pub fn dashboard(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    store::with_store(|store| {
        Response::from_json(&reports::dashboard(
            &store.quotes,
            &store.orders,
            &store.sales,
            &store.purchase_orders,
            &store.purchases,
            &store.receipts,
            &store.supplier_payments,
        ))
    })
}

pub fn report_summary(req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    match run_summary(&req) {
        Ok(summary) => Response::from_json(&summary),
        Err((status, message)) => api_error(status, &message),
    }
}

pub fn print_summary(req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    match run_summary(&req) {
        Ok(summary) => {
            let company = store::with_store(|store| store.company.clone());
            Response::from_html(print::summary_page(&summary, &company))
        }
        Err((status, message)) => api_error(status, &message),
    }
}

pub fn meta(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    store::with_store(|store| {
        Response::from_json(&serde_json::json!({
            "customers": &store.customers,
            "suppliers": &store.suppliers,
            "products": &store.products,
            "company": &store.company,
            "tax_rate": TAX_RATE,
            "today": today(),
        }))
    })
}

pub fn list_quotes(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    store::with_store(|store| Response::from_json(&store.quotes.iter().rev().collect::<Vec<_>>()))
}

pub fn get_quote(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Some(id) = id_from(&ctx) else {
        return api_error(400, "見積 ID が不正です。");
    };
    store::with_store(|store| match store.quote(id) {
        Some(quote) => Response::from_json(quote),
        None => api_error(404, "見積が見つかりません。"),
    })
}

pub async fn create_quote(mut req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let input: QuoteInput = match req.json().await {
        Ok(input) => input,
        Err(_) => return api_error(400, "リクエストの JSON が不正です。"),
    };

    match store::with_store(|store| service::create_quote_inner(store, input)) {
        Ok(quote) => Response::from_json(&quote).map(|response| response.with_status(201)),
        Err((status, message)) => api_error(status, &message),
    }
}

pub async fn update_quote(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Some(id) = id_from(&ctx) else {
        return api_error(400, "見積 ID が不正です。");
    };
    let input: QuoteInput = match req.json().await {
        Ok(input) => input,
        Err(_) => return api_error(400, "リクエストの JSON が不正です。"),
    };

    match store::with_store(|store| service::update_quote_inner(store, id, input)) {
        Ok(quote) => Response::from_json(&quote),
        Err((status, message)) => api_error(status, &message),
    }
}

pub async fn delete_quote(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Some(id) = id_from(&ctx) else {
        return api_error(400, "見積 ID が不正です。");
    };

    match store::with_store(|store| service::delete_quote_inner(store, id)) {
        Ok(()) => Ok(Response::empty()?.with_status(204)),
        Err((status, message)) => api_error(status, &message),
    }
}

pub async fn convert_quote(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Some(id) = id_from(&ctx) else {
        return api_error(400, "見積 ID が不正です。");
    };

    match store::with_store(|store| service::convert_quote_inner(store, id)) {
        Ok(order) => Response::from_json(&order).map(|response| response.with_status(201)),
        Err((status, message)) => api_error(status, &message),
    }
}

pub fn print_quote(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Some(id) = id_from(&ctx) else {
        return api_error(400, "見積 ID が不正です。");
    };
    store::with_store(|store| match store.quote(id) {
        Some(quote) => Response::from_html(print::quote_page(quote, &store.company)),
        None => api_error(404, "見積が見つかりません。"),
    })
}

pub fn list_orders(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    store::with_store(|store| Response::from_json(&store.orders.iter().rev().collect::<Vec<_>>()))
}

pub fn get_order(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Some(id) = id_from(&ctx) else {
        return api_error(400, "受注 ID が不正です。");
    };
    store::with_store(|store| match store.order(id) {
        Some(order) => Response::from_json(order),
        None => api_error(404, "受注が見つかりません。"),
    })
}

pub async fn create_order(mut req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let input: OrderInput = match req.json().await {
        Ok(input) => input,
        Err(_) => return api_error(400, "リクエストの JSON が不正です。"),
    };

    match store::with_store(|store| service::create_order_inner(store, input)) {
        Ok(order) => Response::from_json(&order).map(|response| response.with_status(201)),
        Err((status, message)) => api_error(status, &message),
    }
}

pub async fn update_order(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Some(id) = id_from(&ctx) else {
        return api_error(400, "受注 ID が不正です。");
    };
    let input: OrderInput = match req.json().await {
        Ok(input) => input,
        Err(_) => return api_error(400, "リクエストの JSON が不正です。"),
    };

    match store::with_store(|store| service::update_order_inner(store, id, input)) {
        Ok(order) => Response::from_json(&order),
        Err((status, message)) => api_error(status, &message),
    }
}

pub async fn delete_order(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Some(id) = id_from(&ctx) else {
        return api_error(400, "受注 ID が不正です。");
    };

    match store::with_store(|store| service::delete_order_inner(store, id)) {
        Ok(()) => Ok(Response::empty()?.with_status(204)),
        Err((status, message)) => api_error(status, &message),
    }
}

pub fn print_order(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Some(id) = id_from(&ctx) else {
        return api_error(400, "受注 ID が不正です。");
    };
    store::with_store(|store| match store.order(id) {
        Some(order) => Response::from_html(print::order_page(order, &store.company)),
        None => api_error(404, "受注が見つかりません。"),
    })
}

pub fn list_sales(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    store::with_store(|store| Response::from_json(&store.sales.iter().rev().collect::<Vec<_>>()))
}

pub fn print_sale(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Some(id) = id_from(&ctx) else {
        return api_error(400, "売上 ID が不正です。");
    };
    store::with_store(
        |store| match store.sales.iter().find(|sale| sale.id == id) {
            Some(sale) => Response::from_html(print::sale_page(sale, &store.company)),
            None => api_error(404, "売上が見つかりません。"),
        },
    )
}

pub async fn delete_sale(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Some(id) = id_from(&ctx) else {
        return api_error(400, "売上 ID が不正です。");
    };
    match store::with_store(|store| service::delete_sale_inner(store, id)) {
        Ok(()) => Ok(Response::empty()?.with_status(204)),
        Err((status, message)) => api_error(status, &message),
    }
}

pub async fn create_sale(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Some(order_id) = id_from(&ctx) else {
        return api_error(400, "受注 ID が不正です。");
    };
    let input: SaleInput = match req.json().await {
        Ok(input) => input,
        Err(_) => return api_error(400, "リクエストの JSON が不正です。"),
    };
    match store::with_store(|store| service::create_sale_inner(store, order_id, input)) {
        Ok(sale) => Response::from_json(&sale).map(|response| response.with_status(201)),
        Err((status, message)) => api_error(status, &message),
    }
}

pub fn list_purchase_orders(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    store::with_store(|store| {
        Response::from_json(&store.purchase_orders.iter().rev().collect::<Vec<_>>())
    })
}

pub async fn create_purchase_order(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Some(order_id) = id_from(&ctx) else {
        return api_error(400, "受注 ID が不正です。");
    };
    let input: PurchaseOrderInput = match req.json().await {
        Ok(input) => input,
        Err(_) => return api_error(400, "リクエストの JSON が不正です。"),
    };
    match store::with_store(|store| service::create_purchase_order_inner(store, order_id, input)) {
        Ok(order) => Response::from_json(&order).map(|response| response.with_status(201)),
        Err((status, message)) => api_error(status, &message),
    }
}

pub async fn receive_purchase_order(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Some(id) = id_from(&ctx) else {
        return api_error(400, "発注 ID が不正です。");
    };
    let input: ReceiveInput = match req.json().await {
        Ok(input) => input,
        Err(_) => return api_error(400, "リクエストの JSON が不正です。"),
    };
    match store::with_store(|store| service::receive_purchase_order_inner(store, id, input)) {
        Ok(purchase) => Response::from_json(&purchase).map(|response| response.with_status(201)),
        Err((status, message)) => api_error(status, &message),
    }
}

pub async fn delete_purchase_order(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Some(id) = id_from(&ctx) else {
        return api_error(400, "発注 ID が不正です。");
    };
    match store::with_store(|store| service::delete_purchase_order_inner(store, id)) {
        Ok(()) => Ok(Response::empty()?.with_status(204)),
        Err((status, message)) => api_error(status, &message),
    }
}

pub fn print_purchase_order(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Some(id) = id_from(&ctx) else {
        return api_error(400, "発注 ID が不正です。");
    };
    store::with_store(|store| match store.purchase_order(id) {
        Some(order) => Response::from_html(print::purchase_order_page(order, &store.company)),
        None => api_error(404, "発注が見つかりません。"),
    })
}

pub fn list_purchases(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    store::with_store(|store| {
        Response::from_json(&store.purchases.iter().rev().collect::<Vec<_>>())
    })
}

pub fn list_receipts(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    store::with_store(|store| Response::from_json(&store.receipts.iter().rev().collect::<Vec<_>>()))
}

pub async fn create_receipt(mut req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let input: ReceiptInput = match req.json().await {
        Ok(input) => input,
        Err(_) => return api_error(400, "リクエストの JSON が不正です。"),
    };
    match store::with_store(|store| service::create_receipt_inner(store, input)) {
        Ok(receipt) => Response::from_json(&receipt).map(|response| response.with_status(201)),
        Err((status, message)) => api_error(status, &message),
    }
}

pub fn list_supplier_payments(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    store::with_store(|store| {
        Response::from_json(&store.supplier_payments.iter().rev().collect::<Vec<_>>())
    })
}

pub async fn create_supplier_payment(mut req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let input: SupplierPaymentInput = match req.json().await {
        Ok(input) => input,
        Err(_) => return api_error(400, "リクエストの JSON が不正です。"),
    };
    match store::with_store(|store| service::create_supplier_payment_inner(store, input)) {
        Ok(payment) => Response::from_json(&payment).map(|response| response.with_status(201)),
        Err((status, message)) => api_error(status, &message),
    }
}

#[derive(Debug, Deserialize)]
struct ClosingQuery {
    #[serde(default)]
    kind: String,
    #[serde(default)]
    month: String,
}

pub fn preview_closing(req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let query: ClosingQuery = match req.query() {
        Ok(query) => query,
        Err(_) => return api_error(400, "クエリパラメータが不正です。"),
    };
    match store::with_store(|store| service::closing_rows(store, &query.kind, &query.month)) {
        Ok(rows) => Response::from_json(&serde_json::json!({
            "kind": query.kind,
            "month": query.month,
            "rows": rows,
        })),
        Err((status, message)) => api_error(status, &message),
    }
}

pub fn list_closings(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    store::with_store(|store| {
        let mut rows = store.closings.clone();
        rows.sort_by(|a, b| {
            b.month
                .cmp(&a.month)
                .then_with(|| a.kind.cmp(&b.kind))
                .then_with(|| a.party_code.cmp(&b.party_code))
        });
        Response::from_json(&rows)
    })
}

pub async fn run_closing(mut req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let input: ClosingInput = match req.json().await {
        Ok(input) => input,
        Err(_) => return api_error(400, "リクエストの JSON が不正です。"),
    };
    match store::with_store(|store| service::run_closing_inner(store, input)) {
        Ok(rows) => Response::from_json(&rows).map(|response| response.with_status(201)),
        Err((status, message)) => api_error(status, &message),
    }
}
