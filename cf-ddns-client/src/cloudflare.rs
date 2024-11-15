//! Types and functions for calling the Cloudflare API.

use std::error;
use std::fmt;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;

use cf_ddns::ResponseInfo;
use reqwest::{
    blocking::Client,
    header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE},
};

use serde::Deserialize;
use serde::Serialize;

type CfResponseInfo = ResponseInfo<i32>;

#[derive(Deserialize)]
struct ListResponse<T> {
    success: bool,
    errors: Vec<CfResponseInfo>,
    result: Vec<T>,
}

#[derive(Deserialize)]
struct Response<T> {
    success: bool,
    errors: Vec<CfResponseInfo>,
    result: T,
}

#[derive(Deserialize)]
struct Id {
    id: String,
}

/// A DNS record.
#[derive(Deserialize, Serialize, Clone)]
#[serde(tag = "type")]
pub struct Record {
    /// The record ID.
    pub id: String,

    /// The name of the zone.
    pub zone_name: String,

    /// The name of the record.
    pub name: String,

    /// The content of the record.
    #[serde(flatten)]
    pub content: RecordContent,
}

/// The content of a DNS record.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type")]
pub enum RecordContent {
    /// An `A` record.
    A {
        /// The IPv4 address of the record.
        content: Ipv4Addr,
    },

    /// An `AAAA` record.
    AAAA {
        /// The IPv6 address of the record.
        content: Ipv6Addr,
    },

    /// Another type of record.
    #[serde(other)]
    Other,
}

#[derive(Serialize)]
struct UpdateRecord {
    content: String,
    r#type: String,
}

impl From<IpAddr> for UpdateRecord {
    fn from(ip: IpAddr) -> Self {
        match ip {
            IpAddr::V4(ip) => UpdateRecord {
                content: ip.to_string(),
                r#type: "A".to_string(),
            },
            IpAddr::V6(ip) => UpdateRecord {
                content: ip.to_string(),
                r#type: "AAAA".to_string(),
            },
        }
    }
}

/// A client for interacting with the Cloudflare API.
pub struct Cloudflare {
    client: Client,
}

/// Errors that can occur when interacting with the Cloudflare API.
#[derive(Debug)]
pub enum CloudflareError<T> {
    /// An error occurred while sending a request to the Cloudflare API.
    RequestFailed(reqwest::Error),

    /// An error occurred while parsing the JSON response from the Cloudflare API.
    ResponseNotJson(reqwest::Error),

    /// The Cloudflare API returned an error.
    Error(Vec<CfResponseInfo>),

    /// The response from the Cloudflare API was empty.
    EmptyResult,

    /// An error occurred that is specific to the Cloudflare API being called.
    ApiSpecific(T),
}

impl<T: fmt::Display> fmt::Display for CloudflareError<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::RequestFailed(e) => write!(f, "Failed to send request: {e}."),
            Self::ResponseNotJson(e) => write!(f, "Failed to parse JSON response: {e}."),
            Self::Error(errors) => {
                if errors.is_empty() {
                    write!(f, "The Cloudflare API returned an unknown error.")
                } else if errors.len() == 1 {
                    write!(f, "The Cloudflare API returned an error: {errors:?}")
                } else {
                    write!(f, "The Cloudflare API returned multiple errors: {errors:?}")
                }
            }
            Self::EmptyResult => write!(f, "No results found."),
            Self::ApiSpecific(inner) => write!(f, "{inner}"),
        }
    }
}

impl<T: fmt::Display + fmt::Debug> error::Error for CloudflareError<T> {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::RequestFailed(e) | Self::ResponseNotJson(e) => Some(e),
            Self::Error(_) | Self::EmptyResult | Self::ApiSpecific(_) => None,
        }
    }
}

/// No additional API-specific errors.
#[derive(Debug)]
pub struct NoApiSpecific;

impl fmt::Display for NoApiSpecific {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "An unknown error occurred.")
    }
}

impl error::Error for NoApiSpecific {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

/// Errors that can occur when getting a record ID.
#[derive(Debug)]
pub enum GetRecordIdError {
    /// The record type is not supported.
    InvalidRecordType(String),
}

impl fmt::Display for GetRecordIdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidRecordType(record_type) => {
                write!(f, "The record type '{}' is not supported.", record_type)
            }
        }
    }
}

impl error::Error for GetRecordIdError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
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
    pub fn get_zone_id(&self, name: &str) -> Result<String, CloudflareError<NoApiSpecific>> {
        let response = self
            .client
            .get(format!(
                "https://api.cloudflare.com/client/v4/zones?name={name}"
            ))
            .send()
            .map_err(CloudflareError::RequestFailed)?
            .json::<ListResponse<Id>>()
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

    /// Get the record ID of a Cloudflare DNS record by name.
    pub fn get_record_id(
        &self,
        zone_id: &str,
        name: &str,
    ) -> Result<String, CloudflareError<GetRecordIdError>> {
        let response = self
            .client
            .get(format!("https://api.cloudflare.com/client/v4/zones/{zone_id}/dns_records?name.exact={name}"))
            .send()
            .map_err(CloudflareError::RequestFailed)?
            .json::<ListResponse<Record>>()
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

    /// Update a DNS record with the new content from the given `content`.
    ///
    /// This performs a `PATCH` request to the Cloudflare API. Only the type and value of the record is updated.
    pub fn update_record(
        &self,
        zone_id: &str,
        record_id: &str,
        content: IpAddr,
    ) -> Result<Record, CloudflareError<NoApiSpecific>> {
        let request_body = UpdateRecord::from(content);
        let response = self
            .client
            .patch(format!(
                "https://api.cloudflare.com/client/v4/zones/{zone_id}/dns_records/{record_id}",
            ))
            .json(&request_body)
            .send()
            .map_err(CloudflareError::RequestFailed)?
            .json::<Response<Record>>()
            .map_err(CloudflareError::ResponseNotJson)?;

        if !response.success {
            return Err(CloudflareError::Error(response.errors));
        }

        Ok(response.result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_a_record() {
        let json =
            r#"{"type":"A","id":"123","name":"foo","zone_name":"zone","content":"10.0.0.0"}"#;
        let record: Record = serde_json::from_str(json).unwrap();
        match record.content {
            RecordContent::A { content: ip } => assert_eq!(ip, Ipv4Addr::new(10, 0, 0, 0)),
            _ => panic!("Expected an A record."),
        }
    }

    #[test]
    fn deserialize_aaaa_record() {
        let json = r#"{"type":"AAAA","id":"123","name":"foo","zone_name":"zone","content":"::1"}"#;
        let record: Record = serde_json::from_str(json).unwrap();
        match record.content {
            RecordContent::AAAA { content: ip } => assert_eq!(ip, Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)),
            _ => panic!("Expected an AAAA record."),
        }
    }

    #[test]
    fn deserialize_other_record() {
        let json = r#"{"type":"TXT","id":"123","name":"foo","zone_name":"zone","content":"example"}"#;
        let record: Record = serde_json::from_str(json).unwrap();
        match record.content {
            RecordContent::Other => {}
            _ => panic!("Expected an other record."),
        }
    }
}
