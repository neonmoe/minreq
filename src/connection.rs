use std::io::prelude::*;
use std::io::{Error, ErrorKind};
use std::net::TcpStream;
use std::time::Duration;
use http::{Request, Response};

/// A connection to the server for sending
/// [`Request`](struct.Request.html)s.
pub struct Connection {
    request: Request,
}

impl Connection {
    /// Creates a new `Connection`. See
    /// [`Request`](struct.Request.html) for specifics about *what* is
    /// being sent.
    pub fn new(request: Request) -> Connection {
        Connection { request }
    }

    /// Sends the [`Request`](struct.Request.html), consumes this
    /// connection, and returns a [`Response`](struct.Response.html).
    pub fn send(self) -> Result<Response, Error> {
        let host = self.request.host.clone();
        let bytes = self.request.into_string().into_bytes();

        let mut stream = TcpStream::connect(host)?;
        // Set the timeouts if possible, but they aren't required..
        stream.set_read_timeout(Some(Duration::from_secs(3))).ok();
        stream.set_write_timeout(Some(Duration::from_secs(3))).ok();

        stream.write_all(&bytes)?;
        match read_from_stream(&stream) {
            Ok(response) => Ok(Response::from_string(response)),
            Err(err) => match err.kind() {
                ErrorKind::WouldBlock | ErrorKind::TimedOut => {
                    Err(Error::new(ErrorKind::TimedOut, "Request timed out!"))
                }
                _ => Err(err),
            },
        }
    }
}

/// Reads the stream until it can't or it reaches the end of the HTTP
/// response.
fn read_from_stream(stream: &TcpStream) -> Result<String, Error> {
    let mut response = String::new();
    let mut response_length = None;
    let mut byte_count = 0;

    for byte in stream.bytes() {
        let byte = byte?;
        let c = byte as char;
        response.push(c);
        byte_count += 1;
        if let Some(len) = response_length {
            if byte_count == len {
                // We have reached the end of the HTTP
                // response, break the reading loop.
                break;
            }
        } else if c == '\n' {
            // End of line, try to get the response length
            response_length = get_response_length(response.clone());
        }
    }

    Ok(response)
}

/// Tries to find out how long the whole response will eventually be,
/// in bytes.
fn get_response_length(response: String) -> Option<usize> {
    // Have all the headers have been read and Content-Length found?
    let mut confirm = false;
    // Content-Length, or 0
    let mut length = 0;
    // The length of the headers
    let mut byte_count = 0;
    for line in response.lines() {
        byte_count += line.len() + 2;
        if line.starts_with("Content-Length: ") {
            length = line.clone()[16..].parse::<usize>().unwrap();
        } else if !confirm && line == "" {
            // This is the blank line before the body, headers have
            // been read.
            confirm = true;
            // Also append the length of the header to the length.
            length += byte_count;
            break;
        }
    }
    if confirm {
        Some(length)
    } else {
        None
    }
}
