use crate::{connection::HttpStream, Error};
use std::collections::HashMap;
use std::io::{self, BufReader, Read};
use std::str;

/// An HTTP response.
///
/// Returned by [`Request::send`](struct.Request.html#method.send).
///
/// # Example
///
/// ```no_run
/// # fn main() -> Result<(), minreq::Error> {
/// let response = minreq::get("http://example.com").send()?;
/// println!("{}", response.as_str()?);
/// # Ok(()) }
/// ```
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Response {
    /// The status code of the response, eg. 404.
    pub status_code: i32,
    /// The reason phrase of the response, eg. "Not Found".
    pub reason_phrase: String,
    /// The headers of the response. The header field names (the
    /// keys) are all lowercase.
    pub headers: HashMap<String, String>,
    /// The URL of the resource returned in this response. May differ from the
    /// request URL if it was redirected or typo corrections were applied (e.g.
    /// <http://example.com?foo=bar> would be corrected to
    /// <http://example.com/?foo=bar>).
    pub url: String,

    body: Vec<u8>,
}

impl Response {
    pub(crate) fn create(mut parent: ResponseLazy, is_head: bool) -> Result<Response, Error> {
        let mut body = Vec::new();
        if !is_head && parent.status_code != 204 && parent.status_code != 304 {
            parent.read_to_end(&mut body)?;
        }

        let ResponseLazy {
            status_code,
            reason_phrase,
            headers,
            url,
            ..
        } = parent;

        Ok(Response {
            status_code,
            reason_phrase,
            headers,
            url,
            body,
        })
    }

    /// Returns the body as an `&str`.
    ///
    /// # Errors
    ///
    /// Returns
    /// [`InvalidUtf8InBody`](enum.Error.html#variant.InvalidUtf8InBody)
    /// if the body is not UTF-8, with a description as to why the
    /// provided slice is not UTF-8.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let url = "http://example.org/";
    /// let response = minreq::get(url).send()?;
    /// println!("{}", response.as_str()?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn as_str(&self) -> Result<&str, Error> {
        match str::from_utf8(&self.body) {
            Ok(s) => Ok(s),
            Err(err) => Err(Error::InvalidUtf8InBody(err)),
        }
    }

    /// Returns a reference to the contained bytes of the body. If you
    /// want the `Vec<u8>` itself, use
    /// [`into_bytes()`](#method.into_bytes) instead.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let url = "http://example.org/";
    /// let response = minreq::get(url).send()?;
    /// println!("{:?}", response.as_bytes());
    /// # Ok(())
    /// # }
    /// ```
    pub fn as_bytes(&self) -> &[u8] {
        &self.body
    }

    /// Turns the `Response` into the inner `Vec<u8>`, the bytes that
    /// make up the response's body. If you just need a `&[u8]`, use
    /// [`as_bytes()`](#method.as_bytes) instead.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let url = "http://example.org/";
    /// let response = minreq::get(url).send()?;
    /// println!("{:?}", response.into_bytes());
    /// // This would error, as into_bytes consumes the Response:
    /// // let x = response.status_code;
    /// # Ok(())
    /// # }
    /// ```
    pub fn into_bytes(self) -> Vec<u8> {
        self.body
    }

    /// Converts JSON body to a `struct` using Serde.
    ///
    /// # Errors
    ///
    /// Returns
    /// [`SerdeJsonError`](enum.Error.html#variant.SerdeJsonError) if
    /// Serde runs into a problem, or
    /// [`InvalidUtf8InBody`](enum.Error.html#variant.InvalidUtf8InBody)
    /// if the body is not UTF-8.
    ///
    /// # Example
    /// In case compiler cannot figure out return type you might need to declare it explicitly:
    ///
    /// ```no_run
    /// use serde_json::Value;
    ///
    /// # fn main() -> Result<(), minreq::Error> {
    /// # let url_to_json_resource = "http://example.org/resource.json";
    /// // Value could be any type that implements Deserialize!
    /// let user = minreq::get(url_to_json_resource).send()?.json::<Value>()?;
    /// println!("User name is '{}'", user["name"]);
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "json-using-serde")]
    pub fn json<'a, T>(&'a self) -> Result<T, Error>
    where
        T: serde::de::Deserialize<'a>,
    {
        let str = match self.as_str() {
            Ok(str) => str,
            Err(_) => return Err(Error::InvalidUtf8InResponse),
        };
        match serde_json::from_str(str) {
            Ok(json) => Ok(json),
            Err(err) => Err(Error::SerdeJsonError(err)),
        }
    }
}

/// An HTTP response, which streams bytes as they arrive on the
/// underlying TCP stream via [`std::io::Read`]
///
/// In comparison to [`Response`](struct.Response.html), this is
/// returned from
/// [`send_lazy()`](struct.Request.html#method.send_lazy), where as
/// [`Response`](struct.Response.html) is returned from
/// [`send()`](struct.Request.html#method.send).
/// # Example
/// ```no_run
/// // This is how the normal Response works behind the scenes, and
/// // how you might use ResponseLazy.
/// # fn main() -> Result<(), minreq::Error> {
/// use std::io::Read;
/// let mut response = minreq::get("http://example.com").send_lazy()?;
/// let mut vec = Vec::new();
/// response.read_to_end(&mut vec)?;
/// # Ok(())
/// # }
///
/// ```
pub struct ResponseLazy {
    /// The status code of the response, eg. 404.
    pub status_code: i32,
    /// The reason phrase of the response, eg. "Not Found".
    pub reason_phrase: String,
    /// The headers of the response. The header field names (the
    /// keys) are all lowercase.
    pub headers: HashMap<String, String>,
    /// The URL of the resource returned in this response. May differ from the
    /// request URL if it was redirected or typo corrections were applied (e.g.
    /// <http://example.com?foo=bar> would be corrected to
    /// <http://example.com/?foo=bar>).
    pub url: String,

    stream: BufReader<HttpStream>,
    state: HttpStreamState,
    max_trailing_headers_size: Option<usize>,
}

impl ResponseLazy {
    pub(crate) fn from_stream(
        stream: HttpStream,
        max_headers_size: Option<usize>,
        max_status_line_len: Option<usize>,
    ) -> Result<ResponseLazy, Error> {
        let mut stream = BufReader::new(stream);
        let ResponseMetadata {
            status_code,
            reason_phrase,
            headers,
            state,
            max_trailing_headers_size,
        } = read_metadata(&mut stream, max_headers_size, max_status_line_len)?;

        Ok(ResponseLazy {
            status_code,
            reason_phrase,
            headers,
            url: String::new(),
            stream,
            state,
            max_trailing_headers_size,
        })
    }
}

impl Read for ResponseLazy {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        use HttpStreamState::*;
        match &mut self.state {
            // If we're reading until the TCP stream closes,
            // just... read it!
            EndOnClose => self.stream.read(buf),
            // If we have a content length, read up to the remaining number
            // of bytes, or the buffer size, whichever is smaller.
            ContentLength { to_go } => {
                if *to_go == 0 {
                    return Ok(0);
                }

                let to_read = buf.len().min(*to_go);
                let n = self.stream.read(&mut buf[..to_read])?;
                *to_go -= n;
                Ok(n)
            }
            Chunked { more_chunks, to_go } => read_chunked(
                buf,
                &mut self.stream,
                &mut self.headers,
                self.max_trailing_headers_size,
                more_chunks,
                to_go,
            ),
        }
    }
}

fn read_trailers(
    stream: &mut BufReader<HttpStream>,
    headers: &mut HashMap<String, String>,
    mut max_headers_size: Option<usize>,
) -> Result<(), Error> {
    loop {
        let trailer_line = read_line(stream, max_headers_size, Error::HeadersOverflow)?;
        if let Some(ref mut max_headers_size) = max_headers_size {
            *max_headers_size -= trailer_line.len() + 2;
        }
        if let Some((header, value)) = parse_header(trailer_line) {
            headers.insert(header, value);
        } else {
            break;
        }
    }
    Ok(())
}

fn read_chunked(
    buf: &mut [u8],
    stream: &mut BufReader<HttpStream>,
    headers: &mut HashMap<String, String>,
    max_trailing_headers_size: Option<usize>,
    more_chunks: &mut bool,
    to_go: &mut usize, // In the current chunk
) -> std::io::Result<usize> {
    if !*more_chunks && *to_go == 0 {
        return Ok(0);
    }

    // Save some typing:
    fn bail<E: Into<Box<dyn std::error::Error + Send + Sync>>>(e: E) -> std::io::Result<usize> {
        Err(io::Error::new(io::ErrorKind::Other, e))
    }

    // If we have no bytes left to read in the current chunk,
    // but we're still expecting more chunks,
    // read the length of the next one.
    if *to_go == 0 {
        // Max length of the chunk length line is 1KB: not too long to
        // take up much memory, long enough to tolerate some chunk
        // extensions (which are ignored).

        // Get the size of the next chunk
        let length_line = match read_line(stream, Some(1024), Error::MalformedChunkLength) {
            Ok(line) => line,
            Err(e) => return bail(e),
        };

        // Note: the trim() and check for empty lines shouldn't be
        // needed according to the RFC, but we might as well, it's a
        // small change and it fixes a few servers.
        let incoming_length = if length_line.is_empty() {
            0
        } else {
            let length = if let Some(i) = length_line.find(';') {
                length_line[..i].trim()
            } else {
                length_line.trim()
            };
            match usize::from_str_radix(length, 16) {
                Ok(length) => length,
                Err(e) => return bail(e),
            }
        };

        // If the incoming length is 0, we're done. There's no more chunks.
        // Read the trailers and get out.
        if incoming_length == 0 {
            *more_chunks = false;

            if let Err(err) = read_trailers(stream, headers, max_trailing_headers_size) {
                return bail(err);
            }
            return Ok(0);
        }
        *to_go = incoming_length;
    }

    assert!(*to_go > 0); // If we got here, there's still bytes to read!

    let to_read = buf.len().min(*to_go);
    let bytes_read = stream.read(&mut buf[..to_read])?;
    *to_go -= bytes_read;

    // If we're at the end of the chunk...
    if *to_go == 0 {
        //...read the trailing \r\n of the chunk, and
        // possibly return an error instead.

        // TODO: Maybe this could be written in a way
        // that doesn't discard the last ok buffer if
        // the \r\n reading fails?
        if let Err(err) = read_line(stream, Some(2), Error::MalformedChunkEnd) {
            return bail(err);
        }
    }
    Ok(bytes_read)
}

enum HttpStreamState {
    // No Content-Length, and Transfer-Encoding != chunked, so we just
    // read unti lthe server closes the connection (this should be the
    // fallback, if I read the rfc right).
    EndOnClose,
    // Content-Length was specified, store the number of bytes remaining
    ContentLength { to_go: usize },
    // Transfer-Encoding == chunked, so we need to save two pieces of
    // information: are we expecting more chunks, and how much is there
    // left of the current chunk?
    Chunked { more_chunks: bool, to_go: usize },
}

// This struct is just used in the Response and ResponseLazy
// constructors, but not in their structs, for api-cleanliness
// reasons. (Eg. response.status_code is much cleaner than
// response.meta.status_code or similar.)
struct ResponseMetadata {
    status_code: i32,
    reason_phrase: String,
    headers: HashMap<String, String>,
    state: HttpStreamState,
    max_trailing_headers_size: Option<usize>,
}

fn read_metadata(
    stream: &mut BufReader<HttpStream>,
    mut max_headers_size: Option<usize>,
    max_status_line_len: Option<usize>,
) -> Result<ResponseMetadata, Error> {
    let line = read_line(stream, max_status_line_len, Error::StatusLineOverflow)?;
    let (status_code, reason_phrase) = parse_status_line(&line);

    let mut headers = HashMap::new();
    loop {
        let line = read_line(stream, max_headers_size, Error::HeadersOverflow)?;
        if line.is_empty() {
            // Body starts here
            break;
        }
        if let Some(ref mut max_headers_size) = max_headers_size {
            *max_headers_size -= line.len() + 2;
        }
        if let Some(header) = parse_header(line) {
            headers.insert(header.0, header.1);
        }
    }

    let mut chunked = false;
    let mut content_length = None;
    for (header, value) in &headers {
        // Handle the Transfer-Encoding header
        if header.to_lowercase().trim() == "transfer-encoding"
            && value.to_lowercase().trim() == "chunked"
        {
            chunked = true;
        }

        // Handle the Content-Length header
        if header.to_lowercase().trim() == "content-length" {
            match str::parse::<usize>(value.trim()) {
                Ok(length) => content_length = Some(length),
                Err(_) => return Err(Error::MalformedContentLength),
            }
        }
    }

    let state = if chunked {
        HttpStreamState::Chunked {
            more_chunks: true,
            to_go: 0,
        }
    } else if let Some(length) = content_length {
        HttpStreamState::ContentLength { to_go: length }
    } else {
        HttpStreamState::EndOnClose
    };

    Ok(ResponseMetadata {
        status_code,
        reason_phrase,
        headers,
        state,
        max_trailing_headers_size: max_headers_size,
    })
}

fn read_line(
    stream: &mut BufReader<HttpStream>,
    max_len: Option<usize>,
    overflow_error: Error,
) -> Result<String, Error> {
    let mut bytes = Vec::with_capacity(32);
    for byte in stream.bytes() {
        match byte {
            Ok(byte) => {
                if let Some(max_len) = max_len {
                    if bytes.len() >= max_len {
                        return Err(overflow_error);
                    }
                }
                if byte == b'\n' {
                    if let Some(b'\r') = bytes.last() {
                        bytes.pop();
                    }
                    break;
                } else {
                    bytes.push(byte);
                }
            }
            Err(err) => return Err(Error::IoError(err)),
        }
    }
    String::from_utf8(bytes).map_err(|_error| Error::InvalidUtf8InResponse)
}

fn parse_status_line(line: &str) -> (i32, String) {
    // sample status line format
    // HTTP/1.1 200 OK
    let mut status_code = String::with_capacity(3);
    let mut reason_phrase = String::with_capacity(2);

    let mut spaces = 0;

    for c in line.chars() {
        if spaces >= 2 {
            reason_phrase.push(c);
        }

        if c == ' ' {
            spaces += 1;
        } else if spaces == 1 {
            status_code.push(c);
        }
    }

    if let Ok(status_code) = status_code.parse::<i32>() {
        return (status_code, reason_phrase);
    }

    (503, "Server did not provide a status line".to_string())
}

fn parse_header(mut line: String) -> Option<(String, String)> {
    if let Some(location) = line.find(':') {
        // Trim the first character of the header if it is a space,
        // otherwise return everything after the ':'. This should
        // preserve the behavior in versions <=2.0.1 in most cases
        // (namely, ones where it was valid), where the first
        // character after ':' was always cut off.
        let value = if let Some(sp) = line.get(location + 1..location + 2) {
            if sp == " " {
                line[location + 2..].to_string()
            } else {
                line[location + 1..].to_string()
            }
        } else {
            line[location + 1..].to_string()
        };

        line.truncate(location);
        // Headers should be ascii, I'm pretty sure. If not, please open an issue.
        line.make_ascii_lowercase();
        return Some((line, value));
    }
    None
}
