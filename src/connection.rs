use std::io::prelude::*;
use std::io::{Error, ErrorKind};
use std::net::TcpStream;
use std::time::Duration;
use std::env;
use http::{Request, Response};

/// A connection to the server for sending
/// [`Request`](struct.Request.html)s.
pub struct Connection {
    request: Request,
    timeout: u64,
}

impl Connection {
    /// Creates a new `Connection`. See
    /// [`Request`](struct.Request.html) for specifics about *what* is
    /// being sent.
    pub(crate) fn new(request: Request) -> Connection {
        let timeout = env::var("MINREQ_TIMEOUT")
            .unwrap_or("5".to_string())
            .parse::<u64>()
            .unwrap_or(5);
        Connection { request, timeout }
    }

    /// Sets how long it takes to timeout (in seconds) for this
    /// connection. Usage:
    /// ```no_run
    /// use minreq::Method;
    ///
    /// minreq::create_connection(Method::Get, "https://httpbin.org/ip", None)
    ///     .with_timeout(10)
    ///     .send();
    /// ```
    pub fn with_timeout(mut self, timeout: u64) -> Connection {
        self.timeout = timeout;
        self
    }

    /// Sends the [`Request`](struct.Request.html), consumes this
    /// connection, and returns a [`Response`](struct.Response.html).
    pub fn send(self) -> Result<Response, Error> {
        let host = self.request.host.clone();
        let bytes = self.request.into_string().into_bytes();

        let mut stream = TcpStream::connect(host)?;
        // Set the timeouts if possible, but they aren't required..
        stream
            .set_read_timeout(Some(Duration::from_secs(self.timeout)))
            .ok();
        stream
            .set_write_timeout(Some(Duration::from_secs(self.timeout)))
            .ok();

        stream.write_all(&bytes)?;
        match read_from_stream(&stream) {
            Ok(response) => Ok(Response::from_string(response)),
            Err(err) => match err.kind() {
                ErrorKind::WouldBlock | ErrorKind::TimedOut => Err(Error::new(
                    ErrorKind::TimedOut,
                    format!("Request timed out! Timeout: {:?}", stream.read_timeout()),
                )),
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
    let mut blank_line = false;

    for byte in stream.bytes() {
        let byte = byte?;
        let c = byte as char;
        response.push(c);
        byte_count += 1;
        if c == '\n' {
            // End of line, try to get the response length
            if blank_line {
                response_length = Some(get_response_length(response.clone()));
            }
            blank_line = true;
        } else if c != '\r' {
            // Normal character, reset blank_line
            blank_line = false;
        }

        if let Some(len) = response_length {
            if byte_count == len {
                // We have reached the end of the HTTP
                // response, break the reading loop.
                break;
            }
        }
    }

    Ok(response)
}

/// Tries to find out how long the whole response will eventually be,
/// in bytes.
fn get_response_length(response: String) -> usize {
    // The length of the headers
    let mut byte_count = 0;
    for line in response.lines() {
        byte_count += line.len() + 2;
        if line.starts_with("Content-Length: ") {
            byte_count += line.clone()[16..].parse::<usize>().unwrap();
        }
    }
    byte_count
}
