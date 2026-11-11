// 操作記録の再生ページ。rrweb-player（UMD）を使う。
(() => {
  "use strict";

  const token = (location.pathname.split("/").filter(Boolean).pop() || "").trim();
  const titleElement = document.getElementById("replay-title");
  const metaElement = document.getElementById("replay-meta");
  const sourceElement = document.getElementById("replay-source");
  const playerElement = document.getElementById("player");
  const errorElement = document.getElementById("replay-error");
  const inquiryElement = document.getElementById("inquiry");
  const stepsElement = document.getElementById("steps");
  const stepCountElement = document.getElementById("step-count");
  let player = null;
  let stepInterval = null;

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
    if (Number.isNaN(date.getTime())) return "—";
    return date.toLocaleString("ja-JP", {
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
    });
  }

  function showError(message) {
    errorElement.textContent = message;
    errorElement.hidden = false;
    playerElement.hidden = true;
  }

  function textOf(node, limit = 120) {
    if (!node) return "";
    if (node.type === 3) return node.textContent || "";
    if (node.type !== 2 && node.type !== 0) return "";
    let out = "";
    const children = node.childNodes || [];
    for (const child of children) {
      out += `${textOf(child, limit)} `;
      if (out.length >= limit) break;
    }
    return out;
  }

  function truncate(value, length) {
    const text = String(value || "").replace(/\s+/g, " ").trim();
    if (!text) return "";
    return text.length > length ? `${text.slice(0, length)}…` : text;
  }

  function labelForNode(node) {
    if (!node) return "クリック";
    const attributes = node.attributes || {};
    const text = truncate(textOf(node), 48);
    if (text) return text;
    const aria = truncate(attributes["aria-label"], 48);
    if (aria) return aria;
    const title = truncate(attributes.title, 48);
    if (title) return title;
    const placeholder = truncate(attributes.placeholder, 40);
    if (placeholder) return `入力: ${placeholder}`;
    const name = truncate(attributes.name, 32);
    if (name) return `入力: ${name}`;
    const tag = String(node.tagName || "").toLowerCase();
    const labels = {
      a: "リンク",
      button: "ボタン",
      input: "入力欄",
      select: "選択",
      textarea: "入力欄",
      tr: "一覧の行",
    };
    return labels[tag] || "クリック";
  }

  // フルスナップショットと追加ノードから、ノード ID → 表示名の索引を作る。
  function buildNodeIndex(events) {
    const index = new Map();
    const visit = (node) => {
      if (!node || typeof node !== "object") return;
      if (typeof node.id === "number") index.set(node.id, node);
      for (const child of node.childNodes || []) visit(child);
    };

    for (const event of events) {
      if (event.type === 2 && event.data && event.data.node) {
        visit(event.data.node);
      } else if (event.type === 3 && event.data && event.data.source === 0) {
        for (const added of event.data.adds || []) visit(added.node);
        for (const text of event.data.texts || []) {
          const node = index.get(text.id);
          if (node) node.textContent = text.value;
        }
      }
    }
    return index;
  }

  function isClickEvent(event) {
    return (
      event.type === 3 &&
      event.data &&
      event.data.source === 2 &&
      (event.data.type === 2 || event.data.type === 4)
    );
  }

  function renderInquiry(data) {
    inquiryElement.textContent = "";
    const details = document.createElement("dl");
    const add = (label, value) => {
      if (!value) return;
      const dt = document.createElement("dt");
      dt.textContent = label;
      const dd = document.createElement("dd");
      dd.textContent = value;
      details.append(dt, dd);
    };
    add("お名前", data.name);
    add("メール", data.email);
    add("操作ページ", data.page);
    if (details.children.length) inquiryElement.appendChild(details);

    const message = document.createElement("p");
    message.className = "replay-message";
    message.textContent = data.message || "（お問い合わせ内容はありません）";
    inquiryElement.appendChild(message);
  }

  function renderSteps(events, nodeIndex, firstTimestamp) {
    const steps = events
      .filter(isClickEvent)
      .map((event) => ({
        offset: Math.max(0, event.timestamp - firstTimestamp),
        label: labelForNode(nodeIndex.get(event.data.id)),
      }))
      .slice(0, 80);

    stepCountElement.textContent = steps.length ? `${steps.length} 件` : "";
    stepsElement.textContent = "";
    if (!steps.length) {
      const empty = document.createElement("li");
      empty.className = "empty small";
      empty.textContent = "クリック操作は記録されていません。";
      stepsElement.appendChild(empty);
      return;
    }

    const items = [];
    steps.forEach((step, index) => {
      const item = document.createElement("li");
      const button = document.createElement("button");
      button.type = "button";
      button.className = "replay-step";
      button.dataset.index = String(index);

      const time = document.createElement("span");
      time.className = "replay-step-time";
      time.textContent = formatDuration(step.offset);

      const label = document.createElement("span");
      label.className = "replay-step-label";
      label.textContent = step.label;

      button.append(time, label);
      button.addEventListener("click", () => {
        if (!player || typeof player.goto !== "function") return;
        markActive(index);
        try {
          player.goto(step.offset, true);
        } catch {
          // 再生位置の移動に失敗しても一覧は維持する
        }
      });
      item.appendChild(button);
      stepsElement.appendChild(item);
      items.push(button);
    });

    function markActive(index) {
      items.forEach((item, itemIndex) => {
        item.classList.toggle("active", itemIndex === index);
      });
    }

    if (!player || typeof player.getReplayer !== "function") return;
    try {
      const replayer = player.getReplayer();
      if (!replayer || typeof replayer.getCurrentTime !== "function") return;
      stepInterval = setInterval(() => {
        let current = 0;
        try {
          current = Number(replayer.getCurrentTime()) || 0;
        } catch {
          return;
        }
        let active = 0;
        steps.forEach((step, index) => {
          if (step.offset <= current + 150) active = index;
        });
        markActive(active);
      }, 500);
    } catch {
      // 再生インスタンスが取得できない場合はハイライトなしで続行する
    }
  }

  function renderPlayer(data, events, firstTimestamp) {
    if (!window.rrwebPlayer) {
      showError("再生ライブラリを読み込めませんでした。");
      return;
    }
    if (!events.length || !events.some((event) => event.type === 2)) {
      showError("再生できる操作記録がありません。記録が途中で終了した可能性があります。");
      return;
    }

    const stage = document.querySelector(".replay-stage");
    const available = Math.max(320, (stage ? stage.clientWidth : 1024) - 16);
    const width = Math.min(1280, available);
    const ratio =
      data.viewport_width > 0 && data.viewport_height > 0
        ? data.viewport_height / data.viewport_width
        : 9 / 16;
    const height = Math.round(
      Math.min(720, Math.max(260, width * Math.min(Math.max(ratio, 0.45), 1.2))),
    );

    try {
      player = new window.rrwebPlayer({
        target: playerElement,
        props: {
          events,
          width,
          height,
          maxScale: 1,
          autoPlay: true,
          speed: 1,
          speedOption: [1, 2, 4],
          showController: true,
          showCaptions: false,
          skipInactive: false,
          // 記録対象から除外した操作記録ウィジェットの余白を再生画面に残さない。
          insertStyleRules: [".rr-block { display: none !important; }"],
        },
      });
    } catch (error) {
      showError(`操作記録を再生できませんでした: ${error.message}`);
      return;
    }

    const nodeIndex = buildNodeIndex(events);
    renderSteps(events, nodeIndex, firstTimestamp);
  }

  async function init() {
    if (!token) {
      showError("操作記録の URL が不正です。");
      return;
    }

    let data;
    try {
      const response = await fetch(`/api/replay/${encodeURIComponent(token)}`, {
        headers: { accept: "application/json" },
      });
      data = await response.json().catch(() => null);
      if (!response.ok) {
        throw new Error(data && data.error ? data.error : `HTTP ${response.status}`);
      }
    } catch (error) {
      showError(
        `操作記録を読み込めませんでした: ${error.message}（Worker の再起動でデモデータが初期化された可能性があります）`,
      );
      return;
    }

    const events = Array.isArray(data.events) ? data.events : [];
    const firstTimestamp = events.length ? events[0].timestamp : Date.now();
    const lastTimestamp = events.length ? events[events.length - 1].timestamp : firstTimestamp;
    const duration = data.duration_ms > 0 ? data.duration_ms : lastTimestamp - firstTimestamp;

    document.title = `${data.title || "操作記録"} — 商い帖`;
    titleElement.textContent = data.title || "操作記録";
    metaElement.textContent = [
      formatDateTime(data.created_at),
      formatDuration(duration),
      `${events.length.toLocaleString("ja-JP")} イベント`,
      data.page ? `操作ページ ${data.page}` : "",
    ]
      .filter(Boolean)
      .join(" / ");

    if (data.page && String(data.page).startsWith("#/")) {
      sourceElement.href = `/${data.page}`;
      sourceElement.hidden = false;
    }

    renderInquiry(data);
    renderPlayer(data, events, firstTimestamp);
  }

  window.addEventListener("beforeunload", () => {
    if (stepInterval) clearInterval(stepInterval);
  });

  init();
})();
