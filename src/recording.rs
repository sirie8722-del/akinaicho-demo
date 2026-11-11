//! 操作記録（rrweb）とお問い合わせの API。
//!
//! 記録は isolate 内のインメモリに保持するデモ実装。公開 URL には連番ではなく
//! 推測されにくいトークンを使い、一覧・削除などの管理操作だけ数値 ID を使う。

use worker::*;

use crate::api::api_error;
use crate::models::*;
use crate::store;

fn id_from(ctx: &RouteContext<()>) -> Option<u64> {
    ctx.param("id")
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|id| *id > 0)
}

fn token_from(ctx: &RouteContext<()>) -> Option<String> {
    ctx.param("token")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty() && value.len() <= 64)
}

/// 文字数を上限に切り詰める。マルチバイト文字を壊さない。
fn trim_to(value: Option<String>, max_chars: usize) -> String {
    value
        .unwrap_or_default()
        .trim()
        .chars()
        .take(max_chars)
        .collect()
}

/// 共有 URL 用のトークン。暗号学的な強度はないが、連番の推測は防ぐ。
fn new_token(store: &store::Store) -> String {
    const CHARS: &[u8] = b"23456789abcdefghjkmnpqrstuvwxyz";
    for _ in 0..20 {
        let mut token = String::with_capacity(12);
        for _ in 0..12 {
            let index = (js_sys::Math::random() * CHARS.len() as f64) as usize;
            token.push(CHARS[index.min(CHARS.len() - 1)] as char);
        }
        if !store.recording_token_exists(&token) {
            return token;
        }
    }
    format!("{:x}", js_sys::Date::now() as u64)
}

fn validate_event(event: &serde_json::Value) -> bool {
    event.is_object()
        && event.get("type").and_then(|value| value.as_u64()).is_some()
        && event
            .get("timestamp")
            .and_then(|value| value.as_f64())
            .is_some()
}

pub async fn create(mut req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let input: RecordingStartInput = match req.json().await {
        Ok(input) => input,
        Err(_) => return api_error(400, "リクエストの JSON が不正です。"),
    };

    let now = now_iso();
    let title = trim_to(input.title, 100);
    let page = trim_to(input.page, 300);
    let started_at = trim_to(input.started_at, 40);
    let user_agent = trim_to(input.user_agent, 300);
    let viewport_width = input.viewport_width.unwrap_or(0).min(10_000);
    let viewport_height = input.viewport_height.unwrap_or(0).min(10_000);

    let result = store::with_store(|store| {
        if store.recordings.len() >= store::MAX_RECORDINGS {
            return Err((
                429,
                format!(
                    "操作記録の保存上限（{} 件）に達しました。",
                    store::MAX_RECORDINGS
                ),
            ));
        }

        let id = store.next_recording_id;
        store.next_recording_id += 1;
        let recording = Recording {
            id,
            token: new_token(store),
            title: if title.is_empty() {
                "操作記録".to_string()
            } else {
                title
            },
            message: String::new(),
            name: String::new(),
            email: String::new(),
            page: if page.is_empty() {
                "/".to_string()
            } else {
                page
            },
            user_agent,
            viewport_width,
            viewport_height,
            started_at: if started_at.is_empty() {
                now.clone()
            } else {
                started_at
            },
            created_at: now.clone(),
            updated_at: now.clone(),
            duration_ms: 0,
            event_bytes: 0,
            status: "recording".to_string(),
            events: Vec::new(),
        };

        let summary = recording.summary();
        store.recordings.push(recording);
        Ok(summary)
    });

    match result {
        Ok(summary) => Response::from_json(&summary).map(|response| response.with_status(201)),
        Err((status, message)) => api_error(status, &message),
    }
}

pub async fn append_events(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Some(id) = id_from(&ctx) else {
        return api_error(400, "操作記録 ID が不正です。");
    };

    if let Ok(Some(raw)) = req.headers().get("content-length") {
        if raw
            .parse::<usize>()
            .map(|length| length > store::MAX_EVENT_BYTES_PER_BATCH * 2)
            .unwrap_or(false)
        {
            return api_error(413, "1 回に送信できる操作記録が大きすぎます。");
        }
    }

    let input: RecordingEventsInput = match req.json().await {
        Ok(input) => input,
        Err(_) => return api_error(400, "リクエストの JSON が不正です。"),
    };

    if input.events.is_empty() {
        let result = store::with_store(|store| {
            store
                .recording(id)
                .map(Recording::summary)
                .ok_or((404, "操作記録が見つかりません。".to_string()))
        });
        return match result {
            Ok(summary) => Response::from_json(&summary),
            Err((status, message)) => api_error(status, &message),
        };
    }

    if input.events.len() > store::MAX_EVENTS_PER_BATCH {
        return api_error(
            413,
            &format!(
                "1 回に送信できるイベントは {} 件までです。",
                store::MAX_EVENTS_PER_BATCH
            ),
        );
    }
    if input.events.iter().any(|event| !validate_event(event)) {
        return api_error(400, "操作記録のイベント形式が不正です。");
    }

    let batch_bytes = match serde_json::to_vec(&input.events) {
        Ok(bytes) => bytes.len(),
        Err(_) => return api_error(400, "操作記録を読み取れませんでした。"),
    };
    if batch_bytes > store::MAX_EVENT_BYTES_PER_BATCH {
        return api_error(413, "1 回に送信できる操作記録が大きすぎます。");
    }

    let now = now_iso();
    let result = store::with_store(|store| {
        let recording = store
            .recording_mut(id)
            .ok_or((404, "操作記録が見つかりません。".to_string()))?;

        if recording.status == "submitted" {
            return Err((409, "送信済みの操作記録には追加できません。".to_string()));
        }
        if recording.events.len() + input.events.len() > store::MAX_EVENTS_PER_RECORDING {
            return Err((
                413,
                format!(
                    "操作記録のイベント上限（{} 件）に達しました。",
                    store::MAX_EVENTS_PER_RECORDING
                ),
            ));
        }
        if recording.event_bytes + batch_bytes > store::MAX_EVENT_BYTES_PER_RECORDING {
            return Err((
                413,
                "操作記録のサイズ上限に達しました。短い操作に分けて記録してください。".to_string(),
            ));
        }

        recording.events.extend(input.events);
        recording.event_bytes += batch_bytes;
        recording.updated_at = now;
        Ok(recording.summary())
    });

    match result {
        Ok(summary) => Response::from_json(&summary),
        Err((status, message)) => api_error(status, &message),
    }
}

pub async fn finish(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Some(id) = id_from(&ctx) else {
        return api_error(400, "操作記録 ID が不正です。");
    };
    let input: RecordingFinishInput = match req.json().await {
        Ok(input) => input,
        Err(_) => return api_error(400, "リクエストの JSON が不正です。"),
    };

    let message = trim_to(input.message, 2_000);
    if message.is_empty() {
        return api_error(400, "お問い合わせ内容を入力してください。");
    }
    let title = trim_to(input.title, 100);
    let name = trim_to(input.name, 50);
    let email = trim_to(input.email, 200);
    if !email.is_empty() && (!email.contains('@') || email.contains(char::is_whitespace)) {
        return api_error(400, "メールアドレスの形式が不正です。");
    }
    let duration_ms = input.duration_ms.unwrap_or(0).min(86_400_000);
    let page = trim_to(input.page, 300);
    let now = now_iso();

    let result = store::with_store(|store| {
        let recording = store
            .recording_mut(id)
            .ok_or((404, "操作記録が見つかりません。".to_string()))?;

        if recording.status != "submitted" {
            if !title.is_empty() {
                recording.title = title;
            }
            recording.message = message;
            recording.name = name;
            recording.email = email;
            recording.duration_ms = duration_ms;
            if !page.is_empty() {
                recording.page = page;
            }
            recording.status = "submitted".to_string();
            recording.updated_at = now;
        }
        Ok(recording.summary())
    });

    match result {
        Ok(summary) => Response::from_json(&summary),
        Err((status, message)) => api_error(status, &message),
    }
}

pub fn list(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    store::with_store(|store| {
        let summaries: Vec<RecordingSummary> = store
            .recordings
            .iter()
            .rev()
            .map(Recording::summary)
            .collect();
        Response::from_json(&summaries)
    })
}

pub fn get_by_token(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Some(token) = token_from(&ctx) else {
        return api_error(404, "操作記録が見つかりません。");
    };
    store::with_store(|store| match store.recording_by_token(&token) {
        Some(recording) => Response::from_json(recording),
        None => api_error(404, "操作記録が見つかりません。"),
    })
}

pub fn delete(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Some(id) = id_from(&ctx) else {
        return api_error(400, "操作記録 ID が不正です。");
    };

    let result = store::with_store(|store| {
        let before = store.recordings.len();
        store.recordings.retain(|recording| recording.id != id);
        if store.recordings.len() == before {
            Err((404, "操作記録が見つかりません。".to_string()))
        } else {
            Ok(())
        }
    });

    match result {
        Ok(()) => Ok(Response::empty()?.with_status(204)),
        Err((status, message)) => api_error(status, &message),
    }
}
