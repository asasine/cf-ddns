//! Tests for functions that use wasm-bindgen imported functions.

use std::net::IpAddr;

use wasm_bindgen_test::*;

use cf_ddns::{Error, Response};
use cf_ddns_worker::respond;
use worker::Headers;

#[allow(dead_code)]
#[wasm_bindgen_test]
fn no_header() {
    let resp = respond(&Headers::new());
    assert_eq!(resp, Response::from(Error::HeaderNotFound));
}

#[allow(dead_code)]
#[wasm_bindgen_test]
fn invalid_ip() {
    let invalid_ip = "invalid";
    let headers = Headers::from_iter([("CF-Connecting-IP", invalid_ip)]);
    let resp = respond(&headers);
    assert_eq!(
        resp,
        Response::from(Error::InvalidIp(invalid_ip.to_string()))
    );
}

#[allow(dead_code)]
#[wasm_bindgen_test]
fn ipv4() {
    let ip = "127.0.0.1";
    let headers = Headers::from_iter([("CF-Connecting-IP", ip)]);
    let resp = respond(&headers);
    assert_eq!(
        resp,
        Response::from(IpAddr::V4(ip.parse().expect("ip should be valid.")))
    );
}

#[allow(dead_code)]
#[wasm_bindgen_test]
fn ipv6() {
    let ip = "::1";
    let headers = Headers::from_iter([("CF-Connecting-IP", ip)]);
    let resp = respond(&headers);
    assert_eq!(
        resp,
        Response::from(IpAddr::V6(ip.parse().expect("ip should be valid.")))
    );
}

#[allow(dead_code)]
#[wasm_bindgen_test]
fn ipv6_not_mapped() {
    // This is an [IPv4-mapped IPv6 address](https://en.wikipedia.org/wiki/IPv6#IPv4-mapped_IPv6_addresses)
    // The worker should not map it to an IPv4 address.
    let ip = "::ffff:7f00:1";
    let headers = Headers::from_iter([("CF-Connecting-IP", ip)]);
    let resp = respond(&headers);
    assert_eq!(
        resp,
        Response::from(IpAddr::V6(ip.parse().expect("ip should be valid.")))
    );
}
