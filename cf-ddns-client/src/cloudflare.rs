//! Types and functions for calling the Cloudflare API.

use std::error;
use std::fmt;

use cf_ddns::ResponseInfo;
use reqwest::{
    blocking::Client,
    header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE},
};

use serde::Deserialize;

type CfResponseInfo = ResponseInfo<i32>;

#[derive(Deserialize)]
struct PaginatedResponse<T> {
    success: bool,
    errors: Vec<CfResponseInfo>,
    result: Vec<T>,
}

#[derive(Deserialize)]
struct IdResult {
    id: String,
}

/// A client for interacting with the Cloudflare API.
pub struct Cloudflare {
    client: Client,
}

/// Errors that can occur when interacting with the Cloudflare API.
#[derive(Debug)]
pub enum CloudflareError {
    /// An error occurred while sending a request to the Cloudflare API.
    RequestFailed(reqwest::Error),

    /// An error occurred while parsing the JSON response from the Cloudflare API.
    ResponseNotJson(reqwest::Error),

    /// The Cloudflare API returned an error.
    Error(Vec<CfResponseInfo>),

    /// The response from the Cloudflare API was empty.
    EmptyResult,
}

impl fmt::Display for CloudflareError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::RequestFailed(e) => write!(f, "Failed to send request: {e}."),
            Self::ResponseNotJson(e) => write!(f, "Failed to parse JSON response: {e}."),
            Self::Error(errors) => {
                let errors = errors
                    .iter()
                    .map(|info| format!("{info:?}"))
                    .collect::<Vec<String>>()
                    .join("; ");

                write!(f, "{errors}")
            }
            Self::EmptyResult => write!(f, "No results found."),
        }
    }
}

impl error::Error for CloudflareError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::RequestFailed(e) | Self::ResponseNotJson(e) => Some(e),
            Self::Error(_) | Self::EmptyResult => None,
        }
    }
}

impl Cloudflare {
    /// Create a new Cloudflare client.
    pub fn try_new(token: &str) -> Result<Self, reqwest::Error> {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse().unwrap());
        headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
        let client = Client::builder().default_headers(headers).build()?;
        Ok(Self { client })
    }

    /// Get the zone ID of a Cloudflare DNS zone by name.
    pub fn get_zone_id(&self, name: &str) -> Result<String, CloudflareError> {
        let response: PaginatedResponse<IdResult> = self
            .client
            .get(format!(
                "https://api.cloudflare.com/client/v4/zones?name={}",
                name
            ))
            .send()
            .map_err(CloudflareError::RequestFailed)?
            .json::<PaginatedResponse<IdResult>>()
            .map_err(CloudflareError::ResponseNotJson)?;

        if !response.success {
            return Err(CloudflareError::Error(response.errors));
        }

        Ok(response
            .result
            .first()
            .ok_or(CloudflareError::EmptyResult)?
            .id
            .clone())
    }
}
