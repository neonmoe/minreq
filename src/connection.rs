use crate::{http, Error, Request, Response};
#[cfg(feature = "https")]
use rustls::{self, ClientConfig, ClientSession};
use std::env;
use std::io::{BufReader, BufWriter, Write};
use std::net::{TcpStream, ToSocketAddrs};
#[cfg(feature = "https")]
use std::sync::Arc;
use std::time::Duration;
#[cfg(feature = "https")]
use webpki::DNSNameRef;
#[cfg(feature = "https")]
use webpki_roots::TLS_SERVER_ROOTS;

#[cfg(feature = "https")]
lazy_static::lazy_static! {
    static ref CONFIG: Arc<ClientConfig> = {
        let mut config = ClientConfig::new();
        config
            .root_store
            .add_server_trust_anchors(&TLS_SERVER_ROOTS);
        Arc::new(config)
    };
}

/// A connection to the server for sending
/// [`Request`](struct.Request.html)s.
#[derive(Debug)]
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
        let dns_name = dns_name.split(":").next().unwrap();
        let dns_name = DNSNameRef::try_from_ascii_str(dns_name).unwrap();
        let mut sess = ClientSession::new(&CONFIG, dns_name);

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

        let tcp = match create_tcp_stream(&self.request.host, self.timeout) {
            Ok(stream) => stream,
            Err(err) => {
                return Err(Error::IoError(err));
            }
        };

        // Send request
        let mut stream = BufWriter::new(tcp);
        if let Err(err) = stream.write_all(&bytes) {
            return Err(Error::IoError(err));
        }

        // Receive response
        let tcp = match stream.into_inner() {
            Ok(stream) => stream,
            Err(_) => {
                return Err(Error::Other(
                    "IntoInnerError after writing the request into the TcpStream.",
                ));
            }
        };
        let stream = BufReader::new(tcp);
        match Response::from_stream(stream, is_head) {
            Ok(response) => handle_redirects(self, response),
            Err(err) => Err(err),
        }
    }
}

fn handle_redirects(connection: Connection, response: Response) -> Result<Response, Error> {
    let status_code = response.status_code;
    match status_code {
        301 | 302 | 303 | 307 => {
            let url = response.headers.get("Location");
            if url.is_none() {
                return Err(Error::RedirectLocationMissing);
            }
            let url = url.unwrap();

            if let Some(mut request) = connection.request.redirect_to(url.clone()) {
                if status_code == 303 {
                    match request.method {
                        http::Method::Post | http::Method::Put | http::Method::Delete => {
                            request.method = http::Method::Get;
                        }
                        _ => {}
                    }
                }

                request.send()
            } else {
                Err(Error::InfiniteRedirectionLoop)
            }
        }

        _ => Ok(response),
    }
}

fn create_tcp_stream<A>(host: A, timeout: Option<u64>) -> Result<TcpStream, std::io::Error>
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
