mod setup;

use requests;
use http::Method;
use self::setup::*;

#[test]
fn test_latency() {
    setup();
    let body = get_body(
        requests::create_connection(Method::Get, url("/slow_a"), Some("Q".to_string()))
            .with_timeout(1)
            .send(),
    );
    assert_ne!(body, "j: Q");
}

#[test]
fn test_get() {
    setup();
    let body = get_body(requests::get(url("/a"), Some("Q".to_string())));
    assert_eq!(body, "j: Q");
}

#[test]
fn test_head() {
    setup();
    assert_eq!(get_status_code(requests::head(url("/b"))), 420);
}

#[test]
fn test_post() {
    setup();
    let body = get_body(requests::post(url("/c"), "E".to_string()));
    assert_eq!(body, "l: E");
}

#[test]
fn test_put() {
    setup();
    let body = get_body(requests::put(url("/d"), "R".to_string()));
    assert_eq!(body, "m: R");
}

#[test]
fn test_delete() {
    setup();
    assert_eq!(get_body(requests::delete(url("/e"))), "n:");
}

#[test]
fn test_trace() {
    setup();
    assert_eq!(get_body(requests::trace(url("/f"))), "o:");
}

#[test]
fn test_options() {
    setup();
    let body = get_body(requests::options(url("/g"), Some("U".to_string())));
    assert_eq!(body, "p: U");
}

#[test]
fn test_connect() {
    setup();
    let body = get_body(requests::connect(url("/h"), "I".to_string()));
    assert_eq!(body, "q: I");
}

#[test]
fn test_patch() {
    setup();
    let body = get_body(requests::patch(url("/i"), "O".to_string()));
    assert_eq!(body, "r: O");
}
