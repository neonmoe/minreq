use std::collections::HashMap;
use std::fmt;

/// A URL type for requests.
pub type URL = String;

/// An HTTP request method.
pub enum Method {
    Get,
    Post,
}

impl fmt::Display for Method {
    /// Formats the Method to the form in the HTTP request,
    /// ie. Method::Get -> "GET", Method::Post -> "POST", etc.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Method::Get => write!(f, "GET"),
            &Method::Post => write!(f, "POST"),
        }
    }
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
    pub fn new<T: Into<URL>>(method: Method, url: T, body: Option<URL>) -> Request {
        let (host, resource) = parse_url(url.into());
        let mut headers = HashMap::new();
        if let Some(ref body) = body {
            headers.insert("Content-Length".to_string(), format!("{}", body.len()));
        }
        Request {
            method,
            host,
            resource,
            headers,
            body,
        }
    }

    /// Returns the HTTP request as a `String`, ready to be sent to
    /// the server.
    pub(crate) fn into_string(self) -> String {
        let mut http = String::new();
        // Add the request line and the "Host" header
        http += &format!(
            "{} {} HTTP/1.1\r\nHost: {}\r\n",
            self.method, self.resource, self.host
        );
        // Add other headers
        for (k, v) in self.headers {
            http += &format!("{}: {}\r\n", k, v);
        }
        // Add the body
        http += "\r\n";
        if let Some(body) = self.body {
            http += &format!("{}", body);
        }
        http
    }
}

/// An HTTP response.
pub struct Response {
    /// The headers of the response.
    pub headers: HashMap<String, String>,
    /// The body of the response.
    pub body: String,
}

impl Response {
    pub(crate) fn from_string(response_text: String) -> Response {
        let lines = response_text.lines();
        let mut headers = HashMap::new();
        let mut body = String::new();
        let mut writing_headers = true;
        for line in lines {
            if line.is_empty() {
                writing_headers = false;
                continue;
            }
            if writing_headers {
                if let Some(index) = line.find(":") {
                    let key = line[..index].trim().to_string();
                    let value = line[index..].trim().to_string();
                    headers.insert(key, value);
                }
            } else {
                body += &format!("{}\r\n", line);
            }
        }
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
        } else if slashes == 2 {
            first.push(c);
        }
        if slashes == 3 {
            second.push(c);
        }
    }
    (first, second)
}
