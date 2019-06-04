use crate::{http, Request, Response};
#[cfg(feature = "https")]
use rustls::{self, ClientConfig, ClientSession};
use std::env;
use std::io::{BufReader, BufWriter, Error, ErrorKind, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
#[cfg(feature = "https")]
use std::sync::Arc;
use std::time::Duration;
#[cfg(feature = "https")]
use webpki::DNSNameRef;
#[cfg(feature = "https")]
use webpki_roots::TLS_SERVER_ROOTS;

/// A connection to the server for sending
/// [`Request`](struct.Request.html)s.
pub struct Connection {
    request: Request,
    timeout: Option<u64>,
}

impl Connection {
    /// Creates a new `Connection`. See
    /// [`Request`](struct.Request.html) for specifics about *what* is
    /// being sent.
    pub(crate) fn new(request: Request) -> Connection {
        let timeout = request
            .timeout
            .or_else(|| match env::var("MINREQ_TIMEOUT") {
                Ok(t) => t.parse::<u64>().ok(),
                Err(_) => None,
            });
        Connection { request, timeout }
    }

    /// Sends the [`Request`](struct.Request.html), consumes this
    /// connection, and returns a [`Response`](struct.Response.html).
    #[cfg(feature = "https")]
    pub(crate) fn send_https(self) -> Result<Response, Error> {
        let is_head = self.request.method == http::Method::Head;
        let bytes = self.request.to_string().into_bytes();

        // Rustls setup
        let dns_name = &self.request.host;
        let dns_name = dns_name.split(":").next()?;
        let dns_name = DNSNameRef::try_from_ascii_str(dns_name)?;
        let mut config = ClientConfig::new();
        config
            .root_store
            .add_server_trust_anchors(&TLS_SERVER_ROOTS);
        let mut sess = ClientSession::new(&Arc::new(config), dns_name);

        // IO
        let mut stream = create_tcp_stream(&self.request.host, self.timeout)?;
        let mut tls = rustls::Stream::new(&mut sess, &mut stream);
        tls.write(&bytes)?;
        match read_from_stream(tls, is_head) {
            Ok(result) => handle_redirects(self, Response::from_bytes(result)),
            Err(err) => Err(err),
        }
    }

    /// Sends the [`Request`](struct.Request.html), consumes this
    /// connection, and returns a [`Response`](struct.Response.html).
    pub(crate) fn send(self) -> Result<Response, Error> {
        let is_head = self.request.method == http::Method::Head;
        let bytes = self.request.to_string().into_bytes();

        let tcp = create_tcp_stream(&self.request.host, self.timeout)?;

        // Send request
        let mut stream = BufWriter::new(tcp);
        stream.write_all(&bytes)?;

        // Receive response
        let tcp = stream.into_inner()?;
        let mut stream = BufReader::new(tcp);
        match read_from_stream(&mut stream, is_head) {
            Ok(response) => handle_redirects(self, Response::from_bytes(response)),
            Err(err) => match err.kind() {
                ErrorKind::WouldBlock | ErrorKind::TimedOut => Err(Error::new(
                    ErrorKind::TimedOut,
                    format!(
                        "Request timed out! Timeout: {:?}",
                        stream.get_ref().read_timeout()
                    ),
                )),
                _ => Err(err),
            },
        }
    }
}

fn handle_redirects(connection: Connection, response: Response) -> Result<Response, Error> {
    let status_code = response.status_code;
    match status_code {
        301 | 302 | 303 | 307 => {
            let url = response.headers.get("Location");
            if url.is_none() {
                return Err(Error::new(
                    ErrorKind::Other,
                    "'Location' header missing in redirect.",
                ));
            }
            let url = url.unwrap();
            let mut request = connection.request;

            if request.redirects.contains(&url) {
                Err(Error::new(ErrorKind::Other, "Infinite redirection loop."))
            } else {
                request.redirect_to(url.clone());
                if status_code == 303 {
                    match request.method {
                        http::Method::Post | http::Method::Put | http::Method::Delete => {
                            request.method = http::Method::Get;
                        }
                        _ => {}
                    }
                }

                request.send()
            }
        }

        _ => Ok(response),
    }
}

fn create_tcp_stream<A>(host: A, timeout: Option<u64>) -> Result<TcpStream, Error>
where
    A: ToSocketAddrs,
{
    let stream = TcpStream::connect(host)?;
    if let Some(secs) = timeout {
        let dur = Some(Duration::from_secs(secs));
        stream.set_read_timeout(dur)?;
        stream.set_write_timeout(dur)?;
    }
    Ok(stream)
}

/// Reads the stream until it can't or it reaches the end of the HTTP
/// response.
fn read_from_stream<T: Read>(stream: T, head: bool) -> Result<Vec<u8>, Error> {
    let mut response = Vec::new();
    let mut response_length = None;
    let mut chunked = false;
    let mut expecting_chunk_length = false;
    let mut byte_count = 0;
    let mut last_newline_index = 0;
    let mut blank_line = false;
    let mut status_code = None;

    for byte in stream.bytes() {
        let byte = byte?;
        response.push(byte);
        byte_count += 1;
        if byte == b'\n' {
            if status_code.is_none() {
                // First line
                status_code = Some(http::parse_status_line(&response).0);
            }

            if blank_line {
                // Two consecutive blank lines, body should start here
                if let Some(code) = status_code {
                    if head || code / 100 == 1 || code == 204 || code == 304 {
                        response_length = Some(response.len());
                    }
                }
                if response_length.is_none() {
                    if let Ok(response_str) = std::str::from_utf8(&response) {
                        let len = get_response_length(response_str);
                        response_length = Some(len);
                        if len > response.len() {
                            response.reserve(len - response.len());
                        }
                    }
                }
            } else if let Ok(new_response_length_str) =
                std::str::from_utf8(&response[last_newline_index..])
            {
                if expecting_chunk_length {
                    expecting_chunk_length = false;

                    if let Ok(n) = usize::from_str_radix(new_response_length_str.trim(), 16) {
                        // Cut out the chunk length from the reponse
                        response.truncate(last_newline_index);
                        byte_count = last_newline_index;
                        // Update response length according to the new chunk length
                        if n == 0 {
                            break;
                        } else {
                            response_length = Some(byte_count + n + 2);
                        }
                    }
                } else if let Some((key, value)) = http::parse_header(new_response_length_str) {
                    if key.trim() == "Transfer-Encoding" && value.trim() == "chunked" {
                        chunked = true;
                    }
                }
            }

            blank_line = true;
            last_newline_index = byte_count;
        } else if byte != b'\r' {
            // Normal character, reset blank_line
            blank_line = false;
        }

        if let Some(len) = response_length {
            if byte_count >= len {
                if chunked {
                    // Transfer-Encoding is chunked, next up should be
                    // the next chunk's length.
                    expecting_chunk_length = true;
                } else {
                    // We have reached the end of the HTTP response,
                    // break the reading loop.
                    break;
                }
            }
        }
    }

    Ok(response)
}

/// Tries to find out how long the whole response will eventually be,
/// in bytes.
fn get_response_length(response: &str) -> usize {
    // The length of the headers
    let mut byte_count = 0;
    for line in response.lines() {
        byte_count += line.len() + 2;
        if line.starts_with("Content-Length: ") {
            if let Ok(length) = line[16..].parse::<usize>() {
                byte_count += length;
            }
        }
    }
    byte_count
}
