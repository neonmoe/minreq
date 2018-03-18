extern crate minreq;
mod setup;

use self::setup::*;

#[test]
#[cfg(feature = "https")]
fn test_https() {
    // TODO: Implement this locally.
    assert_eq!(
        get_status_code(minreq::get("https://httpbin.org/status/418").send()),
        418
    );
}

#[test]
fn test_latency() {
    setup();
    let body = get_body(
        minreq::get(url("/slow_a"))
            .with_body("Q".to_string())
            .with_timeout(1)
            .send(),
    );
    assert_ne!(body, "j: Q");
}

#[test]
fn test_headers() {
    setup();
    let body = get_body(
        minreq::get(url("/header_pong"))
            .with_header("Ping", "Qwerty")
            .send(),
    );
    assert_eq!("Qwerty", body);
}

#[test]
fn test_custom_method() {
    use minreq::Method;
    setup();
    let body = get_body(
        minreq::create_request(Method::Custom("GET".to_string()), url("/a"))
            .with_body("Q")
            .send(),
    );
    assert_eq!("j: Q", body);
}

#[test]
fn test_get() {
    setup();
    let body = get_body(minreq::get(url("/a")).with_body("Q").send());
    assert_eq!(body, "j: Q");
}

#[test]
fn test_head() {
    setup();
    assert_eq!(get_status_code(minreq::head(url("/b")).send()), 418);
}

#[test]
fn test_post() {
    setup();
    let body = get_body(minreq::post(url("/c")).with_body("E").send());
    assert_eq!(body, "l: E");
}

#[test]
fn test_put() {
    setup();
    let body = get_body(minreq::put(url("/d")).with_body("R").send());
    assert_eq!(body, "m: R");
}

#[test]
fn test_delete() {
    setup();
    assert_eq!(get_body(minreq::delete(url("/e")).send()), "n:");
}

#[test]
fn test_trace() {
    setup();
    assert_eq!(get_body(minreq::trace(url("/f")).send()), "o:");
}

#[test]
fn test_options() {
    setup();
    let body = get_body(minreq::options(url("/g")).with_body("U").send());
    assert_eq!(body, "p: U");
}

#[test]
fn test_connect() {
    setup();
    let body = get_body(minreq::connect(url("/h")).with_body("I").send());
    assert_eq!(body, "q: I");
}

#[test]
fn test_patch() {
    setup();
    let body = get_body(minreq::patch(url("/i")).with_body("O").send());
    assert_eq!(body, "r: O");
}
