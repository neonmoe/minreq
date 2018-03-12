mod setup;

use requests;
use self::setup::*;

#[test]
fn test_get() {
    setup();
    let response = requests::get(url("/boop")).ok().unwrap();
    assert_eq!(response, "beep");
}

#[test]
fn test_post() {
    setup();
    assert_eq!(requests::post(url("/insert"), "boop").ok().unwrap(), "ok");
    assert_eq!(requests::get(url("/list")).ok().unwrap(), "[\"boop\"]");
    assert_eq!(requests::post(url("/insert"), "beep").ok().unwrap(), "ok");
    assert_eq!(
        requests::get(url("/list")).ok().unwrap(),
        "[\"boop\", \"beep\"]"
    );
}
