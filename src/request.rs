use crate::connection::Connection;
use crate::http_url::{HttpUrl, Port};
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
    pub fn with_param<T: Into<String>, U: Into<String>>(mut self, key: T, value: U) -> Request {
        let key = key.into();
        #[cfg(feature = "urlencoding")]
        let key = urlencoding::encode(&key);
        let value = value.into();
        #[cfg(feature = "urlencoding")]
        let value = urlencoding::encode(&value);

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

    /// Sends this request to the host and collect the *whole* response
    ///
    /// **WARNING:** This does what it says on the tin — so long as the
    /// server keeps sending bytes, they will be appended, in-memory,
    /// to the repsonse. Consider reading from a [`ResponseLazy`] instead.
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
        if parsed_request.url.https {
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
        if parsed_request.url.https {
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
    pub(crate) url: HttpUrl,
    pub(crate) redirects: Vec<HttpUrl>,
    pub(crate) config: Request,
}

impl ParsedRequest {
    #[allow(unused_mut)]
    fn new(mut config: Request) -> Result<ParsedRequest, Error> {
        let mut url = HttpUrl::parse(&config.url, None)?;

        if !config.params.is_empty() {
            if url.path_and_query.contains('?') {
                url.path_and_query.push('&');
            } else {
                url.path_and_query.push('?');
            }
            url.path_and_query.push_str(&config.params);
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
            if url.https {
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
            url,
            redirects: Vec::new(),
            config,
        })
    }

    fn get_http_head(&self) -> String {
        let mut http = String::with_capacity(32);

        // NOTE: As of 2.10.0, the fragment is intentionally left out of the request, based on:
        // - [RFC 3986 section 3.5](https://datatracker.ietf.org/doc/html/rfc3986#section-3.5):
        //   "...the fragment identifier is not used in the scheme-specific
        //   processing of a URI; instead, the fragment identifier is separated
        //   from the rest of the URI prior to a dereference..."
        // - [RFC 7231 section 9.5](https://datatracker.ietf.org/doc/html/rfc7231#section-9.5):
        //   "Although fragment identifiers used within URI references are not
        //   sent in requests..."

        // Add the request line and the "Host" header
        write!(
            http,
            "{} {} HTTP/1.1\r\nHost: {}",
            self.config.method, self.url.path_and_query, self.url.host
        )
        .unwrap();
        if let Port::Explicit(port) = self.url.port {
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
    pub(crate) fn redirect_to(&mut self, url: &str) -> Result<(), Error> {
        if url.contains("://") {
            let mut url = HttpUrl::parse(url, Some(&self.url)).map_err(|_| {
                // TODO: Uncomment this for 3.0
                // Error::InvalidProtocolInRedirect
                Error::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "was redirected to an absolute url with an invalid protocol",
                ))
            })?;
            std::mem::swap(&mut url, &mut self.url);
            self.redirects.push(url);
        } else {
            // The url does not have the protocol part, assuming it's
            // a relative resource.
            let mut absolute_url = String::new();
            self.url.write_base_url_to(&mut absolute_url).unwrap();
            absolute_url.push_str(url);
            let mut url = HttpUrl::parse(&absolute_url, Some(&self.url))?;
            std::mem::swap(&mut url, &mut self.url);
            self.redirects.push(url);
        }

        if self.redirects.len() > self.config.max_redirects {
            Err(Error::TooManyRedirections)
        } else if self
            .redirects
            .iter()
            .any(|redirect_url| redirect_url == &self.url)
        {
            Err(Error::InfiniteRedirectionLoop)
        } else {
            Ok(())
        }
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
        assert_eq!(&req.url.path_and_query, "/test/res?foo=bar&asd=qwe");
    }

    #[test]
    fn test_domain() {
        let req = get("http://www.example.org/test/res").with_param("foo", "bar");
        let req = ParsedRequest::new(req).unwrap();
        assert_eq!(&req.url.host, "www.example.org");
    }

    #[test]
    fn test_protocol() {
        let req =
            ParsedRequest::new(get("http://www.example.org/").with_param("foo", "bar")).unwrap();
        assert!(!req.url.https);
        let req =
            ParsedRequest::new(get("https://www.example.org/").with_param("foo", "bar")).unwrap();
        assert!(req.url.https);
    }
}

#[cfg(all(test, feature = "urlencoding"))]
mod encoding_tests {
    use super::{get, ParsedRequest};

    #[test]
    fn test_with_param() {
        let req = get("http://www.example.org").with_param("foo", "bar");
        let req = ParsedRequest::new(req).unwrap();
        assert_eq!(&req.url.path_and_query, "/?foo=bar");

        let req = get("http://www.example.org").with_param("ówò", "what's this? 👀");
        let req = ParsedRequest::new(req).unwrap();
        assert_eq!(
            &req.url.path_and_query,
            "/?%C3%B3w%C3%B2=what%27s%20this%3F%20%F0%9F%91%80"
        );
    }

    #[test]
    fn test_on_creation() {
        let req = ParsedRequest::new(get("http://www.example.org/?foo=bar#baz")).unwrap();
        assert_eq!(&req.url.path_and_query, "/?foo=bar");

        let req = ParsedRequest::new(get("http://www.example.org/?ówò=what's this? 👀")).unwrap();
        assert_eq!(
            &req.url.path_and_query,
            "/?%C3%B3w%C3%B2=what%27s%20this?%20%F0%9F%91%80"
        );
    }
}
