//! Types and functions for interacting with the Cloudflare DDNS worker.

use std::error;
use std::fmt;
use std::net::IpAddr;

use cf_ddns::Response;
use reqwest::blocking::get;

/// Errors that can occur when getting the IP address from the DDNS worker.
#[derive(Debug)]
pub enum GetIpError {
    /// A request was not successfully sent.
    RequestFailed(reqwest::Error),

    /// The response from the worker was not JSON.
    ResponseNotJson(reqwest::Error),

    /// The response from the worker was not successful.
    UnsuccessfulResponse(Response),
}

impl fmt::Display for GetIpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::RequestFailed(e) => write!(f, "Failed to send request: {e}."),
            Self::ResponseNotJson(e) => write!(f, "Failed to parse JSON response: {e}."),
            Self::UnsuccessfulResponse(response) => {
                let errors = &response.errors;
                write!(f, "Response was not successful: errors: {errors:?}")
            }
        }
    }
}

impl error::Error for GetIpError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::RequestFailed(e) | Self::ResponseNotJson(e) => Some(e),
            Self::UnsuccessfulResponse(_) => None,
        }
    }
}

/// Get the IP address from the given DDNS worker URL.
pub fn get_ip(url: &str) -> Result<IpAddr, GetIpError> {
    let response = get(url)
        .map_err(GetIpError::RequestFailed)?
        .json::<Response>()
        .map_err(GetIpError::ResponseNotJson)?;

    if let Some(ip) = response.result {
        Ok(ip)
    } else {
        Err(GetIpError::UnsuccessfulResponse(response))
    }
}
