use worker::*;

const INDEX_HTML: &str = include_str!("index.html");

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();

    router
        .get("/", |_, _| Response::from_html(INDEX_HTML))
        .get("/api/health", |_, _| {
            let mut res = Response::from_json(&serde_json::json!({
                "status": "ok",
                "runtime": "cloudflare-workers",
                "language": "rust",
            }))?;
            res.headers_mut().set("cache-control", "no-store")?;
            Ok(res)
        })
        .run(req, env)
        .await
}
