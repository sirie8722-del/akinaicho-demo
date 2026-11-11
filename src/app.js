const app = document.getElementById("app");
      const state = {
        meta: null,
        dashboard: null,
        quotes: [],
        orders: [],
        sales: [],
        purchaseOrders: [],
        purchases: [],
        receipts: [],
        supplierPayments: [],
        closings: [],
        closing: { kind: "customer", month: "", preview: null },
        summary: {
          report: "summary",
          source: "orders",
          from: "",
          to: "",
          group: "customer",
          data: null,
        },
      };

      const QUOTE_STATUS = {
        draft: ["下書き", "draft"],
        sent: ["提出済", "sent"],
        accepted: ["受注済", "accepted"],
      };
      const ORDER_STATUS = {
        open: ["未完了", "open"],
        completed: ["完了", "completed"],
      };

      const esc = (value) =>
        String(value ?? "").replace(/[&<>"']/g, (ch) => ({
          "&": "&amp;",
          "<": "&lt;",
          ">": "&gt;",
          '"': "&quot;",
          "'": "&#39;",
        })[ch]);

      const yen = (value) => Number(value || 0).toLocaleString("ja-JP");
      const qty = (value) => {
        const n = Number(value);
        return Number.isInteger(n) ? String(n) : String(Number(n.toFixed(2)));
      };
      const today = () => state.meta?.today || new Date().toISOString().slice(0, 10);

      async function api(path, options = {}) {
        const init = { ...options, headers: { ...(options.headers || {}) } };
        if (init.body !== undefined) init.headers["content-type"] = "application/json";
        const res = await fetch(path, init);
        if (res.status === 204) return null;
        const data = await res.json().catch(() => null);
        if (!res.ok) throw new Error(data?.error || `HTTP ${res.status}`);
        return data;
      }

      let toastTimer;
      function toast(message, type = "ok") {
        const el = document.getElementById("toast");
        el.textContent = message;
        el.className = `show ${type === "error" ? "error" : ""}`;
        clearTimeout(toastTimer);
        toastTimer = setTimeout(() => {
          el.className = "";
        }, 3200);
      }

      function navigate(path) {
        location.hash = path;
      }

      function statusBadge(status, map) {
        const [label, cls] = map[status] || [status, ""];
        return `<span class="status ${cls}">${esc(label)}</span>`;
      }

      function setActiveNav(view) {
        document.querySelectorAll("[data-nav]").forEach((button) => {
          button.classList.toggle("active", button.dataset.nav === view);
        });
      }

      async function refresh() {
        const [
          dashboard,
          quotes,
          orders,
          sales,
          purchaseOrders,
          purchases,
          receipts,
          supplierPayments,
          closings,
        ] = await Promise.all([
          api("/api/dashboard"),
          api("/api/quotes"),
          api("/api/orders"),
          api("/api/sales"),
          api("/api/purchase-orders"),
          api("/api/purchases"),
          api("/api/receipts"),
          api("/api/supplier-payments"),
          api("/api/closings"),
        ]);
        state.dashboard = dashboard;
        state.quotes = quotes;
        state.orders = orders;
        state.sales = sales;
        state.purchaseOrders = purchaseOrders;
        state.purchases = purchases;
        state.receipts = receipts;
        state.supplierPayments = supplierPayments;
        state.closings = closings;
      }

      async function loadMetrics() {
        const element = document.getElementById("metrics");
        if (!element) return;
        try {
          const diag = await api("/api/diagnostics");
          element.textContent = `WASM ${diag.measured.wasm_memory_mib.toFixed(2)} / 128 MiB`;
        } catch {
          element.textContent = "WASM —";
        }
      }

      function currentRoute() {
        const parts = location.hash.replace(/^#/, "").split("/").filter(Boolean);
        if (!parts.length || parts[0] === "dashboard") return { type: "dashboard" };
        if (parts[0] === "summary") return { type: "summary" };
        if (parts[0] === "recordings") return { type: "recordings" };
        if (parts[0] === "payments") return { type: "payments" };
        if (parts[0] === "closings") return { type: "closings" };
        if (parts[0] === "sales") return { type: "sales" };
        if (parts[0] === "purchase-orders") return { type: "purchase-orders" };
        if (parts[0] === "purchases") return { type: "purchases" };
        if (parts[0] === "orders") {
          if (parts[1] === "new") return { type: "order-form" };
          if (parts[1] && parts[2] === "sales") return { type: "sale-form", id: Number(parts[1]) };
          if (parts[1] && parts[2] === "purchase") {
            return { type: "purchase-form", id: Number(parts[1]) };
          }
          if (parts[1] && parts[2] === "edit") return { type: "order-form", id: Number(parts[1]) };
          return { type: "orders" };
        }
        if (parts[0] === "quotes") {
          if (parts[1] === "new") return { type: "quote-form" };
          if (parts[1] && parts[2] === "edit") return { type: "quote-form", id: Number(parts[1]) };
          return { type: "quotes" };
        }
        return { type: "quotes" };
      }

      function render() {
        const route = currentRoute();
        if (route.type === "dashboard") {
          setActiveNav("dashboard");
          renderDashboard();
        } else if (route.type === "orders") {
          setActiveNav("orders");
          renderOrderList();
        } else if (route.type === "order-form") {
          setActiveNav("orders");
          renderOrderForm(route.id);
        } else if (route.type === "sale-form") {
          setActiveNav("orders");
          renderSaleForm(route.id);
        } else if (route.type === "purchase-form") {
          setActiveNav("orders");
          renderPurchaseForm(route.id);
        } else if (route.type === "quote-form") {
          setActiveNav("quotes");
          renderQuoteForm(route.id);
        } else if (route.type === "quotes") {
          setActiveNav("quotes");
          renderQuoteList();
        } else if (route.type === "sales") {
          setActiveNav("sales");
          renderSalesList();
        } else if (route.type === "purchase-orders") {
          setActiveNav("purchase-orders");
          renderPurchaseOrderList();
        } else if (route.type === "purchases") {
          setActiveNav("purchases");
          renderPurchaseList();
        } else if (route.type === "payments") {
          setActiveNav("payments");
          renderPayments();
        } else if (route.type === "closings") {
          setActiveNav("closings");
          renderClosings();
        } else if (route.type === "summary") {
          setActiveNav("summary");
          renderSummary();
        } else if (route.type === "recordings") {
          setActiveNav("recordings");
          if (window.RecorderView) {
            window.RecorderView.render();
          } else {
            app.innerHTML = `<div class="card not-found">操作記録を読み込めませんでした。</div>`;
          }
        } else {
          setActiveNav("dashboard");
          renderDashboard();
        }
      }

      function setupTable({ tableId, searchId, filters = [] }) {
        const table = document.getElementById(tableId);
        if (!table) return;
        const tbody = table.tBodies[0];
        const search = searchId ? document.getElementById(searchId) : null;
        const activeFilters = filters
          .map((filter) => ({ key: filter.key, element: document.getElementById(filter.id) }))
          .filter((filter) => filter.element);

        const apply = () => {
          const query = search ? search.value.trim().toLowerCase() : "";
          [...tbody.rows].forEach((row) => {
            if (!row.dataset.search) return;
            let visible = !query || row.dataset.search.toLowerCase().includes(query);
            for (const filter of activeFilters) {
              if (filter.element.value && row.dataset[filter.key] !== filter.element.value) {
                visible = false;
              }
            }
            row.hidden = !visible;
          });
        };

        if (search) search.addEventListener("input", apply);
        activeFilters.forEach((filter) => filter.element.addEventListener("change", apply));

        let sortIndex = -1;
        let direction = 1;
        const headers = [...table.tHead.rows[0].cells];
        headers.forEach((th, index) => {
          if (th.dataset.nosort !== undefined) return;
          th.classList.add("sortable");
          th.addEventListener("click", () => {
            direction = sortIndex === index ? -direction : 1;
            sortIndex = index;
            headers.forEach((header) => header.classList.remove("asc", "desc"));
            th.classList.add(direction === 1 ? "asc" : "desc");

            const rows = [...tbody.rows].filter((row) => row.dataset.search);
            rows.sort((a, b) => {
              const aCell = a.cells[index];
              const bCell = b.cells[index];
              const aRaw = aCell?.dataset.sort ?? aCell?.textContent.trim() ?? "";
              const bRaw = bCell?.dataset.sort ?? bCell?.textContent.trim() ?? "";
              const aNumber = Number(aRaw.replace(/[¥,]/g, ""));
              const bNumber = Number(bRaw.replace(/[¥,]/g, ""));
              const numeric =
                aCell?.classList.contains("num") &&
                !Number.isNaN(aNumber) &&
                !Number.isNaN(bNumber);
              const comparison = numeric
                ? aNumber - bNumber
                : aRaw.localeCompare(bRaw, "ja", { numeric: true });
              return comparison * direction;
            });
            rows.forEach((row) => tbody.append(row));
          });
        });
      }

      function compactYen(value) {
        const n = Number(value || 0);
        if (Math.abs(n) >= 100000000) return `${(n / 100000000).toFixed(1)}億`;
        if (Math.abs(n) >= 10000) return `${Math.round(n / 10000)}万`;
        return yen(n);
      }

      function jpDate(value) {
        const [year, month, day] = String(value || "").split("-");
        if (!year || !month || !day) return value || "";
        return `${year}年${Number(month)}月${Number(day)}日`;
      }

      function niceMax(value) {
        if (!value || value <= 0) return 1;
        const power = 10 ** Math.floor(Math.log10(value));
        const n = value / power;
        const step = n <= 1 ? 1 : n <= 2 ? 2 : n <= 5 ? 5 : 10;
        return step * power;
      }

      const STATUS_COLORS = {
        draft: "#c9c9c9",
        sent: "#70584f",
        accepted: "#4f705f",
        open: "#f06530",
        completed: "#4f705f",
      };

      function kpiCard(label, value, foot) {
        return `<article class="kpi">
          <div class="kpi-head"><span class="kpi-label">${esc(label)}</span></div>
          <div class="kpi-value">${value}</div>
          <div class="kpi-foot">${foot}</div>
        </article>`;
      }

      function rankList(rows, tone, emptyLabel) {
        if (!rows.length) return `<div class="empty">${esc(emptyLabel)}</div>`;
        const max = rows[0].total || 1;
        return `<div class="rank-list ${tone}">${rows
          .map(
            (row, index) => `<div class="rank-row">
              <span class="rank-no">${String(index + 1).padStart(2, "0")}</span>
              <div class="rank-main">
                <div class="rank-top">
                  <span class="rank-name" title="${esc(row.name)}">${esc(row.name)}</span>
                  <span class="rank-value">¥${yen(row.total)}</span>
                </div>
                <div class="rank-track"><span style="width:${((row.total / max) * 100).toFixed(1)}%"></span></div>
              </div>
              <span class="rank-meta">${(row.share * 100).toFixed(1)}%</span>
            </div>`
          )
          .join("")}</div>`;
      }

      function renderDashboard() {
        const data = state.dashboard;
        if (!data) {
          app.innerHTML = `<div class="card not-found">読み込み中です…</div>`;
          return;
        }
        const k = data.kpis;
        const delta =
          k.sales_prev > 0 ? ((k.sales_month - k.sales_prev) / k.sales_prev) * 100 : null;
        const deltaHtml =
          delta === null
            ? `<span class="kpi-muted">前月同期実績なし</span>`
            : `<span class="delta ${delta >= 0 ? "up" : "down"}">${delta >= 0 ? "▲" : "▼"} ${Math.abs(delta).toFixed(1)}%</span><span class="kpi-muted">前月同期比</span>`;

        const kpis = [
          kpiCard(
            "当月売上",
            `¥${yen(k.sales_month)}`,
            `${deltaHtml}<span class="kpi-muted">／ 前月同期 ¥${yen(k.sales_prev)}</span>`
          ),
          kpiCard(
            "当月入金",
            `¥${yen(k.receipts_month)}`,
            `<span class="kpi-muted">${k.receipt_count} 件の入金</span>`
          ),
          kpiCard(
            "売掛残高",
            `¥${yen(k.receivable)}`,
            `<span class="kpi-muted">得意先 ${state.meta.customers.length} 社</span>`
          ),
          kpiCard(
            "受注残高",
            `¥${yen(k.order_remaining)}`,
            `<span class="kpi-muted">${k.order_remaining_count} 件の受注</span>`
          ),
          kpiCard(
            "当月仕入",
            `¥${yen(k.purchases_month)}`,
            `<span class="kpi-muted">${k.purchase_count} 件の仕入</span>`
          ),
          kpiCard(
            "買掛残高",
            `¥${yen(k.payable)}`,
            `<span class="kpi-muted">仕入先 ${state.meta.suppliers.length} 社</span>`
          ),
        ].join("");

        const chartMax = niceMax(
          Math.max(1, ...data.months.map((month) => Math.max(month.sales, month.purchases)))
        );
        const chartColumns = data.months
          .map((month) => {
            const salesHeight = (month.sales / chartMax) * 100;
            const purchaseHeight = (month.purchases / chartMax) * 100;
            return `<div class="chart-col" title="${esc(month.month)} 売上 ¥${yen(
              month.sales
            )} / 仕入 ¥${yen(month.purchases)}">
              <div class="chart-bars">
                <span class="chart-bar sales" style="height:${month.sales > 0 ? Math.max(salesHeight, 1.5) : 0}%;min-height:${month.sales > 0 ? "2px" : "0"}"></span>
                <span class="chart-bar purchases" style="height:${month.purchases > 0 ? Math.max(purchaseHeight, 1.5) : 0}%;min-height:${month.purchases > 0 ? "2px" : "0"}"></span>
              </div>
              <span class="chart-label ${month.month === data.month ? "current" : ""}">${esc(
                month.month === data.month ? "今月" : month.label
              )}</span>
            </div>`;
          })
          .join("");

        const alerts = [
          {
            count: k.pending_quotes,
            text: "未処理の見積",
            sub: "下書き・提出済",
            href: "#/quotes",
            ok: k.pending_quotes === 0,
          },
          {
            count: k.order_remaining_count,
            text: "受注残のある受注",
            sub: `残高 ¥${yen(k.order_remaining)}`,
            href: "#/orders",
            ok: k.order_remaining_count === 0,
          },
          {
            count: k.open_purchase_orders,
            text: "未仕入の発注",
            sub: "仕入計上待ち",
            href: "#/purchase-orders",
            ok: k.open_purchase_orders === 0,
          },
          {
            count: k.receipt_count,
            text: "今月の入金",
            sub: `¥${yen(k.receipts_month)}`,
            href: "#/payments",
            ok: true,
          },
        ];
        const alertItems = alerts
          .map(
            (alert) => `<li class="${alert.ok ? "ok" : ""}">
              <a href="${alert.href}">
                <span class="alert-count">${alert.count}</span>
                <span class="alert-text">${esc(alert.text)}<span class="alert-sub">${esc(
                  alert.sub
                )}</span></span>
                <span class="alert-arrow">→</span>
              </a>
            </li>`
          )
          .join("");

        const statusBlock = (title, rows) => {
          const total = rows.reduce((sum, row) => sum + row.count, 0);
          if (!total) {
            return `<div class="status-block">
              <div class="status-block-title">${esc(title)}<span>0 件</span></div>
              <div class="empty small">データがありません。</div>
            </div>`;
          }
          return `<div class="status-block">
            <div class="status-block-title">${esc(title)}<span>${total} 件</span></div>
            <div class="seg-bar">${rows
              .map(
                (row) =>
                  `<span class="seg ${esc(row.key)}" style="width:${(
                    (row.count / total) * 100
                  ).toFixed(1)}%"></span>`
              )
              .join("")}</div>
            <div class="seg-legend">${rows
              .map(
                (row) =>
                  `<span style="--dot:${STATUS_COLORS[row.key] || "#cbd5e1"}">${esc(
                    row.label
                  )} ${row.count}</span>`
              )
              .join("")}</div>
          </div>`;
        };

        const activities = data.activities
          .map(
            (activity) => `<tr>
              <td><span class="activity-kind ${esc(activity.kind)}">${esc(activity.label)}</span></td>
              <td>${esc(activity.date)}</td>
              <td class="mono">${esc(activity.number)}</td>
              <td>${esc(activity.party)}</td>
              <td class="num">¥${yen(activity.amount)}</td>
              <td class="actions"><a class="link" href="${esc(activity.href)}">開く</a></td>
            </tr>`
          )
          .join("");

        app.innerHTML = `<section>
          <div class="dash-bar">
            <div class="dash-date">${esc(jpDate(data.as_of))} 時点 ・ ${esc(data.month_label)}</div>
            <div class="dash-actions">
              <button type="button" class="primary" id="dash-new-quote">＋ 新規見積</button>
              <button type="button" class="secondary" id="dash-new-order">＋ 新規受注</button>
              <button type="button" class="secondary" id="dash-receipt">入金を登録</button>
              <button type="button" class="secondary" id="dash-closing">締処理</button>
            </div>
          </div>

          <div class="kpi-grid">${kpis}</div>

          <div class="dash-grid">
            <div class="card chart-card">
              <div class="card-head">
                <span>売上・仕入の推移</span>
                <span class="legend"><i class="sales"></i>売上 <i class="purchases"></i>仕入</span>
              </div>
              <div class="chart-body">
                <div class="chart-axis">
                  <span>¥${compactYen(chartMax)}</span>
                  <span>¥${compactYen(chartMax / 2)}</span>
                  <span>0</span>
                </div>
                <div class="chart-plot">
                  <div class="chart-gridlines"><span></span><span></span><span></span></div>
                  <div class="chart-columns">${chartColumns}</div>
                </div>
              </div>
            </div>
            <div class="card alerts-card">
              <div class="card-head"><span>対応待ち</span><span class="card-sub">${alerts.filter((alert) => !alert.ok).length} 件</span></div>
              <ul class="alert-list">${alertItems}</ul>
            </div>
          </div>

          <div class="card ledger-card">
            <div class="ledger-grid">
              <section class="ledger-col">
                <div class="card-head"><span>得意先別売上</span><span class="card-sub">TOP 5</span></div>
                ${rankList(data.top_customers, "customers", "売上データがありません。")}
              </section>
              <section class="ledger-col">
                <div class="card-head"><span>商品別売上</span><span class="card-sub">TOP 5</span></div>
                ${rankList(data.top_products, "products", "売上データがありません。")}
              </section>
              <section class="ledger-col">
                <div class="card-head"><span>伝票ステータス</span></div>
                <div class="status-blocks">
                  ${statusBlock("見積", data.quote_status)}
                  ${statusBlock("受注", data.order_status)}
                </div>
              </section>
            </div>
          </div>

          <div class="card spaced">
            <div class="card-head"><span>最近の動き</span><a class="link" href="#/sales">売上一覧へ</a></div>
            <div class="table-wrap">
              <table class="list">
                <thead><tr><th>種別</th><th>日付</th><th>番号</th><th>取引先</th><th class="num">金額</th><th></th></tr></thead>
                <tbody>${activities || `<tr><td colspan="6" class="empty">まだ伝票がありません。</td></tr>`}</tbody>
              </table>
            </div>
          </div>
        </section>`;

        document.getElementById("dash-new-quote").addEventListener("click", () => navigate("/quotes/new"));
        document.getElementById("dash-new-order").addEventListener("click", () => navigate("/orders/new"));
        document.getElementById("dash-receipt").addEventListener("click", () => navigate("/payments"));
        document.getElementById("dash-closing").addEventListener("click", () => navigate("/closings"));
      }

      function renderQuoteList() {
        const rows = state.quotes
          .map((quote) => {
            const search = `${quote.number} ${quote.date} ${quote.customer_name} ${quote.subject}`;
            const orderAction =
              quote.status === "accepted" && quote.order_id
                ? `<button type="button" class="link" data-action="order-edit" data-id="${quote.order_id}">受注を開く</button>`
                : `<button type="button" class="link" data-action="quote-convert" data-id="${quote.id}">受注へ</button>`;
            return `<tr data-search="${esc(search)}" data-status="${esc(quote.status)}" data-customer="${esc(quote.customer_code)}">
              <td class="mono">${esc(quote.number)}</td>
              <td>${esc(quote.date)}</td>
              <td>${esc(quote.customer_name)}</td>
              <td class="subject-cell" title="${esc(quote.subject)}">${esc(quote.subject || "—")}</td>
              <td class="num">¥${yen(quote.total)}</td>
              <td>${statusBadge(quote.status, QUOTE_STATUS)}</td>
              <td class="actions">
                <button type="button" class="link" data-action="quote-edit" data-id="${quote.id}">${quote.status === "accepted" ? "表示" : "編集"}</button>
                <button type="button" class="link" data-action="quote-print" data-id="${quote.id}">見積書</button>
                ${orderAction}
                <button type="button" class="link danger" data-action="quote-delete" data-id="${quote.id}">削除</button>
              </td>
            </tr>`;
          })
          .join("");

        app.innerHTML = `<section>
          <div class="card">
            <div class="toolbar">
              <input type="search" id="quote-search" placeholder="番号・得意先・件名で検索" />
              <select id="quote-status-filter" aria-label="状態で絞り込み">
                <option value="">すべての状態</option>
                <option value="draft">下書き</option>
                <option value="sent">提出済</option>
                <option value="accepted">受注済</option>
              </select>
              <button type="button" class="primary" id="new-quote">＋ 新規見積</button>
            </div>
            <div class="table-wrap">
              <table class="list" id="quote-table">
                <thead>
                  <tr><th>番号</th><th>日付</th><th>得意先</th><th class="subject-col">件名</th><th class="num">金額</th><th>状態</th><th data-nosort></th></tr>
                </thead>
                <tbody>${rows || `<tr><td colspan="7" class="empty">見積がありません。「新規見積」から作成してください。</td></tr>`}</tbody>
              </table>
            </div>
          </div>
        </section>`;

        document.getElementById("new-quote").addEventListener("click", () => navigate("/quotes/new"));
        setupTable({
          tableId: "quote-table",
          searchId: "quote-search",
          filters: [{ id: "quote-status-filter", key: "status" }],
        });
      }

      function renderOrderList() {
        const rows = state.orders
          .map((order) => {
            const search = `${order.number} ${order.date} ${order.customer_name} ${order.subject} ${order.customer_order_no}`;
            const saleRemaining = order.lines.reduce(
              (sum, line) => sum + Math.max(0, line.quantity - line.sold_quantity),
              0
            );
            const purchaseRemaining = order.lines.reduce(
              (sum, line) => sum + Math.max(0, line.quantity - line.purchased_quantity),
              0
            );
            const hasFlow = order.lines.some(
              (line) => line.sold_quantity > 0 || line.purchased_quantity > 0
            );
            return `<tr data-search="${esc(search)}" data-status="${esc(order.status)}" data-customer="${esc(order.customer_code)}">
              <td class="mono">${esc(order.number)}</td>
              <td>${esc(order.date)}</td>
              <td>${esc(order.customer_name)}</td>
              <td class="mono">${esc(order.customer_order_no || "—")}</td>
              <td class="subject-cell" title="${esc(order.subject)}">${esc(order.subject || "—")}</td>
              <td class="num">¥${yen(order.total)}</td>
              <td>${statusBadge(order.status, ORDER_STATUS)}</td>
              <td class="actions">
                <button type="button" class="link" data-action="order-edit" data-id="${order.id}">${hasFlow ? "表示" : "編集"}</button>
                ${saleRemaining > 0 ? `<button type="button" class="link" data-action="order-sale" data-id="${order.id}">売上へ</button>` : ""}
                ${purchaseRemaining > 0 ? `<button type="button" class="link" data-action="order-purchase" data-id="${order.id}">発注へ</button>` : ""}
                <button type="button" class="link" data-action="order-print" data-id="${order.id}">注文請書</button>
                <button type="button" class="link danger" data-action="order-delete" data-id="${order.id}">削除</button>
              </td>
            </tr>`;
          })
          .join("");

        app.innerHTML = `<section>
          <div class="card">
            <div class="toolbar">
              <input type="search" id="order-search" placeholder="番号・得意先・注文番号・件名で検索" />
              <select id="order-status-filter" aria-label="状態で絞り込み">
                <option value="">すべての状態</option>
                <option value="open">未完了</option>
                <option value="completed">完了</option>
              </select>
              <button type="button" class="primary" id="new-order">＋ 新規受注</button>
            </div>
            <div class="table-wrap">
              <table class="list" id="order-table">
                <thead>
                  <tr><th>番号</th><th>日付</th><th>得意先</th><th>注文番号</th><th class="subject-col">件名</th><th class="num">金額</th><th>状態</th><th data-nosort></th></tr>
                </thead>
                <tbody>${rows || `<tr><td colspan="8" class="empty">受注がありません。「新規受注」から作成してください。</td></tr>`}</tbody>
              </table>
            </div>
          </div>
        </section>`;

        document.getElementById("new-order").addEventListener("click", () => navigate("/orders/new"));
        setupTable({
          tableId: "order-table",
          searchId: "order-search",
          filters: [{ id: "order-status-filter", key: "status" }],
        });
      }

      function supplierOptions(selectedCode) {
        return state.meta.suppliers
          .map(
            (supplier) =>
              `<option value="${esc(supplier.code)}" ${supplier.code === selectedCode ? "selected" : ""}>${esc(
                supplier.code
              )} ${esc(supplier.name)}</option>`
          )
          .join("");
      }

      function renderSalesList() {
        const rows = state.sales
          .map(
            (sale) => `<tr data-search="${esc(`${sale.number} ${sale.date} ${sale.customer_name} ${sale.subject}`)}" data-customer="${esc(sale.customer_code)}">
              <td class="mono">${esc(sale.number)}</td>
              <td>${esc(sale.date)}</td>
              <td>${esc(sale.customer_name)}</td>
              <td class="mono">${esc(sale.order_number)}</td>
              <td class="subject-cell">${esc(sale.subject || "—")}</td>
              <td class="num">¥${yen(sale.total)}</td>
              <td class="actions">
                <button type="button" class="link" data-action="sale-print" data-id="${sale.id}">納品書</button>
                <button type="button" class="link danger" data-action="sale-delete" data-id="${sale.id}">削除</button>
              </td>
            </tr>`
          )
          .join("");
        app.innerHTML = `<section>
          <div class="card">
            <div class="toolbar">
              <input type="search" id="sale-search" placeholder="番号・得意先・受注番号・件名で検索" />
              <select id="sale-customer-filter" aria-label="得意先で絞り込み">
                <option value="">すべての得意先</option>
                ${state.meta.customers.map((c) => `<option value="${esc(c.code)}">${esc(c.name)}</option>`).join("")}
              </select>
            </div>
            <div class="table-wrap"><table class="list" id="sale-table">
              <thead><tr><th>番号</th><th>日付</th><th>得意先</th><th>受注番号</th><th class="subject-col">件名</th><th class="num">金額</th><th data-nosort></th></tr></thead>
              <tbody>${rows || `<tr><td colspan="7" class="empty">売上がありません。</td></tr>`}</tbody>
            </table></div>
          </div>
        </section>`;

        setupTable({
          tableId: "sale-table",
          searchId: "sale-search",
          filters: [{ id: "sale-customer-filter", key: "customer" }],
        });
      }

      function renderPurchaseOrderList() {
        const rows = state.purchaseOrders
          .map(
            (order) => `<tr data-search="${esc(`${order.number} ${order.date} ${order.supplier_name} ${order.order_number}`)}" data-status="${esc(order.status)}" data-supplier="${esc(order.supplier_code)}">
              <td class="mono">${esc(order.number)}</td>
              <td>${esc(order.date)}</td>
              <td>${esc(order.supplier_name)}</td>
              <td class="mono">${esc(order.order_number)}</td>
              <td class="num">¥${yen(order.total)}</td>
              <td>${order.status === "received" ? `<span class="status completed">仕入済</span>` : `<span class="status open">未仕入</span>`}</td>
              <td class="actions">
                <button type="button" class="link" data-action="po-print" data-id="${order.id}">発注書</button>
                ${order.status === "open" ? `<button type="button" class="link" data-action="po-receive" data-id="${order.id}">仕入計上</button>` : ""}
                ${order.status === "open" ? `<button type="button" class="link danger" data-action="po-delete" data-id="${order.id}">削除</button>` : ""}
              </td>
            </tr>`
          )
          .join("");
        app.innerHTML = `<section>
          <div class="card">
            <div class="toolbar">
              <input type="search" id="po-search" placeholder="番号・仕入先・受注番号で検索" />
              <select id="po-status-filter" aria-label="状態で絞り込み">
                <option value="">すべての状態</option>
                <option value="open">未仕入</option>
                <option value="received">仕入済</option>
              </select>
              <select id="po-supplier-filter" aria-label="仕入先で絞り込み">
                <option value="">すべての仕入先</option>
                ${state.meta.suppliers.map((s) => `<option value="${esc(s.code)}">${esc(s.name)}</option>`).join("")}
              </select>
            </div>
            <div class="table-wrap"><table class="list" id="po-table">
              <thead><tr><th>番号</th><th>日付</th><th>仕入先</th><th>受注番号</th><th class="num">金額</th><th>状態</th><th data-nosort></th></tr></thead>
              <tbody>${rows || `<tr><td colspan="7" class="empty">発注がありません。</td></tr>`}</tbody>
            </table></div>
          </div>
        </section>`;

        setupTable({
          tableId: "po-table",
          searchId: "po-search",
          filters: [
            { id: "po-status-filter", key: "status" },
            { id: "po-supplier-filter", key: "supplier" },
          ],
        });
      }

      function renderPurchaseList() {
        const rows = state.purchases
          .map(
            (purchase) => `<tr data-search="${esc(`${purchase.number} ${purchase.date} ${purchase.supplier_name} ${purchase.purchase_order_number}`)}" data-supplier="${esc(purchase.supplier_code)}">
              <td class="mono">${esc(purchase.number)}</td>
              <td>${esc(purchase.date)}</td>
              <td>${esc(purchase.supplier_name)}</td>
              <td class="mono">${esc(purchase.purchase_order_number)}</td>
              <td class="num">¥${yen(purchase.total)}</td>
            </tr>`
          )
          .join("");
        app.innerHTML = `<section>
          <div class="card">
            <div class="toolbar">
              <input type="search" id="purchase-search" placeholder="番号・仕入先・発注番号で検索" />
              <select id="purchase-supplier-filter" aria-label="仕入先で絞り込み">
                <option value="">すべての仕入先</option>
                ${state.meta.suppliers.map((s) => `<option value="${esc(s.code)}">${esc(s.name)}</option>`).join("")}
              </select>
            </div>
            <div class="table-wrap"><table class="list" id="purchase-table">
              <thead><tr><th>番号</th><th>日付</th><th>仕入先</th><th>発注番号</th><th class="num">金額</th></tr></thead>
              <tbody>${rows || `<tr><td colspan="5" class="empty">仕入がありません。</td></tr>`}</tbody>
            </table></div>
          </div>
        </section>`;

        setupTable({
          tableId: "purchase-table",
          searchId: "purchase-search",
          filters: [{ id: "purchase-supplier-filter", key: "supplier" }],
        });
      }

      function renderPayments() {
        const receiptRows = state.receipts
          .slice(0, 30)
          .map(
            (receipt) => `<tr data-search="${esc(`${receipt.number} ${receipt.date} ${receipt.customer_name} ${receipt.method}`)}" data-customer="${esc(receipt.customer_code)}">
              <td class="mono">${esc(receipt.number)}</td>
              <td>${esc(receipt.date)}</td>
              <td>${esc(receipt.customer_name)}</td>
              <td class="num">¥${yen(receipt.amount)}</td>
              <td>${esc(receipt.method)}</td>
            </tr>`
          )
          .join("");
        const paymentRows = state.supplierPayments
          .slice(0, 30)
          .map(
            (payment) => `<tr data-search="${esc(`${payment.number} ${payment.date} ${payment.supplier_name} ${payment.method}`)}" data-supplier="${esc(payment.supplier_code)}">
              <td class="mono">${esc(payment.number)}</td>
              <td>${esc(payment.date)}</td>
              <td>${esc(payment.supplier_name)}</td>
              <td class="num">¥${yen(payment.amount)}</td>
              <td>${esc(payment.method)}</td>
            </tr>`
          )
          .join("");

        app.innerHTML = `<section>
          <div class="page-head">
            <div><h1>入金・支払</h1><p class="lead">得意先からの入金と、仕入先への支払を登録します。</p></div>
          </div>
          <div class="grid-2">
            <div class="card">
              <div class="card-head">入金の登録</div>
              <form class="slip" id="receipt-form">
                <div class="form-grid">
                  <label class="field">得意先<select name="customer_code">${customerOptions(state.meta.customers[0]?.code)}</select></label>
                  <label class="field">入金日<input type="date" name="date" value="${today()}" /></label>
                  <label class="field">金額<input type="number" name="amount" min="1" step="1" required /></label>
                  <label class="field">方法<select name="method"><option>振込</option><option>現金</option><option>手形</option><option>その他</option></select></label>
                  <label class="field full">備考<input type="text" name="notes" /></label>
                </div>
                <div class="form-actions"><button type="submit" class="primary">入金を登録</button></div>
              </form>
            </div>
            <div class="card">
              <div class="card-head">支払の登録</div>
              <form class="slip" id="supplier-payment-form">
                <div class="form-grid">
                  <label class="field">仕入先<select name="supplier_code">${supplierOptions(state.meta.suppliers[0]?.code)}</select></label>
                  <label class="field">支払日<input type="date" name="date" value="${today()}" /></label>
                  <label class="field">金額<input type="number" name="amount" min="1" step="1" required /></label>
                  <label class="field">方法<select name="method"><option>振込</option><option>現金</option><option>手形</option><option>その他</option></select></label>
                  <label class="field full">備考<input type="text" name="notes" /></label>
                </div>
                <div class="form-actions"><button type="submit" class="primary">支払を登録</button></div>
              </form>
            </div>
          </div>
          <div class="grid-2 spaced">
            <div class="card">
              <div class="card-head">最近の入金 <input type="search" id="receipt-search" placeholder="検索" /></div>
              <div class="table-wrap"><table class="list" id="receipt-table">
                <thead><tr><th>番号</th><th>日付</th><th>得意先</th><th class="num">金額</th><th>方法</th></tr></thead>
                <tbody>${receiptRows || `<tr><td colspan="5" class="empty">入金がありません。</td></tr>`}</tbody>
              </table></div>
            </div>
            <div class="card">
              <div class="card-head">最近の支払 <input type="search" id="supplier-payment-search" placeholder="検索" /></div>
              <div class="table-wrap"><table class="list" id="supplier-payment-table">
                <thead><tr><th>番号</th><th>日付</th><th>仕入先</th><th class="num">金額</th><th>方法</th></tr></thead>
                <tbody>${paymentRows || `<tr><td colspan="5" class="empty">支払がありません。</td></tr>`}</tbody>
              </table></div>
            </div>
          </div>
        </section>`;

        document.getElementById("receipt-form").addEventListener("submit", async (event) => {
          event.preventDefault();
          const form = event.currentTarget;
          const data = new FormData(form);
          try {
            await api("/api/receipts", {
              method: "POST",
              body: JSON.stringify({
                customer_code: data.get("customer_code"),
                date: data.get("date"),
                amount: Number(data.get("amount")),
                method: data.get("method"),
                notes: data.get("notes"),
              }),
            });
            await refresh();
            toast("入金を登録しました。");
            renderPayments();
          } catch (error) {
            toast(error.message, "error");
          }
        });

        document
          .getElementById("supplier-payment-form")
          .addEventListener("submit", async (event) => {
            event.preventDefault();
            const form = event.currentTarget;
            const data = new FormData(form);
            try {
              await api("/api/supplier-payments", {
                method: "POST",
                body: JSON.stringify({
                  supplier_code: data.get("supplier_code"),
                  date: data.get("date"),
                  amount: Number(data.get("amount")),
                  method: data.get("method"),
                  notes: data.get("notes"),
                }),
              });
              await refresh();
              toast("支払を登録しました。");
              renderPayments();
            } catch (error) {
              toast(error.message, "error");
            }
          });

        setupTable({ tableId: "receipt-table", searchId: "receipt-search" });
        setupTable({ tableId: "supplier-payment-table", searchId: "supplier-payment-search" });
      }

      function renderSaleForm(orderId) {
        const order = state.orders.find((item) => item.id === orderId);
        if (!order) {
          app.innerHTML = `<div class="card not-found">受注が見つかりません。</div>`;
          return;
        }
        const lines = order.lines.filter((line) => line.quantity - line.sold_quantity > 0.0001);
        if (lines.length === 0) {
          app.innerHTML = `<section><div class="page-head"><div><h1>売上計上 ${esc(order.number)}</h1></div></div><div class="card"><p class="empty">売上残がありません。</p></div></section>`;
          return;
        }

        const rows = lines
          .map((line) => {
            const remaining = line.quantity - line.sold_quantity;
            return `<tr data-line-id="${line.id}">
              <td class="c-pick" data-label="計上"><input type="checkbox" class="pick" checked /></td>
              <td class="c-product" data-label="商品">${esc(line.name)}</td>
              <td class="c-stat num" data-label="受注">${qty(line.quantity)}</td>
              <td class="c-stat num" data-label="売上済">${qty(line.sold_quantity)}</td>
              <td class="c-stat num" data-label="残">${qty(remaining)}</td>
              <td class="c-qty" data-label="今回数量"><input class="flow-qty" type="number" min="0" max="${remaining}" step="0.01" value="${remaining}" /></td>
              <td class="c-price num" data-label="単価">¥${yen(line.unit_price)}</td>
              <td class="c-amount num flow-amount" data-label="金額">¥${yen(Math.round(remaining * line.unit_price))}</td>
            </tr>`;
          })
          .join("");

        app.innerHTML = `<section>
          <div class="page-head"><div><h1>売上計上 ${esc(order.number)}</h1><p class="lead">${esc(order.customer_name)} / ${esc(order.subject || "")}</p></div></div>
          <div class="card"><form class="slip" id="sale-form">
            <div class="form-grid">
              <label class="field">売上日<input type="date" name="date" value="${today()}" /></label>
              <label class="field full">備考<input type="text" name="notes" value="" /></label>
            </div>
            <div class="section-title"><span>受注残から売上計上する明細</span></div>
            <div class="table-wrap"><table class="line-editor">
              <thead><tr><th></th><th>商品</th><th class="num">受注</th><th class="num">売上済</th><th class="num">残</th><th>今回数量</th><th class="num">単価</th><th class="num">金額</th></tr></thead>
              <tbody id="flow-body">${rows}</tbody>
            </table></div>
            <div class="totals-box">
              <div><span>小計</span><strong id="flow-subtotal">¥0</strong></div>
              <div><span>消費税 (10%)</span><strong id="flow-tax">¥0</strong></div>
              <div class="grand"><span>合計</span><strong id="flow-total">¥0</strong></div>
            </div>
            <div class="form-actions">
              <button type="button" class="secondary" id="cancel-button">戻る</button>
              <button type="submit" class="primary">売上計上</button>
            </div>
          </form></div>
        </section>`;

        const tbody = document.getElementById("flow-body");
        const recalc = () => {
          let subtotal = 0;
          [...tbody.querySelectorAll("tr")].forEach((row) => {
            const picked = row.querySelector(".pick").checked;
            const quantity = Number(row.querySelector(".flow-qty").value) || 0;
            const line = order.lines.find((item) => item.id === Number(row.dataset.lineId));
            const amount = picked ? Math.round(quantity * line.unit_price) : 0;
            row.querySelector(".flow-amount").textContent = `¥${yen(amount)}`;
            subtotal += amount;
          });
          const tax = Math.round(subtotal * 0.1);
          document.getElementById("flow-subtotal").textContent = `¥${yen(subtotal)}`;
          document.getElementById("flow-tax").textContent = `¥${yen(tax)}`;
          document.getElementById("flow-total").textContent = `¥${yen(subtotal + tax)}`;
        };
        tbody.addEventListener("input", recalc);
        tbody.addEventListener("change", recalc);
        recalc();

        document.getElementById("cancel-button").addEventListener("click", () => navigate("/orders"));
        document.getElementById("sale-form").addEventListener("submit", async (event) => {
          event.preventDefault();
          const data = new FormData(event.currentTarget);
          const selected = [...tbody.querySelectorAll("tr")]
            .filter((row) => row.querySelector(".pick").checked)
            .map((row) => ({
              line_id: Number(row.dataset.lineId),
              quantity: Number(row.querySelector(".flow-qty").value),
            }))
            .filter((line) => line.quantity > 0);
          try {
            await api(`/api/orders/${order.id}/sales`, {
              method: "POST",
              body: JSON.stringify({
                date: data.get("date"),
                notes: data.get("notes"),
                lines: selected,
              }),
            });
            await refresh();
            toast("売上を計上しました。");
            navigate("/sales");
          } catch (error) {
            toast(error.message, "error");
          }
        });
      }

      function renderPurchaseForm(orderId) {
        const order = state.orders.find((item) => item.id === orderId);
        if (!order) {
          app.innerHTML = `<div class="card not-found">受注が見つかりません。</div>`;
          return;
        }
        const lines = order.lines.filter(
          (line) => line.quantity - line.purchased_quantity > 0.0001
        );
        if (lines.length === 0) {
          app.innerHTML = `<section><div class="page-head"><div><h1>発注 ${esc(order.number)}</h1></div></div><div class="card"><p class="empty">発注残がありません。</p></div></section>`;
          return;
        }

        const rows = lines
          .map((line) => {
            const remaining = line.quantity - line.purchased_quantity;
            const product = state.meta.products.find((item) => item.code === line.product_code);
            const cost = product?.cost_price ?? line.unit_price;
            return `<tr data-line-id="${line.id}" data-cost="${cost}">
              <td class="c-pick" data-label="計上"><input type="checkbox" class="pick" checked /></td>
              <td class="c-product" data-label="商品">${esc(line.name)}</td>
              <td class="c-stat num" data-label="受注">${qty(line.quantity)}</td>
              <td class="c-stat num" data-label="発注済">${qty(line.purchased_quantity)}</td>
              <td class="c-stat num" data-label="残">${qty(remaining)}</td>
              <td class="c-qty" data-label="今回数量"><input class="flow-qty" type="number" min="0" max="${remaining}" step="0.01" value="${remaining}" /></td>
              <td class="c-price num" data-label="仕入単価">¥${yen(cost)}</td>
              <td class="c-amount num flow-amount" data-label="金額">¥${yen(Math.round(remaining * cost))}</td>
            </tr>`;
          })
          .join("");

        app.innerHTML = `<section>
          <div class="page-head"><div><h1>発注 ${esc(order.number)}</h1><p class="lead">${esc(order.customer_name)} / ${esc(order.subject || "")}</p></div></div>
          <div class="card"><form class="slip" id="purchase-form">
            <div class="form-grid">
              <label class="field">仕入先<select name="supplier_code">${supplierOptions(state.meta.suppliers[0]?.code)}</select></label>
              <label class="field">発注日<input type="date" name="date" value="${today()}" /></label>
              <label class="field full">備考<input type="text" name="notes" value="" /></label>
            </div>
            <div class="section-title"><span>受注残から発注する明細</span></div>
            <div class="table-wrap"><table class="line-editor">
              <thead><tr><th></th><th>商品</th><th class="num">受注</th><th class="num">発注済</th><th class="num">残</th><th>今回数量</th><th class="num">仕入単価</th><th class="num">金額</th></tr></thead>
              <tbody id="flow-body">${rows}</tbody>
            </table></div>
            <div class="totals-box">
              <div><span>小計</span><strong id="flow-subtotal">¥0</strong></div>
              <div><span>消費税 (10%)</span><strong id="flow-tax">¥0</strong></div>
              <div class="grand"><span>合計</span><strong id="flow-total">¥0</strong></div>
            </div>
            <div class="form-actions">
              <button type="button" class="secondary" id="cancel-button">戻る</button>
              <button type="submit" class="primary">発注する</button>
            </div>
          </form></div>
        </section>`;

        const tbody = document.getElementById("flow-body");
        const recalc = () => {
          let subtotal = 0;
          [...tbody.querySelectorAll("tr")].forEach((row) => {
            const picked = row.querySelector(".pick").checked;
            const quantity = Number(row.querySelector(".flow-qty").value) || 0;
            const cost = Number(row.dataset.cost);
            const amount = picked ? Math.round(quantity * cost) : 0;
            row.querySelector(".flow-amount").textContent = `¥${yen(amount)}`;
            subtotal += amount;
          });
          const tax = Math.round(subtotal * 0.1);
          document.getElementById("flow-subtotal").textContent = `¥${yen(subtotal)}`;
          document.getElementById("flow-tax").textContent = `¥${yen(tax)}`;
          document.getElementById("flow-total").textContent = `¥${yen(subtotal + tax)}`;
        };
        tbody.addEventListener("input", recalc);
        tbody.addEventListener("change", recalc);
        recalc();

        document.getElementById("cancel-button").addEventListener("click", () => navigate("/orders"));
        document.getElementById("purchase-form").addEventListener("submit", async (event) => {
          event.preventDefault();
          const data = new FormData(event.currentTarget);
          const selected = [...tbody.querySelectorAll("tr")]
            .filter((row) => row.querySelector(".pick").checked)
            .map((row) => ({
              line_id: Number(row.dataset.lineId),
              quantity: Number(row.querySelector(".flow-qty").value),
            }))
            .filter((line) => line.quantity > 0);
          try {
            await api(`/api/orders/${order.id}/purchase`, {
              method: "POST",
              body: JSON.stringify({
                supplier_code: data.get("supplier_code"),
                date: data.get("date"),
                notes: data.get("notes"),
                lines: selected,
              }),
            });
            await refresh();
            toast("発注を作成しました。");
            navigate("/purchase-orders");
          } catch (error) {
            toast(error.message, "error");
          }
        });
      }

      function renderClosings() {
        if (!state.closing.month) state.closing.month = today().slice(0, 7);
        app.innerHTML = `<section>
          <div class="page-head"><div><h1>締処理</h1><p class="lead">得意先・仕入先ごとに月次の売上/仕入と入金/支払を締め、残高を確定します。</p></div></div>
          <div class="card">
            <form class="summary-filter" id="closing-filter">
              <label>区分<select name="kind">
                <option value="customer" ${state.closing.kind === "customer" ? "selected" : ""}>得意先締</option>
                <option value="supplier" ${state.closing.kind === "supplier" ? "selected" : ""}>仕入先締</option>
              </select></label>
              <label>対象月<input type="month" name="month" value="${esc(state.closing.month)}" /></label>
              <div class="summary-actions">
                <button type="submit" class="primary">プレビュー</button>
                <button type="button" class="secondary" id="closing-run">締を実行</button>
              </div>
            </form>
            <div id="closing-preview"></div>
          </div>
          <div class="card spaced">
            <div class="card-head">締処理履歴 <input type="search" id="closing-history-search" placeholder="検索" /></div>
            <div class="table-wrap"><table class="list" id="closing-history-table">
              <thead><tr><th>対象月</th><th>区分</th><th>コード</th><th>名称</th><th class="num">当月</th><th class="num">入出金</th><th class="num">残高</th></tr></thead>
              <tbody>${state.closings
                .map(
                  (row) => `<tr data-search="${esc(`${row.month} ${row.kind} ${row.party_code} ${row.party_name}`)}">
                    <td>${esc(row.month)}</td>
                    <td>${row.kind === "customer" ? "得意先" : "仕入先"}</td>
                    <td class="mono">${esc(row.party_code)}</td>
                    <td>${esc(row.party_name)}</td>
                    <td class="num">¥${yen(row.sales_total)}</td>
                    <td class="num">¥${yen(row.payments)}</td>
                    <td class="num"><strong>¥${yen(row.balance)}</strong></td>
                  </tr>`
                )
                .join("") || `<tr><td colspan="7" class="empty">締処理履歴がありません。</td></tr>`}</tbody>
            </table></div>
          </div>
        </section>`;

        const form = document.getElementById("closing-filter");
        form.addEventListener("submit", (event) => {
          event.preventDefault();
          state.closing.kind = form.kind.value;
          state.closing.month = form.month.value;
          loadClosingPreview();
        });

        document.getElementById("closing-run").addEventListener("click", async () => {
          if (!confirm(`${state.closing.month} の締処理を実行しますか？`)) return;
          try {
            await api("/api/closings", {
              method: "POST",
              body: JSON.stringify({ kind: state.closing.kind, month: state.closing.month }),
            });
            await refresh();
            toast("締処理を実行しました。");
            renderClosings();
          } catch (error) {
            toast(error.message, "error");
          }
        });

        setupTable({ tableId: "closing-history-table", searchId: "closing-history-search" });
        loadClosingPreview();
      }

      async function loadClosingPreview() {
        const box = document.getElementById("closing-preview");
        if (!box) return;
        box.innerHTML = `<div class="empty">読み込み中…</div>`;
        try {
          const data = await api(
            `/api/closings/preview?kind=${state.closing.kind}&month=${state.closing.month}`
          );
          state.closing.preview = data;
          const isCustomer = data.kind === "customer";
          const rows = data.rows
            .map(
              (row) => `<tr>
                <td class="mono">${esc(row.party_code)}</td>
                <td>${esc(row.party_name)}</td>
                <td class="num">¥${yen(row.previous_balance)}</td>
                <td class="num">¥${yen(row.sales_total)}</td>
                <td class="num">¥${yen(row.tax)}</td>
                <td class="num">¥${yen(row.payments)}</td>
                <td class="num"><strong>¥${yen(row.balance)}</strong></td>
              </tr>`
            )
            .join("");
          box.innerHTML = `<div class="summary-meta"><span>${esc(data.month)} / ${isCustomer ? "得意先締" : "仕入先締"}</span></div>
            <div class="table-wrap"><table class="list">
              <thead><tr><th>コード</th><th>名称</th><th class="num">前回残高</th><th class="num">${isCustomer ? "売上" : "仕入"}</th><th class="num">消費税</th><th class="num">${isCustomer ? "入金" : "支払"}</th><th class="num">残高</th></tr></thead>
              <tbody>${rows || `<tr><td colspan="7" class="empty">対象データがありません。</td></tr>`}</tbody>
            </table></div>`;
        } catch (error) {
          box.innerHTML = `<div class="empty">${esc(error.message)}</div>`;
        }
      }

      function summaryParams() {
        const params = new URLSearchParams();
        if (state.summary.from) params.set("from", state.summary.from);
        if (state.summary.to) params.set("to", state.summary.to);
        params.set("group", state.summary.group);
        params.set("source", state.summary.source);
        return params;
      }

      function summaryTable(data) {
        const rows = data.rows
          .map(
            (row) => `<tr>
              <td>${esc(row.label)}</td>
              <td class="num">${row.orders}</td>
              <td class="num">${qty(row.quantity)}</td>
              <td class="num">¥${yen(row.subtotal)}</td>
              <td class="num">¥${yen(row.tax)}</td>
              <td class="num"><strong>¥${yen(row.total)}</strong></td>
            </tr>`
          )
          .join("");

        return `<div class="summary-meta">
            <span>集計: <strong>${esc(data.title)} / ${esc(data.group_label)}</strong></span>
            <span>対象期間: <strong>${data.from || data.to ? `${esc(data.from || "指定なし")} 〜 ${esc(data.to || "指定なし")}` : "全期間"}</strong></span>
          </div>
          <div class="table-wrap">
            <table class="list" id="summary-table">
              <thead>
                <tr>
                  <th>区分</th>
                  <th class="num">件数</th>
                  <th class="num">数量</th>
                  <th class="num">小計</th>
                  <th class="num">消費税</th>
                  <th class="num">合計</th>
                </tr>
              </thead>
              <tbody>${rows || `<tr><td colspan="6" class="empty">対象データがありません。</td></tr>`}</tbody>
              <tfoot>
                <tr>
                  <td><strong>合計</strong></td>
                  <td class="num"><strong>${data.totals.orders}</strong></td>
                  <td class="num"><strong>${qty(data.totals.quantity)}</strong></td>
                  <td class="num"><strong>¥${yen(data.totals.subtotal)}</strong></td>
                  <td class="num"><strong>¥${yen(data.totals.tax)}</strong></td>
                  <td class="num"><strong>¥${yen(data.totals.total)}</strong></td>
                </tr>
              </tfoot>
            </table>
          </div>`;
      }

      function remainingTable(data) {
        const rows = data.rows
          .map(
            (row) => `<tr>
              <td class="mono">${esc(row.document)}</td>
              <td>${esc(row.party)}</td>
              <td>${esc(row.product)}</td>
              <td class="num">${qty(row.quantity)}</td>
              <td class="num">¥${yen(row.amount)}</td>
            </tr>`
          )
          .join("");
        return `<div class="summary-meta"><span>${esc(data.title)}</span></div>
          <div class="table-wrap"><table class="list">
            <thead><tr><th>伝票</th><th>相手先</th><th>商品</th><th class="num">残数量</th><th class="num">残金額</th></tr></thead>
            <tbody>${rows || `<tr><td colspan="5" class="empty">残がありません。</td></tr>`}</tbody>
            <tfoot><tr><td colspan="3"><strong>合計</strong></td><td class="num"><strong>${qty(data.total_quantity)}</strong></td><td class="num"><strong>¥${yen(data.total_amount)}</strong></td></tr></tfoot>
          </table></div>`;
      }

      function balanceTable(data) {
        const customerRows = data.customers
          .map(
            (row) => `<tr><td class="mono">${esc(row.code)}</td><td>${esc(row.name)}</td><td class="num">¥${yen(row.balance)}</td></tr>`
          )
          .join("");
        const supplierRows = data.suppliers
          .map(
            (row) => `<tr><td class="mono">${esc(row.code)}</td><td>${esc(row.name)}</td><td class="num">¥${yen(row.balance)}</td></tr>`
          )
          .join("");
        return `<div class="grid-2">
          <div>
            <div class="card-head">売掛残高（得意先）</div>
            <div class="table-wrap"><table class="list">
              <thead><tr><th>コード</th><th>名称</th><th class="num">残高</th></tr></thead>
              <tbody>${customerRows || `<tr><td colspan="3" class="empty">データがありません。</td></tr>`}</tbody>
              <tfoot><tr><td colspan="2"><strong>合計</strong></td><td class="num"><strong>¥${yen(data.customer_total)}</strong></td></tr></tfoot>
            </table></div>
          </div>
          <div>
            <div class="card-head">買掛残高（仕入先）</div>
            <div class="table-wrap"><table class="list">
              <thead><tr><th>コード</th><th>名称</th><th class="num">残高</th></tr></thead>
              <tbody>${supplierRows || `<tr><td colspan="3" class="empty">データがありません。</td></tr>`}</tbody>
              <tfoot><tr><td colspan="2"><strong>合計</strong></td><td class="num"><strong>¥${yen(data.supplier_total)}</strong></td></tr></tfoot>
            </table></div>
          </div>
        </div>`;
      }

      async function loadSummary() {
        const box = document.getElementById("summary-result");
        if (!box) return;
        box.innerHTML = `<div class="empty">集計中…</div>`;
        try {
          let data;
          if (state.summary.report === "order_remaining") {
            data = await api("/api/reports/order-remaining");
          } else if (state.summary.report === "purchase_remaining") {
            data = await api("/api/reports/purchase-remaining");
          } else if (state.summary.report === "balances") {
            data = await api("/api/reports/balances");
          } else {
            data = await api(`/api/reports/summary?${summaryParams()}`);
          }
          state.summary.data = data;
          if (state.summary.report === "order_remaining" || state.summary.report === "purchase_remaining") {
            box.innerHTML = remainingTable(data);
          } else if (state.summary.report === "balances") {
            box.innerHTML = balanceTable(data);
          } else {
            box.innerHTML = summaryTable(data);
          }
        } catch (error) {
          box.innerHTML = `<div class="empty">${esc(error.message)}</div>`;
        }
      }

      function renderSummary() {
        const isSummary = state.summary.report === "summary";
        app.innerHTML = `<section>
          <div class="page-head">
            <div>
              <h1>集計表</h1>
              <p class="lead">売上・受注の集計、受注残・発注残、売掛・買掛残高を確認できます。</p>
            </div>
          </div>
          <div class="card">
            <form class="summary-filter" id="summary-filter">
              <label>帳票
                <select name="report">
                  <option value="summary" ${isSummary ? "selected" : ""}>売上 / 受注集計</option>
                  <option value="order_remaining" ${state.summary.report === "order_remaining" ? "selected" : ""}>受注残一覧</option>
                  <option value="purchase_remaining" ${state.summary.report === "purchase_remaining" ? "selected" : ""}>発注残一覧</option>
                  <option value="balances" ${state.summary.report === "balances" ? "selected" : ""}>売掛・買掛残高</option>
                </select>
              </label>
              <span id="summary-options" style="display:${isSummary ? "contents" : "none"}">
                <label>対象
                  <select name="source">
                    <option value="orders" ${state.summary.source === "orders" ? "selected" : ""}>受注</option>
                    <option value="sales" ${state.summary.source === "sales" ? "selected" : ""}>売上</option>
                  </select>
                </label>
                <label>集計区分
                  <select name="group">
                    <option value="customer" ${state.summary.group === "customer" ? "selected" : ""}>得意先別</option>
                    <option value="product" ${state.summary.group === "product" ? "selected" : ""}>商品別</option>
                    <option value="month" ${state.summary.group === "month" ? "selected" : ""}>月別</option>
                  </select>
                </label>
                <label>開始日<input type="date" name="from" value="${esc(state.summary.from)}" /></label>
                <label>終了日<input type="date" name="to" value="${esc(state.summary.to)}" /></label>
              </span>
              <div class="summary-actions">
                <button type="submit" class="primary">表示</button>
                <button type="button" class="secondary" id="summary-print" style="display:${isSummary ? "" : "none"}">集計表を印刷</button>
              </div>
            </form>
            <div id="summary-result"></div>
          </div>
          <p class="foot-note">商品別の消費税は明細単位で計算しているため、伝票単位の税額合計と数円異なる場合があります。受注残・発注残は未処理数量を表示します。</p>
        </section>`;

        const form = document.getElementById("summary-filter");
        form.addEventListener("submit", (event) => {
          event.preventDefault();
          state.summary.report = form.report.value;
          if (state.summary.report === "summary") {
            state.summary.source = form.source.value;
            state.summary.group = form.group.value;
            state.summary.from = form.from.value;
            state.summary.to = form.to.value;
          }
          renderSummary();
        });

        document.getElementById("summary-print").addEventListener("click", () => {
          window.open(`/reports/summary/print?${summaryParams()}`, "_blank", "noopener");
        });

        loadSummary();
      }

      function customerOptions(selectedCode) {
        return state.meta.customers
          .map(
            (customer) =>
              `<option value="${esc(customer.code)}" ${customer.code === selectedCode ? "selected" : ""}>${esc(
                customer.code
              )} ${esc(customer.name)}</option>`
          )
          .join("");
      }

      function productOptions(selectedCode) {
        return (
          `<option value="">（商品を選択）</option>` +
          state.meta.products
            .map(
              (product) =>
                `<option value="${esc(product.code)}" ${product.code === selectedCode ? "selected" : ""}>${esc(
                  product.code
                )} ${esc(product.name)}</option>`
            )
            .join("")
        );
      }

      function addLine(tbody, line = null) {
        const product =
          state.meta.products.find((item) => item.code === line?.product_code) || state.meta.products[0];
        const code = line?.product_code || product?.code || "";
        const tr = document.createElement("tr");
        tr.innerHTML = `
          <td class="c-drag"><span class="drag-handle" draggable="true" title="ドラッグで並べ替え">⠿</span></td>
          <td class="c-product" data-label="商品"><select class="product">${productOptions(code)}</select></td>
          <td class="c-qty" data-label="数量"><input class="qty" type="number" min="0" step="0.01" value="${line ? qty(line.quantity) : 1}" /></td>
          <td class="c-unit" data-label="単位"><input class="unit" type="text" value="${esc(line?.unit ?? product?.unit ?? "")}" /></td>
          <td class="c-price" data-label="単価"><input class="price" type="number" min="0" step="1" value="${line ? line.unit_price : product?.unit_price ?? 0}" /></td>
          <td class="c-amount num amount" data-label="金額">¥0</td>
          <td class="c-note" data-label="備考"><input class="note" type="text" value="${esc(line?.note ?? "")}" placeholder="任意" /></td>
          <td class="c-tax" data-label=""><label class="check"><input type="checkbox" class="taxable" ${line ? (line.taxable ? "checked" : "") : "checked"} />課税</label></td>
          <td class="c-remove"><button type="button" class="icon remove" title="行を削除">×</button></td>`;

        tbody.append(tr);

        const handle = tr.querySelector(".drag-handle");
        handle.addEventListener("dragstart", (event) => {
          event.dataTransfer.effectAllowed = "move";
          event.dataTransfer.setData("text/plain", "");
          tr.classList.add("dragging");
        });
        handle.addEventListener("dragend", () => {
          tr.classList.remove("dragging");
          updateTotals(tbody);
        });
        tr.addEventListener("dragover", (event) => {
          const dragging = tbody.querySelector("tr.dragging");
          if (!dragging || dragging === tr) return;
          event.preventDefault();
          const rect = tr.getBoundingClientRect();
          const after = event.clientY - rect.top > rect.height / 2;
          tbody.insertBefore(dragging, after ? tr.nextSibling : tr);
        });
        tr.addEventListener("drop", (event) => event.preventDefault());

        tr.querySelector(".product").addEventListener("change", () => {
          const selected = state.meta.products.find(
            (item) => item.code === tr.querySelector(".product").value
          );
          if (selected) {
            tr.querySelector(".unit").value = selected.unit;
            tr.querySelector(".price").value = selected.unit_price;
            tr.querySelector(".taxable").checked = selected.taxable;
          }
          updateRow(tr);
          updateTotals(tbody);
        });
        tr.addEventListener("input", () => {
          updateRow(tr);
          updateTotals(tbody);
        });
        tr.querySelector(".remove").addEventListener("click", () => {
          if (tbody.children.length > 1) {
            tr.remove();
          } else {
            tr.querySelector(".qty").value = "1";
            tr.querySelector(".price").value = "0";
            tr.querySelector(".note").value = "";
          }
          updateTotals(tbody);
        });

        updateRow(tr);
        updateTotals(tbody);
      }

      function updateRow(tr) {
        const quantity = Number(tr.querySelector(".qty").value) || 0;
        const price = Number(tr.querySelector(".price").value) || 0;
        tr.querySelector(".amount").textContent = `¥${yen(Math.round(quantity * price))}`;
      }

      function updateTotals(tbody) {
        let subtotal = 0;
        let taxable = 0;
        [...tbody.querySelectorAll("tr")].forEach((tr) => {
          const quantity = Number(tr.querySelector(".qty").value) || 0;
          const price = Number(tr.querySelector(".price").value) || 0;
          const amount = Math.round(quantity * price);
          subtotal += amount;
          if (tr.querySelector(".taxable").checked) taxable += amount;
        });
        const tax = Math.round(taxable * 0.1);
        document.getElementById("subtotal").textContent = `¥${yen(subtotal)}`;
        document.getElementById("tax").textContent = `¥${yen(tax)}`;
        document.getElementById("total").textContent = `¥${yen(subtotal + tax)}`;
      }

      function collectLines(tbody) {
        return [...tbody.querySelectorAll("tr")]
          .map((tr) => ({
            product_code: tr.querySelector(".product").value || null,
            quantity: Number(tr.querySelector(".qty").value),
            unit: tr.querySelector(".unit").value,
            unit_price: Number(tr.querySelector(".price").value),
            taxable: tr.querySelector(".taxable").checked,
            note: tr.querySelector(".note").value,
          }))
          .filter((line) => line.quantity > 0 || line.unit_price > 0);
      }

      function lineEditor() {
        return `<div class="section-title"><span>明細</span><button type="button" class="secondary" id="add-line">＋ 明細を追加</button></div>
          <div class="table-wrap">
            <table class="line-editor">
              <thead><tr><th></th><th>商品</th><th>数量</th><th>単位</th><th>単価</th><th class="num">金額</th><th>備考</th><th>課税</th><th></th></tr></thead>
              <tbody id="line-body"></tbody>
            </table>
          </div>
          <div class="totals-box">
            <div><span>小計</span><strong id="subtotal">¥0</strong></div>
            <div><span>消費税 (10%)</span><strong id="tax">¥0</strong></div>
            <div class="grand"><span>合計</span><strong id="total">¥0</strong></div>
          </div>`;
      }

      function renderQuoteForm(id) {
        const quote = id ? state.quotes.find((item) => item.id === id) : null;
        if (id && !quote) {
          app.innerHTML = `<div class="card not-found">見積が見つかりません。<button type="button" class="link" data-nav="quotes">一覧へ戻る</button></div>`;
          return;
        }
        const readOnly = quote?.status === "accepted";
        const disabled = readOnly ? "disabled" : "";
        const value = (key, fallback = "") => (quote ? quote[key] ?? fallback : fallback);

        app.innerHTML = `<section>
          <div class="page-head">
            <div>
              <h1>${quote ? `見積 ${esc(quote.number)}` : "新規見積"}</h1>
              <p class="lead">見積日・得意先・明細を入力して保存します。金額は数量 × 単価から自動計算されます。</p>
            </div>
          </div>
          ${readOnly ? `<p class="notice">この見積は受注に変換済みのため編集できません。内容の確認と帳票の印刷のみ可能です。</p>` : ""}
          <div class="card">
            <form class="slip" id="quote-form">
              <div class="form-grid">
                <label class="field">得意先<select name="customer_code" ${disabled}>${customerOptions(value("customer_code"))}</select></label>
                <label class="field">見積日<input type="date" name="date" value="${esc(value("date", today()))}" ${disabled} /></label>
                <label class="field">件名<input type="text" name="subject" value="${esc(value("subject"))}" ${disabled} placeholder="例: オフィスPC環境更新のご提案" /></label>
                <label class="field">有効期限<input type="date" name="valid_until" value="${esc(value("valid_until"))}" ${disabled} /></label>
                <label class="field">受渡期日<input type="date" name="delivery_date" value="${esc(value("delivery_date"))}" ${disabled} /></label>
                <label class="field">受渡場所<input type="text" name="delivery_place" value="${esc(value("delivery_place"))}" ${disabled} /></label>
                <label class="field">取引方法・支払条件<input type="text" name="payment_terms" value="${esc(value("payment_terms"))}" ${disabled} /></label>
                <label class="field">状態<select name="status" ${disabled}>
                  <option value="draft" ${value("status", "draft") === "draft" ? "selected" : ""}>下書き</option>
                  <option value="sent" ${value("status", "draft") === "sent" ? "selected" : ""}>提出済</option>
                  <option value="accepted" ${value("status", "draft") === "accepted" ? "selected" : ""}>受注済</option>
                </select></label>
                <label class="field full">備考<textarea name="notes" rows="3" ${disabled}>${esc(value("notes"))}</textarea></label>
              </div>
              ${lineEditor()}
              <div class="form-actions">
                <button type="button" class="secondary" id="cancel-button">戻る</button>
                ${readOnly ? "" : `<button type="submit" class="primary">保存</button>`}
                ${quote ? `<button type="button" class="secondary" data-action="quote-print" data-id="${quote.id}">見積書を印刷</button>` : ""}
              </div>
            </form>
          </div>
        </section>`;

        const tbody = document.getElementById("line-body");
        (quote?.lines?.length ? quote.lines : [null]).forEach((line) => addLine(tbody, line));
        if (readOnly) {
          tbody.querySelectorAll("input, select, button").forEach((element) => {
            element.disabled = true;
          });
          tbody.querySelectorAll(".drag-handle").forEach((handle) => {
            handle.draggable = false;
          });
          document.getElementById("add-line").style.display = "none";
        }
        document.getElementById("cancel-button").addEventListener("click", () => navigate("/quotes"));
        if (!readOnly) {
          document.getElementById("add-line").addEventListener("click", () => addLine(tbody));
        }

        const form = document.getElementById("quote-form");
        form.addEventListener("submit", async (event) => {
          event.preventDefault();
          const submit = form.querySelector('button[type="submit"]');
          submit.disabled = true;
          try {
            const data = new FormData(form);
            const payload = {
              customer_code: data.get("customer_code"),
              date: data.get("date"),
              subject: data.get("subject"),
              valid_until: data.get("valid_until"),
              delivery_date: data.get("delivery_date"),
              delivery_place: data.get("delivery_place"),
              payment_terms: data.get("payment_terms"),
              notes: data.get("notes"),
              status: data.get("status"),
              lines: collectLines(tbody),
            };
            if (quote) {
              await api(`/api/quotes/${quote.id}`, { method: "PUT", body: JSON.stringify(payload) });
              toast("見積を更新しました。");
            } else {
              await api("/api/quotes", { method: "POST", body: JSON.stringify(payload) });
              toast("見積を作成しました。");
            }
            await refresh();
            navigate("/quotes");
          } catch (error) {
            toast(error.message, "error");
            submit.disabled = false;
          }
        });
      }

      function renderOrderForm(id) {
        const order = id ? state.orders.find((item) => item.id === id) : null;
        if (id && !order) {
          app.innerHTML = `<div class="card not-found">受注が見つかりません。<button type="button" class="link" data-nav="orders">一覧へ戻る</button></div>`;
          return;
        }
        const value = (key, fallback = "") => (order ? order[key] ?? fallback : fallback);
        const readOnly =
          order?.lines.some(
            (line) => line.sold_quantity > 0 || line.purchased_quantity > 0
          ) || false;

        app.innerHTML = `<section>
          <div class="page-head">
            <div>
              <h1>${order ? `受注 ${esc(order.number)}` : "新規受注"}</h1>
              <p class="lead">受注日・得意先・納期・明細を入力して保存します。見積から変換した場合は見積番号を保持します。</p>
            </div>
          </div>
          ${readOnly ? `<p class="notice">売上または発注が発生しているため、この受注は編集できません。</p>` : ""}
          ${order?.quote_number ? `<p class="notice">見積 ${esc(order.quote_number)} から変換された受注です。</p>` : ""}
          <div class="card">
            <form class="slip" id="order-form">
              <div class="form-grid">
                <label class="field">得意先<select name="customer_code">${customerOptions(value("customer_code"))}</select></label>
                <label class="field">受注日<input type="date" name="date" value="${esc(value("date", today()))}" /></label>
                <label class="field">注文番号<input type="text" name="customer_order_no" value="${esc(value("customer_order_no"))}" placeholder="例: PO-2026-0182" /></label>
                <label class="field">納期<input type="date" name="delivery_date" value="${esc(value("delivery_date"))}" /></label>
                <label class="field">件名<input type="text" name="subject" value="${esc(value("subject"))}" /></label>
                <label class="field">状態<select name="status">
                  <option value="open" ${value("status", "open") === "open" ? "selected" : ""}>未完了</option>
                  <option value="completed" ${value("status", "open") === "completed" ? "selected" : ""}>完了</option>
                </select></label>
                <label class="field full">備考<textarea name="notes" rows="3">${esc(value("notes"))}</textarea></label>
              </div>
              ${lineEditor()}
              <div class="form-actions">
                <button type="button" class="secondary" id="cancel-button">戻る</button>
                ${readOnly ? "" : `<button type="submit" class="primary">保存</button>`}
                ${order ? `<button type="button" class="secondary" data-action="order-print" data-id="${order.id}">注文請書を印刷</button>` : ""}
              </div>
            </form>
          </div>
        </section>`;

        const tbody = document.getElementById("line-body");
        const form = document.getElementById("order-form");
        (order?.lines?.length ? order.lines : [null]).forEach((line) => addLine(tbody, line));
        if (readOnly) {
          form.querySelectorAll("input, select, textarea").forEach((element) => {
            element.disabled = true;
          });
          tbody.querySelectorAll("button").forEach((element) => {
            element.disabled = true;
          });
          tbody.querySelectorAll(".drag-handle").forEach((handle) => {
            handle.draggable = false;
          });
          document.getElementById("add-line").style.display = "none";
        }
        document.getElementById("cancel-button").addEventListener("click", () => navigate("/orders"));
        if (!readOnly) {
          document.getElementById("add-line").addEventListener("click", () => addLine(tbody));
        }

        form.addEventListener("submit", async (event) => {
          event.preventDefault();
          const submit = form.querySelector('button[type="submit"]');
          submit.disabled = true;
          try {
            const data = new FormData(form);
            const payload = {
              customer_code: data.get("customer_code"),
              date: data.get("date"),
              customer_order_no: data.get("customer_order_no"),
              delivery_date: data.get("delivery_date"),
              subject: data.get("subject"),
              notes: data.get("notes"),
              status: data.get("status"),
              lines: collectLines(tbody),
            };
            if (order) {
              await api(`/api/orders/${order.id}`, { method: "PUT", body: JSON.stringify(payload) });
              toast("受注を更新しました。");
            } else {
              await api("/api/orders", { method: "POST", body: JSON.stringify(payload) });
              toast("受注を作成しました。");
            }
            await refresh();
            navigate("/orders");
          } catch (error) {
            toast(error.message, "error");
            submit.disabled = false;
          }
        });
      }

      document.addEventListener("click", async (event) => {
        const navButton = event.target.closest("[data-nav]");
        if (navButton) {
          navigate(`/${navButton.dataset.nav}`);
          return;
        }

        const actionButton = event.target.closest("[data-action]");
        if (!actionButton) return;
        const id = Number(actionButton.dataset.id);
        const action = actionButton.dataset.action;

        try {
          if (action === "quote-edit") {
            navigate(`/quotes/${id}/edit`);
          } else if (action === "order-edit") {
            navigate(`/orders/${id}/edit`);
          } else if (action === "order-sale") {
            navigate(`/orders/${id}/sales`);
          } else if (action === "order-purchase") {
            navigate(`/orders/${id}/purchase`);
          } else if (action === "sale-print") {
            window.open(`/sales/${id}/print`, "_blank", "noopener");
          } else if (action === "po-print") {
            window.open(`/purchase-orders/${id}/print`, "_blank", "noopener");
          } else if (action === "sale-delete") {
            if (!confirm("この売上を削除しますか？ 受注の売上済数量も戻ります。")) return;
            await api(`/api/sales/${id}`, { method: "DELETE" });
            await refresh();
            toast("売上を削除しました。");
            render();
          } else if (action === "po-receive") {
            if (!confirm("この発注を仕入計上しますか？")) return;
            await api(`/api/purchase-orders/${id}/receive`, {
              method: "POST",
              body: JSON.stringify({ date: today() }),
            });
            await refresh();
            toast("仕入を計上しました。");
            render();
          } else if (action === "po-delete") {
            if (!confirm("この発注を削除しますか？")) return;
            await api(`/api/purchase-orders/${id}`, { method: "DELETE" });
            await refresh();
            toast("発注を削除しました。");
            render();
          } else if (action === "quote-print") {
            window.open(`/quotes/${id}/print`, "_blank", "noopener");
          } else if (action === "order-print") {
            window.open(`/orders/${id}/print`, "_blank", "noopener");
          } else if (action === "quote-convert") {
            if (!confirm("この見積を受注に変換しますか？")) return;
            const order = await api(`/api/quotes/${id}/convert`, { method: "POST" });
            await refresh();
            toast(`受注 ${order.number} を作成しました。`);
            navigate(`/orders/${order.id}/edit`);
          } else if (action === "quote-delete") {
            if (!confirm("この見積を削除しますか？")) return;
            await api(`/api/quotes/${id}`, { method: "DELETE" });
            await refresh();
            toast("見積を削除しました。");
            render();
          } else if (action === "order-delete") {
            if (!confirm("この受注を削除しますか？")) return;
            await api(`/api/orders/${id}`, { method: "DELETE" });
            await refresh();
            toast("受注を削除しました。");
            render();
          }
        } catch (error) {
          toast(error.message, "error");
        }
      });

      async function init() {
        try {
          state.meta = await api("/api/meta");
          await refresh();
          render();
          loadMetrics();
        } catch (error) {
          app.innerHTML = `<div class="card not-found">読み込みに失敗しました: ${esc(error.message)}</div>`;
        }
        window.addEventListener("hashchange", render);
      }

      init();
