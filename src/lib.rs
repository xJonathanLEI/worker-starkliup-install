use worker::*;

mod utils;

const STORAGE_BINDING: &str = "STORAGE";
const SECRET_ADMIN_TOKEN: &str = "ADMIN_TOKEN";

const SCRIPT_KEY: &str = "script";

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or_else(|| "unknown region".into())
    );
}

#[event(fetch)]
pub async fn handle_fetch(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);

    // Prints error message to console on panic
    utils::set_panic_hook();

    Router::new()
        .get_async("/", |_req, ctx| async move {
            match ctx.kv(STORAGE_BINDING)?.get(SCRIPT_KEY).bytes().await? {
                Some(script) => match String::from_utf8(script) {
                    Ok(script) => Response::ok(script),
                    Err(_) => Response::error("Internal server error: invalid script", 500),
                },
                None => Response::error("Internal server error: install script not set up", 500),
            }
        })
        .post_async("/admin/script", |mut req, ctx| async move {
            if let Err(err) = bearer_auth(&req, &ctx).await {
                return Ok(err);
            }

            let new_script = req.text().await?;
            ctx.kv(STORAGE_BINDING)?
                .put_bytes(SCRIPT_KEY, new_script.as_bytes())?
                .execute()
                .await?;

            Response::ok("OK")
        })
        .or_else_any_method("", |_req, _ctx| Response::error("Not found", 404))
        .run(req, env)
        .await
}

async fn bearer_auth(req: &Request, ctx: &RouteContext<()>) -> std::result::Result<(), Response> {
    let admin_token = ctx
        .secret(SECRET_ADMIN_TOKEN)
        .map_err(|_| Response::error("Internal server error: secret missing", 500).unwrap())?
        .to_string();

    let header_auth = match req.headers().get("Authorization").unwrap() {
        Some(value) => value,
        None => return Err(Response::error("Auth header missing", 403).unwrap()),
    };
    if header_auth == format!("Bearer {admin_token}") {
        Ok(())
    } else {
        Err(Response::error("Invalid token", 403).unwrap())
    }
}
