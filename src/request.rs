use crate::connection::Connection;
#[cfg(feature = "proxy")]
use crate::proxy::Proxy;
use crate::{Error, Response, ResponseLazy};
use std::collections::HashMap;
use std::fmt;
use std::fmt::Write;

/// A URL type for requests.
pub type URL = String;

/// An HTTP request method.
#[derive(Clone, PartialEq, Eq, Debug)]
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

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Port {
    ImplicitHttp,
    ImplicitHttps,
    Explicit(u32),
}

impl Port {
    pub(crate) fn port(self) -> u32 {
        match self {
            Port::ImplicitHttp => 80,
            Port::ImplicitHttps => 443,
            Port::Explicit(port) => port,
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
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Request {
    pub(crate) method: Method,
    url: URL,
    params: String,
    headers: HashMap<String, String>,
    body: Option<Vec<u8>>,
    pub(crate) timeout: Option<u64>,
    pub(crate) max_headers_size: Option<usize>,
    pub(crate) max_status_line_len: Option<usize>,
    max_redirects: usize,
    #[cfg(feature = "proxy")]
    pub(crate) proxy: Option<Proxy>,
}

impl Request {
    /// Creates a new HTTP `Request`.
    ///
    /// This is only the request's data, it is not sent yet. For
    /// sending the request, see [`send`](struct.Request.html#method.send).
    ///
    /// If `urlencoding` is not enabled, it is the responsibility of the
    /// user to ensure there are no illegal characters in the URL.
    ///
    /// If `urlencoding` is enabled, the resource part of the URL will be
    /// encoded. Any URL special characters (e.g. &, #, =) are not encoded
    /// as they are assumed to be meaningful parameters etc.
    pub fn new<T: Into<URL>>(method: Method, url: T) -> Request {
        Request {
            method,
            url: url.into(),
            params: String::new(),
            headers: HashMap::new(),
            body: None,
            timeout: None,
            max_headers_size: None,
            max_status_line_len: None,
            max_redirects: 100,
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

    /// Adds given key and value as query parameter to request url
    /// (resource).
    ///
    /// If `urlencoding` is not enabled, it is the responsibility
    /// of the user to ensure there are no illegal characters in the
    /// key or value.
    ///
    /// If `urlencoding` is enabled, the key and value are both encoded.
    #[cfg_attr(not(urlencoding), allow(clippy::needless_borrow))]
    pub fn with_param<T: AsRef<str>, U: AsRef<str>>(mut self, key: T, value: U) -> Request {
        let key = key.as_ref();
        #[cfg(feature = "urlencoding")]
        let key = urlencoding::encode(key);
        let value = value.as_ref();
        #[cfg(feature = "urlencoding")]
        let value = urlencoding::encode(value);

        if !self.params.is_empty() {
            self.params.push('&');
        }
        self.params.push_str(&key);
        self.params.push('=');
        self.params.push_str(&value);
        self
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

    /// Sets the maximum size of all the headers this request will
    /// accept.
    ///
    /// If this limit is passed, the request will close the connection
    /// and return an [Error::HeadersOverflow] error.
    ///
    /// The maximum length is counted in bytes, including line-endings
    /// and other whitespace. Both normal and trailing headers count
    /// towards this cap.
    ///
    /// `None` disables the cap, and may cause the program to use any
    /// amount of memory if the server responds with a lot of headers
    /// (or an infinite amount). In minreq versions 2.x.x, the default
    /// is None, so setting this manually is recommended when talking
    /// to untrusted servers.
    pub fn with_max_headers_size<S: Into<Option<usize>>>(mut self, max_headers_size: S) -> Request {
        self.max_headers_size = max_headers_size.into();
        self
    }

    /// Sets the maximum length of the status line this request will
    /// accept.
    ///
    /// If this limit is passed, the request will close the connection
    /// and return an [Error::StatusLineOverflow] error.
    ///
    /// The maximum length is counted in bytes, including the
    /// line-ending `\r\n`.
    ///
    /// `None` disables the cap, and may cause the program to use any
    /// amount of memory if the server responds with a long (or
    /// infinite) status line. In minreq versions 2.x.x, the default
    /// is None, so setting this manually is recommended when talking
    /// to untrusted servers.
    pub fn with_max_status_line_length<S: Into<Option<usize>>>(
        mut self,
        max_status_line_len: S,
    ) -> Request {
        self.max_status_line_len = max_status_line_len.into();
        self
    }

    /// Sets the proxy to use.
    #[cfg(feature = "proxy")]
    pub fn with_proxy(mut self, proxy: Proxy) -> Request {
        self.proxy = Some(proxy);
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
    pub fn send(self) -> Result<Response, Error> {
        let parsed_request = ParsedRequest::new(self)?;
        if parsed_request.https {
            #[cfg(any(feature = "rustls", feature = "openssl", feature = "native-tls"))]
            {
                let is_head = parsed_request.config.method == Method::Head;
                let response = Connection::new(parsed_request).send_https()?;
                Response::create(response, is_head)
            }
            #[cfg(not(any(feature = "rustls", feature = "openssl", feature = "native-tls")))]
            {
                Err(Error::HttpsFeatureNotEnabled)
            }
        } else {
            let is_head = parsed_request.config.method == Method::Head;
            let response = Connection::new(parsed_request).send()?;
            Response::create(response, is_head)
        }
    }

    /// Sends this request to the host, loaded lazily.
    ///
    /// # Errors
    ///
    /// See [`send`](struct.Request.html#method.send).
    pub fn send_lazy(self) -> Result<ResponseLazy, Error> {
        let parsed_request = ParsedRequest::new(self)?;
        if parsed_request.https {
            #[cfg(any(feature = "rustls", feature = "openssl", feature = "native-tls"))]
            {
                Connection::new(parsed_request).send_https()
            }
            #[cfg(not(any(feature = "rustls", feature = "openssl", feature = "native-tls")))]
            {
                Err(Error::HttpsFeatureNotEnabled)
            }
        } else {
            Connection::new(parsed_request).send()
        }
    }
}

pub(crate) struct ParsedRequest {
    pub(crate) host: URL,
    pub(crate) port: Port,
    resource: URL,
    pub(crate) https: bool,
    pub(crate) redirects: Vec<(bool, URL, URL)>,
    pub(crate) config: Request,
}

impl ParsedRequest {
    #[allow(unused_mut)]
    fn new(mut config: Request) -> Result<ParsedRequest, Error> {
        let (https, host, port, mut resource) = parse_url(&config.url)?;

        if !config.params.is_empty() {
            if resource.contains('?') {
                resource.push('&');
            } else {
                resource.push('?');
            }

            resource.push_str(&config.params);
        }

        #[cfg(feature = "proxy")]
        // Set default proxy from environment variables
        //
        // Curl documentation: https://everything.curl.dev/usingcurl/proxies/env
        //
        // Accepted variables are `http_proxy`, `https_proxy`, `HTTPS_PROXY`, `ALL_PROXY`
        //
        // Note: https://everything.curl.dev/usingcurl/proxies/env#http_proxy-in-lower-case-only
        if config.proxy.is_none() {
            // Set HTTP proxies if request's protocol is HTTPS and they're given
            if https {
                if let Ok(proxy) =
                    std::env::var("https_proxy").map_err(|_| std::env::var("HTTPS_PROXY"))
                {
                    if let Ok(proxy) = Proxy::new(proxy) {
                        config.proxy = Some(proxy);
                    }
                }
            }
            // Set HTTP proxies if request's protocol is HTTP and they're given
            else if let Ok(proxy) = std::env::var("http_proxy") {
                if let Ok(proxy) = Proxy::new(proxy) {
                    config.proxy = Some(proxy);
                }
            }
            // Set any given proxies if neither of HTTP/HTTPS were given
            else if let Ok(proxy) =
                std::env::var("all_proxy").map_err(|_| std::env::var("ALL_PROXY"))
            {
                if let Ok(proxy) = Proxy::new(proxy) {
                    config.proxy = Some(proxy);
                }
            }
        }

        Ok(ParsedRequest {
            host,
            port,
            resource,
            https,
            redirects: Vec::new(),
            config,
        })
    }

    fn get_http_head(&self) -> String {
        let mut http = String::with_capacity(32);

        // Add the request line and the "Host" header
        write!(
            http,
            "{} {} HTTP/1.1\r\nHost: {}",
            self.config.method, self.resource, self.host
        )
        .unwrap();
        if let Port::Explicit(port) = self.port {
            write!(http, ":{}", port).unwrap();
        }
        http += "\r\n";

        // Add other headers
        for (k, v) in &self.config.headers {
            write!(http, "{}: {}\r\n", k, v).unwrap();
        }

        if self.config.method == Method::Post
            || self.config.method == Method::Put
            || self.config.method == Method::Patch
        {
            let not_length = |key: &String| {
                let key = key.to_lowercase();
                key != "content-length" && key != "transfer-encoding"
            };
            if self.config.headers.keys().all(not_length) {
                // A user agent SHOULD send a Content-Length in a request message when no Transfer-Encoding
                // is sent and the request method defines a meaning for an enclosed payload body.
                // refer: https://tools.ietf.org/html/rfc7230#section-3.3.2

                // A client MUST NOT send a message body in a TRACE request.
                // refer: https://tools.ietf.org/html/rfc7231#section-4.3.8
                // similar line found for GET, HEAD, CONNECT and DELETE.

                http += "Content-Length: 0\r\n";
            }
        }

        http += "\r\n";
        http
    }

    /// Returns the HTTP request as bytes, ready to be sent to
    /// the server.
    pub(crate) fn as_bytes(&self) -> Vec<u8> {
        let mut head = self.get_http_head().into_bytes();
        if let Some(body) = &self.config.body {
            head.extend(body);
        }
        head
    }

    /// Returns the redirected version of this Request, unless an
    /// infinite redirection loop was detected, or the redirection
    /// limit was reached.
    pub(crate) fn redirect_to(&mut self, url: URL) -> Result<(), Error> {
        // If the redirected resource does not have a fragment, but
        // the original URL did, the fragment should be preserved over
        // redirections. See RFC 7231 section 7.1.2.
        let inherit_fragment = |resource: String, original_resource: &str| {
            if resource.chars().any(|c| c == '#') {
                resource
            } else {
                let mut original_resource_split = original_resource.split('#');
                if let Some(fragment) = original_resource_split.nth(1) {
                    format!("{}#{}", resource, fragment)
                } else {
                    resource
                }
            }
        };

        if url.contains("://") {
            let (mut https, mut host, mut port, resource) = parse_url(&url).map_err(|_| {
                // TODO: Uncomment this for 3.0
                // Error::InvalidProtocolInRedirect
                Error::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "was redirected to an absolute url with an invalid protocol",
                ))
            })?;
            let mut resource = inherit_fragment(resource, &self.resource);
            std::mem::swap(&mut https, &mut self.https);
            std::mem::swap(&mut host, &mut self.host);
            std::mem::swap(&mut port, &mut self.port);
            std::mem::swap(&mut resource, &mut self.resource);
            self.redirects.push((https, host, resource));
        } else {
            // The url does not have the protocol part, assuming it's
            // a relative resource.
            let mut resource = inherit_fragment(url, &self.resource);
            std::mem::swap(&mut resource, &mut self.resource);
            self.redirects
                .push((self.https, self.host.clone(), resource));
        }

        let is_this_url = |(https_, host_, resource_): &(bool, URL, URL)| {
            resource_ == &self.resource && host_ == &self.host && https_ == &self.https
        };

        if self.redirects.len() > self.config.max_redirects {
            Err(Error::TooManyRedirections)
        } else if self.redirects.iter().any(is_this_url) {
            Err(Error::InfiniteRedirectionLoop)
        } else {
            Ok(())
        }
    }
}

fn parse_url(url: impl AsRef<str>) -> Result<(bool, URL, Port, URL), Error> {
    enum UrlParseStatus {
        Host,
        Port,
        Resource,
    }

    let url = url.as_ref();
    let (url, https) = if let Some(after_protocol) = url.strip_prefix("http://") {
        (after_protocol, false)
    } else if let Some(after_protocol) = url.strip_prefix("https://") {
        (after_protocol, true)
    } else {
        // TODO: Uncomment this for 3.0
        // return Err(Error::InvalidProtocol);
        return Err(Error::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            "was redirected to an absolute url with an invalid protocol",
        )));
    };

    let mut host = URL::new();
    let mut port = String::new();
    let mut resource = URL::new();
    let mut status = UrlParseStatus::Host;
    for c in url.chars() {
        match status {
            UrlParseStatus::Host => {
                match c {
                    '/' | '?' => {
                        // Tolerate typos like: www.example.com?some=params
                        status = UrlParseStatus::Resource;
                        resource.push(c);
                    }
                    ':' => status = UrlParseStatus::Port,
                    _ => host.push(c),
                }
            }
            UrlParseStatus::Port => match c {
                '/' | '?' => {
                    status = UrlParseStatus::Resource;
                    resource.push(c);
                }
                _ => port.push(c),
            },
            #[cfg(not(feature = "urlencoding"))]
            UrlParseStatus::Resource => resource.push(c),
            #[cfg(feature = "urlencoding")]
            UrlParseStatus::Resource => match c {
                // All URL-'safe' characters, plus URL 'special
                // characters' like &, #, =, / ,?
                '0'..='9'
                | 'A'..='Z'
                | 'a'..='z'
                | '-'
                | '.'
                | '_'
                | '~'
                | '&'
                | '#'
                | '='
                | '/'
                | '?' => {
                    resource.push(c);
                }
                // There is probably a simpler way to do this, but this
                // method avoids any heap allocations (except extending
                // `resource`)
                _ => {
                    // Any UTF-8 character can fit in 4 bytes
                    let mut utf8_buf = [0u8; 4];
                    // Bytes fill buffer from the front
                    c.encode_utf8(&mut utf8_buf);
                    // Slice disregards the unused portion of the buffer
                    utf8_buf[..c.len_utf8()].iter().for_each(|byte| {
                        // Convert byte to URL escape, e.g. %21 for b'!'
                        let rem = *byte % 16;
                        let right_char = to_hex_digit(rem);
                        let left_char = to_hex_digit((*byte - rem) >> 4);
                        resource.push('%');
                        resource.push(left_char);
                        resource.push(right_char);
                    });
                }
            },
        }
    }
    // Ensure the resource is *something*
    if resource.is_empty() {
        resource += "/";
    }
    // Set appropriate port
    let port = port.parse::<u32>().map(Port::Explicit).unwrap_or_else(|_| {
        if https {
            Port::ImplicitHttps
        } else {
            Port::ImplicitHttp
        }
    });
    Ok((https, host, port, resource))
}

// https://github.com/kornelski/rust_urlencoding/blob/a4df8027ab34a86a63f1be727965cf101556403f/src/enc.rs#L130-L136
// Converts a UTF-8 byte to a single hexadecimal character
#[cfg(feature = "urlencoding")]
fn to_hex_digit(digit: u8) -> char {
    match digit {
        0..=9 => (b'0' + digit) as char,
        10..=255 => (b'A' - 10 + digit) as char,
    }
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

#[cfg(test)]
mod parsing_tests {
    use super::{get, ParsedRequest};

    #[test]
    fn test_multiple_params() {
        let req = get("http://www.example.org/test/res")
            .with_param("foo", "bar")
            .with_param("asd", "qwe");
        let req = ParsedRequest::new(req).unwrap();
        assert_eq!(&req.resource, "/test/res?foo=bar&asd=qwe");
    }

    #[test]
    fn test_domain() {
        let req = get("http://www.example.org/test/res").with_param("foo", "bar");
        let req = ParsedRequest::new(req).unwrap();
        assert_eq!(&req.host, "www.example.org");
    }

    #[test]
    fn test_protocol() {
        let req =
            ParsedRequest::new(get("http://www.example.org/").with_param("foo", "bar")).unwrap();
        assert!(!req.https);
        let req =
            ParsedRequest::new(get("https://www.example.org/").with_param("foo", "bar")).unwrap();
        assert!(req.https);
    }
}

#[cfg(all(test, feature = "urlencoding"))]
mod encoding_tests {
    use super::{get, ParsedRequest};

    #[test]
    fn test_with_param() {
        let req = get("http://www.example.org").with_param("foo", "bar");
        let req = ParsedRequest::new(req).unwrap();
        assert_eq!(&req.resource, "/?foo=bar");

        let req = get("http://www.example.org").with_param("ówò", "what's this? 👀");
        let req = ParsedRequest::new(req).unwrap();
        assert_eq!(
            &req.resource,
            "/?%C3%B3w%C3%B2=what%27s%20this%3F%20%F0%9F%91%80"
        );
    }

    #[test]
    fn test_on_creation() {
        let req = ParsedRequest::new(get("http://www.example.org/?foo=bar#baz")).unwrap();
        assert_eq!(&req.resource, "/?foo=bar#baz");

        let req = ParsedRequest::new(get("http://www.example.org/?ówò=what's this? 👀")).unwrap();
        assert_eq!(
            &req.resource,
            "/?%C3%B3w%C3%B2=what%27s%20this?%20%F0%9F%91%80"
        );
    }
}
