use std::collections::HashSet;

use crate::models::*;
use crate::store::{self, Store};

fn ensure_capacity(len: usize) -> Result<(), (u16, String)> {
    if len >= store::MAX_RECORDS {
        return Err((
            429,
            format!(
                "デモデータの登録上限（{} 件）に達しました。",
                store::MAX_RECORDS
            ),
        ));
    }
    Ok(())
}

fn clean(value: Option<String>) -> String {
    value.unwrap_or_default().trim().to_string()
}

fn party_snapshot(customer: &Customer) -> (String, String, String) {
    (
        customer.name.clone(),
        customer.name2.clone(),
        customer.honorific.clone(),
    )
}

pub(crate) fn create_sale_inner(
    store: &mut Store,
    order_id: u64,
    input: SaleInput,
) -> Result<Sale, (u16, String)> {
    ensure_capacity(store.sales.len())?;
    let order = store
        .order(order_id)
        .cloned()
        .ok_or((404, "受注が見つかりません。".to_string()))?;
    if input.lines.is_empty() {
        return Err((400, "売上計上する明細を選択してください。".to_string()));
    }
    let date = normalize_date(&input.date).map_err(|message| (400, message))?;

    let mut seen = HashSet::new();
    let mut sale_lines = Vec::new();
    for flow in &input.lines {
        if !seen.insert(flow.line_id) {
            return Err((400, "同じ明細が重複しています。".to_string()));
        }
        let line = order
            .lines
            .iter()
            .find(|line| line.id == flow.line_id)
            .ok_or((400, "受注明細が見つかりません。".to_string()))?;
        let remaining = line.quantity - line.sold_quantity;
        if !flow.quantity.is_finite() || flow.quantity <= 0.0 {
            return Err((400, "数量は 0 より大きい値を入力してください。".to_string()));
        }
        if flow.quantity > remaining + 0.0001 {
            return Err((400, format!("{} の数量が受注残を超えています。", line.name)));
        }

        let line_id = store.next_line_id;
        store.next_line_id += 1;
        sale_lines.push(LineItem {
            id: line_id,
            quantity: flow.quantity,
            amount: round_yen(flow.quantity * line.unit_price as f64),
            source_line_id: Some(line.id),
            sold_quantity: 0.0,
            purchased_quantity: 0.0,
            ..line.clone()
        });
    }

    let (subtotal, tax, total) = totals(&sale_lines);
    let id = store.next_sale_id;
    store.next_sale_id += 1;
    let sale = Sale {
        id,
        number: format!("S-{id:04}"),
        date,
        order_id: Some(order.id),
        order_number: order.number.clone(),
        quote_number: order.quote_number.clone(),
        customer_code: order.customer_code.clone(),
        customer_name: order.customer_name.clone(),
        customer_name2: order.customer_name2.clone(),
        honorific: order.honorific.clone(),
        subject: order.subject.clone(),
        notes: clean(input.notes),
        lines: sale_lines,
        subtotal,
        tax,
        total,
    };
    store.sales.push(sale.clone());

    if let Some(order) = store.order_mut(order_id) {
        for line in &sale.lines {
            if let Some(source_id) = line.source_line_id {
                if let Some(order_line) = order.lines.iter_mut().find(|item| item.id == source_id) {
                    order_line.sold_quantity += line.quantity;
                }
            }
        }
        let complete = order
            .lines
            .iter()
            .all(|line| line.sold_quantity + 0.0001 >= line.quantity);
        if complete {
            order.status = "completed".to_string();
        }
    }

    Ok(sale)
}

pub(crate) fn delete_sale_inner(store: &mut Store, id: u64) -> Result<(), (u16, String)> {
    let Some(index) = store.sales.iter().position(|sale| sale.id == id) else {
        return Err((404, "売上が見つかりません。".to_string()));
    };
    let sale = store.sales.remove(index);
    if let Some(order_id) = sale.order_id {
        if let Some(order) = store.order_mut(order_id) {
            for line in &sale.lines {
                if let Some(source_id) = line.source_line_id {
                    if let Some(order_line) =
                        order.lines.iter_mut().find(|item| item.id == source_id)
                    {
                        order_line.sold_quantity =
                            (order_line.sold_quantity - line.quantity).max(0.0);
                    }
                }
            }
            let complete = order
                .lines
                .iter()
                .all(|line| line.sold_quantity + 0.0001 >= line.quantity);
            order.status = if complete { "completed" } else { "open" }.to_string();
        }
    }
    Ok(())
}

pub(crate) fn create_purchase_order_inner(
    store: &mut Store,
    order_id: u64,
    input: PurchaseOrderInput,
) -> Result<PurchaseOrder, (u16, String)> {
    ensure_capacity(store.purchase_orders.len())?;
    let order = store
        .order(order_id)
        .cloned()
        .ok_or((404, "受注が見つかりません。".to_string()))?;
    let supplier = store
        .supplier(input.supplier_code.trim())
        .cloned()
        .ok_or((400, "仕入先を選択してください。".to_string()))?;
    if input.lines.is_empty() {
        return Err((400, "発注する明細を選択してください。".to_string()));
    }
    let date = normalize_date(&input.date).map_err(|message| (400, message))?;

    let mut seen = HashSet::new();
    let mut lines = Vec::new();
    for flow in &input.lines {
        if !seen.insert(flow.line_id) {
            return Err((400, "同じ明細が重複しています。".to_string()));
        }
        let line = order
            .lines
            .iter()
            .find(|line| line.id == flow.line_id)
            .ok_or((400, "受注明細が見つかりません。".to_string()))?;
        let remaining = line.quantity - line.purchased_quantity;
        if !flow.quantity.is_finite() || flow.quantity <= 0.0 {
            return Err((400, "数量は 0 より大きい値を入力してください。".to_string()));
        }
        if flow.quantity > remaining + 0.0001 {
            return Err((400, format!("{} の数量が発注残を超えています。", line.name)));
        }
        let cost_price = store
            .product(&line.product_code)
            .map(|product| product.cost_price)
            .unwrap_or(line.unit_price);
        let line_id = store.next_line_id;
        store.next_line_id += 1;
        lines.push(LineItem {
            id: line_id,
            quantity: flow.quantity,
            unit_price: cost_price,
            amount: round_yen(flow.quantity * cost_price as f64),
            source_line_id: Some(line.id),
            sold_quantity: 0.0,
            purchased_quantity: 0.0,
            ..line.clone()
        });
    }

    let (subtotal, tax, total) = totals(&lines);
    let id = store.next_purchase_order_id;
    store.next_purchase_order_id += 1;
    let purchase_order = PurchaseOrder {
        id,
        number: format!("PO-{id:04}"),
        date,
        order_id: Some(order.id),
        order_number: order.number.clone(),
        supplier_code: supplier.code.clone(),
        supplier_name: supplier.name.clone(),
        supplier_honorific: supplier.honorific.clone(),
        status: "open".to_string(),
        notes: clean(input.notes),
        lines,
        subtotal,
        tax,
        total,
    };
    store.purchase_orders.push(purchase_order.clone());

    if let Some(order) = store.order_mut(order_id) {
        for line in &purchase_order.lines {
            if let Some(source_id) = line.source_line_id {
                if let Some(order_line) = order.lines.iter_mut().find(|item| item.id == source_id) {
                    order_line.purchased_quantity += line.quantity;
                }
            }
        }
    }

    Ok(purchase_order)
}

pub(crate) fn receive_purchase_order_inner(
    store: &mut Store,
    id: u64,
    input: ReceiveInput,
) -> Result<Purchase, (u16, String)> {
    ensure_capacity(store.purchases.len())?;
    let purchase_order = store
        .purchase_order(id)
        .cloned()
        .ok_or((404, "発注が見つかりません。".to_string()))?;
    if purchase_order.status == "received" {
        return Err((409, "この発注はすでに仕入計上済みです。".to_string()));
    }
    let date = normalize_date(&input.date).map_err(|message| (400, message))?;

    let mut lines = Vec::with_capacity(purchase_order.lines.len());
    for line in &purchase_order.lines {
        let line_id = store.next_line_id;
        store.next_line_id += 1;
        lines.push(LineItem {
            id: line_id,
            source_line_id: Some(line.id),
            sold_quantity: 0.0,
            purchased_quantity: 0.0,
            ..line.clone()
        });
    }
    let (subtotal, tax, total) = totals(&lines);
    let purchase_id = store.next_purchase_id;
    store.next_purchase_id += 1;
    let purchase = Purchase {
        id: purchase_id,
        number: format!("PI-{purchase_id:04}"),
        date,
        purchase_order_id: Some(purchase_order.id),
        purchase_order_number: purchase_order.number.clone(),
        supplier_code: purchase_order.supplier_code.clone(),
        supplier_name: purchase_order.supplier_name.clone(),
        supplier_honorific: purchase_order.supplier_honorific.clone(),
        notes: purchase_order.notes.clone(),
        lines,
        subtotal,
        tax,
        total,
    };
    store.purchases.push(purchase.clone());
    if let Some(order) = store.purchase_order_mut(id) {
        order.status = "received".to_string();
    }
    Ok(purchase)
}

pub(crate) fn delete_purchase_order_inner(store: &mut Store, id: u64) -> Result<(), (u16, String)> {
    let Some(index) = store
        .purchase_orders
        .iter()
        .position(|order| order.id == id)
    else {
        return Err((404, "発注が見つかりません。".to_string()));
    };
    if store.purchase_orders[index].status == "received" {
        return Err((409, "仕入計上済みの発注は削除できません。".to_string()));
    }
    let purchase_order = store.purchase_orders.remove(index);
    if let Some(order_id) = purchase_order.order_id {
        if let Some(order) = store.order_mut(order_id) {
            for line in &purchase_order.lines {
                if let Some(source_id) = line.source_line_id {
                    if let Some(order_line) =
                        order.lines.iter_mut().find(|item| item.id == source_id)
                    {
                        order_line.purchased_quantity =
                            (order_line.purchased_quantity - line.quantity).max(0.0);
                    }
                }
            }
        }
    }
    Ok(())
}

pub(crate) fn create_receipt_inner(
    store: &mut Store,
    input: ReceiptInput,
) -> Result<Receipt, (u16, String)> {
    ensure_capacity(store.receipts.len())?;
    let customer = store
        .customer(input.customer_code.trim())
        .cloned()
        .ok_or((400, "得意先を選択してください。".to_string()))?;
    if input.amount <= 0 {
        return Err((
            400,
            "入金額は 0 より大きい値を入力してください。".to_string(),
        ));
    }
    let date = normalize_date(&input.date).map_err(|message| (400, message))?;
    let id = store.next_receipt_id;
    store.next_receipt_id += 1;
    let receipt = Receipt {
        id,
        number: format!("R-{id:04}"),
        date,
        customer_code: customer.code,
        customer_name: customer.name,
        amount: input.amount,
        method: input.method.unwrap_or_else(|| "振込".to_string()),
        notes: clean(input.notes),
    };
    store.receipts.push(receipt.clone());
    Ok(receipt)
}

pub(crate) fn create_supplier_payment_inner(
    store: &mut Store,
    input: SupplierPaymentInput,
) -> Result<SupplierPayment, (u16, String)> {
    ensure_capacity(store.supplier_payments.len())?;
    let supplier = store
        .supplier(input.supplier_code.trim())
        .cloned()
        .ok_or((400, "仕入先を選択してください。".to_string()))?;
    if input.amount <= 0 {
        return Err((
            400,
            "支払額は 0 より大きい値を入力してください。".to_string(),
        ));
    }
    let date = normalize_date(&input.date).map_err(|message| (400, message))?;
    let id = store.next_supplier_payment_id;
    store.next_supplier_payment_id += 1;
    let payment = SupplierPayment {
        id,
        number: format!("D-{id:04}"),
        date,
        supplier_code: supplier.code,
        supplier_name: supplier.name,
        amount: input.amount,
        method: input.method.unwrap_or_else(|| "振込".to_string()),
        notes: clean(input.notes),
    };
    store.supplier_payments.push(payment.clone());
    Ok(payment)
}

pub(crate) fn validate_closing_month(month: &str) -> Result<(), (u16, String)> {
    let bytes = month.as_bytes();
    if bytes.len() != 7
        || bytes[4] != b'-'
        || !bytes
            .iter()
            .enumerate()
            .all(|(i, b)| i == 4 || b.is_ascii_digit())
    {
        return Err((400, "対象月は YYYY-MM 形式で指定してください。".to_string()));
    }
    let year: u32 = month[0..4]
        .parse()
        .map_err(|_| (400, "対象月が不正です。".to_string()))?;
    let month_number: u32 = month[5..7]
        .parse()
        .map_err(|_| (400, "対象月が不正です。".to_string()))?;
    if !(1..=9999).contains(&year) || !(1..=12).contains(&month_number) {
        return Err((400, "対象月が不正です。".to_string()));
    }
    Ok(())
}

pub(crate) fn closing_rows(
    store: &Store,
    kind: &str,
    month: &str,
) -> Result<Vec<ClosingPreviewRow>, (u16, String)> {
    validate_closing_month(month)?;
    let start = format!("{month}-01");

    match kind {
        "customer" => Ok(store
            .customers
            .iter()
            .map(|customer| {
                let sales_total: i64 = store
                    .sales
                    .iter()
                    .filter(|sale| {
                        sale.customer_code == customer.code && sale.date.starts_with(month)
                    })
                    .map(|sale| sale.total)
                    .sum();
                let tax: i64 = store
                    .sales
                    .iter()
                    .filter(|sale| {
                        sale.customer_code == customer.code && sale.date.starts_with(month)
                    })
                    .map(|sale| sale.tax)
                    .sum();
                let payments: i64 = store
                    .receipts
                    .iter()
                    .filter(|receipt| {
                        receipt.customer_code == customer.code && receipt.date.starts_with(month)
                    })
                    .map(|receipt| receipt.amount)
                    .sum();
                let previous_balance: i64 = store
                    .sales
                    .iter()
                    .filter(|sale| {
                        sale.customer_code == customer.code && sale.date.as_str() < start.as_str()
                    })
                    .map(|sale| sale.total)
                    .sum::<i64>()
                    - store
                        .receipts
                        .iter()
                        .filter(|receipt| {
                            receipt.customer_code == customer.code
                                && receipt.date.as_str() < start.as_str()
                        })
                        .map(|receipt| receipt.amount)
                        .sum::<i64>();

                ClosingPreviewRow {
                    kind: kind.to_string(),
                    party_code: customer.code.clone(),
                    party_name: customer.name.clone(),
                    month: month.to_string(),
                    sales_total,
                    tax,
                    payments,
                    previous_balance,
                    balance: previous_balance + sales_total - payments,
                }
            })
            .collect()),
        "supplier" => Ok(store
            .suppliers
            .iter()
            .map(|supplier| {
                let purchase_total: i64 = store
                    .purchases
                    .iter()
                    .filter(|purchase| {
                        purchase.supplier_code == supplier.code && purchase.date.starts_with(month)
                    })
                    .map(|purchase| purchase.total)
                    .sum();
                let tax: i64 = store
                    .purchases
                    .iter()
                    .filter(|purchase| {
                        purchase.supplier_code == supplier.code && purchase.date.starts_with(month)
                    })
                    .map(|purchase| purchase.tax)
                    .sum();
                let payments: i64 = store
                    .supplier_payments
                    .iter()
                    .filter(|payment| {
                        payment.supplier_code == supplier.code && payment.date.starts_with(month)
                    })
                    .map(|payment| payment.amount)
                    .sum();
                let previous_balance: i64 = store
                    .purchases
                    .iter()
                    .filter(|purchase| {
                        purchase.supplier_code == supplier.code
                            && purchase.date.as_str() < start.as_str()
                    })
                    .map(|purchase| purchase.total)
                    .sum::<i64>()
                    - store
                        .supplier_payments
                        .iter()
                        .filter(|payment| {
                            payment.supplier_code == supplier.code
                                && payment.date.as_str() < start.as_str()
                        })
                        .map(|payment| payment.amount)
                        .sum::<i64>();

                ClosingPreviewRow {
                    kind: kind.to_string(),
                    party_code: supplier.code.clone(),
                    party_name: supplier.name.clone(),
                    month: month.to_string(),
                    sales_total: purchase_total,
                    tax,
                    payments,
                    previous_balance,
                    balance: previous_balance + purchase_total - payments,
                }
            })
            .collect()),
        _ => Err((400, "締処理区分が不正です。".to_string())),
    }
}

pub(crate) fn run_closing_inner(
    store: &mut Store,
    input: ClosingInput,
) -> Result<Vec<ClosingRecord>, (u16, String)> {
    ensure_capacity(store.closings.len())?;
    let rows = closing_rows(store, &input.kind, &input.month)?;
    if store
        .closings
        .iter()
        .any(|record| record.kind == input.kind && record.month == input.month)
    {
        return Err((409, "この対象月はすでに締処理済みです。".to_string()));
    }

    let mut created = Vec::new();
    for row in rows {
        if row.sales_total == 0 && row.payments == 0 && row.previous_balance == 0 {
            continue;
        }
        let id = store.next_closing_id;
        store.next_closing_id += 1;
        let record = ClosingRecord {
            id,
            kind: row.kind,
            party_code: row.party_code,
            party_name: row.party_name,
            month: row.month,
            sales_total: row.sales_total,
            tax: row.tax,
            payments: row.payments,
            previous_balance: row.previous_balance,
            balance: row.balance,
            closed_at: today(),
        };
        store.closings.push(record.clone());
        created.push(record);
    }

    if created.is_empty() {
        return Err((400, "締処理対象のデータがありません。".to_string()));
    }
    Ok(created)
}

pub(crate) fn create_quote_inner(
    store: &mut Store,
    input: QuoteInput,
) -> Result<Quote, (u16, String)> {
    ensure_capacity(store.quotes.len())?;
    let customer = store
        .customer(input.customer_code.trim())
        .cloned()
        .ok_or((400, "得意先を選択してください。".to_string()))?;
    let date = normalize_date(&input.date).map_err(|message| (400, message))?;
    let lines = store
        .build_lines(&input.lines)
        .map_err(|message| (400, message))?;
    let (subtotal, tax, total) = totals(&lines);
    let status = match input.status.as_deref() {
        Some("sent") => "sent",
        _ => "draft",
    };

    let id = store.next_quote_id;
    store.next_quote_id += 1;
    let (customer_name, customer_name2, honorific) = party_snapshot(&customer);

    let quote = Quote {
        id,
        number: format!("Q-{id:04}"),
        date,
        customer_code: customer.code.clone(),
        customer_name,
        customer_name2,
        honorific,
        subject: clean(input.subject),
        valid_until: normalize_optional_date(input.valid_until.as_deref())
            .map_err(|message| (400, message))?,
        delivery_date: normalize_optional_date(input.delivery_date.as_deref())
            .map_err(|message| (400, message))?,
        delivery_place: clean(input.delivery_place),
        payment_terms: clean(input.payment_terms),
        notes: clean(input.notes),
        status: status.to_string(),
        order_id: None,
        lines,
        subtotal,
        tax,
        total,
        updated_at: today(),
    };

    store.quotes.push(quote.clone());
    Ok(quote)
}

pub(crate) fn update_quote_inner(
    store: &mut Store,
    id: u64,
    input: QuoteInput,
) -> Result<Quote, (u16, String)> {
    let Some(existing) = store.quote(id) else {
        return Err((404, "見積が見つかりません。".to_string()));
    };
    if existing.status == "accepted" {
        return Err((409, "受注済みの見積は編集できません。".to_string()));
    }

    let customer = store
        .customer(input.customer_code.trim())
        .cloned()
        .ok_or((400, "得意先を選択してください。".to_string()))?;
    let date = normalize_date(&input.date).map_err(|message| (400, message))?;
    let lines = store
        .build_lines(&input.lines)
        .map_err(|message| (400, message))?;
    let (subtotal, tax, total) = totals(&lines);
    let status = match input.status.as_deref() {
        Some("sent") => "sent",
        _ => "draft",
    };
    let valid_until =
        normalize_optional_date(input.valid_until.as_deref()).map_err(|message| (400, message))?;
    let delivery_date = normalize_optional_date(input.delivery_date.as_deref())
        .map_err(|message| (400, message))?;
    let (customer_name, customer_name2, honorific) = party_snapshot(&customer);

    let Some(quote) = store.quote_mut(id) else {
        return Err((404, "見積が見つかりません。".to_string()));
    };
    quote.date = date;
    quote.customer_code = customer.code.clone();
    quote.customer_name = customer_name;
    quote.customer_name2 = customer_name2;
    quote.honorific = honorific;
    quote.subject = clean(input.subject);
    quote.valid_until = valid_until;
    quote.delivery_date = delivery_date;
    quote.delivery_place = clean(input.delivery_place);
    quote.payment_terms = clean(input.payment_terms);
    quote.notes = clean(input.notes);
    quote.status = status.to_string();
    quote.lines = lines;
    quote.subtotal = subtotal;
    quote.tax = tax;
    quote.total = total;
    quote.updated_at = today();
    Ok(quote.clone())
}

pub(crate) fn delete_quote_inner(store: &mut Store, id: u64) -> Result<(), (u16, String)> {
    let Some(index) = store.quotes.iter().position(|quote| quote.id == id) else {
        return Err((404, "見積が見つかりません。".to_string()));
    };
    let removed = store.quotes.remove(index);
    if let Some(order_id) = removed.order_id {
        if let Some(order) = store.order_mut(order_id) {
            order.quote_id = None;
            order.quote_number.clear();
        }
    }
    Ok(())
}

pub(crate) fn convert_quote_inner(store: &mut Store, id: u64) -> Result<Order, (u16, String)> {
    let quote = store
        .quote(id)
        .cloned()
        .ok_or((404, "見積が見つかりません。".to_string()))?;
    if quote.order_id.is_some() {
        return Err((409, "この見積はすでに受注に変換されています。".to_string()));
    }
    if quote.lines.is_empty() {
        return Err((400, "明細がない見積は受注に変換できません。".to_string()));
    }

    let mut lines = Vec::with_capacity(quote.lines.len());
    for line in &quote.lines {
        let line_id = store.next_line_id;
        store.next_line_id += 1;
        lines.push(LineItem {
            id: line_id,
            source_line_id: Some(line.id),
            ..line.clone()
        });
    }
    let (subtotal, tax, total) = totals(&lines);
    let order_id = store.next_order_id;
    store.next_order_id += 1;

    let order = Order {
        id: order_id,
        number: format!("O-{order_id:04}"),
        date: today(),
        quote_id: Some(quote.id),
        quote_number: quote.number.clone(),
        customer_code: quote.customer_code.clone(),
        customer_name: quote.customer_name.clone(),
        customer_name2: quote.customer_name2.clone(),
        honorific: quote.honorific.clone(),
        customer_order_no: String::new(),
        subject: quote.subject.clone(),
        delivery_date: quote.delivery_date.clone(),
        notes: quote.notes.clone(),
        status: "open".to_string(),
        lines,
        subtotal,
        tax,
        total,
        updated_at: today(),
    };
    store.orders.push(order.clone());

    if let Some(source) = store.quote_mut(id) {
        source.status = "accepted".to_string();
        source.order_id = Some(order_id);
    }

    Ok(order)
}

pub(crate) fn create_order_inner(
    store: &mut Store,
    input: OrderInput,
) -> Result<Order, (u16, String)> {
    ensure_capacity(store.orders.len())?;
    let customer = store
        .customer(input.customer_code.trim())
        .cloned()
        .ok_or((400, "得意先を選択してください。".to_string()))?;
    let date = normalize_date(&input.date).map_err(|message| (400, message))?;
    let lines = store
        .build_lines(&input.lines)
        .map_err(|message| (400, message))?;
    let (subtotal, tax, total) = totals(&lines);
    let delivery_date = normalize_optional_date(input.delivery_date.as_deref())
        .map_err(|message| (400, message))?;
    let status = match input.status.as_deref() {
        Some("completed") => "completed",
        _ => "open",
    };
    let id = store.next_order_id;
    store.next_order_id += 1;
    let (customer_name, customer_name2, honorific) = party_snapshot(&customer);

    let order = Order {
        id,
        number: format!("O-{id:04}"),
        date,
        quote_id: None,
        quote_number: String::new(),
        customer_code: customer.code.clone(),
        customer_name,
        customer_name2,
        honorific,
        customer_order_no: clean(input.customer_order_no),
        subject: clean(input.subject),
        delivery_date,
        notes: clean(input.notes),
        status: status.to_string(),
        lines,
        subtotal,
        tax,
        total,
        updated_at: today(),
    };

    store.orders.push(order.clone());
    Ok(order)
}

pub(crate) fn update_order_inner(
    store: &mut Store,
    id: u64,
    input: OrderInput,
) -> Result<Order, (u16, String)> {
    let Some(order) = store.order(id) else {
        return Err((404, "受注が見つかりません。".to_string()));
    };
    if order
        .lines
        .iter()
        .any(|line| line.sold_quantity > 0.0 || line.purchased_quantity > 0.0)
    {
        return Err((
            409,
            "売上または発注が発生している受注は編集できません。".to_string(),
        ));
    }

    let customer = store
        .customer(input.customer_code.trim())
        .cloned()
        .ok_or((400, "得意先を選択してください。".to_string()))?;
    let date = normalize_date(&input.date).map_err(|message| (400, message))?;
    let lines = store
        .build_lines(&input.lines)
        .map_err(|message| (400, message))?;
    let (subtotal, tax, total) = totals(&lines);
    let delivery_date = normalize_optional_date(input.delivery_date.as_deref())
        .map_err(|message| (400, message))?;
    let status = match input.status.as_deref() {
        Some("completed") => "completed",
        _ => "open",
    };
    let (customer_name, customer_name2, honorific) = party_snapshot(&customer);

    let Some(order) = store.order_mut(id) else {
        return Err((404, "受注が見つかりません。".to_string()));
    };
    order.date = date;
    order.customer_code = customer.code.clone();
    order.customer_name = customer_name;
    order.customer_name2 = customer_name2;
    order.honorific = honorific;
    order.customer_order_no = clean(input.customer_order_no);
    order.subject = clean(input.subject);
    order.delivery_date = delivery_date;
    order.notes = clean(input.notes);
    order.status = status.to_string();
    order.lines = lines;
    order.subtotal = subtotal;
    order.tax = tax;
    order.total = total;
    order.updated_at = today();
    Ok(order.clone())
}

pub(crate) fn delete_order_inner(store: &mut Store, id: u64) -> Result<(), (u16, String)> {
    let Some(index) = store.orders.iter().position(|order| order.id == id) else {
        return Err((404, "受注が見つかりません。".to_string()));
    };
    if store.sales.iter().any(|sale| sale.order_id == Some(id))
        || store
            .purchase_orders
            .iter()
            .any(|order| order.order_id == Some(id))
    {
        return Err((
            409,
            "売上または発注が発生している受注は削除できません。".to_string(),
        ));
    }
    let removed = store.orders.remove(index);
    if let Some(quote_id) = removed.quote_id {
        if let Some(quote) = store.quote_mut(quote_id) {
            quote.order_id = None;
            if quote.status == "accepted" {
                quote.status = "sent".to_string();
            }
        }
    }
    Ok(())
}
