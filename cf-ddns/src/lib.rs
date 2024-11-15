//! Common code for the cf-ddns worker and client.
#![deny(missing_docs)]

use std::error;
use std::fmt;
use std::net::IpAddr;

use serde::{Deserialize, Serialize};

/// The possible errors that can occur in the cf-ddns worker.
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum Error {
    /// The `CF-Connecting-IP` header was not found in the request.
    HeaderNotFound,

    /// The IP address in the `CF-Connecting-IP` header is invalid. Contains the invalid IP address.
    InvalidIp(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::HeaderNotFound => write!(f, "CF-Connecting-IP header not found."),
            Error::InvalidIp(ip) => write!(f, "Invalid IP address: {}", ip),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

impl From<Error> for ResponseInfo<Error> {
    fn from(err: Error) -> Self {
        let message = err.to_string();
        Self { code: err, message }
    }
}

/// A possible error or message included alongside a response.
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct ResponseInfo<TCode> {
    /// The error code. Could be an enum variant, an integer, or a string.
    ///
    /// If it's an enum variant, it should be deserializable to the type `TCode` and may contain additional details.
    pub code: TCode,

    /// A human-readable message describing the error or message.
    pub message: String,
}

/// The response from the cf-ddns worker.
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Response {
    /// Whether the request was successful.
    pub success: bool,

    /// Any errors that occurred during the request.
    pub errors: Vec<ResponseInfo<Error>>,

    /// The result of the request, if successful.
    pub result: Option<IpAddr>,
}

impl From<IpAddr> for Response {
    fn from(ip: IpAddr) -> Self {
        Self {
            success: true,
            errors: vec![],
            result: Some(ip),
        }
    }
}

impl From<Error> for Response {
    fn from(err: Error) -> Self {
        Self {
            success: false,
            errors: vec![err.into()],
            result: None,
        }
    }
}

impl From<Result<IpAddr, Error>> for Response {
    fn from(result: Result<IpAddr, Error>) -> Self {
        match result {
            Ok(ip) => ip.into(),
            Err(err) => err.into(),
        }
    }
}
