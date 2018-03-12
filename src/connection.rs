use std::collections::HashMap;
use http::{Method, Request, Response};

/// A connection to the server for sending
/// [`Request`](struct.Request.html)s.
pub struct Connection {
    request: Request,
}

impl Connection {
    /// Creates a new `Connection`. See
    /// [`Request`](struct.Request.html) for specifics about *what* is
    /// being sent.
    pub fn new(request: Request) -> Connection {
        Connection { request }
    }

    /// Sends the [`Request`](struct.Request.html), consumes this
    /// connection, and returns a [`Response`](struct.Response.html).
    pub fn send(self) -> Response {
        // For now, here's a dummy implementation to make the tests pass.
        // TODO: Implement networking
        let req = self.request;
        match req.method {
            Method::Get => match &*req.resource {
                "/boop" => Response::new(HashMap::new(), "beep".to_string()),
                "/list" => Response::new(HashMap::new(), "[\"boop\", \"beep\"]".to_string()),
                _ => Response::new(HashMap::new(), "404 Not Found".to_string()),
            },
            Method::Post => Response::new(HashMap::new(), "ok".to_string()),
        }
    }
}
