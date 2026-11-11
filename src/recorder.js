// 操作記録（rrweb）とお問い合わせのフロントエンド。
//
// app.js より先に読み込み、記録用のフローティング UI と一覧画面を提供する。
// rrweb 本体（約 280KB）は記録開始時まで読み込まない。
(() => {
  "use strict";

  const MAX_EVENTS = 4500;
  const FLUSH_INTERVAL_MS = 5000;
  const MAX_BATCH = 100;
  const MAX_BATCH_BYTES = 1_500_000;

  const rec = {
    active: false,
    starting: false,
    submitting: false,
    id: null,
    token: null,
    stop: null,
    startedAt: 0,
    duration: 0,
    queue: [],
    flushing: null,
    total: 0,
    eventCount: 0,
    uploadErrorShown: false,
    limitReached: false,
    tickTimer: null,
    flushTimer: null,
    view: "idle",
  };

  const statusMap = {
    recording: ["記録中", "open"],
    submitted: ["送信済", "accepted"],
  };

  function escapeHtml(value) {
    return String(value ?? "").replace(/[&<>"']/g, (ch) => ({
      "&": "&amp;",
      "<": "&lt;",
      ">": "&gt;",
      '"': "&quot;",
      "'": "&#39;",
    })[ch]);
  }

  function notify(message, type = "ok") {
    if (typeof toast === "function") {
      toast(message, type);
    } else {
      console.log(`[recorder] ${message}`);
    }
  }

  async function apiRequest(path, options) {
    if (typeof api !== "function") {
      throw new Error("アプリの初期化に失敗しました。");
    }
    return api(path, options);
  }

  function formatDuration(ms) {
    const total = Math.max(0, Math.round(Number(ms || 0) / 1000));
    const hours = Math.floor(total / 3600);
    const minutes = Math.floor((total % 3600) / 60);
    const seconds = total % 60;
    if (hours > 0) {
      return `${hours}:${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
    }
    return `${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
  }

  function formatDateTime(value) {
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) return String(value || "—");
    return date.toLocaleString("ja-JP", {
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
    });
  }

  function truncate(value, length = 44) {
    const text = String(value ?? "").replace(/\s+/g, " ").trim();
    if (!text) return "—";
    return text.length > length ? `${text.slice(0, length)}…` : text;
  }

  function recordingStatus(status) {
    const [label, className] = statusMap[status] || [status, ""];
    return `<span class="status ${className}">${escapeHtml(label)}</span>`;
  }

  function suggestedTitle() {
    const key = (location.hash || "").replace(/^#\/?/, "").split("/")[0] || "dashboard";
    const labels = {
      dashboard: "ダッシュボード",
      quotes: "見積",
      orders: "受注",
      sales: "売上",
      "purchase-orders": "発注",
      purchases: "仕入",
      payments: "入出金",
      closings: "締処理",
      summary: "集計",
    };
    return `${labels[key] || "画面"}の操作について`;
  }

  // ---- rrweb ---------------------------------------------------------------
  //
  // rrweb 本体は /recorder.js に同梱している（別 URL のスクリプト読み込みを
  // ブロックする拡張機能やキャッシュの影響を避けるため）。ここでは同梱コードが
  // 初期化されているかだけを確認する。

  function loadRrweb() {
    if (window.rrweb && typeof window.rrweb.record === "function") {
      return Promise.resolve(window.rrweb);
    }
    return Promise.reject(
      new Error("操作記録ライブラリを初期化できませんでした。ページを再読み込みしてください。"),
    );
  }

  // ---- ウィジェット ---------------------------------------------------------

  const widget = document.createElement("div");
  widget.id = "recorder-widget";
  widget.className = "recorder-widget rr-block";
  widget.innerHTML = `
    <section class="recorder-panel" id="recorder-panel" aria-label="操作記録とお問い合わせ" hidden>
      <header class="recorder-head">
        <strong>操作を記録してお問い合わせ</strong>
        <button type="button" class="recorder-close" data-recorder-close aria-label="閉じる">×</button>
      </header>

      <div class="recorder-body" data-recorder-view="idle">
        <p class="recorder-note">見積作成などの操作をそのまま記録し、お問い合わせに添付できます。担当者は同じ画面を再生して、困った箇所を確認できます。</p>
        <ul class="recorder-points">
          <li>入力した文字は伏せ字（マスク）で保存します。</li>
          <li>記録はこのブラウザ内で作成され、送信後に共有リンクが発行されます。</li>
          <li>記録中も画面の操作はそのまま続けられます。</li>
        </ul>
        <p class="recorder-hint" data-recorder-error hidden></p>
        <button type="button" class="primary recorder-wide" data-recorder-start>記録を開始</button>
      </div>

      <div class="recorder-body" data-recorder-view="recording" hidden>
        <p class="recorder-live" aria-live="polite">
          <span class="recorder-dot" aria-hidden="true"></span>
          <span data-recorder-time>00:00</span>
          <span data-recorder-events>0 イベント</span>
        </p>
        <p class="recorder-hint" data-recorder-error hidden></p>
        <div class="recorder-actions">
          <button type="button" class="primary" data-recorder-stop>記録を停止</button>
          <button type="button" class="secondary" data-recorder-cancel>破棄</button>
        </div>
      </div>

      <form class="recorder-body" data-recorder-view="review" hidden>
        <p class="recorder-summary" data-recorder-summary></p>
        <p class="recorder-note">記録した操作にお問い合わせ内容を添えて送信します。</p>
        <label class="recorder-field">
          <span>件名</span>
          <input name="title" maxlength="100" required />
        </label>
        <label class="recorder-field">
          <span>お名前 <small>任意</small></span>
          <input name="name" maxlength="50" autocomplete="name" />
        </label>
        <label class="recorder-field">
          <span>メールアドレス <small>任意</small></span>
          <input name="email" type="email" maxlength="200" autocomplete="email" />
        </label>
        <label class="recorder-field">
          <span>お問い合わせ内容</span>
          <textarea name="message" rows="4" maxlength="2000" required placeholder="どの操作で困ったか、期待した結果などを書いてください。"></textarea>
        </label>
        <p class="recorder-hint" data-recorder-form-error hidden></p>
        <div class="recorder-actions">
          <button type="submit" class="primary" data-recorder-submit>送信する</button>
          <button type="button" class="secondary" data-recorder-cancel>破棄</button>
        </div>
      </form>

      <div class="recorder-body" data-recorder-view="done" hidden>
        <p class="recorder-note">お問い合わせを受け付けました。以下のリンクから、記録した操作を再生できます。</p>
        <a class="recorder-link" data-recorder-link target="_blank" rel="noopener"></a>
        <div class="recorder-actions">
          <button type="button" class="secondary" data-recorder-copy>リンクをコピー</button>
          <button type="button" class="primary" data-recorder-again>閉じる</button>
        </div>
      </div>
    </section>

    <div class="recorder-mini" data-recorder-mini hidden>
      <button type="button" class="recorder-mini-status" data-recorder-mini-open aria-label="記録の詳細を開く">
        <span class="recorder-dot" aria-hidden="true"></span>
        <span data-recorder-mini-time>00:00</span>
        <span class="recorder-mini-events" data-recorder-mini-events>0件</span>
      </button>
      <button type="button" class="recorder-mini-stop" data-recorder-mini-stop>停止</button>
    </div>

    <button type="button" class="recorder-fab" data-recorder-fab aria-expanded="false" aria-controls="recorder-panel">
      <span class="recorder-fab-dot" aria-hidden="true"></span>
      <span data-recorder-fab-label>操作記録</span>
    </button>
  `;

  const panel = widget.querySelector("#recorder-panel");
  const fab = widget.querySelector("[data-recorder-fab]");
  const fabLabel = widget.querySelector("[data-recorder-fab-label]");
  const mini = widget.querySelector("[data-recorder-mini]");
  const miniTime = widget.querySelector("[data-recorder-mini-time]");
  const miniEvents = widget.querySelector("[data-recorder-mini-events]");

  function showView(view) {
    rec.view = view;
    widget.querySelectorAll("[data-recorder-view]").forEach((element) => {
      element.hidden = element.dataset.recorderView !== view;
    });
    updateFab();
  }

  function showError(selector, message) {
    const element = widget.querySelector(selector);
    if (!element) return;
    element.textContent = message || "";
    element.hidden = !message;
  }

  function updateFab() {
    const label = rec.active
      ? `記録中 ${formatDuration(Date.now() - rec.startedAt)}`
      : rec.view === "review"
        ? "記録を確認"
        : "操作記録";
    fabLabel.textContent = label;
    fab.classList.toggle("recording", rec.active);
    fab.hidden = rec.active;
    mini.hidden = !rec.active;
    if (rec.active) {
      miniTime.textContent = formatDuration(Date.now() - rec.startedAt);
      miniEvents.textContent = `${rec.total}件`;
    }
  }

  function updateClock() {
    if (!rec.active) return;
    widget.querySelector("[data-recorder-time]").textContent = formatDuration(
      Date.now() - rec.startedAt,
    );
    widget.querySelector("[data-recorder-events]").textContent = `${rec.total} イベント`;
    updateFab();
  }

  function openPanel() {
    panel.hidden = false;
    fab.setAttribute("aria-expanded", "true");
  }

  function closePanel() {
    panel.hidden = true;
    fab.setAttribute("aria-expanded", "false");
  }

  // ---- 記録の開始・停止・送信 ----------------------------------------------

  function resetState() {
    rec.active = false;
    rec.starting = false;
    rec.submitting = false;
    rec.id = null;
    rec.token = null;
    rec.stop = null;
    rec.startedAt = 0;
    rec.duration = 0;
    rec.queue = [];
    rec.flushing = null;
    rec.total = 0;
    rec.eventCount = 0;
    rec.uploadErrorShown = false;
    rec.limitReached = false;
    if (rec.tickTimer) clearInterval(rec.tickTimer);
    if (rec.flushTimer) clearInterval(rec.flushTimer);
    rec.tickTimer = null;
    rec.flushTimer = null;
    showError("[data-recorder-error]", "");
    showError("[data-recorder-form-error]", "");
    showView("idle");
  }

  function stopRrweb() {
    if (rec.stop) {
      try {
        rec.stop();
      } catch {
        // 記録停止時の例外は無視してよい
      }
      rec.stop = null;
    }
  }

  function clearTimers() {
    if (rec.tickTimer) clearInterval(rec.tickTimer);
    if (rec.flushTimer) clearInterval(rec.flushTimer);
    rec.tickTimer = null;
    rec.flushTimer = null;
  }

  // イベント数だけでなくバイト数でも分割する。一覧画面の全行追加など、
  // 1 イベントが数百 KB になることがあるため。
  function takeBatch() {
    const encoder = new TextEncoder();
    const batch = [];
    let bytes = 0;
    while (rec.queue.length && batch.length < MAX_BATCH) {
      const event = rec.queue[0];
      const size = encoder.encode(JSON.stringify(event)).length;
      if (batch.length > 0 && bytes + size > MAX_BATCH_BYTES) break;
      rec.queue.shift();
      batch.push(event);
      bytes += size;
      if (bytes >= MAX_BATCH_BYTES) break;
    }
    return batch;
  }

  async function flushBatch() {
    if (rec.flushing) return rec.flushing;
    if (!rec.id || !rec.queue.length) return;
    const batch = takeBatch();
    rec.flushing = apiRequest(`/api/recordings/${rec.id}/events`, {
      method: "POST",
      body: JSON.stringify({ events: batch }),
    })
      .then((summary) => {
        if (summary && typeof summary.event_count === "number") {
          rec.eventCount = summary.event_count;
        }
        rec.uploadErrorShown = false;
        return summary;
      })
      .catch((error) => {
        // 保存上限に達した場合は、これ以上送っても成功しないため破棄して
        // ここまでの記録で送信できるようにする。
        if (/サイズ上限|イベント上限|大きすぎ/.test(error.message)) {
          rec.queue = [];
          rec.limitReached = true;
          if (!rec.uploadErrorShown) {
            rec.uploadErrorShown = true;
            notify(
              "記録の保存上限に達しました。ここまでの操作内容でお問い合わせできます。",
              "error",
            );
          }
          if (rec.active) {
            setTimeout(() => {
              if (rec.active) stopRecording(true);
            }, 0);
          }
          return null;
        }

        // 通信エラーなどは、失敗したバッチを先頭に戻して次回の送信で再試行する
        rec.queue = batch.concat(rec.queue);
        if (!rec.uploadErrorShown) {
          rec.uploadErrorShown = true;
          notify(`記録の送信に失敗しました: ${error.message}`, "error");
        }
        throw error;
      })
      .finally(() => {
        rec.flushing = null;
      });
    return rec.flushing;
  }

  async function flushAll() {
    let attempts = 0;
    while (true) {
      if (rec.flushing) {
        try {
          await rec.flushing;
        } catch {
          // 直前の失敗は下の flushBatch で再試行する
        }
      }
      if (!rec.queue.length) return;
      await flushBatch();
      attempts += 1;
      if (attempts > 100) {
        throw new Error("記録を送信できませんでした。通信環境を確認してください。");
      }
    }
  }

  async function startRecording() {
    if (rec.active || rec.starting || rec.submitting) return;
    rec.starting = true;
    openPanel();
    showView("idle");
    showError("[data-recorder-error]", "");
    const startButton = widget.querySelector("[data-recorder-start]");
    const originalLabel = startButton.textContent;
    startButton.disabled = true;
    startButton.textContent = "準備中…";

    let created = null;
    try {
      await loadRrweb();
      created = await apiRequest("/api/recordings", {
        method: "POST",
        body: JSON.stringify({
          page: location.hash || "/",
          user_agent: navigator.userAgent,
          viewport_width: window.innerWidth,
          viewport_height: window.innerHeight,
          started_at: new Date().toISOString(),
        }),
      });

      rec.id = created.id;
      rec.token = created.token;
      rec.startedAt = Date.now();
      rec.duration = 0;
      rec.queue = [];
      rec.total = 0;
      rec.eventCount = 0;
      rec.uploadErrorShown = false;
      rec.limitReached = false;
      rec.active = true;

      rec.stop = window.rrweb.record({
        emit(event) {
          rec.total += 1;
          rec.queue.push(event);
          if (rec.active && rec.total >= MAX_EVENTS) {
            stopRecording(true);
          }
        },
        maskAllInputs: true,
        maskInputOptions: { password: true, email: true, tel: true },
        maskTextSelector: "[data-recording-mask]",
        blockSelector: ".rr-block",
        recordCanvas: false,
        collectFonts: false,
        inlineStylesheet: true,
        sampling: {
          mousemove: 100,
          mouseInteraction: true,
          scroll: 100,
          media: 800,
          input: "last",
        },
        checkoutEveryNms: 60000,
      });

      if (typeof rec.stop !== "function") {
        throw new Error("操作記録を開始できませんでした。");
      }

      showView("recording");
      // 記録中は大きなパネルを閉じ、右下のコンパクトなバーだけにする。
      closePanel();
      updateClock();
      rec.tickTimer = setInterval(updateClock, 1000);
      rec.flushTimer = setInterval(() => {
        flushBatch().catch(() => {
          // 通知は flushBatch 内で 1 回だけ行う
        });
      }, FLUSH_INTERVAL_MS);
      notify("操作の記録を開始しました。");
    } catch (error) {
      rec.active = false;
      stopRrweb();
      clearTimers();
      if (created && created.id) {
        apiRequest(`/api/recordings/${created.id}`, { method: "DELETE" }).catch(() => {});
      }
      showView("idle");
      showError("[data-recorder-error]", error.message);
      notify(error.message, "error");
    } finally {
      rec.starting = false;
      startButton.disabled = false;
      startButton.textContent = originalLabel;
    }
  }

  async function stopRecording(auto = false) {
    if (!rec.active) return;
    rec.active = false;
    stopRrweb();
    clearTimers();
    rec.duration = Math.max(0, Date.now() - rec.startedAt);
    updateFab();

    showView("review");
    openPanel();
    widget.querySelector("[data-recorder-summary]").textContent =
      `${formatDuration(rec.duration)} / ${rec.total} イベント / ${location.hash || "/"}`;
    const form = widget.querySelector('form[data-recorder-view="review"]');
    form.elements.title.value = suggestedTitle();
    form.elements.message.value = "";
    form.elements.message.focus();
    showError("[data-recorder-form-error]", "");

    try {
      await flushAll();
      widget.querySelector("[data-recorder-summary]").textContent =
        `${formatDuration(rec.duration)} / ${rec.eventCount} イベント / ${location.hash || "/"}`;
      if (rec.limitReached) {
        showError(
          "[data-recorder-form-error]",
          "記録の保存上限に達したため、以降の操作は含まれません。ここまでの内容で送信できます。",
        );
      }
    } catch (error) {
      showError("[data-recorder-form-error]", `記録を保存できませんでした: ${error.message}`);
    }

    if (auto) {
      notify("記録の上限に達したため停止しました。内容を確認して送信してください。", "error");
    } else {
      notify("記録を停止しました。お問い合わせ内容を入力してください。");
    }
  }

  async function discardRecording() {
    if (!confirm("この操作記録を破棄しますか？")) return;
    const id = rec.id;
    rec.active = false;
    stopRrweb();
    clearTimers();
    resetState();
    if (id) {
      try {
        await apiRequest(`/api/recordings/${id}`, { method: "DELETE" });
        notify("操作記録を破棄しました。");
      } catch (error) {
        notify(`記録の削除に失敗しました: ${error.message}`, "error");
      }
    }
  }

  async function submitInquiry(event) {
    event.preventDefault();
    if (rec.submitting || !rec.id) return;
    const form = event.currentTarget;
    const submit = form.querySelector("[data-recorder-submit]");
    showError("[data-recorder-form-error]", "");

    const payload = {
      title: form.elements.title.value,
      name: form.elements.name.value,
      email: form.elements.email.value,
      message: form.elements.message.value,
      duration_ms: rec.duration,
      page: location.hash || "/",
    };

    rec.submitting = true;
    submit.disabled = true;
    submit.textContent = "送信中…";
    try {
      await flushAll();
      const summary = await apiRequest(`/api/recordings/${rec.id}/finish`, {
        method: "POST",
        body: JSON.stringify(payload),
      });
      rec.token = summary.token;
      rec.eventCount = summary.event_count;
      const link = `${location.origin}/replay/${encodeURIComponent(summary.token)}`;
      const linkElement = widget.querySelector("[data-recorder-link]");
      linkElement.href = link;
      linkElement.textContent = link;
      showView("done");
      notify("お問い合わせを送信しました。");
    } catch (error) {
      showError("[data-recorder-form-error]", `送信できませんでした: ${error.message}`);
      notify(error.message, "error");
    } finally {
      rec.submitting = false;
      submit.disabled = false;
      submit.textContent = "送信する";
    }
  }

  // ---- 一覧画面 -------------------------------------------------------------

  async function renderRecordings() {
    if (typeof app === "undefined" || !app) return;
    app.innerHTML = `<section>
      <div class="card">
        <div class="toolbar recorder-toolbar">
          <div class="recorder-list-head">
            <strong>操作記録とお問い合わせ</strong>
            <span>操作を記録すると、担当者が同じ画面を再生して手順を確認できます。入力値はマスクされます。</span>
          </div>
          <button type="button" class="primary" id="recording-start">＋ 操作を記録</button>
        </div>
        <div class="table-wrap">
          <table class="list">
            <thead>
              <tr>
                <th>受付</th>
                <th>状態</th>
                <th>件名</th>
                <th>お問い合わせ</th>
                <th>記録日時</th>
                <th>長さ</th>
                <th class="num">イベント</th>
                <th data-nosort></th>
              </tr>
            </thead>
            <tbody data-recording-rows>
              <tr><td colspan="8" class="empty">読み込み中…</td></tr>
            </tbody>
          </table>
        </div>
      </div>
    </section>`;

    const startButton = document.getElementById("recording-start");
    if (startButton) {
      startButton.addEventListener("click", () => {
        openPanel();
        startRecording();
      });
    }

    let recordings = [];
    try {
      recordings = await apiRequest("/api/recordings");
    } catch (error) {
      const tbody = document.querySelector("[data-recording-rows]");
      if (tbody) {
        tbody.innerHTML = `<tr><td colspan="8" class="empty">操作記録を読み込めませんでした: ${escapeHtml(error.message)}</td></tr>`;
      }
      return;
    }

    // 読み込み中に別画面へ移動していたら描画しない
    if (typeof currentRoute === "function" && currentRoute().type !== "recordings") return;

    const tbody = document.querySelector("[data-recording-rows]");
    if (!tbody) return;

    if (!recordings.length) {
      tbody.innerHTML = `<tr><td colspan="8" class="empty">操作記録はまだありません。「操作を記録」から、お問い合わせしたい操作を記録できます。</td></tr>`;
      return;
    }

    tbody.innerHTML = recordings
      .map((recording) => {
        const duration =
          recording.duration_ms > 0 ? formatDuration(recording.duration_ms) : "—";
        const replay = recording.token
          ? `<a class="link" href="/replay/${encodeURIComponent(recording.token)}" target="_blank" rel="noopener">再生</a>`
          : "";
        return `<tr>
          <td class="mono">#${recording.id}</td>
          <td>${recordingStatus(recording.status)}</td>
          <td class="subject-cell" title="${escapeHtml(recording.title)}">${escapeHtml(recording.title || "操作記録")}</td>
          <td class="subject-cell" title="${escapeHtml(recording.message)}">${escapeHtml(truncate(recording.message))}</td>
          <td>${escapeHtml(formatDateTime(recording.created_at))}</td>
          <td class="mono">${escapeHtml(duration)}</td>
          <td class="num">${Number(recording.event_count || 0).toLocaleString("ja-JP")}</td>
          <td class="actions">
            ${replay}
            <button type="button" class="link danger" data-recording-delete="${recording.id}">削除</button>
          </td>
        </tr>`;
      })
      .join("");
  }

  // ---- イベント -------------------------------------------------------------

  fab.addEventListener("click", () => {
    if (panel.hidden) {
      openPanel();
    } else {
      closePanel();
    }
  });

  widget.querySelector("[data-recorder-close]").addEventListener("click", closePanel);
  widget.querySelector("[data-recorder-start]").addEventListener("click", startRecording);
  widget.querySelector("[data-recorder-stop]").addEventListener("click", () => stopRecording(false));
  widget.querySelector("[data-recorder-mini-open]").addEventListener("click", () => {
    if (rec.active) showView("recording");
    openPanel();
  });
  widget.querySelector("[data-recorder-mini-stop]").addEventListener("click", () => stopRecording(false));
  widget.querySelectorAll("[data-recorder-cancel]").forEach((button) => {
    button.addEventListener("click", discardRecording);
  });
  widget.querySelector('form[data-recorder-view="review"]').addEventListener("submit", submitInquiry);
  widget.querySelector("[data-recorder-copy]").addEventListener("click", async () => {
    const linkElement = widget.querySelector("[data-recorder-link]");
    try {
      await navigator.clipboard.writeText(linkElement.href);
      notify("共有リンクをコピーしました。");
    } catch {
      notify("リンクをコピーできませんでした。表示された URL を選択してコピーしてください。", "error");
    }
  });
  widget.querySelector("[data-recorder-again]").addEventListener("click", () => {
    resetState();
    closePanel();
  });

  document.addEventListener("click", async (event) => {
    const button = event.target.closest("[data-recording-delete]");
    if (!button) return;
    event.preventDefault();
    const id = button.dataset.recordingDelete;
    if (!id || !confirm("この操作記録を削除しますか？")) return;
    try {
      await apiRequest(`/api/recordings/${id}`, { method: "DELETE" });
      notify("操作記録を削除しました。");
      renderRecordings();
    } catch (error) {
      notify(error.message, "error");
    }
  });

  window.addEventListener("beforeunload", (event) => {
    if (rec.active) {
      event.preventDefault();
      event.returnValue = "";
    }
  });

  if (document.body) {
    document.body.appendChild(widget);
  } else {
    document.addEventListener("DOMContentLoaded", () => document.body.appendChild(widget));
  }

  window.RecorderView = {
    render: renderRecordings,
    open: openPanel,
    start: startRecording,
  };
})();
