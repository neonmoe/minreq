mod setup;

use requests;
use self::setup::*;

#[test]
fn test_requests_get() {
    setup();
    assert_eq!(
        requests::get(url("/boop"), None).unwrap().body.trim(),
        "beep"
    );
}

#[test]
fn test_requests_post() {
    setup();
    assert_eq!(
        requests::post(url("/clear"), None).unwrap().body.trim(),
        "ok"
    );
    assert_eq!(
        requests::post(url("/insert"), Some("boop".to_string()))
            .unwrap()
            .body
            .trim(),
        "ok"
    );
    assert_eq!(
        requests::post(url("/insert"), Some("beep".to_string()))
            .unwrap()
            .body
            .trim(),
        "ok"
    );
    assert_eq!(
        requests::get(url("/list"), None).unwrap().body.trim(),
        "[\"boop\", \"beep\"]"
    );
}
