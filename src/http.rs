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
    /// The HTTP request method.
    pub method: Method,
    /// The HTTP request's "Host" field.
    pub host: URL,
    /// The requested resource.
    pub resource: URL,
    /// The additional headers.
    pub headers: HashMap<String, String>,
    /// The optional body of the request.
    pub body: Option<String>,
}

impl Request {
    /// Creates a new HTTP `Request`.
    ///
    /// This is only the request's data, it is not sent here. For
    /// sending the request, see [`get`](fn.get.html).
    pub fn new(method: Method, url: URL) -> Request {
        let (host, resource) = parse_url(url);
        Request {
            method,
            host,
            resource,
            headers: HashMap::new(),
            body: None,
        }
    }

    pub fn with_body(mut self, body: String) -> Request {
        self.body = Some(body);
        self
    }
}

/// An HTTP response.
pub struct Response {
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl Response {
    /// Creates a new HTTP `Response`.
    ///
    /// This is returned from the server after a
    /// [`Request`](struct.Request.html) has been sent.
    pub fn new(headers: HashMap<String, String>, body: String) -> Response {
        Response { headers, body }
    }
}

fn parse_url(url: URL) -> (URL, URL) {
    let mut first = URL::new();
    let mut second = URL::new();
    let mut slashes = 0;
    for c in url.chars() {
        if c == '/' {
            slashes += 1;
        }
        if slashes < 3 {
            first.push(c);
        } else {
            second.push(c);
        }
    }
    (first, second)
}
