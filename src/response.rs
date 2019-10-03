use crate::Error;
use std::collections::HashMap;
use std::io::{Bytes, Read};
use std::str;

struct ResponseIter<T: Read> {
    bytes: Bytes<T>,
    chunked: bool,
    chunks_done: bool,
    expected_bytes: usize,
}

impl<T: Read> ResponseIter<T> {
    fn new(bytes: Bytes<T>, chunked: bool, expected_bytes: usize) -> ResponseIter<T> {
        ResponseIter {
            bytes,
            chunked,
            chunks_done: false,
            expected_bytes,
        }
    }
}

impl<T: Read> Iterator for ResponseIter<T> {
    // u8 is the byte that was read, usize is how much you should
    // reserve in a Vec if you're pushing the bytes into it for
    // optimal operation.
    type Item = Result<(u8, usize), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.chunked {
            if self.chunks_done {
                return None;
            }

            if self.expected_bytes == 0 {
                // Read the end of the last chunk first, return error if encountered
                if let Err(err) = read_line(&mut self.bytes) {
                    return Some(Err(err));
                }

                // Get the size of the next chunk
                let count_line = match read_line(&mut self.bytes) {
                    Ok(line) => line,
                    Err(err) => return Some(Err(err)),
                };
                match str::parse::<usize>(&count_line) {
                    Ok(incoming_count) => {
                        if incoming_count == 0 {
                            self.chunks_done = true;
                            return None;
                        }
                        self.expected_bytes = incoming_count;
                    }
                    Err(_) => return Some(Err(Error::MalformedChunkLength)),
                }
            }
        }

        // Read the next byte
        if self.expected_bytes > 0 {
            self.expected_bytes -= 1;

            if let Some(byte) = self.bytes.next() {
                return match byte {
                    Ok(byte) => Some(Ok((byte, self.expected_bytes + 1))),
                    Err(err) => Some(Err(Error::IoError(err))),
                };
            }
        }

        // Either we loaded all the bytes expected, or ran out of bytes
        None
    }
}

// This struct is just used in the Response and ResponseLazy
// constructors, but not in their structs, for api-cleanliness
// reasons. (Eg. response.status_code is much cleaner than
// response.meta.status_code or similar.)
struct ResponseMetadata {
    status_code: i32,
    reason_phrase: String,
    headers: HashMap<String, String>,
    chunked: bool,
    content_length: Option<usize>,
}

fn read_metadata<T: Read>(stream: &mut Bytes<T>) -> Result<ResponseMetadata, Error> {
    let (status_code, reason_phrase) = parse_status_line(read_line(stream)?);
    let mut headers = HashMap::new();
    let mut chunked = false;
    let mut content_length = None;

    // Read headers
    loop {
        let line = read_line(stream)?;
        if line.len() == 0 {
            // Body starts here
            break;
        }
        if let Some(header) = parse_header(line) {
            if !chunked
                && &header.0.to_lowercase() == "transfer-encoding"
                && &header.1.to_lowercase() == "chunked"
            {
                chunked = true;
            }
            if content_length.is_none() && &header.0.to_lowercase() == "content-length" {
                match str::parse::<usize>(&header.1.trim()) {
                    Ok(length) => content_length = Some(length),
                    Err(_) => return Err(Error::MalformedContentLength),
                }
            }
            headers.insert(header.0, header.1);
        }
    }

    Ok(ResponseMetadata {
        status_code,
        reason_phrase,
        headers,
        chunked,
        content_length,
    })
}

/// An HTTP response.
#[derive(Debug, Clone)]
pub struct Response {
    /// The status code of the response, eg. 404.
    pub status_code: i32,
    /// The reason phrase of the response, eg. "Not Found".
    pub reason_phrase: String,
    /// The headers of the response.
    pub headers: HashMap<String, String>,
    /// The body of the response.
    pub body: Vec<u8>,
}

impl Response {
    pub(crate) fn from_stream<T: Read>(stream: T, is_head: bool) -> Result<Response, Error> {
        let mut stream = stream.bytes();
        let ResponseMetadata {
            status_code,
            reason_phrase,
            headers,
            chunked,
            content_length,
        } = read_metadata(&mut stream)?;

        // Read body (if needed)
        let bodyless_status = status_code / 100 == 1 || status_code == 204 || status_code == 304;
        let body = if is_head || bodyless_status {
            // No body!
            Vec::new()
        } else {
            let count = content_length.unwrap_or(0);
            let iter = ResponseIter::new(stream, chunked, count);
            let mut collected = Vec::with_capacity(count);
            for byte in iter {
                match byte {
                    Ok((byte, length)) => {
                        collected.reserve(length);
                        collected.push(byte)
                    }
                    Err(err) => return Err(err),
                }
            }
            collected
        };

        Ok(Response {
            status_code,
            reason_phrase,
            headers,
            body,
        })
    }

    /// Returns a `&str` constructed from the body of the
    /// `Response`. Shorthand for
    /// `std::str::from_utf8(&response.body)`.
    ///
    /// Returns a `Result`, as it is possible that the returned bytes
    /// are not valid UTF-8: the message can be corrupted on the
    /// server's side, it could be still loading (`flush()` not yet
    /// called, for example), or the returned message could simply not
    /// be valid UTF-8.
    ///
    /// Usage:
    /// ```no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let url = "http://example.org/";
    /// let response = minreq::get(url).send()?;
    /// println!("{}", response.as_str()?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn as_str(&self) -> Result<&str, str::Utf8Error> {
        str::from_utf8(&self.body)
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

fn read_line<T: Read>(stream: &mut Bytes<T>) -> Result<String, Error> {
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
        Ok(line) => Ok(line.to_string()),
        Err(_) => Err(Error::InvalidUtf8InResponse),
    }
}

fn parse_status_line(line: String) -> (i32, String) {
    let mut split = line.split(' ');
    if let Some(code) = split.nth(1) {
        if let Ok(code) = code.parse::<i32>() {
            if let Some(reason) = split.next() {
                return (code, reason.to_string());
            }
        }
    }
    (503, "Server did not provide a status line".to_owned())
}

fn parse_header(mut line: String) -> Option<(String, String)> {
    if let Some(location) = line.find(':') {
        let value = line.split_off(location + 1);
        line.truncate(location);
        return Some((line, value));
    }
    None
}
