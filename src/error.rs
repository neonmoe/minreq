use std::error;
use std::fmt;
use std::io;

/// Represents an error while receiving or parsing an HTTP response.
#[derive(Debug)]
pub enum MinreqError {
    /// Ran into an IO problem while loading the response.
    IOError(io::Error),
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

    /// This is a special error case, one that should never be
    /// returned! Think of this as a cleaner alternative to calling
    /// `unreachable!()` inside the library. If you come across this,
    /// please open an issue, and include the string inside this
    /// error, as it can be used to locate the problem.
    Other(&'static str),
}

impl fmt::Display for MinreqError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use MinreqError::*;
        match self {
            IOError(err) => write!(f, "{}", err),
            MalformedChunkLength => write!(f, "non-usize chunk length with transfer-encoding: chunked"),
            MalformedContentLength => write!(f, "non-usize content length"),
            RedirectLocationMissing => write!(f, "redirection location header missing"),
            InfiniteRedirectionLoop => write!(f, "infinite redirection loop detected"),
            InvalidUtf8InResponse => write!(f, "response contained invalid utf-8 where valid utf-8 was expected"),
            Other(msg) => write!(f, "error in minreq: please open an issue in the minreq repo, include the following: '{}'", msg),
        }
    }
}

impl error::Error for MinreqError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        use MinreqError::*;
        match self {
            IOError(err) => Some(err),
            _ => None,
        }
    }
}
