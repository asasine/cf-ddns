use worker::*;

use std::net::{IpAddr, Ipv4Addr};

/// The possible responses from the Cloudflare Worker.
#[derive(Debug, PartialEq)]
pub enum Responses {
    Ok(Ipv4Addr),
    BadRequest(CfConnectingIpError),
}

impl From<Responses> for Result<Response> {
    fn from(resp: Responses) -> Self {
        match resp {
            Responses::Ok(ip) => Response::ok(ip.to_string()),
            Responses::BadRequest(err) => Ok(err.into()),
        }
    }
}

/// Possible errors that can occur when trying to get the connecting IP address.
#[derive(Debug, PartialEq)]
pub enum CfConnectingIpError {
    /// The `CF-Connecting-IP` header was not found in the request.
    HeaderNotFound,

    /// The IP address in the `CF-Connecting-IP` header is invalid. Contains the invalid IP address.
    InvalidIp(String),

    /// The IP address in the `CF-Connecting-IP` header is an IPv6 address, which is not supported.
    V6NotSupported,
}

impl From<CfConnectingIpError> for Response {
    fn from(err: CfConnectingIpError) -> Self {
        match err {
            CfConnectingIpError::HeaderNotFound => {
                Response::error("CF-Connecting-IP header not found.", 400)
            }
            CfConnectingIpError::InvalidIp(ip) => {
                Response::error(format!("Invalid IP address: {}", ip), 400)
            }
            CfConnectingIpError::V6NotSupported => Response::error("IPv6 is not supported.", 400),
        }
        .expect("400 should be a valid status code.")
    }
}

pub fn respond(headers: &Headers) -> Responses {
    headers
        .get("CF-Connecting-IP")
        .expect("CF-Connecting-IP should be a valid header identifier.")
        .ok_or(Err(CfConnectingIpError::HeaderNotFound))
        .and_then(|ip| {
            ip.parse::<IpAddr>()
                .map_err(|_| Err(CfConnectingIpError::InvalidIp(ip)))
        })
        .map(|ip| ip.to_canonical()) // ::ffff:7f00:1 -> 127.0.0.1
        .map_or_else(std::convert::identity, |ip| match ip {
            IpAddr::V4(addr) => Ok(addr),
            IpAddr::V6(_) => Err(CfConnectingIpError::V6NotSupported),
        })
        .map_or_else(Responses::BadRequest, Responses::Ok)
}

#[event(fetch)]
async fn fetch(req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();
    respond(req.headers()).into()
}
