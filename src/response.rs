use crate::MinreqError;
use std::collections::HashMap;
use std::io::{Bytes, Read};
use std::str;

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
    pub(crate) fn from_stream<T: Read>(stream: T, is_head: bool) -> Result<Response, MinreqError> {
        let stream_ = stream;
        let mut stream = stream_.bytes();
        let (status_code, reason_phrase) = parse_status_line(read_line(&mut stream)?);
        let mut headers = HashMap::new();
        let mut chunked = false;
        let mut expected_size = None;

        // Read headers
        loop {
            let line = read_line(&mut stream)?;
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
                if expected_size.is_none() && &header.0.to_lowercase() == "content-length" {
                    match str::parse::<usize>(&header.1.trim()) {
                        Ok(length) => expected_size = Some(length),
                        Err(_) => return Err(MinreqError::MalformedContentLength),
                    }
                }
                headers.insert(header.0, header.1);
            }
        }

        // Read body (if needed)
        let ignore_body =
            is_head || status_code / 100 == 1 || status_code == 204 || status_code == 304;
        let body = if ignore_body {
            Vec::new()
        } else if chunked {
            // Transfer-Encoding: chunked
            let mut body = Vec::new();
            loop {
                match str::parse::<usize>(&read_line(&mut stream)?) {
                    Ok(incoming_count) => {
                        if incoming_count == 0 {
                            break;
                        }
                        body.reserve(incoming_count);
                        for byte in &mut stream {
                            if let Ok(byte) = byte {
                                body.push(byte);
                            }
                        }
                    }
                    Err(_) => return Err(MinreqError::MalformedChunkLength),
                }
            }
            body
        } else if let Some(content_length) = expected_size {
            stream.take(content_length).filter_map(|b| b.ok()).collect()
        } else {
            // NOTE: Maybe this should return an error? The HTTP
            // standard says that it is valid to assume that a message
            // ends after the server closes the connection, but that
            // Content-Length should be respected over that: so if
            // there is no Content-Length, maybe falling back on
            // waiting for the server to close the connection is fine?
            stream.filter_map(|b| b.ok()).collect()
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
    pub fn json<'a, T>(&'a self) -> Result<T, serde_json::Error>
    where
        T: serde::de::Deserialize<'a>,
    {
        serde_json::from_str(&self.body)
    }
}

fn read_line<T: Read>(stream: &mut Bytes<T>) -> Result<String, MinreqError> {
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
                return Err(MinreqError::IOError(err));
            }
        }
    }
    match String::from_utf8(bytes) {
        Ok(line) => Ok(line.to_string()),
        Err(_) => Err(MinreqError::InvalidUtf8InResponse),
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
