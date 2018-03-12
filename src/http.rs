use std::collections::HashMap;

/// An URL for requests.
pub type URL = String;

/// An HTTP request method.
pub enum Method {
    Get,
    Post,
}

/// An HTTP request.
pub struct Request {
    method: Method,
    host: URL,
    resource: URL,
    headers: HashMap<String, String>,
    body: Option<String>,
}

impl Request {
    /// Creates a new HTTP `Request`.
    ///
    /// This is only the request's data, it is not sent here. For
    /// sending the request, see
    /// [`Connection`](../connection/struct.Connection.html).
    pub fn new(method: Method, host: URL, resource: URL) -> Request {
        Request {
            method,
            host,
            resource,
            headers: HashMap::new(),
            body: None,
        }
    }

    pub fn with_body(mut self, body: String) -> Request {
        self.body = body;
        self
    }
}

/// An HTTP response.
pub struct Response {
    headers: HashMap<String, String>,
    body: String,
}

impl Response {
    /// Creates a new HTTP `Response`.
    ///
    /// This is returned from the server after a
    /// [`Request`](struct.Request.html) has been sent.
    pub fn new(headers: HashMap, body: String) -> Response {
        Response { headers, body }
    }
}
