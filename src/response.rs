use crate::{connection::HttpStream, Error};
use std::collections::HashMap;
use std::io::{Bytes, Read};
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
#[derive(Clone, PartialEq, Debug)]
pub struct Response {
    /// The status code of the response, eg. 404.
    pub status_code: i32,
    /// The reason phrase of the response, eg. "Not Found".
    pub reason_phrase: String,
    /// The headers of the response. The header field names (the
    /// keys) are all lowercase.
    pub headers: HashMap<String, String>,

    body: Vec<u8>,
}

impl Response {
    pub(crate) fn create(mut parent: ResponseLazy, is_head: bool) -> Result<Response, Error> {
        let mut body = Vec::new();
        if !is_head && parent.status_code != 204 && parent.status_code != 304 {
            for byte in &mut parent {
                let (byte, length) = byte?;
                body.reserve(length);
                body.push(byte);
            }
        }

        let ResponseLazy {
            status_code,
            reason_phrase,
            headers,
            ..
        } = parent;

        Ok(Response {
            status_code,
            reason_phrase,
            headers,
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
    /// use serde_derive::Deserialize;
    ///
    /// #[derive(Deserialize)]
    /// struct User {
    ///     name: String,
    ///     email: String,
    /// }
    ///
    /// # fn main() -> Result<(), minreq::Error> {
    /// # let url_to_json_resource = "http://example.org/resource.json";
    /// let user_name = minreq::get(url_to_json_resource).send()?
    ///     .json::<User>()? // explicitly declared type `User`
    ///     .name;
    /// println!("User name is '{}'", &user_name);
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

/// An HTTP response, which is loaded lazily.
///
/// In comparison to [`Response`](struct.Response.html), this is
/// returned from
/// [`send_lazy()`](struct.Request.html#method.send_lazy), where as
/// [`Response`](struct.Response.html) is returned from
/// [`send()`](struct.Request.html#method.send).
///
/// In practice, "lazy loading" means that the bytes are only loaded
/// as you iterate through them. The bytes are provided in the form of
/// a `Result<(u8, usize), minreq::Error>`, as the reading operation
/// can fail in various ways. The `u8` is the actual byte that was
/// read, and `usize` is how many bytes we are expecting to read in
/// the future (including this byte). Note, however, that the `usize`
/// can change, particularly when the `Transfer-Encoding` is
/// `chunked`: then it will reflect how many bytes are left of the
/// current chunk.
///
/// # Example
/// ```no_run
/// // This is how the normal Response works behind the scenes, and
/// // how you might use ResponseLazy.
/// # fn main() -> Result<(), minreq::Error> {
/// let response = minreq::get("http://httpbin.org/ip").send_lazy()?;
/// let mut vec = Vec::new();
/// for result in response {
///     let (byte, length) = result?;
///     vec.reserve(length);
///     vec.push(byte);
/// }
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

    stream: Bytes<HttpStream>,
    state: HttpStreamState,
}

impl ResponseLazy {
    pub(crate) fn from_stream(stream: HttpStream) -> Result<ResponseLazy, Error> {
        let mut stream = stream.bytes();
        let ResponseMetadata {
            status_code,
            reason_phrase,
            headers,
            state,
        } = read_metadata(&mut stream)?;

        Ok(ResponseLazy {
            status_code,
            reason_phrase,
            headers,
            stream,
            state,
        })
    }
}

impl Iterator for ResponseLazy {
    type Item = Result<(u8, usize), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        use HttpStreamState::*;
        match self.state {
            EndOnClose => read_until_closed(&mut self.stream),
            ContentLength(ref mut length) => read_with_content_length(&mut self.stream, length),
            Chunked(ref mut expecting_chunks, ref mut length, ref mut content_length) => {
                read_chunked(
                    &mut self.stream,
                    &mut self.headers,
                    expecting_chunks,
                    length,
                    content_length,
                )
            }
        }
    }
}

fn read_until_closed(bytes: &mut Bytes<HttpStream>) -> Option<<ResponseLazy as Iterator>::Item> {
    if let Some(byte) = bytes.next() {
        match byte {
            Ok(byte) => Some(Ok((byte, 1))),
            Err(err) => Some(Err(Error::IoError(err))),
        }
    } else {
        None
    }
}

fn read_with_content_length(
    bytes: &mut Bytes<HttpStream>,
    content_length: &mut usize,
) -> Option<<ResponseLazy as Iterator>::Item> {
    if *content_length > 0 {
        *content_length -= 1;

        if let Some(byte) = bytes.next() {
            match byte {
                Ok(byte) => return Some(Ok((byte, *content_length + 1))),
                Err(err) => return Some(Err(Error::IoError(err))),
            }
        }
    }
    None
}

fn read_trailers(
    bytes: &mut Bytes<HttpStream>,
    headers: &mut HashMap<String, String>,
) -> Result<(), Error> {
    loop {
        let trailer_line = read_line(bytes)?;
        if let Some((header, value)) = parse_header(trailer_line) {
            headers.insert(header, value);
        } else {
            break;
        }
    }
    Ok(())
}

fn read_chunked(
    bytes: &mut Bytes<HttpStream>,
    headers: &mut HashMap<String, String>,
    expecting_more_chunks: &mut bool,
    chunk_length: &mut usize,
    content_length: &mut usize,
) -> Option<<ResponseLazy as Iterator>::Item> {
    if !*expecting_more_chunks && *chunk_length == 0 {
        return None;
    }

    if *chunk_length == 0 {
        // Get the size of the next chunk
        let length_line = match read_line(bytes) {
            Ok(line) => line,
            Err(err) => return Some(Err(err)),
        };
        match usize::from_str_radix(&length_line, 16) {
            Ok(incoming_length) => {
                if incoming_length == 0 {
                    if let Err(err) = read_trailers(bytes, headers) {
                        return Some(Err(err));
                    }

                    *expecting_more_chunks = false;
                    headers.insert("content-length".to_owned(), (*content_length).to_string());
                    headers.remove("transfer-encoding");
                    return None;
                }
                *chunk_length = incoming_length;
                *content_length += incoming_length;
            }
            Err(_) => return Some(Err(Error::MalformedChunkLength)),
        }
    }

    if *chunk_length > 0 {
        *chunk_length -= 1;
        if let Some(byte) = bytes.next() {
            match byte {
                Ok(byte) => {
                    // If we're at the end of the chunk...
                    if *chunk_length == 0 {
                        //...read the trailing \r\n of the chunk, and
                        // possibly return an error instead.

                        // TODO: Maybe this could be written in a way
                        // that doesn't discard the last ok byte if
                        // the \r\n reading fails?
                        if let Err(err) = read_line(bytes) {
                            return Some(Err(err));
                        }
                    }

                    return Some(Ok((byte, *chunk_length + 1)));
                }
                Err(err) => return Some(Err(Error::IoError(err))),
            }
        }
    }

    None
}

enum HttpStreamState {
    // No Content-Length, and Transfer-Encoding != chunked, so we just
    // read unti lthe server closes the connection (this should be the
    // fallback, if I read the rfc right).
    EndOnClose,
    // Content-Length was specified, read that amount of bytes
    ContentLength(usize),
    // Transfer-Encoding == chunked, so we need to save two pieces of
    // information: are we expecting more chunks, how much is there
    // left of the current chunk, and how much have we read? The last
    // number is needed in order to provide an accurate Content-Length
    // header after loading all the bytes.
    Chunked(bool, usize, usize),
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
}

fn read_metadata(stream: &mut Bytes<HttpStream>) -> Result<ResponseMetadata, Error> {
    let (status_code, reason_phrase) = parse_status_line(read_line(stream)?);

    let mut headers = HashMap::new();
    loop {
        let line = read_line(stream)?;
        if line.is_empty() {
            // Body starts here
            break;
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
        HttpStreamState::Chunked(true, 0, 0)
    } else if let Some(length) = content_length {
        HttpStreamState::ContentLength(length)
    } else {
        HttpStreamState::EndOnClose
    };

    Ok(ResponseMetadata {
        status_code,
        reason_phrase,
        headers,
        state,
    })
}

fn read_line(stream: &mut Bytes<HttpStream>) -> Result<String, Error> {
    let mut bytes = Vec::with_capacity(32);
    for byte in stream {
        match byte {
            Ok(byte) => {
                if byte == b'\n' {
                    // Pop the \r off, as HTTP lines end in \r\n.
                    bytes.pop();
                    break;
                } else {
                    bytes.push(byte);
                }
            }
            Err(err) => {
                return Err(Error::IoError(err));
            }
        }
    }
    match String::from_utf8(bytes) {
        Ok(line) => Ok(line),
        Err(_) => Err(Error::InvalidUtf8InResponse),
    }
}

fn parse_status_line(line: String) -> (i32, String) {
    let mut split = line.split(' ');
    if let Some(code) = split.nth(1) {
        if let Ok(code) = code.parse::<i32>() {
            if let Some(reason) = split.next() {
                return (code, reason.to_owned());
            }
        }
    }
    (503, "Server did not provide a status line".to_owned())
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
                line[location + 2..].to_owned()
            } else {
                line[location + 1..].to_owned()
            }
        } else {
            line[location + 1..].to_owned()
        };

        line.truncate(location);
        // Headers should be ascii, I'm pretty sure. If not, please open an issue.
        line.make_ascii_lowercase();
        return Some((line, value));
    }
    None
}
