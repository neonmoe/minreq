use std::io::{Error, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use std::env;
use http::{Request, Response};
#[cfg(feature = "https")]
use std::sync::Arc;
#[cfg(feature = "https")]
use rustls::{self, ClientConfig, ClientSession};
#[cfg(feature = "https")]
use webpki::DNSNameRef;
#[cfg(feature = "https")]
use webpki_roots::TLS_SERVER_ROOTS;

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
        let timeout;
        if let Some(t) = request.timeout {
            timeout = t;
        } else {
            timeout = env::var("MINREQ_TIMEOUT")
                .unwrap_or("5".to_string()) // Not defined -> 5
                .parse::<u64>()
                .unwrap_or(5); // NaN -> 5
        }
        Connection { request, timeout }
    }

    /// Sends the [`Request`](struct.Request.html), consumes this
    /// connection, and returns a [`Response`](struct.Response.html).
    #[cfg(feature = "https")]
    pub(crate) fn send_https(self) -> Result<Response, Error> {
        let host = self.request.host.clone();
        let bytes = self.request.into_string().into_bytes();

        // Rustls setup
        let dns_name = host.clone();
        let dns_name = dns_name.split(":").next().unwrap();
        let dns_name = DNSNameRef::try_from_ascii_str(dns_name).unwrap();
        let mut config = ClientConfig::new();
        config
            .root_store
            .add_server_trust_anchors(&TLS_SERVER_ROOTS);
        let mut sess = ClientSession::new(&Arc::new(config), dns_name);

        // IO
        let mut stream = create_tcp_stream(host, self.timeout)?;
        let mut tls = rustls::Stream::new(&mut sess, &mut stream);
        tls.write(&bytes)?;
        match read_from_stream(tls) {
            Ok(result) => Ok(Response::from_string(result)),
            Err(err) => Err(err),
        }
    }

    /// Sends the [`Request`](struct.Request.html), consumes this
    /// connection, and returns a [`Response`](struct.Response.html).
    pub(crate) fn send(self) -> Result<Response, Error> {
        let host = self.request.host.clone();
        let bytes = self.request.into_string().into_bytes();

        // IO
        let mut stream = create_tcp_stream(host, self.timeout)?;
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

fn create_tcp_stream(host: String, timeout: u64) -> Result<TcpStream, Error> {
    let stream = TcpStream::connect(host)?;
    let timeout = Some(Duration::from_secs(timeout));
    stream.set_read_timeout(timeout).ok();
    stream.set_write_timeout(timeout).ok();
    Ok(stream)
}

/// Reads the stream until it can't or it reaches the end of the HTTP
/// response.
fn read_from_stream<T: Read>(stream: T) -> Result<String, Error> {
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
            if blank_line && response_length.is_none() {
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
