use wasm_bindgen::prelude::*;
use worker::*;

/// Workers Free プランの上限（2026-10 時点）。
pub const FREE_PLAN_CPU_MS: f64 = 10.0;
pub const FREE_PLAN_MEMORY_MIB: f64 = 128.0;
pub const FREE_PLAN_MEMORY_BYTES: usize = 128 * 1024 * 1024;
pub const FREE_PLAN_REQUESTS_PER_DAY: u64 = 100_000;

const BYTES_PER_MIB: f64 = 1024.0 * 1024.0;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = performance, js_name = now)]
    fn performance_now() -> f64;
}

/// Workers ランタイムの `performance.now()`。ミリ秒（小数あり）。
pub fn now_ms() -> f64 {
    performance_now()
}

/// このモジュールの WebAssembly リニアメモリ使用量。
///
/// isolate 全体（JS ヒープを含む）のメモリではない点に注意。
pub fn wasm_memory_bytes() -> usize {
    #[cfg(target_arch = "wasm32")]
    {
        core::arch::wasm32::memory_size::<0>() * 64 * 1024
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        0
    }
}

pub fn mib(bytes: usize) -> f64 {
    bytes as f64 / BYTES_PER_MIB
}

pub fn round3(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}

/// レスポンスに計測値を付与する。ヘッダ設定の失敗はリクエストを失敗させない。
pub fn annotate(res: &mut Response, wall_ms: f64, mem_start: usize, mem_end: usize) {
    let headers = res.headers_mut();
    let _ = headers.set("server-timing", &format!("app;dur={wall_ms:.3}"));
    let _ = headers.set("x-worker-wall-time-ms", &format!("{wall_ms:.3}"));
    let _ = headers.set(
        "x-wasm-memory-kib",
        &format!("{}", mem_end.max(mem_start) / 1024),
    );
}

/// 1 リクエスト分の計測値をログに出す。
pub fn log_request(
    method: &str,
    path: &str,
    status: u16,
    wall_ms: f64,
    mem_start: usize,
    mem_end: usize,
) {
    console_log!(
        "[metrics] {} {} -> {} | wall={:.3}ms | wasm_mem={:.3}MiB (start={:.3}MiB) | cpu_limit={}ms | near_cpu_limit={}",
        method,
        path,
        status,
        wall_ms,
        mib(mem_end),
        mib(mem_start),
        FREE_PLAN_CPU_MS,
        signal::is_near_cpu_limit(),
    );
}
