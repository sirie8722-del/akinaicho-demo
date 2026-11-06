use std::cell::RefCell;

use crate::models::*;

pub struct Store {
    pub customers: Vec<Customer>,
    pub suppliers: Vec<Supplier>,
    pub products: Vec<Product>,
    pub company: Company,
    pub quotes: Vec<Quote>,
    pub orders: Vec<Order>,
    pub sales: Vec<Sale>,
    pub purchase_orders: Vec<PurchaseOrder>,
    pub purchases: Vec<Purchase>,
    pub receipts: Vec<Receipt>,
    pub supplier_payments: Vec<SupplierPayment>,
    pub closings: Vec<ClosingRecord>,
    pub next_quote_id: u64,
    pub next_order_id: u64,
    pub next_line_id: u64,
    pub next_sale_id: u64,
    pub next_purchase_order_id: u64,
    pub next_purchase_id: u64,
    pub next_receipt_id: u64,
    pub next_supplier_payment_id: u64,
    pub next_closing_id: u64,
}

thread_local! {
    /// 学習用のインメモリストア。isolate ごとに保持され、再起動で初期データに戻る。
    static STORE: RefCell<Store> = RefCell::new(Store::seed());
}

pub fn with_store<T>(f: impl FnOnce(&mut Store) -> T) -> T {
    STORE.with(|store| f(&mut store.borrow_mut()))
}

impl Store {
    /// 見積 100 件・受注 100 件のデモデータを作る。
    /// 先頭 30 件の見積は受注に変換済みで、受注と明細 ID がリンクしている。
    pub fn seed() -> Self {
        let customers = vec![
            customer(
                "C001",
                "株式会社サンプル商事",
                "情報システム部  山田 太郎",
                "御中",
                "山田 太郎",
                "100-0001",
                "東京都千代田区丸の内1-1-1",
                "03-1234-5678",
            ),
            customer(
                "C002",
                "テスト工業株式会社",
                "総務部  佐藤 花子",
                "御中",
                "佐藤 花子",
                "220-0012",
                "神奈川県横浜市西区みなとみらい2-2-1",
                "045-234-5678",
            ),
            customer(
                "C003",
                "有限会社みどり商店",
                "代表  鈴木 一郎",
                "様",
                "鈴木 一郎",
                "530-0001",
                "大阪府大阪市北区梅田3-3-3",
                "06-3456-7890",
            ),
            customer(
                "C004",
                "株式会社ブルースカイ",
                "購買部  高橋 美咲",
                "御中",
                "高橋 美咲",
                "460-0008",
                "愛知県名古屋市中区栄4-4-4",
                "052-456-7890",
            ),
            customer(
                "C005",
                "シンプル株式会社",
                "管理部  伊藤 健",
                "御中",
                "伊藤 健",
                "810-0001",
                "福岡県福岡市中央区天神5-5-5",
                "092-567-8901",
            ),
        ];

        let products = vec![
            product("P001", "ノートPC スタンダードモデル", "台", 128_000, 96_000),
            product("P002", "ワイヤレスマウス", "個", 3_500, 2_200),
            product("P003", "27インチモニター", "台", 42_000, 30_000),
            product("P004", "年間保守サポート", "式", 60_000, 38_000),
            product("P005", "初期セットアップ作業", "式", 25_000, 15_000),
            product("P006", "デスクトップPC 高性能モデル", "台", 98_000, 72_000),
            product("P007", "USB-C ドッキングステーション", "個", 18_000, 11_000),
            product("P008", "オンサイト作業", "式", 35_000, 22_000),
        ];

        let suppliers = vec![
            Supplier {
                code: "S001".to_string(),
                name: "株式会社サプライヤーA".to_string(),
                honorific: "御中".to_string(),
            },
            Supplier {
                code: "S002".to_string(),
                name: "サプライヤーB 株式会社".to_string(),
                honorific: "御中".to_string(),
            },
            Supplier {
                code: "S003".to_string(),
                name: "有限会社パーツ工房".to_string(),
                honorific: "様".to_string(),
            },
        ];

        let company = Company {
            name: "株式会社サンプルシステム".to_string(),
            postal_code: "150-0002".to_string(),
            address1: "東京都渋谷区渋谷2-2-2 サンプルビル 5F".to_string(),
            address2: "営業部 情報ソリューショングループ".to_string(),
            tel: "03-9876-5432".to_string(),
            fax: "03-9876-5433".to_string(),
            registration_no: "T1234567890123".to_string(),
            bank_name: "サンプル銀行".to_string(),
            bank_branch: "渋谷支店".to_string(),
            bank_account_type: "普通".to_string(),
            bank_account_number: "1234567".to_string(),
        };

        let subjects = [
            "オフィスPC環境更新",
            "ネットワーク構築",
            "保守契約更新",
            "機器追加購入",
            "基幹システム周辺機器",
        ];
        let places = ["貴社 本社", "貴社 支社", "貴社 物流センター"];

        let mut quotes = Vec::with_capacity(100);
        let mut orders = Vec::with_capacity(100);
        let mut next_line_id = 1u64;

        for index in 0..100usize {
            let id = (index + 1) as u64;
            let customer = customers[index % customers.len()].clone();
            let date = date_days_ago((index * 3) as u32);
            let linked = index < 30;
            let subject = format!("{} 案件 {:03}", subjects[index % subjects.len()], index + 1);
            let line_count = 1 + (index % 3);
            let mut quote_lines = Vec::with_capacity(line_count);

            for line_index in 0..line_count {
                let product = products[(index + line_index) % products.len()].clone();
                let quantity = ((index + line_index) % 3 + 1) as f64;
                quote_lines.push(LineItem {
                    id: next_line_id,
                    product_code: product.code,
                    name: product.name,
                    quantity,
                    unit: product.unit,
                    unit_price: product.unit_price,
                    amount: round_yen(quantity * product.unit_price as f64),
                    taxable: product.taxable,
                    note: if line_index == 0 && index % 5 == 0 {
                        "デモ明細".to_string()
                    } else {
                        String::new()
                    },
                    source_line_id: None,
                    sold_quantity: 0.0,
                    purchased_quantity: 0.0,
                });
                next_line_id += 1;
            }

            let (subtotal, tax, total) = totals(&quote_lines);
            let number = format!("Q-{id:04}");
            quotes.push(Quote {
                id,
                number: number.clone(),
                date: date.clone(),
                customer_code: customer.code.clone(),
                customer_name: customer.name.clone(),
                customer_name2: customer.name2.clone(),
                honorific: customer.honorific.clone(),
                subject: subject.clone(),
                valid_until: date.clone(),
                delivery_date: date.clone(),
                delivery_place: places[index % places.len()].to_string(),
                payment_terms: "納品後 30日以内".to_string(),
                notes: "デモデータです。".to_string(),
                status: if linked {
                    "accepted".to_string()
                } else if index % 4 == 0 {
                    "draft".to_string()
                } else {
                    "sent".to_string()
                },
                order_id: if linked { Some(id) } else { None },
                lines: quote_lines,
                subtotal,
                tax,
                total,
                updated_at: date.clone(),
            });

            let order_lines = if linked {
                let source = quotes.last().expect("quote was just pushed");
                source
                    .lines
                    .iter()
                    .map(|line| {
                        let order_line = LineItem {
                            id: next_line_id,
                            source_line_id: Some(line.id),
                            ..line.clone()
                        };
                        next_line_id += 1;
                        order_line
                    })
                    .collect()
            } else {
                let mut lines = Vec::with_capacity(line_count);
                for line_index in 0..line_count {
                    let product = products[(index + line_index + 2) % products.len()].clone();
                    let quantity = ((index + line_index) % 4 + 1) as f64;
                    lines.push(LineItem {
                        id: next_line_id,
                        product_code: product.code,
                        name: product.name,
                        quantity,
                        unit: product.unit,
                        unit_price: product.unit_price,
                        amount: round_yen(quantity * product.unit_price as f64),
                        taxable: product.taxable,
                        note: String::new(),
                        source_line_id: None,
                        sold_quantity: 0.0,
                        purchased_quantity: 0.0,
                    });
                    next_line_id += 1;
                }
                lines
            };

            let (order_subtotal, order_tax, order_total) = totals(&order_lines);
            orders.push(Order {
                id,
                number: format!("O-{id:04}"),
                date: date.clone(),
                quote_id: if linked { Some(id) } else { None },
                quote_number: if linked { number } else { String::new() },
                customer_code: customer.code,
                customer_name: customer.name,
                customer_name2: customer.name2,
                honorific: customer.honorific,
                customer_order_no: format!("PO-{:04}", 2000 + index),
                subject,
                delivery_date: date.clone(),
                notes: "デモデータです。".to_string(),
                status: if index % 5 == 0 {
                    "completed".to_string()
                } else {
                    "open".to_string()
                },
                lines: order_lines,
                subtotal: order_subtotal,
                tax: order_tax,
                total: order_total,
                updated_at: date,
            });
        }

        let mut sales = Vec::new();
        let mut purchase_orders = Vec::new();
        let mut purchases = Vec::new();
        let mut receipts = Vec::new();
        let mut supplier_payments = Vec::new();
        let mut next_sale_id = 1u64;
        let mut next_purchase_order_id = 1u64;
        let mut next_purchase_id = 1u64;
        let mut next_receipt_id = 1u64;
        let mut next_supplier_payment_id = 1u64;

        for index in 0..orders.len() {
            // 受注 → 売上（一部の受注は数量の半分だけ売上計上）
            if index < 70 {
                let order = orders[index].clone();
                let full = index < 45;
                let mut sale_lines = Vec::new();
                for line in &order.lines {
                    let quantity = if full {
                        line.quantity
                    } else {
                        line.quantity * 0.5
                    };
                    if quantity <= 0.0 {
                        continue;
                    }
                    sale_lines.push(LineItem {
                        id: next_line_id,
                        source_line_id: Some(line.id),
                        quantity,
                        amount: round_yen(quantity * line.unit_price as f64),
                        sold_quantity: 0.0,
                        purchased_quantity: 0.0,
                        ..line.clone()
                    });
                    next_line_id += 1;
                }

                if !sale_lines.is_empty() {
                    let (subtotal, tax, total) = totals(&sale_lines);
                    let id = next_sale_id;
                    next_sale_id += 1;
                    sales.push(Sale {
                        id,
                        number: format!("S-{id:04}"),
                        date: order.date.clone(),
                        order_id: Some(order.id),
                        order_number: order.number.clone(),
                        quote_number: order.quote_number.clone(),
                        customer_code: order.customer_code.clone(),
                        customer_name: order.customer_name.clone(),
                        customer_name2: order.customer_name2.clone(),
                        honorific: order.honorific.clone(),
                        subject: order.subject.clone(),
                        notes: order.notes.clone(),
                        lines: sale_lines.clone(),
                        subtotal,
                        tax,
                        total,
                    });

                    for line in &sale_lines {
                        if let Some(source_id) = line.source_line_id {
                            if let Some(order_line) = orders[index]
                                .lines
                                .iter_mut()
                                .find(|item| item.id == source_id)
                            {
                                order_line.sold_quantity += line.quantity;
                            }
                        }
                    }
                }
            }

            // 受注 → 発注 → 仕入（一部のみ発注、発注の一部を仕入計上）
            if index < 40 {
                let order = orders[index].clone();
                let supplier = suppliers[index % suppliers.len()].clone();
                let full = index < 25;
                let mut po_lines = Vec::new();

                for line in &order.lines {
                    let quantity = if full {
                        line.quantity
                    } else {
                        line.quantity * 0.5
                    };
                    if quantity <= 0.0 {
                        continue;
                    }
                    let cost_price = products
                        .iter()
                        .find(|product| product.code == line.product_code)
                        .map(|product| product.cost_price)
                        .unwrap_or(line.unit_price);
                    po_lines.push(LineItem {
                        id: next_line_id,
                        source_line_id: Some(line.id),
                        quantity,
                        unit_price: cost_price,
                        amount: round_yen(quantity * cost_price as f64),
                        sold_quantity: 0.0,
                        purchased_quantity: 0.0,
                        ..line.clone()
                    });
                    next_line_id += 1;
                }

                if !po_lines.is_empty() {
                    let (subtotal, tax, total) = totals(&po_lines);
                    let po_id = next_purchase_order_id;
                    next_purchase_order_id += 1;
                    let po_number = format!("PO-{po_id:04}");
                    purchase_orders.push(PurchaseOrder {
                        id: po_id,
                        number: po_number.clone(),
                        date: order.date.clone(),
                        order_id: Some(order.id),
                        order_number: order.number.clone(),
                        supplier_code: supplier.code.clone(),
                        supplier_name: supplier.name.clone(),
                        supplier_honorific: supplier.honorific.clone(),
                        status: "open".to_string(),
                        notes: order.notes.clone(),
                        lines: po_lines.clone(),
                        subtotal,
                        tax,
                        total,
                    });

                    for line in &po_lines {
                        if let Some(source_id) = line.source_line_id {
                            if let Some(order_line) = orders[index]
                                .lines
                                .iter_mut()
                                .find(|item| item.id == source_id)
                            {
                                order_line.purchased_quantity += line.quantity;
                            }
                        }
                    }

                    if index < 20 {
                        let (subtotal, tax, total) = totals(&po_lines);
                        let purchase_lines = po_lines
                            .iter()
                            .map(|line| {
                                let purchase_line = LineItem {
                                    id: next_line_id,
                                    source_line_id: Some(line.id),
                                    sold_quantity: 0.0,
                                    purchased_quantity: 0.0,
                                    ..line.clone()
                                };
                                next_line_id += 1;
                                purchase_line
                            })
                            .collect();

                        let purchase_id = next_purchase_id;
                        next_purchase_id += 1;
                        purchases.push(Purchase {
                            id: purchase_id,
                            number: format!("PI-{purchase_id:04}"),
                            date: order.date.clone(),
                            purchase_order_id: Some(po_id),
                            purchase_order_number: po_number,
                            supplier_code: supplier.code.clone(),
                            supplier_name: supplier.name.clone(),
                            supplier_honorific: supplier.honorific.clone(),
                            notes: order.notes.clone(),
                            lines: purchase_lines,
                            subtotal,
                            tax,
                            total,
                        });

                        if let Some(po) = purchase_orders.iter_mut().find(|item| item.id == po_id) {
                            po.status = "received".to_string();
                        }
                    }
                }
            }
        }

        // 受注の完納状態を売上数量から更新
        for order in &mut orders {
            let complete = order
                .lines
                .iter()
                .all(|line| line.sold_quantity + 0.0001 >= line.quantity);
            order.status = if complete { "completed" } else { "open" }.to_string();
        }

        // 売上に対する入金（全額・一部）
        for (sale_index, sale) in sales.iter().enumerate() {
            let amount = if sale_index < 35 {
                sale.total
            } else if sale_index < 45 {
                sale.total / 2
            } else {
                continue;
            };
            let id = next_receipt_id;
            next_receipt_id += 1;
            receipts.push(Receipt {
                id,
                number: format!("R-{id:04}"),
                date: sale.date.clone(),
                customer_code: sale.customer_code.clone(),
                customer_name: sale.customer_name.clone(),
                amount,
                method: if id.is_multiple_of(2) {
                    "振込"
                } else {
                    "現金"
                }
                .to_string(),
                notes: String::new(),
            });
        }

        // 仕入に対する支払
        for (purchase_index, purchase) in purchases.iter().enumerate() {
            if purchase_index >= 12 {
                continue;
            }
            let id = next_supplier_payment_id;
            next_supplier_payment_id += 1;
            supplier_payments.push(SupplierPayment {
                id,
                number: format!("D-{id:04}"),
                date: purchase.date.clone(),
                supplier_code: purchase.supplier_code.clone(),
                supplier_name: purchase.supplier_name.clone(),
                amount: purchase.total,
                method: "振込".to_string(),
                notes: String::new(),
            });
        }

        Self {
            customers,
            suppliers,
            products,
            company,
            quotes,
            orders,
            sales,
            purchase_orders,
            purchases,
            receipts,
            supplier_payments,
            closings: Vec::new(),
            next_quote_id: 101,
            next_order_id: 101,
            next_line_id,
            next_sale_id,
            next_purchase_order_id,
            next_purchase_id,
            next_receipt_id,
            next_supplier_payment_id,
            next_closing_id: 1,
        }
    }

    pub fn customer(&self, code: &str) -> Option<&Customer> {
        self.customers.iter().find(|customer| customer.code == code)
    }

    pub fn quote(&self, id: u64) -> Option<&Quote> {
        self.quotes.iter().find(|quote| quote.id == id)
    }

    pub fn quote_mut(&mut self, id: u64) -> Option<&mut Quote> {
        self.quotes.iter_mut().find(|quote| quote.id == id)
    }

    pub fn order(&self, id: u64) -> Option<&Order> {
        self.orders.iter().find(|order| order.id == id)
    }

    pub fn order_mut(&mut self, id: u64) -> Option<&mut Order> {
        self.orders.iter_mut().find(|order| order.id == id)
    }

    pub fn supplier(&self, code: &str) -> Option<&Supplier> {
        self.suppliers.iter().find(|supplier| supplier.code == code)
    }

    pub fn product(&self, code: &str) -> Option<&Product> {
        self.products.iter().find(|product| product.code == code)
    }

    pub fn purchase_order(&self, id: u64) -> Option<&PurchaseOrder> {
        self.purchase_orders.iter().find(|order| order.id == id)
    }

    pub fn purchase_order_mut(&mut self, id: u64) -> Option<&mut PurchaseOrder> {
        self.purchase_orders.iter_mut().find(|order| order.id == id)
    }

    pub fn build_lines(&mut self, inputs: &[LineInput]) -> Result<Vec<LineItem>, String> {
        if inputs.len() > 100 {
            return Err("明細は 100 行までです。".to_string());
        }

        let mut lines = Vec::new();
        for input in inputs {
            let product = input
                .product_code
                .as_deref()
                .and_then(|code| self.products.iter().find(|product| product.code == code))
                .cloned();

            let name = input
                .name
                .as_deref()
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .map(str::to_string)
                .or_else(|| product.as_ref().map(|product| product.name.clone()));

            let Some(name) = name else {
                if input.quantity == 0.0 && input.unit_price == 0 {
                    continue;
                }
                return Err("商品を選択するか、商品名を入力してください。".to_string());
            };

            if !input.quantity.is_finite() || input.quantity <= 0.0 || input.quantity > 1_000_000.0
            {
                return Err("数量は 0 より大きい値を入力してください。".to_string());
            }
            if input.unit_price < 0 || input.unit_price > 1_000_000_000 {
                return Err("単価が不正です。".to_string());
            }

            let unit = input
                .unit
                .as_deref()
                .map(str::trim)
                .filter(|unit| !unit.is_empty())
                .map(str::to_string)
                .or_else(|| product.as_ref().map(|product| product.unit.clone()))
                .unwrap_or_default();

            let taxable = input.taxable.unwrap_or_else(|| {
                product
                    .as_ref()
                    .map(|product| product.taxable)
                    .unwrap_or(true)
            });

            let amount = round_yen(input.quantity * input.unit_price as f64);
            let id = self.next_line_id;
            self.next_line_id += 1;

            lines.push(LineItem {
                id,
                product_code: input
                    .product_code
                    .clone()
                    .or_else(|| product.map(|product| product.code))
                    .unwrap_or_default(),
                name,
                quantity: input.quantity,
                unit,
                unit_price: input.unit_price,
                amount,
                taxable,
                note: input.note.as_deref().unwrap_or("").trim().to_string(),
                source_line_id: None,
                sold_quantity: 0.0,
                purchased_quantity: 0.0,
            });
        }

        if lines.is_empty() {
            return Err("明細を 1 行以上入力してください。".to_string());
        }
        Ok(lines)
    }
}

#[allow(clippy::too_many_arguments)]
fn customer(
    code: &str,
    name: &str,
    name2: &str,
    honorific: &str,
    contact_name: &str,
    postal_code: &str,
    address: &str,
    tel: &str,
) -> Customer {
    Customer {
        code: code.to_string(),
        name: name.to_string(),
        name2: name2.to_string(),
        honorific: honorific.to_string(),
        contact_name: contact_name.to_string(),
        postal_code: postal_code.to_string(),
        address: address.to_string(),
        tel: tel.to_string(),
    }
}

fn product(code: &str, name: &str, unit: &str, unit_price: i64, cost_price: i64) -> Product {
    Product {
        code: code.to_string(),
        name: name.to_string(),
        unit: unit.to_string(),
        unit_price,
        cost_price,
        taxable: true,
    }
}
