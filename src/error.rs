use std::error;
use std::fmt;
use std::io;

/// Represents an error while sending, receiving, or parsing an HTTP response.
#[derive(Debug)]
pub enum Error {
    #[cfg(feature = "json-using-serde")]
    /// Ran into a serde error.
    SerdeJsonError(serde_json::Error),

    /// Ran into an IO problem while loading the response.
    IoError(io::Error),
    /// Couldn't parse the incoming chunk's length while receiving a
    /// response with the header `Transfer-Encoding: chunked`.
    MalformedChunkLength,
    /// Couldn't parse the `Content-Length` header's value as an
    /// `usize`.
    MalformedContentLength,
    /// The response was a redirection, but the `Location` header is
    /// missing.
    RedirectLocationMissing,
    /// The response redirections caused an infinite redirection loop.
    InfiniteRedirectionLoop,
    /// The response contained invalid UTF-8 where it should be valid
    /// (eg. headers).
    InvalidUtf8InResponse,
    /// The response body contains invalid UTF-8, so the `as_str()`
    /// conversion failed.
    // FIXME: Add the inner Utf8Error here?
    InvalidUtf8InBody,
    /// Tried to send a secure request (ie. the url started with
    /// `https://`), but the crate's `https` feature was not enabled,
    /// and as such, a connection cannot be made.
    HttpsFeatureNotEnabled,
    /// The provided url contained a domain that has non-ASCII
    /// characters, but it could not be converted into punycode
    /// because the `punycode` feature was not enabled.
    PunycodeFeatureNotEnabled,
    /// The provided url contained a domain that has non-ASCII
    /// characters, and could not be converted into punycode. It is
    /// probably not an actual domain.
    PunycodeConversionFailed,

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
            IoError(err) => write!(f, "{}", err),
            MalformedChunkLength => write!(f, "non-usize chunk length with transfer-encoding: chunked"),
            MalformedContentLength => write!(f, "non-usize content length"),
            RedirectLocationMissing => write!(f, "redirection location header missing"),
            InfiniteRedirectionLoop => write!(f, "infinite redirection loop detected"),
            InvalidUtf8InResponse => write!(f, "response contained invalid utf-8 where valid utf-8 was expected"),
            InvalidUtf8InBody => write!(f, "response body contains invalid utf-8, so it can't be converted into a string"),
            HttpsFeatureNotEnabled => write!(f, "request url contains https:// but the https feature is not enabled"),
            PunycodeFeatureNotEnabled => write!(f, "non-ascii urls needs to be converted into punycode, and the feature is missing"),
            PunycodeConversionFailed => write!(f, "non-ascii url conversion to punycode failed"),
            Other(msg) => write!(f, "error in minreq: please open an issue in the minreq repo, include the following: '{}'", msg),

            #[cfg(feature = "json-using-serde")]
            SerdeJsonError(err) => write!(f, "{}", err),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        use Error::*;
        match self {
            IoError(err) => Some(err),
            #[cfg(feature = "json-using-serde")]
            SerdeJsonError(err) => Some(err),
            _ => None,
        }
    }
}
