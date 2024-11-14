//! Tests for functions that use wasm-bindgen imported functions.

use std::net::Ipv4Addr;

use wasm_bindgen_test::*;

use cf_ddns_worker::*;
use worker::Headers;

#[allow(dead_code)]
#[wasm_bindgen_test]
fn no_header() {
    let resp = respond(&Headers::new());
    assert_eq!(
        resp,
        Responses::BadRequest(CfConnectingIpError::HeaderNotFound)
    );
}

#[allow(dead_code)]
#[wasm_bindgen_test]
fn invalid_ip() {
    let headers = Headers::from_iter([("CF-Connecting-IP", "invalid")]);
    let resp = respond(&headers);
    assert_eq!(
        resp,
        Responses::BadRequest(CfConnectingIpError::InvalidIp("invalid".into()))
    );
}

#[allow(dead_code)]
#[wasm_bindgen_test]
fn ipv4() {
    let headers = Headers::from_iter([("CF-Connecting-IP", "127.0.0.1")]);
    let resp = respond(&headers);
    assert_eq!(resp, Responses::Ok(Ipv4Addr::new(127, 0, 0, 1)));
}

#[allow(dead_code)]
#[wasm_bindgen_test]
fn ipv6() {
    let headers = Headers::from_iter([("CF-Connecting-IP", "::1")]);
    let resp = respond(&headers);
    assert_eq!(
        resp,
        Responses::BadRequest(CfConnectingIpError::V6NotSupported)
    );
}

#[allow(dead_code)]
#[wasm_bindgen_test]
fn ipv6_mapped() {
    let headers = Headers::from_iter([("CF-Connecting-IP", "::ffff:7f00:1")]);
    let resp = respond(&headers);
    assert_eq!(resp, Responses::Ok(Ipv4Addr::new(127, 0, 0, 1)));
}
