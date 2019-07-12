use crate::connection::Connection;
use std::collections::HashMap;
use std::fmt;
use std::io::Error;

/// A URL type for requests.
pub type URL = String;

/// An HTTP request method.
#[derive(Clone, PartialEq, Debug)]
pub enum Method {
    /// The GET method
    Get,
    /// The HEAD method
    Head,
    /// The POST method
    Post,
    /// The PUT method
    Put,
    /// The DELETE method
    Delete,
    /// The CONNECT method
    Connect,
    /// The OPTIONS method
    Options,
    /// The TRACE method
    Trace,
    /// The PATCH method
    Patch,
    /// A custom method, use with care: the string will be embedded in
    /// your request as-is.
    Custom(String),
}

impl fmt::Display for Method {
    /// Formats the Method to the form in the HTTP request,
    /// ie. Method::Get -> "GET", Method::Post -> "POST", etc.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Method::Get => write!(f, "GET"),
            Method::Head => write!(f, "HEAD"),
            Method::Post => write!(f, "POST"),
            Method::Put => write!(f, "PUT"),
            Method::Delete => write!(f, "DELETE"),
            Method::Connect => write!(f, "CONNECT"),
            Method::Options => write!(f, "OPTIONS"),
            Method::Trace => write!(f, "TRACE"),
            Method::Patch => write!(f, "PATCH"),
            Method::Custom(ref s) => write!(f, "{}", s),
        }
    }
}

/// An HTTP request.
pub struct Request {
    pub(crate) method: Method,
    pub(crate) host: URL,
    resource: URL,
    headers: HashMap<String, String>,
    body: Option<String>,
    pub(crate) timeout: Option<u64>,
    https: bool,
    pub(crate) redirects: Vec<URL>,
}

impl Request {
    /// Creates a new HTTP `Request`.
    ///
    /// This is only the request's data, it is not sent yet. For
    /// sending the request, see [`send`](struct.Request.html#method.send).
    pub fn new<T: Into<URL>>(method: Method, url: T) -> Request {
        let (host, resource, https) = parse_url(url.into());
        Request {
            method,
            host,
            resource,
            headers: HashMap::new(),
            body: None,
            timeout: None,
            https,
            redirects: Vec::new(),
        }
    }

    /// Adds a header to the request this is called on. Use this
    /// function to add headers to your requests.
    pub fn with_header<T: Into<String>, U: Into<String>>(mut self, key: T, value: U) -> Request {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Sets the request body.
    pub fn with_body<T: Into<String>>(mut self, body: T) -> Request {
        let body = body.into();
        let body_length = body.len();
        self.body = Some(body);
        self.with_header("Content-Length", format!("{}", body_length))
    }

    /// Converts given argument to JSON and sets it as body.
    #[cfg(feature = "json-using-serde")]
    pub fn with_json<T: serde::ser::Serialize>(mut self, body: &T) -> Result<Request, serde_json::Error> {
        self.headers.insert("Content-Type".to_string(), "application/json; charset=UTF-8".to_string());
        Ok(self.with_body(serde_json::to_string(&body)?))
    }

    /// Sets the request timeout.
    pub fn with_timeout(mut self, timeout: u64) -> Request {
        self.timeout = Some(timeout);
        self
    }

    /// Sends this request to the host.
    #[cfg(feature = "https")]
    pub fn send(self) -> Result<Response, Error> {
        if self.https {
            Connection::new(self).send_https()
        } else {
            Connection::new(self).send()
        }
    }

    /// Sends this request to the host.
    #[cfg(not(feature = "https"))]
    pub fn send(self) -> Result<Response, Error> {
        if self.https {
            panic!("Can't send requests to urls that start with https:// when the `https` feature is not enabled!")
        } else {
            Connection::new(self).send()
        }
    }

    /// Returns the HTTP request as a `String`, ready to be sent to
    /// the server.
    pub(crate) fn to_string(&self) -> String {
        let mut http = String::new();
        // Add the request line and the "Host" header
        http += &format!(
            "{} {} HTTP/1.1\r\nHost: {}\r\n",
            self.method, self.resource, self.host
        );
        // Add other headers
        for (k, v) in &self.headers {
            http += &format!("{}: {}\r\n", k, v);
        }
        // Add the body
        http += "\r\n";
        if let Some(ref body) = &self.body {
            http += body;
        }
        http
    }

    pub(crate) fn redirect_to(&mut self, url: URL) {
        self.redirects
            .push(create_url(&self.host, &self.resource, self.https));

        let (host, resource, https) = parse_url(url);
        self.host = host;
        self.resource = resource;
        self.https = https;
    }
}

/// An HTTP response.
pub struct Response {
    /// The status code of the response, eg. 404.
    pub status_code: i32,
    /// The reason phrase of the response, eg. "Not Found".
    pub reason_phrase: String,
    /// The headers of the response.
    pub headers: HashMap<String, String>,
    /// The body of the response.
    pub body: String,
    /// The body of the response, as raw bytes.
    pub body_bytes: Vec<u8>,
}

impl Response {
    pub(crate) fn from_bytes(bytes: Vec<u8>) -> Response {
        let (status_code, reason_phrase) = parse_status_line(&bytes);
        let (headers, body_bytes) = parse_http_response_content(&bytes);
        Response {
            status_code,
            reason_phrase,
            headers,
            // FIXME: Make body an Option?
            body: std::str::from_utf8(&body_bytes).unwrap_or("").to_owned(),
            body_bytes,
        }
    }

    /// Converts JSON body to a `struct` using Serde.
    ///
    /// In case compiler cannot figure out return type you might need to declare it explicitly:
    ///
    /// ```no_run
    /// use serde_derive::Deserialize;
    ///
    /// #[derive(Deserialize)]
    /// struct User {
    ///     name: String,
    ///     email: String,
    /// }
    ///
    /// # fn main() {
    /// # let url_to_json_resource = "http://example.org/resource.json";
    /// let user_name = minreq::get(url_to_json_resource)
    ///     .send().unwrap()
    ///     .json::<User>().unwrap() // explicitly declared type `User`
    ///     .name;
    /// println!("User name is '{}'", &user_name);
    /// # }
    /// ```
    #[cfg(feature = "json-using-serde")]
    pub fn json<'a, T>(&'a self) -> Result<T, serde_json::Error>
    where T: serde::de::Deserialize<'a> {
        serde_json::from_str(&self.body)
    }
}

fn create_url(host: &str, resource: &str, https: bool) -> URL {
    let prefix = if https { "https://" } else { "http://" };
    return format!("{}{}{}", prefix, host, resource);
}

fn parse_url(url: URL) -> (URL, URL, bool) {
    let mut first = URL::new();
    let mut second = URL::new();
    let mut slashes = 0;
    for c in url.chars() {
        if c == '/' {
            slashes += 1;
        } else if slashes == 2 {
            first.push(c);
        }
        if slashes >= 3 {
            second.push(c);
        }
    }
    // Ensure the resource is *something*
    if second.is_empty() {
        second += "/";
    }
    // Set appropriate port
    let https = url.starts_with("https://");
    if !first.contains(':') {
        first += if https { ":443" } else { ":80" };
    }
    (first, second, https)
}

pub(crate) fn parse_status_line(http_response: &[u8]) -> (i32, String) {
    let (line, _) = split_at(http_response, "\r\n");
    if let Ok(line) = std::str::from_utf8(line) {
        let mut split = line.split(' ');
        if let Some(code) = split.nth(1) {
            if let Ok(code) = code.parse::<i32>() {
                if let Some(reason) = split.next() {
                    return (code, reason.to_string());
                }
            }
        }
    }
    (503, "Server did not provide a status line".to_string())
}

fn parse_http_response_content(http_response: &[u8]) -> (HashMap<String, String>, Vec<u8>) {
    let (headers_text, body) = split_at(http_response, "\r\n\r\n");

    let mut headers = HashMap::new();
    let mut status_line = true;
    if let Ok(headers_text) = std::str::from_utf8(headers_text) {
        for line in headers_text.lines() {
            if status_line {
                status_line = false;
                continue;
            } else if let Some((key, value)) = parse_header(line) {
                headers.insert(key, value);
            }
        }
    }

    (headers, body.to_vec())
}

fn split_at<'a>(bytes: &'a [u8], splitter: &str) -> (&'a [u8], &'a [u8]) {
    for i in 0..bytes.len() - splitter.len() {
        if let Ok(s) = std::str::from_utf8(&bytes[i..i + splitter.len()]) {
            if s == splitter {
                return (&bytes[..i], &bytes[i + splitter.len()..]);
            }
        }
    }
    (bytes, &[])
}

pub(crate) fn parse_header(line: &str) -> Option<(String, String)> {
    if let Some(index) = line.find(':') {
        let key = line[..index].trim().to_string();
        let value = line[(index + 1)..].trim().to_string();
        Some((key, value))
    } else {
        None
    }
}
