use crate::connection::Connection;
#[cfg(feature = "proxy")]
use crate::proxy::Proxy;
use crate::{Error, Response, ResponseLazy};
use std::collections::HashMap;
use std::fmt;

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
///
/// Generally created by the [`minreq::get`](fn.get.html)-style
/// functions, corresponding to the HTTP method we want to use.
///
/// # Example
///
/// ```
/// let request = minreq::post("http://example.com");
/// ```
///
/// After creating the request, you would generally call
/// [`send`](struct.Request.html#method.send) or
/// [`send_lazy`](struct.Request.html#method.send_lazy) on it, as it
/// doesn't do much on its own.
#[derive(Clone, PartialEq, Debug)]
pub struct Request {
    pub(crate) method: Method,
    pub(crate) host: URL,
    resource: URL,
    headers: HashMap<String, String>,
    body: Option<Vec<u8>>,
    pub(crate) timeout: Option<u64>,
    max_redirects: usize,
    https: bool,
    pub(crate) redirects: Vec<(bool, URL, URL)>,
    #[cfg(feature = "proxy")]
    pub(crate) proxy: Option<Proxy>,
}

impl Request {
    /// Creates a new HTTP `Request`.
    ///
    /// This is only the request's data, it is not sent yet. For
    /// sending the request, see [`send`](struct.Request.html#method.send).
    pub fn new<T: Into<URL>>(method: Method, url: T) -> Request {
        let (https, host, resource) = parse_url(url.into());
        Request {
            method,
            host,
            resource,
            headers: HashMap::new(),
            body: None,
            timeout: None,
            max_redirects: 100,
            https,
            redirects: Vec::new(),
            #[cfg(feature = "proxy")]
            proxy: None,
        }
    }

    /// Adds a header to the request this is called on. Use this
    /// function to add headers to your requests.
    pub fn with_header<T: Into<String>, U: Into<String>>(mut self, key: T, value: U) -> Request {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Sets the request body.
    pub fn with_body<T: Into<Vec<u8>>>(mut self, body: T) -> Request {
        let body = body.into();
        let body_length = body.len();
        self.body = Some(body);
        self.with_header("Content-Length", format!("{}", body_length))
    }

    /// Converts given argument to JSON and sets it as body.
    ///
    /// # Errors
    ///
    /// Returns
    /// [`SerdeJsonError`](enum.Error.html#variant.SerdeJsonError) if
    /// Serde runs into a problem when converting `body` into a
    /// string.
    #[cfg(feature = "json-using-serde")]
    pub fn with_json<T: serde::ser::Serialize>(mut self, body: &T) -> Result<Request, Error> {
        self.headers.insert(
            "Content-Type".to_string(),
            "application/json; charset=UTF-8".to_string(),
        );
        match serde_json::to_string(&body) {
            Ok(json) => Ok(self.with_body(json)),
            Err(err) => Err(Error::SerdeJsonError(err)),
        }
    }

    /// Sets the request timeout in seconds.
    pub fn with_timeout(mut self, timeout: u64) -> Request {
        self.timeout = Some(timeout);
        self
    }

    /// Sets the max redirects we follow until giving up. 100 by
    /// default.
    ///
    /// Warning: setting this to a very high number, such as 1000, may
    /// cause a stack overflow if that many redirects are followed. If
    /// you have a use for so many redirects that the stack overflow
    /// becomes a problem, please open an issue.
    pub fn with_max_redirects(mut self, max_redirects: usize) -> Request {
        self.max_redirects = max_redirects;
        self
    }

    /// Sends this request to the host.
    ///
    /// # Errors
    ///
    /// Returns `Err` if we run into an error while sending the
    /// request, or receiving/parsing the response. The specific error
    /// is described in the `Err`, and it can be any
    /// [`minreq::Error`](enum.Error.html) except
    /// [`SerdeJsonError`](enum.Error.html#variant.SerdeJsonError) and
    /// [`InvalidUtf8InBody`](enum.Error.html#variant.InvalidUtf8InBody).
    #[cfg(feature = "https")]
    pub fn send(self) -> Result<Response, Error> {
        if self.https {
            let is_head = self.method == Method::Head;
            let response = Connection::new(self).send_https()?;
            Response::create(response, is_head)
        } else {
            let is_head = self.method == Method::Head;
            let response = Connection::new(self).send()?;
            Response::create(response, is_head)
        }
    }

    /// Sends this request to the host, loaded lazily.
    ///
    /// # Errors
    ///
    /// See [`send`](struct.Request.html#method.send).
    #[cfg(feature = "https")]
    pub fn send_lazy(self) -> Result<ResponseLazy, Error> {
        if self.https {
            Connection::new(self).send_https()
        } else {
            Connection::new(self).send()
        }
    }

    /// Sends this request to the host.
    ///
    /// # Errors
    ///
    /// Returns `Err` if we run into an error while sending the
    /// request, or receiving/parsing the response. The specific error
    /// is described in the `Err`, and it can be any
    /// [`minreq::Error`](enum.Error.html) except
    /// [`SerdeJsonError`](enum.Error.html#variant.SerdeJsonError) and
    /// [`InvalidUtf8InBody`](enum.Error.html#variant.InvalidUtf8InBody).
    #[cfg(not(feature = "https"))]
    pub fn send(self) -> Result<Response, Error> {
        if self.https {
            Err(Error::HttpsFeatureNotEnabled)
        } else {
            let is_head = self.method == Method::Head;
            let response = Connection::new(self).send()?;
            Response::create(response, is_head)
        }
    }

    /// Sends this request to the host, loaded lazily.
    ///
    /// # Errors
    ///
    /// See [`send`](struct.Request.html#method.send).
    #[cfg(not(feature = "https"))]
    pub fn send_lazy(self) -> Result<ResponseLazy, Error> {
        if self.https {
            Err(Error::HttpsFeatureNotEnabled)
        } else {
            Connection::new(self).send()
        }
    }

    fn get_http_head(&self) -> String {
        let mut http = String::with_capacity(32);
        // Add the request line and the "Host" header
        http += &format!(
            "{} {} HTTP/1.1\r\nHost: {}\r\n",
            self.method, self.resource, self.host
        );
        // Add other headers
        for (k, v) in &self.headers {
            http += &format!("{}: {}\r\n", k, v);
        }
        http += "\r\n";
        http
    }

    /// Returns the HTTP request as bytes, ready to be sent to
    /// the server.
    pub(crate) fn as_bytes(&self) -> Vec<u8> {
        let mut head = self.get_http_head().into_bytes();
        if let Some(body) = &self.body {
            head.extend(body);
        }
        head
    }

    /// Returns the redirected version of this Request, unless an infinite redirection loop was detected.
    pub(crate) fn redirect_to(mut self, url: URL) -> Result<Request, Error> {
        self.redirects.push((self.https, self.host, self.resource));

        let (https, host, resource) = parse_url(url);
        self.host = host;
        self.resource = resource;
        self.https = https;

        let is_this_url = |(https_, host_, resource_): &(bool, URL, URL)| {
            *resource_ == self.resource && *host_ == self.host && *https_ == https
        };

        if self.redirects.len() > self.max_redirects {
            Err(Error::TooManyRedirections)
        } else if self.redirects.iter().any(is_this_url) {
            Err(Error::InfiniteRedirectionLoop)
        } else {
            Ok(self)
        }
    }

    /// Sets the proxy to use.
    #[cfg(feature = "proxy")]
    pub fn with_proxy(mut self, proxy: Proxy) -> Request {
        self.proxy = Some(proxy);
        self
    }
}

fn parse_url(url: URL) -> (bool, URL, URL) {
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
    (https, first, second)
}

/// Alias for [Request::new](struct.Request.html#method.new) with `method` set to
/// [Method::Get](enum.Method.html).
pub fn get<T: Into<URL>>(url: T) -> Request {
    Request::new(Method::Get, url)
}

/// Alias for [Request::new](struct.Request.html#method.new) with `method` set to
/// [Method::Head](enum.Method.html).
pub fn head<T: Into<URL>>(url: T) -> Request {
    Request::new(Method::Head, url)
}

/// Alias for [Request::new](struct.Request.html#method.new) with `method` set to
/// [Method::Post](enum.Method.html).
pub fn post<T: Into<URL>>(url: T) -> Request {
    Request::new(Method::Post, url)
}

/// Alias for [Request::new](struct.Request.html#method.new) with `method` set to
/// [Method::Put](enum.Method.html).
pub fn put<T: Into<URL>>(url: T) -> Request {
    Request::new(Method::Put, url)
}

/// Alias for [Request::new](struct.Request.html#method.new) with `method` set to
/// [Method::Delete](enum.Method.html).
pub fn delete<T: Into<URL>>(url: T) -> Request {
    Request::new(Method::Delete, url)
}

/// Alias for [Request::new](struct.Request.html#method.new) with `method` set to
/// [Method::Connect](enum.Method.html).
pub fn connect<T: Into<URL>>(url: T) -> Request {
    Request::new(Method::Connect, url)
}

/// Alias for [Request::new](struct.Request.html#method.new) with `method` set to
/// [Method::Options](enum.Method.html).
pub fn options<T: Into<URL>>(url: T) -> Request {
    Request::new(Method::Options, url)
}

/// Alias for [Request::new](struct.Request.html#method.new) with `method` set to
/// [Method::Trace](enum.Method.html).
pub fn trace<T: Into<URL>>(url: T) -> Request {
    Request::new(Method::Trace, url)
}

/// Alias for [Request::new](struct.Request.html#method.new) with `method` set to
/// [Method::Patch](enum.Method.html).
pub fn patch<T: Into<URL>>(url: T) -> Request {
    Request::new(Method::Patch, url)
}
