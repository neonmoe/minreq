use crate::{Error, Method, Request, ResponseLazy};
#[cfg(feature = "https")]
use rustls::{self, ClientConfig, ClientSession, StreamOwned};
use std::env;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::net::TcpStream;
#[cfg(feature = "https")]
use std::sync::Arc;
use std::time::{Duration, Instant};
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

type UnsecuredStream = BufReader<TcpStream>;
#[cfg(feature = "https")]
type SecuredStream = StreamOwned<ClientSession, TcpStream>;

pub(crate) enum HttpStream {
    Unsecured(UnsecuredStream, Option<Instant>),
    #[cfg(feature = "https")]
    Secured(Box<SecuredStream>, Option<Instant>),
}

impl HttpStream {
    fn create_unsecured(reader: UnsecuredStream, timeout_at: Option<Instant>) -> HttpStream {
        HttpStream::Unsecured(reader, timeout_at)
    }

    #[cfg(feature = "https")]
    fn create_secured(reader: SecuredStream, timeout_at: Option<Instant>) -> HttpStream {
        HttpStream::Secured(Box::new(reader), timeout_at)
    }
}

impl Read for HttpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let timeout = |tcp: &TcpStream, timeout_at: Option<Instant>| {
            if let Some(timeout_at) = timeout_at {
                let now = Instant::now();
                if timeout_at <= now {
                    return Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "The request's timeout was reached.",
                    ));
                } else {
                    tcp.set_read_timeout(Some(timeout_at - now)).ok();
                }
            }
            Ok(())
        };

        match self {
            HttpStream::Unsecured(inner, timeout_at) => {
                timeout(inner.get_ref(), *timeout_at)?;
                inner.read(buf)
            }
            #[cfg(feature = "https")]
            HttpStream::Secured(inner, timeout_at) => {
                timeout(inner.get_ref(), *timeout_at)?;
                inner.read(buf)
            }
        }
    }
}

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
    pub(crate) fn send_https(mut self) -> Result<ResponseLazy, Error> {
        self.request.host = ensure_ascii_host(self.request.host)?;
        let bytes = self.request.as_bytes();
        let timeout_duration = self.timeout.map(|d| Duration::from_secs(d));
        let timeout_at = timeout_duration.map(|d| Instant::now() + d);

        // Rustls setup
        let dns_name = &self.request.host;
        let dns_name = dns_name.split(':').next().unwrap();
        let dns_name = DNSNameRef::try_from_ascii_str(dns_name).unwrap();
        let sess = ClientSession::new(&CONFIG, dns_name);

        let tcp = TcpStream::connect(&self.request.host)?;

        // Send request
        let mut tls = StreamOwned::new(sess, tcp);
        tls.get_ref().set_write_timeout(timeout_duration).ok();
        tls.write(&bytes)?;

        // Receive request
        let response = ResponseLazy::from_stream(HttpStream::create_secured(tls, timeout_at))?;
        handle_redirects(self, response)
    }

    /// Sends the [`Request`](struct.Request.html), consumes this
    /// connection, and returns a [`Response`](struct.Response.html).
    pub(crate) fn send(mut self) -> Result<ResponseLazy, Error> {
        self.request.host = ensure_ascii_host(self.request.host)?;
        let bytes = self.request.as_bytes();
        let timeout_duration = self.timeout.map(|d| Duration::from_secs(d));
        let timeout_at = timeout_duration.map(|d| Instant::now() + d);

        let tcp = TcpStream::connect(&self.request.host)?;

        // Send request
        let mut stream = BufWriter::new(tcp);
        stream.get_ref().set_write_timeout(timeout_duration).ok();
        stream.write_all(&bytes)?;

        // Receive response
        let tcp = match stream.into_inner() {
            Ok(tcp) => tcp,
            Err(_) => {
                return Err(Error::Other(
                    "IntoInnerError after writing the request into the TcpStream.",
                ));
            }
        };
        let stream = HttpStream::create_unsecured(BufReader::new(tcp), timeout_at);
        let response = ResponseLazy::from_stream(stream)?;
        handle_redirects(self, response)
    }
}

fn handle_redirects(connection: Connection, response: ResponseLazy) -> Result<ResponseLazy, Error> {
    let status_code = response.status_code;
    let url = response.headers.get("location");
    if let Some(request) = get_redirect(connection, status_code, url) {
        request?.send_lazy()
    } else {
        Ok(response)
    }
}

fn get_redirect(
    connection: Connection,
    status_code: i32,
    url: Option<&String>,
) -> Option<Result<Request, Error>> {
    match status_code {
        301 | 302 | 303 | 307 => {
            if url.is_none() {
                return Some(Err(Error::RedirectLocationMissing));
            }
            let url = url.unwrap();

            match connection.request.redirect_to(url.clone()) {
                Ok(mut request) => {
                    if status_code == 303 {
                        match request.method {
                            Method::Post | Method::Put | Method::Delete => {
                                request.method = Method::Get;
                            }
                            _ => {}
                        }
                    }

                    Some(Ok(request))
                }
                Err(err) => Some(Err(err)),
            }
        }

        _ => None,
    }
}

fn ensure_ascii_host(host: String) -> Result<String, Error> {
    if host.is_ascii() {
        Ok(host)
    } else {
        #[cfg(not(feature = "punycode"))]
        {
            Err(Error::PunycodeFeatureNotEnabled)
        }

        #[cfg(feature = "punycode")]
        {
            let mut result = String::with_capacity(host.len() * 2);
            for s in host.split('.') {
                if s.is_ascii() {
                    result += s;
                } else {
                    match punycode::encode(s) {
                        Ok(s) => result = result + "xn--" + &s,
                        Err(_) => return Err(Error::PunycodeConversionFailed),
                    }
                }
                result += ".";
            }
            result.truncate(result.len() - 1); // Remove the trailing dot
            Ok(result)
        }
    }
}
