mod setup;

use requests;
use self::setup::*;

#[test]
fn test_requests() {
    setup();
    assert_eq!(requests::delete(url("/list")).unwrap().body.trim(), "ok");
    assert_eq!(
        requests::put(url("/insert"), "boop".to_string())
            .unwrap()
            .body
            .trim(),
        "ok"
    );
    assert_eq!(
        requests::put(url("/insert"), "beep".to_string())
            .unwrap()
            .body
            .trim(),
        "ok"
    );
    assert_eq!(
        requests::get(url("/list"), None).unwrap().body.trim(),
        "[\"boop\", \"beep\"]"
    );
    assert_eq!(requests::delete(url("/list")).unwrap().body.trim(), "ok");
    assert_eq!(requests::get(url("/list"), None).unwrap().body.trim(), "[]");
}
