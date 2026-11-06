use serde::{Deserialize, Serialize};

/// 消費税率（PoC では単一税率）。
pub const TAX_RATE: f64 = 0.10;

#[derive(Clone, Debug, Serialize)]
pub struct Customer {
    pub code: String,
    pub name: String,
    pub name2: String,
    pub honorific: String,
    pub contact_name: String,
    pub postal_code: String,
    pub address: String,
    pub tel: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct Supplier {
    pub code: String,
    pub name: String,
    pub honorific: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct Product {
    pub code: String,
    pub name: String,
    pub unit: String,
    pub unit_price: i64,
    pub cost_price: i64,
    pub taxable: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct Company {
    pub name: String,
    pub postal_code: String,
    pub address1: String,
    pub address2: String,
    pub tel: String,
    pub fax: String,
    pub registration_no: String,
    pub bank_name: String,
    pub bank_branch: String,
    pub bank_account_type: String,
    pub bank_account_number: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct LineItem {
    pub id: u64,
    pub product_code: String,
    pub name: String,
    pub quantity: f64,
    pub unit: String,
    pub unit_price: i64,
    pub amount: i64,
    pub taxable: bool,
    pub note: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_line_id: Option<u64>,
    /// 受注明細の売上済数量（受注明細でのみ使用）
    pub sold_quantity: f64,
    /// 受注明細の発注済数量（受注明細でのみ使用）
    pub purchased_quantity: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct Sale {
    pub id: u64,
    pub number: String,
    pub date: String,
    pub order_id: Option<u64>,
    pub order_number: String,
    pub quote_number: String,
    pub customer_code: String,
    pub customer_name: String,
    pub customer_name2: String,
    pub honorific: String,
    pub subject: String,
    pub notes: String,
    pub lines: Vec<LineItem>,
    pub subtotal: i64,
    pub tax: i64,
    pub total: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct PurchaseOrder {
    pub id: u64,
    pub number: String,
    pub date: String,
    pub order_id: Option<u64>,
    pub order_number: String,
    pub supplier_code: String,
    pub supplier_name: String,
    pub supplier_honorific: String,
    /// open | received
    pub status: String,
    pub notes: String,
    pub lines: Vec<LineItem>,
    pub subtotal: i64,
    pub tax: i64,
    pub total: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct Purchase {
    pub id: u64,
    pub number: String,
    pub date: String,
    pub purchase_order_id: Option<u64>,
    pub purchase_order_number: String,
    pub supplier_code: String,
    pub supplier_name: String,
    pub supplier_honorific: String,
    pub notes: String,
    pub lines: Vec<LineItem>,
    pub subtotal: i64,
    pub tax: i64,
    pub total: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct Receipt {
    pub id: u64,
    pub number: String,
    pub date: String,
    pub customer_code: String,
    pub customer_name: String,
    pub amount: i64,
    pub method: String,
    pub notes: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct SupplierPayment {
    pub id: u64,
    pub number: String,
    pub date: String,
    pub supplier_code: String,
    pub supplier_name: String,
    pub amount: i64,
    pub method: String,
    pub notes: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ClosingRecord {
    pub id: u64,
    pub kind: String,
    pub party_code: String,
    pub party_name: String,
    pub month: String,
    pub sales_total: i64,
    pub tax: i64,
    pub payments: i64,
    pub previous_balance: i64,
    pub balance: i64,
    pub closed_at: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ClosingPreviewRow {
    pub kind: String,
    pub party_code: String,
    pub party_name: String,
    pub month: String,
    pub sales_total: i64,
    pub tax: i64,
    pub payments: i64,
    pub previous_balance: i64,
    pub balance: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct Quote {
    pub id: u64,
    pub number: String,
    pub date: String,
    pub customer_code: String,
    pub customer_name: String,
    pub customer_name2: String,
    pub honorific: String,
    pub subject: String,
    pub valid_until: String,
    pub delivery_date: String,
    pub delivery_place: String,
    pub payment_terms: String,
    pub notes: String,
    /// draft | sent | accepted
    pub status: String,
    pub order_id: Option<u64>,
    pub lines: Vec<LineItem>,
    pub subtotal: i64,
    pub tax: i64,
    pub total: i64,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct Order {
    pub id: u64,
    pub number: String,
    pub date: String,
    pub quote_id: Option<u64>,
    pub quote_number: String,
    pub customer_code: String,
    pub customer_name: String,
    pub customer_name2: String,
    pub honorific: String,
    pub customer_order_no: String,
    pub subject: String,
    pub delivery_date: String,
    pub notes: String,
    /// open | completed
    pub status: String,
    pub lines: Vec<LineItem>,
    pub subtotal: i64,
    pub tax: i64,
    pub total: i64,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct LineInput {
    #[serde(default)]
    pub product_code: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    pub quantity: f64,
    #[serde(default)]
    pub unit: Option<String>,
    pub unit_price: i64,
    #[serde(default)]
    pub taxable: Option<bool>,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct QuoteInput {
    pub customer_code: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub subject: Option<String>,
    #[serde(default)]
    pub valid_until: Option<String>,
    #[serde(default)]
    pub delivery_date: Option<String>,
    #[serde(default)]
    pub delivery_place: Option<String>,
    #[serde(default)]
    pub payment_terms: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub lines: Vec<LineInput>,
}

#[derive(Debug, Deserialize)]
pub struct OrderInput {
    pub customer_code: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub customer_order_no: Option<String>,
    #[serde(default)]
    pub subject: Option<String>,
    #[serde(default)]
    pub delivery_date: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub lines: Vec<LineInput>,
}

#[derive(Debug, Deserialize)]
pub struct FlowLineInput {
    pub line_id: u64,
    pub quantity: f64,
}

#[derive(Debug, Deserialize)]
pub struct SaleInput {
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub lines: Vec<FlowLineInput>,
}

#[derive(Debug, Deserialize)]
pub struct PurchaseOrderInput {
    pub supplier_code: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub lines: Vec<FlowLineInput>,
}

#[derive(Debug, Deserialize)]
pub struct ReceiveInput {
    #[serde(default)]
    pub date: String,
}

#[derive(Debug, Deserialize)]
pub struct ReceiptInput {
    pub customer_code: String,
    #[serde(default)]
    pub date: String,
    pub amount: i64,
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SupplierPaymentInput {
    pub supplier_code: String,
    #[serde(default)]
    pub date: String,
    pub amount: i64,
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ClosingInput {
    pub kind: String,
    pub month: String,
}

pub fn round_yen(value: f64) -> i64 {
    value.round() as i64
}

/// (小計, 消費税, 合計)
pub fn totals(lines: &[LineItem]) -> (i64, i64, i64) {
    let subtotal: i64 = lines.iter().map(|line| line.amount).sum();
    let taxable: i64 = lines
        .iter()
        .filter(|line| line.taxable)
        .map(|line| line.amount)
        .sum();
    let tax = round_yen(taxable as f64 * TAX_RATE);
    (subtotal, tax, subtotal + tax)
}

pub fn today() -> String {
    date_days_ago(0)
}

/// 今日から `days` 日前の日付（YYYY-MM-DD）。デモデータの生成用。
pub fn date_days_ago(days: u32) -> String {
    let now = js_sys::Date::new_0();
    let target = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(
        now.get_time() - f64::from(days) * 86_400_000.0,
    ));
    format!(
        "{:04}-{:02}-{:02}",
        target.get_full_year(),
        target.get_month() + 1,
        target.get_date()
    )
}

pub fn normalize_date(raw: &str) -> Result<String, String> {
    let value = raw.trim();
    if value.is_empty() {
        return Ok(today());
    }
    if is_valid_date(value) {
        Ok(value.to_string())
    } else {
        Err("日付は YYYY-MM-DD 形式で入力してください。".to_string())
    }
}

pub fn normalize_optional_date(raw: Option<&str>) -> Result<String, String> {
    let value = raw.unwrap_or("").trim();
    if value.is_empty() {
        return Ok(String::new());
    }
    if is_valid_date(value) {
        Ok(value.to_string())
    } else {
        Err("日付は YYYY-MM-DD 形式で入力してください。".to_string())
    }
}

fn is_valid_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return false;
    }
    if !bytes
        .iter()
        .enumerate()
        .all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit())
    {
        return false;
    }
    let month: u32 = value[5..7].parse().unwrap_or(0);
    let day: u32 = value[8..10].parse().unwrap_or(0);
    (1..=12).contains(&month) && (1..=31).contains(&day)
}

pub fn format_yen(value: i64) -> String {
    let negative = value < 0;
    let digits = value.abs().to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3 + 1);
    for (index, ch) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    if negative {
        format!("-{out}")
    } else {
        out
    }
}

pub fn format_qty(value: f64) -> String {
    if (value.fract()).abs() < 0.0001 {
        format!("{value:.0}")
    } else {
        format!("{value:.2}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

pub fn jp_date(value: &str) -> String {
    if value.len() != 10 {
        return value.to_string();
    }
    let year = &value[0..4];
    let month: u32 = value[5..7].parse().unwrap_or(0);
    let day: u32 = value[8..10].parse().unwrap_or(0);
    if month == 0 || day == 0 {
        value.to_string()
    } else {
        format!("{year}年{month}月{day}日")
    }
}

pub fn escape_html(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}
