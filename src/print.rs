use crate::models::*;
use crate::reports::Summary;

fn page(title: &str, body: &str) -> String {
    format!(
        r#"<!doctype html>
<html lang="ja">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title}</title>
<style>
  :root {{ color-scheme: light; }}
  * {{ box-sizing: border-box; }}
  body {{
    margin: 0;
    background: #e2e8f0;
    color: #0f172a;
    font-family: "Yu Mincho", "Hiragino Mincho ProN", "Noto Serif JP", serif;
  }}
  .toolbar {{
    position: sticky;
    top: 0;
    z-index: 10;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.7rem 1.2rem;
    background: #0f172a;
    color: #e2e8f0;
    font-family: system-ui, sans-serif;
    font-size: 0.85rem;
  }}
  .toolbar .actions {{ display: flex; gap: 0.5rem; }}
  .toolbar button, .toolbar a {{
    padding: 0.45rem 0.9rem;
    border: 1px solid #475569;
    border-radius: 0.4rem;
    background: transparent;
    color: inherit;
    font: inherit;
    text-decoration: none;
    cursor: pointer;
  }}
  .toolbar button.primary {{ background: #38bdf8; border-color: #38bdf8; color: #082f49; font-weight: 700; }}
  .paper {{
    width: 210mm;
    min-height: 297mm;
    margin: 1.2rem auto 2rem;
    padding: 14mm 14mm 12mm;
    background: #fff;
    box-shadow: 0 18px 50px rgb(15 23 42 / 0.25);
  }}
  .doc-head {{ display: flex; align-items: flex-start; justify-content: space-between; gap: 1rem; }}
  .doc-title {{
    margin: 0;
    font-size: 26pt;
    letter-spacing: 0.35em;
    color: #008080;
    font-weight: 700;
  }}
  .doc-no {{ text-align: right; font-size: 10.5pt; line-height: 1.9; }}
  .doc-no strong {{ font-size: 12pt; }}
  .party {{ margin-top: 8mm; }}
  .party .name {{
    display: inline-block;
    min-width: 90mm;
    padding-bottom: 1mm;
    border-bottom: 1px solid #008080;
    font-size: 14pt;
    font-weight: 700;
  }}
  .party .honorific {{ margin-left: 2mm; font-size: 11pt; }}
  .party .sub {{ margin-top: 1.5mm; font-size: 9.5pt; color: #475569; }}
  .lead {{ margin: 6mm 0 3mm; font-size: 10.5pt; }}
  .subject {{
    margin: 3mm 0;
    padding: 2.5mm 3mm;
    border-left: 3px solid #008080;
    background: #f0fdfa;
    font-size: 11pt;
    font-weight: 700;
  }}
  .conditions {{
    width: 100%;
    margin: 4mm 0;
    border-collapse: collapse;
    font-size: 9.5pt;
  }}
  .conditions th, .conditions td {{
    padding: 1.8mm 2mm;
    border-bottom: 1px solid #cbd5e1;
    text-align: left;
    vertical-align: top;
  }}
  .conditions th {{ width: 22mm; color: #0f766e; font-weight: 700; white-space: nowrap; }}
  table.lines {{
    width: 100%;
    margin-top: 5mm;
    border-collapse: collapse;
    font-size: 10pt;
  }}
  table.lines th {{
    padding: 2.2mm 2mm;
    border-top: 1.5px solid #008080;
    border-bottom: 1.5px solid #008080;
    background: #f0fdfa;
    color: #0f766e;
    font-size: 9pt;
  }}
  table.lines td {{
    padding: 2mm;
    border-bottom: 1px solid #dbe4ea;
    vertical-align: top;
  }}
  table.lines tfoot td {{
    padding: 2.2mm 2mm;
    border-top: 1.5px solid #008080;
    border-bottom: 3px double #008080;
    font-weight: 700;
  }}
  .num {{ text-align: right; white-space: nowrap; font-variant-numeric: tabular-nums; }}
  .center {{ text-align: center; }}
  .line-note {{ margin-top: 0.6mm; color: #64748b; font-size: 8pt; }}
  .totals {{
    width: 72mm;
    margin: 5mm 0 0 auto;
    border-collapse: collapse;
    font-size: 10.5pt;
  }}
  .totals th, .totals td {{ padding: 1.8mm 2.5mm; border-bottom: 1px solid #cbd5e1; }}
  .totals th {{ text-align: left; font-weight: 500; color: #0f766e; }}
  .totals .grand {{ border-top: 1.5px solid #008080; border-bottom: 3px double #008080; font-size: 13pt; font-weight: 700; }}
  .notes {{
    margin-top: 6mm;
    padding-top: 3mm;
    border-top: 1px dashed #cbd5e1;
    font-size: 9.5pt;
    white-space: pre-wrap;
  }}
  .company {{
    margin-top: 10mm;
    padding-top: 4mm;
    border-top: 1px solid #cbd5e1;
    text-align: right;
    font-size: 9.5pt;
    line-height: 1.7;
  }}
  .company .name {{ font-size: 12pt; font-weight: 700; }}
  .bank {{ margin-top: 3mm; text-align: right; font-size: 9pt; color: #334155; }}
  @media print {{
    @page {{ size: A4; margin: 14mm; }}
    body {{ background: #fff; }}
    .toolbar {{ display: none; }}
    .paper {{ width: auto; min-height: auto; margin: 0; padding: 0; box-shadow: none; }}
  }}
</style>
</head>
<body>
<div class="toolbar">
  <span>{title}</span>
  <span class="actions">
    <a href="/">アプリに戻る</a>
    <button class="primary" onclick="window.print()">印刷 / PDF 保存</button>
  </span>
</div>
<main class="paper">
{body}
</main>
</body>
</html>"#
    )
}

fn blank(value: &str) -> String {
    if value.trim().is_empty() {
        "—".to_string()
    } else {
        escape_html(value)
    }
}

fn company_block(company: &Company) -> String {
    format!(
        r#"<div class="company">
  <div class="name">{name}</div>
  <div>〒{postal} {address1}</div>
  <div>{address2}</div>
  <div>TEL {tel} / FAX {fax}</div>
  <div>登録番号 {registration}</div>
</div>"#,
        name = escape_html(&company.name),
        postal = escape_html(&company.postal_code),
        address1 = escape_html(&company.address1),
        address2 = escape_html(&company.address2),
        tel = escape_html(&company.tel),
        fax = escape_html(&company.fax),
        registration = escape_html(&company.registration_no),
    )
}

fn customer_block(name: &str, name2: &str, honorific: &str) -> String {
    let mut html = format!(
        r#"<div class="party">
  <div><span class="name">{name}</span><span class="honorific">{honorific}</span></div>"#,
        name = escape_html(name),
        honorific = escape_html(honorific),
    );
    if !name2.trim().is_empty() {
        html.push_str(&format!(r#"<div class="sub">{}</div>"#, escape_html(name2)));
    }
    html.push_str("</div>");
    html
}

fn line_rows(lines: &[LineItem]) -> String {
    lines
        .iter()
        .enumerate()
        .map(|(index, line)| {
            let note = if line.note.trim().is_empty() {
                String::new()
            } else {
                format!(
                    r#"<div class="line-note">{}</div>"#,
                    escape_html(&line.note)
                )
            };
            format!(
                r#"<tr>
  <td class="center">{no}</td>
  <td>{name}{note}</td>
  <td class="num">{qty}</td>
  <td class="center">{unit}</td>
  <td class="num">{price}</td>
  <td class="num">{amount}</td>
</tr>"#,
                no = index + 1,
                name = escape_html(&line.name),
                note = note,
                qty = format_qty(line.quantity),
                unit = escape_html(&line.unit),
                price = format_yen(line.unit_price),
                amount = format_yen(line.amount),
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn lines_table(lines: &[LineItem]) -> String {
    format!(
        r#"<table class="lines">
  <thead>
    <tr>
      <th style="width:10mm">No.</th>
      <th>品名</th>
      <th style="width:20mm">数量</th>
      <th style="width:14mm">単位</th>
      <th style="width:28mm">単価</th>
      <th style="width:30mm">金額</th>
    </tr>
  </thead>
  <tbody>
{rows}
  </tbody>
</table>"#,
        rows = line_rows(lines),
    )
}

fn totals_table(subtotal: i64, tax: i64, total: i64) -> String {
    format!(
        r#"<table class="totals">
  <tr><th>小計</th><td class="num">{subtotal}</td></tr>
  <tr><th>消費税 (10%)</th><td class="num">{tax}</td></tr>
  <tr class="grand"><th>合計</th><td class="num">{total}</td></tr>
</table>"#,
        subtotal = format_yen(subtotal),
        tax = format_yen(tax),
        total = format_yen(total),
    )
}

pub fn quote_page(quote: &Quote, company: &Company) -> String {
    let body = format!(
        r#"<div class="doc-head">
  <h1 class="doc-title">御 見 積 書</h1>
  <div class="doc-no">
    <div><strong>No. {number}</strong></div>
    <div>{date}</div>
  </div>
</div>
{party}
<p class="lead">下記の通り御見積申し上げます。</p>
<div class="subject">件名: {subject}</div>
<table class="conditions">
  <tr><th>有効期限</th><td>{valid_until}</td><th>受渡期日</th><td>{delivery_date}</td></tr>
  <tr><th>受渡場所</th><td>{delivery_place}</td><th>取引方法</th><td>{payment_terms}</td></tr>
</table>
{lines}
{totals}
{notes}
{company}
<div class="bank">お振込先: {bank_name} {bank_branch} {bank_type} {bank_number}</div>"#,
        number = escape_html(&quote.number),
        date = jp_date(&quote.date),
        party = customer_block(
            &quote.customer_name,
            &quote.customer_name2,
            &quote.honorific
        ),
        subject = blank(&quote.subject),
        valid_until = blank(&quote.valid_until),
        delivery_date = blank(&quote.delivery_date),
        delivery_place = blank(&quote.delivery_place),
        payment_terms = blank(&quote.payment_terms),
        lines = lines_table(&quote.lines),
        totals = totals_table(quote.subtotal, quote.tax, quote.total),
        notes = if quote.notes.trim().is_empty() {
            String::new()
        } else {
            format!(
                r#"<div class="notes"><strong>備考</strong><br>{}</div>"#,
                escape_html(&quote.notes)
            )
        },
        company = company_block(company),
        bank_name = escape_html(&company.bank_name),
        bank_branch = escape_html(&company.bank_branch),
        bank_type = escape_html(&company.bank_account_type),
        bank_number = escape_html(&company.bank_account_number),
    );

    page(&format!("見積書 {}", quote.number), &body)
}

pub fn order_page(order: &Order, company: &Company) -> String {
    let quote_ref = if order.quote_number.trim().is_empty() {
        "—".to_string()
    } else {
        escape_html(&order.quote_number)
    };
    let body = format!(
        r#"<div class="doc-head">
  <h1 class="doc-title">注 文 請 書</h1>
  <div class="doc-no">
    <div><strong>No. {number}</strong></div>
    <div>{date}</div>
  </div>
</div>
{party}
<p class="lead">下記の通りご注文承ります。</p>
<div class="subject">件名: {subject}</div>
<table class="conditions">
  <tr><th>注文番号</th><td>{customer_order_no}</td><th>納期</th><td>{delivery_date}</td></tr>
  <tr><th>見積番号</th><td>{quote_ref}</td><th>状態</th><td>{status}</td></tr>
</table>
{lines}
{totals}
{notes}
{company}"#,
        number = escape_html(&order.number),
        date = jp_date(&order.date),
        party = customer_block(
            &order.customer_name,
            &order.customer_name2,
            &order.honorific
        ),
        subject = blank(&order.subject),
        customer_order_no = blank(&order.customer_order_no),
        delivery_date = blank(&order.delivery_date),
        quote_ref = quote_ref,
        status = if order.status == "completed" {
            "完了"
        } else {
            "未完了"
        },
        lines = lines_table(&order.lines),
        totals = totals_table(order.subtotal, order.tax, order.total),
        notes = if order.notes.trim().is_empty() {
            String::new()
        } else {
            format!(
                r#"<div class="notes"><strong>備考</strong><br>{}</div>"#,
                escape_html(&order.notes)
            )
        },
        company = company_block(company),
    );

    page(&format!("注文請書 {}", order.number), &body)
}

pub fn purchase_order_page(order: &PurchaseOrder, company: &Company) -> String {
    let body = format!(
        r#"<div class="doc-head">
  <h1 class="doc-title">発 注 書</h1>
  <div class="doc-no">
    <div><strong>No. {number}</strong></div>
    <div>{date}</div>
  </div>
</div>
<div class="party">
  <div><span class="name">{supplier}</span><span class="honorific">{honorific}</span></div>
</div>
<p class="lead">下記の通り発注いたします。</p>
<table class="conditions">
  <tr><th>受注番号</th><td>{order_number}</td><th>状態</th><td>{status}</td></tr>
</table>
{lines}
{totals}
{notes}
{company}"#,
        number = escape_html(&order.number),
        date = jp_date(&order.date),
        supplier = escape_html(&order.supplier_name),
        honorific = escape_html(&order.supplier_honorific),
        order_number = blank(&order.order_number),
        status = if order.status == "received" {
            "仕入済"
        } else {
            "未仕入"
        },
        lines = lines_table(&order.lines),
        totals = totals_table(order.subtotal, order.tax, order.total),
        notes = if order.notes.trim().is_empty() {
            String::new()
        } else {
            format!(
                r#"<div class="notes"><strong>備考</strong><br>{}</div>"#,
                escape_html(&order.notes)
            )
        },
        company = company_block(company),
    );

    page(&format!("発注書 {}", order.number), &body)
}

pub fn sale_page(sale: &Sale, company: &Company) -> String {
    let body = format!(
        r#"<div class="doc-head">
  <h1 class="doc-title">納 品 書</h1>
  <div class="doc-no">
    <div><strong>No. {number}</strong></div>
    <div>{date}</div>
  </div>
</div>
{party}
<p class="lead">下記の通り納品いたします。</p>
<div class="subject">件名: {subject}</div>
<table class="conditions">
  <tr><th>受注番号</th><td>{order_number}</td><th>見積番号</th><td>{quote_number}</td></tr>
</table>
{lines}
{totals}
{notes}
{company}"#,
        number = escape_html(&sale.number),
        date = jp_date(&sale.date),
        party = customer_block(&sale.customer_name, &sale.customer_name2, &sale.honorific),
        subject = blank(&sale.subject),
        order_number = blank(&sale.order_number),
        quote_number = blank(&sale.quote_number),
        lines = lines_table(&sale.lines),
        totals = totals_table(sale.subtotal, sale.tax, sale.total),
        notes = if sale.notes.trim().is_empty() {
            String::new()
        } else {
            format!(
                r#"<div class="notes"><strong>備考</strong><br>{}</div>"#,
                escape_html(&sale.notes)
            )
        },
        company = company_block(company),
    );

    page(&format!("納品書 {}", sale.number), &body)
}

pub fn summary_page(summary: &Summary, company: &Company) -> String {
    let period = match (summary.from.is_empty(), summary.to.is_empty()) {
        (true, true) => "全期間".to_string(),
        (false, true) => format!("{} 以降", jp_date(&summary.from)),
        (true, false) => format!("{} まで", jp_date(&summary.to)),
        (false, false) => format!("{} 〜 {}", jp_date(&summary.from), jp_date(&summary.to)),
    };

    let rows = summary
        .rows
        .iter()
        .map(|row| {
            format!(
                r#"<tr>
  <td>{label}</td>
  <td class="num">{orders}</td>
  <td class="num">{quantity}</td>
  <td class="num">{subtotal}</td>
  <td class="num">{tax}</td>
  <td class="num">{total}</td>
</tr>"#,
                label = escape_html(&row.label),
                orders = row.orders,
                quantity = format_qty(row.quantity),
                subtotal = format_yen(row.subtotal),
                tax = format_yen(row.tax),
                total = format_yen(row.total),
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let doc_title = if summary.source == "sales" {
        "売 上 集 計 表"
    } else {
        "受 注 集 計 表"
    };
    let note_text = if summary.source == "sales" {
        "集計は売上データを対象にしています。商品別の消費税は明細単位で計算しているため、伝票単位の税額合計と数円異なる場合があります。商品別の件数は、その商品を含む売上の件数です。"
    } else {
        "集計は受注データを対象にしています。商品別の消費税は明細単位で計算しているため、伝票単位の税額合計と数円異なる場合があります。商品別の件数は、その商品を含む受注の件数です。"
    };

    let body = format!(
        r#"<div class="doc-head">
  <h1 class="doc-title">{doc_title}</h1>
  <div class="doc-no">
    <div><strong>{group_label}</strong></div>
    <div>{period}</div>
  </div>
</div>
<table class="conditions">
  <tr><th>対象期間</th><td>{period}</td><th>集計区分</th><td>{group_label}</td></tr>
  <tr><th>対象件数</th><td>{orders} 件</td><th>作成日</th><td>{created}</td></tr>
</table>
<table class="lines">
  <thead>
    <tr>
      <th>区分</th>
      <th style="width:16mm" class="num">件数</th>
      <th style="width:18mm" class="num">数量</th>
      <th style="width:30mm" class="num">小計</th>
      <th style="width:26mm" class="num">消費税</th>
      <th style="width:30mm" class="num">合計</th>
    </tr>
  </thead>
  <tbody>
{rows}
  </tbody>
  <tfoot>
    <tr>
      <td>合計</td>
      <td class="num">{total_orders}</td>
      <td class="num">{total_quantity}</td>
      <td class="num">{total_subtotal}</td>
      <td class="num">{total_tax}</td>
      <td class="num">{total_total}</td>
    </tr>
  </tfoot>
</table>
<p class="notes">{note_text}</p>
{company}"#,
        group_label = escape_html(&summary.group_label),
        period = escape_html(&period),
        doc_title = doc_title,
        orders = summary.totals.orders,
        created = jp_date(&today()),
        rows = rows,
        total_orders = summary.totals.orders,
        total_quantity = format_qty(summary.totals.quantity),
        total_subtotal = format_yen(summary.totals.subtotal),
        total_tax = format_yen(summary.totals.tax),
        total_total = format_yen(summary.totals.total),
        note_text = note_text,
        company = company_block(company),
    );

    page(&summary.title, &body)
}
