use std::collections::{BTreeMap, HashMap, HashSet};

use serde::Serialize;

use crate::models::*;

#[derive(Clone)]
pub struct SummaryDoc {
    pub id: u64,
    pub date: String,
    pub customer_code: String,
    pub customer_name: String,
    pub lines: Vec<LineItem>,
    pub subtotal: i64,
    pub tax: i64,
    pub total: i64,
}

impl From<&Order> for SummaryDoc {
    fn from(order: &Order) -> Self {
        Self {
            id: order.id,
            date: order.date.clone(),
            customer_code: order.customer_code.clone(),
            customer_name: order.customer_name.clone(),
            lines: order.lines.clone(),
            subtotal: order.subtotal,
            tax: order.tax,
            total: order.total,
        }
    }
}

impl From<&Sale> for SummaryDoc {
    fn from(sale: &Sale) -> Self {
        Self {
            id: sale.id,
            date: sale.date.clone(),
            customer_code: sale.customer_code.clone(),
            customer_name: sale.customer_name.clone(),
            lines: sale.lines.clone(),
            subtotal: sale.subtotal,
            tax: sale.tax,
            total: sale.total,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SummaryRow {
    pub key: String,
    pub label: String,
    pub orders: u64,
    pub quantity: f64,
    pub subtotal: i64,
    pub tax: i64,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct Summary {
    pub source: String,
    pub title: String,
    pub from: String,
    pub to: String,
    pub group: String,
    pub group_label: String,
    pub rows: Vec<SummaryRow>,
    pub totals: SummaryRow,
}

pub fn summarize_orders(
    orders: &[Order],
    from: &str,
    to: &str,
    group: &str,
) -> Result<Summary, String> {
    let docs: Vec<SummaryDoc> = orders.iter().map(SummaryDoc::from).collect();
    summarize_docs(&docs, from, to, group, "orders", "受注集計表")
}

pub fn summarize_sales(
    sales: &[Sale],
    from: &str,
    to: &str,
    group: &str,
) -> Result<Summary, String> {
    let docs: Vec<SummaryDoc> = sales.iter().map(SummaryDoc::from).collect();
    summarize_docs(&docs, from, to, group, "sales", "売上集計表")
}

fn summarize_docs(
    docs: &[SummaryDoc],
    from: &str,
    to: &str,
    group: &str,
    source: &str,
    title: &str,
) -> Result<Summary, String> {
    if !matches!(group, "customer" | "product" | "month") {
        return Err("集計区分が不正です。".to_string());
    }
    if !from.is_empty() && !to.is_empty() && from > to {
        return Err("開始日は終了日以前にしてください。".to_string());
    }

    let filtered: Vec<&SummaryDoc> = docs
        .iter()
        .filter(|doc| from.is_empty() || doc.date.as_str() >= from)
        .filter(|doc| to.is_empty() || doc.date.as_str() <= to)
        .collect();

    let mut rows: Vec<SummaryRow> = match group {
        "customer" => summarize_parties(&filtered, false),
        "month" => summarize_parties(&filtered, true),
        _ => summarize_products(&filtered),
    };

    rows.sort_by(|a, b| b.total.cmp(&a.total).then_with(|| a.label.cmp(&b.label)));

    let totals = SummaryRow {
        key: "total".to_string(),
        label: "合計".to_string(),
        orders: filtered.len() as u64,
        quantity: rows.iter().map(|row| row.quantity).sum(),
        subtotal: rows.iter().map(|row| row.subtotal).sum(),
        tax: rows.iter().map(|row| row.tax).sum(),
        total: rows.iter().map(|row| row.total).sum(),
    };

    let group_label = match group {
        "customer" => "得意先別",
        "month" => "月別",
        _ => "商品別",
    };

    Ok(Summary {
        source: source.to_string(),
        title: title.to_string(),
        from: from.to_string(),
        to: to.to_string(),
        group: group.to_string(),
        group_label: group_label.to_string(),
        rows,
        totals,
    })
}

fn summarize_parties(docs: &[&SummaryDoc], by_month: bool) -> Vec<SummaryRow> {
    let mut map: BTreeMap<String, SummaryRow> = BTreeMap::new();

    for doc in docs {
        let (key, label) = if by_month {
            let key = doc.date.get(0..7).unwrap_or("unknown").to_string();
            let label = month_label(&key);
            (key, label)
        } else {
            (doc.customer_code.clone(), doc.customer_name.clone())
        };

        let row = map.entry(key.clone()).or_insert_with(|| SummaryRow {
            key: key.clone(),
            label,
            orders: 0,
            quantity: 0.0,
            subtotal: 0,
            tax: 0,
            total: 0,
        });
        row.orders += 1;
        row.quantity += doc.lines.iter().map(|line| line.quantity).sum::<f64>();
        row.subtotal += doc.subtotal;
        row.tax += doc.tax;
        row.total += doc.total;
    }

    map.into_values().collect()
}

fn summarize_products(docs: &[&SummaryDoc]) -> Vec<SummaryRow> {
    let mut map: HashMap<String, SummaryRow> = HashMap::new();
    let mut order_ids: HashMap<String, HashSet<u64>> = HashMap::new();

    for doc in docs {
        for line in &doc.lines {
            let key = if line.product_code.is_empty() {
                line.name.clone()
            } else {
                line.product_code.clone()
            };
            let line_tax = if line.taxable {
                round_yen(line.amount as f64 * TAX_RATE)
            } else {
                0
            };

            let row = map.entry(key.clone()).or_insert_with(|| SummaryRow {
                key: key.clone(),
                label: line.name.clone(),
                orders: 0,
                quantity: 0.0,
                subtotal: 0,
                tax: 0,
                total: 0,
            });
            row.quantity += line.quantity;
            row.subtotal += line.amount;
            row.tax += line_tax;
            row.total += line.amount + line_tax;
            order_ids.entry(key).or_default().insert(doc.id);
        }
    }

    for (key, row) in map.iter_mut() {
        if let Some(ids) = order_ids.get(key) {
            row.orders = ids.len() as u64;
        }
    }

    map.into_values().collect()
}

fn month_label(key: &str) -> String {
    let parts: Vec<&str> = key.split('-').collect();
    if parts.len() >= 2 {
        let month = parts[1].parse::<u32>().unwrap_or(0);
        format!("{}年{}月", parts[0], month)
    } else {
        key.to_string()
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct RemainingRow {
    pub document: String,
    pub party: String,
    pub product: String,
    pub quantity: f64,
    pub amount: i64,
}

#[derive(Debug, Serialize)]
pub struct RemainingReport {
    pub title: String,
    pub rows: Vec<RemainingRow>,
    pub total_quantity: f64,
    pub total_amount: i64,
}

pub fn order_remaining(orders: &[Order]) -> RemainingReport {
    let mut rows = Vec::new();
    for order in orders {
        for line in &order.lines {
            let remaining = line.quantity - line.sold_quantity;
            if remaining <= 0.0001 {
                continue;
            }
            rows.push(RemainingRow {
                document: order.number.clone(),
                party: order.customer_name.clone(),
                product: line.name.clone(),
                quantity: remaining,
                amount: round_yen(remaining * line.unit_price as f64),
            });
        }
    }
    rows.sort_by(|a, b| {
        b.document
            .cmp(&a.document)
            .then_with(|| a.product.cmp(&b.product))
    });
    remaining_report("受注残一覧", rows)
}

pub fn purchase_remaining(purchase_orders: &[PurchaseOrder]) -> RemainingReport {
    let mut rows = Vec::new();
    for order in purchase_orders {
        if order.status != "open" {
            continue;
        }
        for line in &order.lines {
            rows.push(RemainingRow {
                document: order.number.clone(),
                party: order.supplier_name.clone(),
                product: line.name.clone(),
                quantity: line.quantity,
                amount: line.amount,
            });
        }
    }
    rows.sort_by(|a, b| {
        b.document
            .cmp(&a.document)
            .then_with(|| a.product.cmp(&b.product))
    });
    remaining_report("発注残一覧", rows)
}

fn remaining_report(title: &str, rows: Vec<RemainingRow>) -> RemainingReport {
    RemainingReport {
        title: title.to_string(),
        total_quantity: rows.iter().map(|row| row.quantity).sum(),
        total_amount: rows.iter().map(|row| row.amount).sum(),
        rows,
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct BalanceRow {
    pub code: String,
    pub name: String,
    pub balance: i64,
}

#[derive(Debug, Serialize)]
pub struct BalanceReport {
    pub customers: Vec<BalanceRow>,
    pub suppliers: Vec<BalanceRow>,
    pub customer_total: i64,
    pub supplier_total: i64,
}

#[derive(Debug, Serialize)]
pub struct DashboardKpi {
    pub sales_month: i64,
    pub sales_prev: i64,
    pub sales_count: u64,
    pub receipts_month: i64,
    pub receipt_count: u64,
    pub receivable: i64,
    pub order_remaining: i64,
    pub order_remaining_count: u64,
    pub purchases_month: i64,
    pub purchase_count: u64,
    pub payable: i64,
    pub pending_quotes: u64,
    pub open_purchase_orders: u64,
}

#[derive(Debug, Serialize)]
pub struct DashboardMonth {
    pub month: String,
    pub label: String,
    pub sales: i64,
    pub purchases: i64,
}

#[derive(Debug, Serialize)]
pub struct DashboardRank {
    pub code: String,
    pub name: String,
    pub total: i64,
    pub count: u64,
    pub share: f64,
}

#[derive(Debug, Serialize)]
pub struct DashboardStatus {
    pub key: String,
    pub label: String,
    pub count: u64,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct DashboardActivity {
    pub kind: String,
    pub label: String,
    pub date: String,
    pub number: String,
    pub party: String,
    pub amount: i64,
    pub href: String,
}

#[derive(Debug, Serialize)]
pub struct Dashboard {
    pub as_of: String,
    pub month: String,
    pub month_label: String,
    pub kpis: DashboardKpi,
    pub months: Vec<DashboardMonth>,
    pub top_customers: Vec<DashboardRank>,
    pub top_products: Vec<DashboardRank>,
    pub quote_status: Vec<DashboardStatus>,
    pub order_status: Vec<DashboardStatus>,
    pub activities: Vec<DashboardActivity>,
}

/// トップページ用の集計。売上・入金・仕入・残高・順位・直近の動きを 1 回で返す。
#[allow(clippy::too_many_arguments)]
pub fn dashboard(
    quotes: &[Quote],
    orders: &[Order],
    sales: &[Sale],
    purchase_orders: &[PurchaseOrder],
    purchases: &[Purchase],
    receipts: &[Receipt],
    supplier_payments: &[SupplierPayment],
) -> Dashboard {
    let as_of = today();
    let month = as_of.get(0..7).unwrap_or_default().to_string();
    let prev_month = previous_month(&month);
    let day_of_month = as_of.get(8..10).unwrap_or("31");

    let sales_month = sales
        .iter()
        .filter(|sale| sale.date.starts_with(month.as_str()))
        .map(|sale| sale.total)
        .sum::<i64>();
    // 月初の「前月比」が過度に悪化しないよう、前月は同じ日付までで比較する。
    let sales_prev = sales
        .iter()
        .filter(|sale| {
            sale.date.starts_with(prev_month.as_str())
                && sale.date.get(8..10).is_none_or(|day| day <= day_of_month)
        })
        .map(|sale| sale.total)
        .sum::<i64>();
    let sales_count = sales
        .iter()
        .filter(|sale| sale.date.starts_with(month.as_str()))
        .count() as u64;
    let receipts_month = receipts
        .iter()
        .filter(|receipt| receipt.date.starts_with(month.as_str()))
        .map(|receipt| receipt.amount)
        .sum::<i64>();
    let receipt_count = receipts
        .iter()
        .filter(|receipt| receipt.date.starts_with(month.as_str()))
        .count() as u64;
    let purchases_month = purchases
        .iter()
        .filter(|purchase| purchase.date.starts_with(month.as_str()))
        .map(|purchase| purchase.total)
        .sum::<i64>();
    let purchase_count = purchases
        .iter()
        .filter(|purchase| purchase.date.starts_with(month.as_str()))
        .count() as u64;

    let receivable = sales.iter().map(|sale| sale.total).sum::<i64>()
        - receipts.iter().map(|receipt| receipt.amount).sum::<i64>();
    let payable = purchases.iter().map(|purchase| purchase.total).sum::<i64>()
        - supplier_payments
            .iter()
            .map(|payment| payment.amount)
            .sum::<i64>();

    let mut order_remaining = 0i64;
    let mut order_remaining_count = 0u64;
    for order in orders {
        let mut has_remaining = false;
        for line in &order.lines {
            let remaining = line.quantity - line.sold_quantity;
            if remaining <= 0.0001 {
                continue;
            }
            let amount = round_yen(remaining * line.unit_price as f64);
            let tax = if line.taxable {
                round_yen(amount as f64 * TAX_RATE)
            } else {
                0
            };
            order_remaining += amount + tax;
            has_remaining = true;
        }
        if has_remaining {
            order_remaining_count += 1;
        }
    }

    let pending_quotes = quotes
        .iter()
        .filter(|quote| quote.status == "draft" || quote.status == "sent")
        .count() as u64;
    let open_purchase_orders = purchase_orders
        .iter()
        .filter(|order| order.status == "open")
        .count() as u64;

    let months = recent_months(12)
        .into_iter()
        .map(|key| {
            let sales_total = sales
                .iter()
                .filter(|sale| sale.date.starts_with(key.as_str()))
                .map(|sale| sale.total)
                .sum::<i64>();
            let purchase_total = purchases
                .iter()
                .filter(|purchase| purchase.date.starts_with(key.as_str()))
                .map(|purchase| purchase.total)
                .sum::<i64>();
            DashboardMonth {
                label: short_month_label(&key),
                month: key,
                sales: sales_total,
                purchases: purchase_total,
            }
        })
        .collect();

    Dashboard {
        month_label: month_label(&month),
        as_of,
        month,
        kpis: DashboardKpi {
            sales_month,
            sales_prev,
            sales_count,
            receipts_month,
            receipt_count,
            receivable,
            order_remaining,
            order_remaining_count,
            purchases_month,
            purchase_count,
            payable,
            pending_quotes,
            open_purchase_orders,
        },
        months,
        top_customers: top_customers(sales),
        top_products: top_products(sales),
        quote_status: quote_statuses(quotes),
        order_status: order_statuses(orders),
        activities: recent_activities(sales, receipts, purchase_orders, purchases),
    }
}

fn previous_month(key: &str) -> String {
    let year = key
        .get(0..4)
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(2000);
    let month = key
        .get(5..7)
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(1);
    if month <= 1 {
        format!("{:04}-12", year - 1)
    } else {
        format!("{year:04}-{:02}", month - 1)
    }
}

fn recent_months(count: usize) -> Vec<String> {
    let as_of = today();
    let mut year = as_of
        .get(0..4)
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(2000);
    let mut month = as_of
        .get(5..7)
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(1);
    let mut keys = Vec::with_capacity(count);
    for _ in 0..count {
        keys.push(format!("{year:04}-{month:02}"));
        month -= 1;
        if month == 0 {
            month = 12;
            year -= 1;
        }
    }
    keys.reverse();
    keys
}

fn short_month_label(key: &str) -> String {
    let month = key
        .get(5..7)
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);
    format!("{month}月")
}

fn top_customers(sales: &[Sale]) -> Vec<DashboardRank> {
    let mut map: HashMap<String, (String, i64, u64)> = HashMap::new();
    for sale in sales {
        let entry = map
            .entry(sale.customer_code.clone())
            .or_insert_with(|| (sale.customer_name.clone(), 0, 0));
        entry.1 += sale.total;
        entry.2 += 1;
    }
    let grand = sales.iter().map(|sale| sale.total).sum::<i64>();
    let mut rows: Vec<DashboardRank> = map
        .into_iter()
        .map(|(code, (name, total, count))| DashboardRank {
            code,
            name,
            total,
            count,
            share: if grand > 0 {
                total as f64 / grand as f64
            } else {
                0.0
            },
        })
        .collect();
    rows.sort_by(|a, b| b.total.cmp(&a.total).then_with(|| a.name.cmp(&b.name)));
    rows.truncate(5);
    rows
}

fn top_products(sales: &[Sale]) -> Vec<DashboardRank> {
    let mut map: HashMap<String, (String, i64, HashSet<u64>)> = HashMap::new();
    for sale in sales {
        for line in &sale.lines {
            let key = if line.product_code.is_empty() {
                line.name.clone()
            } else {
                line.product_code.clone()
            };
            let tax = if line.taxable {
                round_yen(line.amount as f64 * TAX_RATE)
            } else {
                0
            };
            let entry = map
                .entry(key)
                .or_insert_with(|| (line.name.clone(), 0, HashSet::new()));
            entry.1 += line.amount + tax;
            entry.2.insert(sale.id);
        }
    }
    let grand = map.values().map(|(_, total, _)| *total).sum::<i64>();
    let mut rows: Vec<DashboardRank> = map
        .into_iter()
        .map(|(code, (name, total, docs))| DashboardRank {
            code,
            name,
            total,
            count: docs.len() as u64,
            share: if grand > 0 {
                total as f64 / grand as f64
            } else {
                0.0
            },
        })
        .collect();
    rows.sort_by(|a, b| b.total.cmp(&a.total).then_with(|| a.name.cmp(&b.name)));
    rows.truncate(5);
    rows
}

fn quote_statuses(quotes: &[Quote]) -> Vec<DashboardStatus> {
    [
        ("draft", "下書き"),
        ("sent", "提出済"),
        ("accepted", "受注済"),
    ]
    .into_iter()
    .map(|(key, label)| {
        let docs: Vec<&Quote> = quotes.iter().filter(|quote| quote.status == key).collect();
        DashboardStatus {
            key: key.to_string(),
            label: label.to_string(),
            count: docs.len() as u64,
            total: docs.iter().map(|quote| quote.total).sum(),
        }
    })
    .collect()
}

fn order_statuses(orders: &[Order]) -> Vec<DashboardStatus> {
    [("open", "未完了"), ("completed", "完了")]
        .into_iter()
        .map(|(key, label)| {
            let docs: Vec<&Order> = orders.iter().filter(|order| order.status == key).collect();
            DashboardStatus {
                key: key.to_string(),
                label: label.to_string(),
                count: docs.len() as u64,
                total: docs.iter().map(|order| order.total).sum(),
            }
        })
        .collect()
}

fn recent_activities(
    sales: &[Sale],
    receipts: &[Receipt],
    purchase_orders: &[PurchaseOrder],
    purchases: &[Purchase],
) -> Vec<DashboardActivity> {
    let mut rows = Vec::new();
    for sale in sales {
        rows.push(DashboardActivity {
            kind: "sale".to_string(),
            label: "売上".to_string(),
            date: sale.date.clone(),
            number: sale.number.clone(),
            party: sale.customer_name.clone(),
            amount: sale.total,
            href: "#/sales".to_string(),
        });
    }
    for receipt in receipts {
        rows.push(DashboardActivity {
            kind: "receipt".to_string(),
            label: "入金".to_string(),
            date: receipt.date.clone(),
            number: receipt.number.clone(),
            party: receipt.customer_name.clone(),
            amount: receipt.amount,
            href: "#/payments".to_string(),
        });
    }
    for order in purchase_orders {
        rows.push(DashboardActivity {
            kind: "purchase_order".to_string(),
            label: "発注".to_string(),
            date: order.date.clone(),
            number: order.number.clone(),
            party: order.supplier_name.clone(),
            amount: order.total,
            href: "#/purchase-orders".to_string(),
        });
    }
    for purchase in purchases {
        rows.push(DashboardActivity {
            kind: "purchase".to_string(),
            label: "仕入".to_string(),
            date: purchase.date.clone(),
            number: purchase.number.clone(),
            party: purchase.supplier_name.clone(),
            amount: purchase.total,
            href: "#/purchases".to_string(),
        });
    }
    rows.sort_by(|a, b| b.date.cmp(&a.date).then_with(|| b.number.cmp(&a.number)));
    rows.truncate(8);
    rows
}

pub fn balances(
    customers: &[Customer],
    suppliers: &[Supplier],
    sales: &[Sale],
    receipts: &[Receipt],
    purchases: &[Purchase],
    supplier_payments: &[SupplierPayment],
) -> BalanceReport {
    let customer_rows = customers
        .iter()
        .map(|customer| {
            let sales_total: i64 = sales
                .iter()
                .filter(|sale| sale.customer_code == customer.code)
                .map(|sale| sale.total)
                .sum();
            let paid: i64 = receipts
                .iter()
                .filter(|receipt| receipt.customer_code == customer.code)
                .map(|receipt| receipt.amount)
                .sum();
            BalanceRow {
                code: customer.code.clone(),
                name: customer.name.clone(),
                balance: sales_total - paid,
            }
        })
        .collect::<Vec<_>>();

    let supplier_rows = suppliers
        .iter()
        .map(|supplier| {
            let purchase_total: i64 = purchases
                .iter()
                .filter(|purchase| purchase.supplier_code == supplier.code)
                .map(|purchase| purchase.total)
                .sum();
            let paid: i64 = supplier_payments
                .iter()
                .filter(|payment| payment.supplier_code == supplier.code)
                .map(|payment| payment.amount)
                .sum();
            BalanceRow {
                code: supplier.code.clone(),
                name: supplier.name.clone(),
                balance: purchase_total - paid,
            }
        })
        .collect::<Vec<_>>();

    BalanceReport {
        customer_total: customer_rows.iter().map(|row| row.balance).sum(),
        supplier_total: supplier_rows.iter().map(|row| row.balance).sum(),
        customers: customer_rows,
        suppliers: supplier_rows,
    }
}
