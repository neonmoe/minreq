use std::collections::HashMap;
use http::{Request, Response};

/// A connection to the server for sending
/// [`Request`](../http/struct.Request.html)s.
pub struct Connection {
    request: Request,
}

impl Connection {
    pub fn new(request: Request) -> Connection {
        Connection { request }
    }

    pub fn send() -> Response {
        Response::new(HashMap::new(), String::new())
    }
}
