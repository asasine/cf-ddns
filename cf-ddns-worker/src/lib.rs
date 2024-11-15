use worker::*;

use std::net::IpAddr;

use cf_ddns::{Error, Response as CfDdnsResponse};

/// Given the headers from a request, respond with a [`CfDdnsResponse`].
pub fn respond(headers: &Headers) -> CfDdnsResponse {
    headers
        .get("CF-Connecting-IP")
        .expect("CF-Connecting-IP should be a valid header identifier.")
        .ok_or(Err(Error::HeaderNotFound))
        .and_then(|ip| ip.parse::<IpAddr>().map_err(|_| Err(Error::InvalidIp(ip))))
        .or_else(std::convert::identity)
        .into()
}

#[event(fetch)]
async fn fetch(req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();
    let response = respond(req.headers());
    let code = if response.success { 200 } else { 400 };
    Response::builder().with_status(code).from_json(&response)
}
