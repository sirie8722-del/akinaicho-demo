use std::cell::RefCell;
use std::collections::HashMap;

use worker::{Env, Request};

/// 1 IP あたりの更新系リクエスト上限（1 分あたり）。
const MUTATION_WINDOW_MS: f64 = 60_000.0;
const MAX_MUTATIONS_PER_WINDOW: u32 = 60;
const MAX_TRACKED_IPS: usize = 4096;

thread_local! {
    static MUTATION_WINDOWS: RefCell<HashMap<String, (f64, u32)>> = RefCell::new(HashMap::new());
}

/// 公開デモ向けの簡易ガード。
///
/// - `DEMO_READ_ONLY=true` が設定されていれば更新系を拒否する。
/// - 本番（`cf-connecting-ip` あり）では IP ごとに 60 更新/分に制限する。
/// - ローカル開発・CI（`127.0.0.1` / `::1` / ヘッダなし）は制限しない。
///
/// isolate ごとのメモリで保持するベストエフォートの制限です。
pub fn check_mutation(req: &Request, env: &Env) -> Result<(), (u16, String)> {
    let read_only = env
        .var("DEMO_READ_ONLY")
        .ok()
        .map(|value| value.to_string().eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    if read_only {
        return Err((403, "このデモは読み取り専用です。".to_string()));
    }

    let Some(ip) = req.headers().get("cf-connecting-ip").ok().flatten() else {
        return Ok(());
    };
    if ip.is_empty() || ip == "127.0.0.1" || ip == "::1" {
        return Ok(());
    }

    let now = js_sys::Date::now();
    let allowed = MUTATION_WINDOWS.with(|windows| {
        let mut windows = windows.borrow_mut();
        if windows.len() > MAX_TRACKED_IPS {
            windows.retain(|_, (start, _)| now - *start < MUTATION_WINDOW_MS);
        }
        let entry = windows.entry(ip).or_insert((now, 0));
        if now - entry.0 >= MUTATION_WINDOW_MS {
            *entry = (now, 0);
        }
        if entry.1 >= MAX_MUTATIONS_PER_WINDOW {
            false
        } else {
            entry.1 += 1;
            true
        }
    });

    if allowed {
        Ok(())
    } else {
        Err((
            429,
            "リクエストが多すぎます。しばらく待ってから再試行してください。".to_string(),
        ))
    }
}
