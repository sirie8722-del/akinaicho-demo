#!/usr/bin/env node
// ローカルで「無料枠の目安」と見積・受注の一連の流れを確認するスクリプト。
// 使い方: mise run dev を起動したまま、別ターミナルで mise run verify
// 環境変数: BASE_URL（既定 http://127.0.0.1:8787）, ROUNDS（既定 100）

const BASE = (process.env.BASE_URL ?? "http://127.0.0.1:8787").replace(/\/$/, "");
const ROUNDS = Number(process.env.ROUNDS ?? 100);
const MEMORY_LIMIT_MIB = 128;

const fmt = (value, digits = 2) => Number(value).toFixed(digits);
const fmtWall = (value) => (value === 0 ? "<1" : fmt(value, 3));

function percentile(values, p) {
  if (values.length === 0) return Number.NaN;
  const sorted = [...values].sort((a, b) => a - b);
  const index = Math.min(sorted.length - 1, Math.ceil(p * sorted.length) - 1);
  return sorted[index];
}

async function request(path, init) {
  const res = await fetch(BASE + path, init);
  const text = await res.text();
  let body = null;
  if (text) {
    try {
      body = JSON.parse(text);
    } catch {
      // HTML など JSON 以外のレスポンス
    }
  }
  return { res, text, body };
}

async function json(path, init) {
  const response = await request(path, init);
  if (!response.res.ok) {
    throw new Error(`${path}: HTTP ${response.res.status} ${response.text.slice(0, 200)}`);
  }
  return response;
}

async function postJson(path, body, method = "POST") {
  return json(path, {
    method,
    headers: { "content-type": "application/json" },
    body: JSON.stringify(body),
  });
}

async function createQuote(meta, suffix) {
  const payload = {
    customer_code: meta.customers[0].code,
    date: meta.today,
    subject: `検証用見積 ${suffix}`,
    valid_until: meta.today,
    delivery_date: meta.today,
    delivery_place: "検証場所",
    payment_terms: "納品後 30日以内",
    notes: "verify",
    status: "draft",
    lines: [
      {
        product_code: meta.products[0].code,
        quantity: 2,
        unit: meta.products[0].unit,
        unit_price: meta.products[0].unit_price,
        taxable: true,
      },
      {
        product_code: meta.products[1].code,
        quantity: 3,
        unit: meta.products[1].unit,
        unit_price: meta.products[1].unit_price,
        taxable: true,
      },
    ],
  };
  const created = await postJson("/api/quotes", payload);
  return created.body;
}

async function main() {
  console.log("Cloudflare Workers 無料枠チェック（ローカル）");
  console.log(`BASE_URL: ${BASE}\n`);

  await json("/api/health");
  const diag = await json("/api/diagnostics");
  const meta = (await json("/api/meta")).body;

  const limits = diag.body.free_plan;
  const initial = diag.body.measured;

  console.log("--- /api/diagnostics");
  console.log(
    `WASM リニアメモリ : ${fmt(initial.wasm_memory_mib, 3)} MiB / ` +
      `${limits.memory_mib_per_isolate} MiB (${fmt(initial.wasm_memory_percent_of_limit, 2)}%)`
  );
  console.log(`CPU 制限          : ${limits.cpu_ms_per_request} ms / リクエスト`);
  console.log(`リクエスト上限    : ${limits.requests_per_day.toLocaleString("ja-JP")} / 日`);
  console.log(
    `CPU 制限シグナル  : supported=${diag.body.cpu_limit_signal.supported}, ` +
      `near_limit=${diag.body.cpu_limit_signal.near_limit}`
  );
  console.log(`マスタ            : 得意先 ${meta.customers.length} 件 / 商品 ${meta.products.length} 件`);

  const quoteList = (await json("/api/quotes")).body;
  const orderList = (await json("/api/orders")).body;
  console.log(`デモデータ        : 見積 ${quoteList.length} 件 / 受注 ${orderList.length} 件\n`);
  if (quoteList.length < 100 || orderList.length < 100) {
    throw new Error("デモデータが 100 件ずつありません");
  }

  // --- ダッシュボード -------------------------------------------------------------
  console.log("--- ダッシュボード");
  const dashboard = (await json("/api/dashboard")).body;
  console.log(
    `KPI               : 当月売上 ¥${dashboard.kpis.sales_month.toLocaleString("ja-JP")} / ` +
      `売掛 ¥${dashboard.kpis.receivable.toLocaleString("ja-JP")} / ` +
      `受注残 ¥${dashboard.kpis.order_remaining.toLocaleString("ja-JP")}`
  );
  console.log(
    `推移 / ランキング : ${dashboard.months.length} か月 / ` +
      `得意先 ${dashboard.top_customers.length} 件 / 商品 ${dashboard.top_products.length} 件`
  );
  console.log(
    `対応待ち          : 見積 ${dashboard.kpis.pending_quotes} / ` +
      `受注残 ${dashboard.kpis.order_remaining_count} / ` +
      `未仕入 ${dashboard.kpis.open_purchase_orders}\n`
  );
  if (dashboard.months.length !== 12) {
    throw new Error("ダッシュボードの月次推移が 12 か月分ありません");
  }
  if (dashboard.top_customers.length !== 5 || dashboard.top_products.length !== 5) {
    throw new Error("ダッシュボードのランキングが 5 件ずつありません");
  }
  if (dashboard.activities.length === 0) {
    throw new Error("ダッシュボードの最近の動きが空です");
  }

  // --- 集計表 ---------------------------------------------------------------
  console.log("--- 集計表");
  const orderTotal = orderList.reduce((sum, order) => sum + order.total, 0);
  for (const group of ["customer", "product", "month"]) {
    const summary = (await json(`/api/reports/summary?group=${group}`)).body;
    console.log(
      `${summary.group_label.padEnd(5)}: ${summary.rows.length} 行 / ` +
        `件数 ${summary.totals.orders} / 合計 ¥${summary.totals.total.toLocaleString("ja-JP")}`
    );
    if (summary.rows.length === 0) {
      throw new Error(`集計結果が空です: ${group}`);
    }
    if ((group === "customer" || group === "month") && summary.totals.total !== orderTotal) {
      throw new Error(`集計合計が受注合計と一致しません: ${group}`);
    }
  }
  const summaryPrint = await request("/reports/summary/print?group=customer");
  if (!summaryPrint.res.ok || !summaryPrint.text.includes("受 注 集 計 表")) {
    throw new Error("集計表の帳票が出力できません");
  }
  console.log(`集計表出力        : ${summaryPrint.text.length} bytes\n`);

  // --- 見積→受注→売上→入金 / 受注→発注→仕入→支払 ------------------------------
  console.log("--- 販売・調達フロー");
  const sales = (await json("/api/sales")).body;
  const purchaseOrders = (await json("/api/purchase-orders")).body;
  const purchases = (await json("/api/purchases")).body;
  console.log(
    `初期データ        : 売上 ${sales.length} / 発注 ${purchaseOrders.length} / 仕入 ${purchases.length}`
  );

  const openOrder = orderList.find((order) =>
    order.lines.every(
      (line) => line.sold_quantity === 0 && line.purchased_quantity === 0
    )
  );
  if (!openOrder) throw new Error("フロー検証に使える未処理の受注がありません");
  const flowLine = openOrder.lines[0];

  const newSale = await postJson(`/api/orders/${openOrder.id}/sales`, {
    date: meta.today,
    notes: "verify",
    lines: [{ line_id: flowLine.id, quantity: 1 }],
  });
  console.log(`売上計上          : ${newSale.body.number} (受注 ${openOrder.number})`);

  const newPo = await postJson(`/api/orders/${openOrder.id}/purchase`, {
    supplier_code: meta.suppliers[0].code,
    date: meta.today,
    notes: "verify",
    lines: [{ line_id: flowLine.id, quantity: 1 }],
  });
  console.log(`発注作成          : ${newPo.body.number} (仕入先 ${newPo.body.supplier_name})`);

  const poPrint = await request(`/purchase-orders/${newPo.body.id}/print`);
  if (!poPrint.res.ok || !poPrint.text.includes("発 注 書")) {
    throw new Error("発注書の帳票が出力できません");
  }

  const received = await postJson(
    `/api/purchase-orders/${newPo.body.id}/receive`,
    { date: meta.today }
  );
  console.log(`仕入計上          : ${received.body.number} (発注 ${newPo.body.number})`);

  const receipt = await postJson("/api/receipts", {
    customer_code: openOrder.customer_code,
    date: meta.today,
    amount: newSale.body.total,
    method: "振込",
  });
  const supplierPayment = await postJson("/api/supplier-payments", {
    supplier_code: meta.suppliers[0].code,
    date: meta.today,
    amount: received.body.total,
    method: "振込",
  });
  console.log(`入金 / 支払       : ${receipt.body.number} / ${supplierPayment.body.number}`);

  const salePrint = await request(`/sales/${newSale.body.id}/print`);
  if (!salePrint.res.ok || !salePrint.text.includes("納 品 書")) {
    throw new Error("納品書の帳票が出力できません");
  }
  console.log(`納品書出力        : ${salePrint.text.length} bytes`);

  const month = meta.today.slice(0, 7);
  const closingPreview = (await json(`/api/closings/preview?kind=customer&month=${month}`)).body;
  const closingRun = await request("/api/closings", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ kind: "customer", month }),
  });
  const closingOk = closingRun.res.status === 201 || closingRun.res.status === 409;
  if (!closingOk) throw new Error(`締処理に失敗しました: HTTP ${closingRun.res.status}`);
  console.log(
    `締処理            : ${month} プレビュー ${closingPreview.rows.length} 件 / ` +
      `実行 ${closingRun.res.status === 201 ? "OK" : "済み"}`
  );

  const salesByMonth = (await json("/api/reports/summary?source=sales&group=month")).body;
  const orderRemaining = (await json("/api/reports/order-remaining")).body;
  const purchaseRemaining = (await json("/api/reports/purchase-remaining")).body;
  const balances = (await json("/api/reports/balances")).body;
  if (salesByMonth.totals.total <= 0) throw new Error("売上集計が空です");
  if (orderRemaining.rows.length === 0) throw new Error("受注残が空です");
  if (purchaseRemaining.rows.length === 0) throw new Error("発注残が空です");
  console.log(
    `追加集計          : 売上 ${salesByMonth.totals.total} / 受注残 ${orderRemaining.rows.length} 行 / ` +
      `発注残 ${purchaseRemaining.rows.length} 行 / 売掛 ${balances.customer_total}`
  );
  console.log("");

  // ウォームアップ
  for (let i = 0; i < 10; i += 1) {
    await request("/api/quotes");
  }

  const serverMs = [];
  const clientMs = [];
  const memKib = [];

  for (let i = 0; i < ROUNDS; i += 1) {
    const started = performance.now();
    const response = await request("/api/quotes");
    clientMs.push(performance.now() - started);

    if (!response.res.ok) {
      throw new Error(`GET /api/quotes: HTTP ${response.res.status}`);
    }

    const timing = response.res.headers.get("server-timing") ?? "";
    const match = timing.match(/dur=([0-9.]+)/);
    if (match) serverMs.push(Number(match[1]));

    const memory = response.res.headers.get("x-wasm-memory-kib");
    if (memory) memKib.push(Number(memory));
  }

  console.log(`--- GET /api/quotes x ${ROUNDS}`);
  console.log(
    `サーバ内 wall time: p50=${fmtWall(percentile(serverMs, 0.5))}ms  ` +
      `p95=${fmtWall(percentile(serverMs, 0.95))}ms  max=${fmtWall(Math.max(...serverMs))}ms`
  );
  console.log(
    `クライアント往復  : p50=${fmt(percentile(clientMs, 0.5), 2)}ms  ` +
      `p95=${fmt(percentile(clientMs, 0.95), 2)}ms  max=${fmt(Math.max(...clientMs), 2)}ms`
  );
  console.log(`WASM メモリ max   : ${fmt(Math.max(...memKib) / 1024, 3)} MiB`);
  console.log("[INFO] workerd の performance.now() は約 1ms 分解能のため、0 は <1ms と表示しています。\n");

  // --- 見積 → 更新 → 受注変換 → 帳票 ---------------------------------------
  console.log("--- 見積・受注フロー");
  const quote = await createQuote(meta, "flow");
  console.log(`見積作成          : ${quote.number} 小計=${quote.subtotal} 税=${quote.tax} 合計=${quote.total}`);

  if (quote.subtotal !== 266500 || quote.tax !== 26650 || quote.total !== 293150) {
    throw new Error(`見積の計算が想定と違います: ${JSON.stringify({ subtotal: quote.subtotal, tax: quote.tax, total: quote.total })}`);
  }

  const updated = await postJson(
    `/api/quotes/${quote.id}`,
    {
      ...quote,
      subject: "検証用見積（更新後）",
      status: "sent",
      lines: quote.lines,
    },
    "PUT"
  );
  console.log(`見積更新          : ${updated.body.number} status=${updated.body.status}`);

  const converted = await json(`/api/quotes/${quote.id}/convert`, { method: "POST" });
  const order = converted.body;
  console.log(`受注変換          : ${order.number} (quote=${order.quote_number}) 合計=${order.total}`);

  const quoteAfter = (await json(`/api/quotes/${quote.id}`)).body;
  if (quoteAfter.status !== "accepted" || quoteAfter.order_id !== order.id) {
    throw new Error("見積が受注済みに更新されていません");
  }

  const quotePrint = await request(`/quotes/${quote.id}/print`);
  const orderPrint = await request(`/orders/${order.id}/print`);
  if (!quotePrint.res.ok || !quotePrint.text.includes("御 見 積 書")) {
    throw new Error("見積書の帳票が出力できません");
  }
  if (!orderPrint.res.ok || !orderPrint.text.includes("注 文 請 書")) {
    throw new Error("注文請書の帳票が出力できません");
  }
  console.log(`帳票出力          : 見積書 ${quotePrint.text.length} bytes / 注文請書 ${orderPrint.text.length} bytes`);

  await json(`/api/orders/${order.id}`, { method: "DELETE" });
  await json(`/api/quotes/${quote.id}`, { method: "DELETE" });
  console.log("後片付け          : 受注・見積を削除\n");

  // --- CRUD を繰り返してメモリが増え続けないか確認 ---------------------------
  const before = (await json("/api/diagnostics")).body.measured.wasm_memory_mib;
  for (let i = 0; i < 20; i += 1) {
    const q = await createQuote(meta, `loop-${i}`);
    const saved = await postJson(
      `/api/quotes/${q.id}`,
      { ...q, subject: `更新 ${i}`, status: "sent", lines: q.lines },
      "PUT"
    );
    const convertedLoop = await json(`/api/quotes/${saved.body.id}/convert`, { method: "POST" });
    await json(`/api/orders/${convertedLoop.body.id}`, { method: "DELETE" });
    await json(`/api/quotes/${q.id}`, { method: "DELETE" });
  }
  const after = (await json("/api/diagnostics")).body.measured;

  console.log("--- 見積 CRUD 20 回後のメモリ");
  console.log(
    `before=${fmt(before, 3)} MiB  after=${fmt(after.wasm_memory_mib, 3)} MiB  ` +
      `diff=${fmt(after.wasm_memory_mib - before, 3)} MiB\n`
  );

  const problems = [];
  if (initial.wasm_memory_mib > MEMORY_LIMIT_MIB) {
    problems.push("WASM メモリが 128 MiB を超えています");
  }
  if (after.wasm_memory_mib > MEMORY_LIMIT_MIB) {
    problems.push("CRUD 後の WASM メモリが 128 MiB を超えています");
  }
  if (serverMs.length === 0) {
    problems.push("Server-Timing ヘッダが取得できません");
  }

  console.log("--- 判定");
  if (problems.length === 0) {
    console.log("[OK] メモリは無料枠の範囲内です。");
  } else {
    for (const problem of problems) console.log(`[NG] ${problem}`);
  }
  console.log("[INFO] CPU 時間はローカルでは実測できません。上記は wall time です。");
  console.log(
    "[INFO] 正確な CPU 時間・1 日あたりのリクエスト数は、デプロイ後に Workers Logs / ダッシュボードで確認してください。"
  );

  if (problems.length > 0) process.exitCode = 1;
}

main().catch((error) => {
  console.error(`[ERROR] ${error.message}`);
  console.error("`mise run dev` で開発サーバーを起動してから実行してください。");
  process.exitCode = 1;
});
