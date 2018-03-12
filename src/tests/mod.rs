mod setup;

use requests;
use self::setup::*;

#[test]
fn test_get() {
    setup();
    let response = requests::get(url("/boop")).body;
    assert_eq!(response, "beep");
}

#[test]
fn test_post() {
    setup();
    assert_eq!(
        requests::post(url("/insert"), "boop".to_string()).body,
        "ok"
    );
    assert_eq!(
        requests::post(url("/insert"), "beep".to_string()).body,
        "ok"
    );
    assert_eq!(requests::get(url("/list")).body, "[\"boop\", \"beep\"]");
}
