use std::{error, fmt, io, str};

/// Represents an error while sending, receiving, or parsing an HTTP response.
#[derive(Debug)]
// TODO: Make non-exhaustive for 3.0?
pub enum Error {
    #[cfg(feature = "json-using-serde")]
    /// Ran into a Serde error.
    SerdeJsonError(serde_json::Error),
    /// The response body contains invalid UTF-8, so the `as_str()`
    /// conversion failed.
    InvalidUtf8InBody(str::Utf8Error),

    #[cfg(feature = "rustls")]
    /// Ran into a rustls error while creating the connection.
    RustlsCreateConnection(rustls::Error),
    /// Ran into an IO problem while loading the response.
    IoError(io::Error),
    /// Couldn't parse the incoming chunk's length while receiving a
    /// response with the header `Transfer-Encoding: chunked`.
    MalformedChunkLength,
    /// The chunk did not end after reading the previously read amount
    /// of bytes.
    MalformedChunkEnd,
    /// Couldn't parse the `Content-Length` header's value as an
    /// `usize`.
    MalformedContentLength,
    /// The response contains headers whose total size surpasses
    /// [Request::with_max_headers_size](crate::request::Request::with_max_headers_size).
    HeadersOverflow,
    /// The response's status line length surpasses
    /// [Request::with_max_status_line_size](crate::request::Request::with_max_status_line_length).
    StatusLineOverflow,
    /// [ToSocketAddrs](std::net::ToSocketAddrs) did not resolve to an
    /// address.
    AddressNotFound,
    /// The response was a redirection, but the `Location` header is
    /// missing.
    RedirectLocationMissing,
    /// The response redirections caused an infinite redirection loop.
    InfiniteRedirectionLoop,
    /// Followed
    /// [`max_redirections`](struct.Request.html#method.with_max_redirections)
    /// redirections, won't follow any more.
    TooManyRedirections,
    /// The response contained invalid UTF-8 where it should be valid
    /// (eg. headers), so the response cannot interpreted correctly.
    InvalidUtf8InResponse,
    /// The provided url contained a domain that has non-ASCII
    /// characters, and could not be converted into punycode. It is
    /// probably not an actual domain.
    PunycodeConversionFailed,
    /// Tried to send a secure request (ie. the url started with
    /// `https://`), but the crate's `https` feature was not enabled,
    /// and as such, a connection cannot be made.
    HttpsFeatureNotEnabled,
    /// The provided url contained a domain that has non-ASCII
    /// characters, but it could not be converted into punycode
    /// because the `punycode` feature was not enabled.
    PunycodeFeatureNotEnabled,
    /// The provided proxy information was not properly formatted. See
    /// [Proxy::new](crate::Proxy::new) for the valid format.
    BadProxy,
    /// The provided credentials were rejected by the proxy server.
    BadProxyCreds,
    /// The provided proxy credentials were malformed.
    ProxyConnect,
    /// The provided credentials were rejected by the proxy server.
    InvalidProxyCreds,
    // TODO: Uncomment these two for 3.0
    // /// The URL does not start with http:// or https://.
    // InvalidProtocol,
    // /// The URL ended up redirecting to an URL that does not start
    // /// with http:// or https://.
    // InvalidProtocolInRedirect,
    /// This is a special error case, one that should never be
    /// returned! Think of this as a cleaner alternative to calling
    /// `unreachable!()` inside the library. If you come across this,
    /// please open an issue, and include the string inside this
    /// error, as it can be used to locate the problem.
    Other(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            #[cfg(feature = "json-using-serde")]
            SerdeJsonError(err) => write!(f, "{}", err),
            IoError(err) => write!(f, "{}", err),
            InvalidUtf8InBody(err) => write!(f, "{}", err),

            #[cfg(feature = "rustls")]
            RustlsCreateConnection(err) => write!(f, "error creating rustls connection: {}", err),
            MalformedChunkLength => write!(f, "non-usize chunk length with transfer-encoding: chunked"),
            MalformedChunkEnd => write!(f, "chunk did not end after reading the expected amount of bytes"),
            MalformedContentLength => write!(f, "non-usize content length"),
            HeadersOverflow => write!(f, "the headers' total size surpassed max_headers_size"),
            StatusLineOverflow => write!(f, "the status line length surpassed max_status_line_length"),
            AddressNotFound => write!(f, "could not resolve host to a socket address"),
            RedirectLocationMissing => write!(f, "redirection location header missing"),
            InfiniteRedirectionLoop => write!(f, "infinite redirection loop detected"),
            TooManyRedirections => write!(f, "too many redirections (over the max)"),
            InvalidUtf8InResponse => write!(f, "response contained invalid utf-8 where valid utf-8 was expected"),
            HttpsFeatureNotEnabled => write!(f, "request url contains https:// but the https feature is not enabled"),
            PunycodeFeatureNotEnabled => write!(f, "non-ascii urls needs to be converted into punycode, and the feature is missing"),
            PunycodeConversionFailed => write!(f, "non-ascii url conversion to punycode failed"),
            BadProxy => write!(f, "the provided proxy information is malformed"),
            BadProxyCreds => write!(f, "the provided proxy credentials are malformed"),
            ProxyConnect => write!(f, "could not connect to the proxy server"),
            InvalidProxyCreds => write!(f, "the provided proxy credentials are invalid"),
            // TODO: Uncomment these two for 3.0
            // InvalidProtocol => write!(f, "the url does not start with http:// or https://"),
            // InvalidProtocolInRedirect => write!(f, "got redirected to an absolute url which does not start with http:// or https://"),
            Other(msg) => write!(f, "error in minreq: please open an issue in the minreq repo, include the following: '{}'", msg),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        use Error::*;
        match self {
            #[cfg(feature = "json-using-serde")]
            SerdeJsonError(err) => Some(err),
            IoError(err) => Some(err),
            InvalidUtf8InBody(err) => Some(err),
            #[cfg(feature = "rustls")]
            RustlsCreateConnection(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(other: io::Error) -> Error {
        Error::IoError(other)
    }
}
