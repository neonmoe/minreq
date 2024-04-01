#[cfg(all(
    not(feature = "rustls"),
    any(feature = "openssl", feature = "native-tls")
))]
use crate::native_tls::{TlsConnector, TlsStream};
use crate::request::ParsedRequest;
use crate::{Error, Method, ResponseLazy};
#[cfg(feature = "https-rustls")]
use once_cell::sync::Lazy;
#[cfg(feature = "rustls")]
use rustls::{
    self, ClientConfig, ClientConnection, OwnedTrustAnchor, RootCertStore, ServerName, StreamOwned,
};
#[cfg(feature = "rustls")]
use std::convert::TryFrom;
use std::env;
use std::io::{self, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
#[cfg(feature = "rustls")]
use std::sync::Arc;
use std::time::{Duration, Instant};
#[cfg(feature = "rustls-webpki")]
use webpki_roots::TLS_SERVER_ROOTS;

#[cfg(feature = "rustls")]
static CONFIG: Lazy<Arc<ClientConfig>> = Lazy::new(|| {
    let mut root_certificates = RootCertStore::empty();

    // Try to load native certs
    #[cfg(feature = "https-rustls-probe")]
    if let Ok(os_roots) = rustls_native_certs::load_native_certs() {
        for root_cert in os_roots {
            // Ignore erroneous OS certificates, there's nothing
            // to do differently in that situation anyways.
            let _ = root_certificates.add(&rustls::Certificate(root_cert.0));
        }
    }

    #[allow(deprecated)] // Need to use add_server_trust_anchors to compile with rustls 0.21.1
    root_certificates.add_server_trust_anchors(TLS_SERVER_ROOTS.iter().map(|ta| {
        OwnedTrustAnchor::from_subject_spki_name_constraints(
            ta.subject,
            ta.spki,
            ta.name_constraints,
        )
    }));
    let config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_certificates)
        .with_no_client_auth();
    Arc::new(config)
});

type UnsecuredStream = TcpStream;
#[cfg(feature = "rustls")]
type SecuredStream = StreamOwned<ClientConnection, TcpStream>;
#[cfg(all(
    not(feature = "rustls"),
    any(feature = "openssl", feature = "native-tls")
))]
type SecuredStream = TlsStream<TcpStream>;

pub(crate) enum HttpStream {
    Unsecured(UnsecuredStream, Option<Instant>),
    #[cfg(any(feature = "rustls", feature = "openssl", feature = "native-tls"))]
    Secured(Box<SecuredStream>, Option<Instant>),
}

impl HttpStream {
    fn create_unsecured(reader: UnsecuredStream, timeout_at: Option<Instant>) -> HttpStream {
        HttpStream::Unsecured(reader, timeout_at)
    }

    #[cfg(any(feature = "rustls", feature = "openssl", feature = "native-tls"))]
    fn create_secured(reader: SecuredStream, timeout_at: Option<Instant>) -> HttpStream {
        HttpStream::Secured(Box::new(reader), timeout_at)
    }
}

fn timeout_err() -> io::Error {
    io::Error::new(
        io::ErrorKind::TimedOut,
        "the timeout of the request was reached",
    )
}

fn timeout_at_to_duration(timeout_at: Option<Instant>) -> Result<Option<Duration>, io::Error> {
    if let Some(timeout_at) = timeout_at {
        if let Some(duration) = timeout_at.checked_duration_since(Instant::now()) {
            Ok(Some(duration))
        } else {
            Err(timeout_err())
        }
    } else {
        Ok(None)
    }
}

impl Read for HttpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let timeout = |tcp: &TcpStream, timeout_at: Option<Instant>| -> io::Result<()> {
            let _ = tcp.set_read_timeout(timeout_at_to_duration(timeout_at)?);
            Ok(())
        };

        let result = match self {
            HttpStream::Unsecured(inner, timeout_at) => {
                timeout(inner, *timeout_at)?;
                inner.read(buf)
            }
            #[cfg(any(feature = "rustls", feature = "openssl", feature = "native-tls"))]
            HttpStream::Secured(inner, timeout_at) => {
                timeout(inner.get_ref(), *timeout_at)?;
                inner.read(buf)
            }
        };
        match result {
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                // We're a blocking socket, so EWOULDBLOCK indicates a timeout
                Err(timeout_err())
            }
            r => r,
        }
    }
}

/// A connection to the server for sending
/// [`Request`](struct.Request.html)s.
pub struct Connection {
    request: ParsedRequest,
    timeout_at: Option<Instant>,
}

impl Connection {
    /// Creates a new `Connection`. See [Request] and [ParsedRequest]
    /// for specifics about *what* is being sent.
    pub(crate) fn new(request: ParsedRequest) -> Connection {
        let timeout = request
            .config
            .timeout
            .or_else(|| match env::var("MINREQ_TIMEOUT") {
                Ok(t) => t.parse::<u64>().ok(),
                Err(_) => None,
            });
        let timeout_at = timeout.map(|t| Instant::now() + Duration::from_secs(t));
        Connection {
            request,
            timeout_at,
        }
    }

    /// Returns the timeout duration for operations that should end at
    /// timeout and are starting "now".
    ///
    /// The Result will be Err if the timeout has already passed.
    fn timeout(&self) -> Result<Option<Duration>, io::Error> {
        let timeout = timeout_at_to_duration(self.timeout_at);
        log::trace!("Timeout requested, it is currently: {:?}", timeout);
        timeout
    }

    /// Sends the [`Request`](struct.Request.html), consumes this
    /// connection, and returns a [`Response`](struct.Response.html).
    #[cfg(feature = "rustls")]
    pub(crate) fn send_https(mut self) -> Result<ResponseLazy, Error> {
        enforce_timeout(self.timeout_at, move || {
            self.request.url.host = ensure_ascii_host(self.request.url.host)?;
            let bytes = self.request.as_bytes();

            // Rustls setup
            log::trace!("Setting up TLS parameters for {}.", self.request.url.host);
            let dns_name = match ServerName::try_from(&*self.request.url.host) {
                Ok(result) => result,
                Err(err) => return Err(Error::IoError(io::Error::new(io::ErrorKind::Other, err))),
            };
            let sess = ClientConnection::new(CONFIG.clone(), dns_name)
                .map_err(Error::RustlsCreateConnection)?;

            log::trace!("Establishing TCP connection to {}.", self.request.url.host);
            let tcp = self.connect()?;

            // Send request
            log::trace!("Establishing TLS session to {}.", self.request.url.host);
            let mut tls = StreamOwned::new(sess, tcp); // I don't think this actually does any communication.
            log::trace!("Writing HTTPS request to {}.", self.request.url.host);
            let _ = tls.get_ref().set_write_timeout(self.timeout()?);
            tls.write_all(&bytes)?;

            // Receive request
            log::trace!("Reading HTTPS response from {}.", self.request.url.host);
            let response = ResponseLazy::from_stream(
                HttpStream::create_secured(tls, self.timeout_at),
                self.request.config.max_headers_size,
                self.request.config.max_status_line_len,
            )?;
            handle_redirects(self, response)
        })
    }

    /// Sends the [`Request`](struct.Request.html), consumes this
    /// connection, and returns a [`Response`](struct.Response.html).
    #[cfg(all(
        not(feature = "rustls"),
        any(feature = "openssl", feature = "native-tls")
    ))]
    pub(crate) fn send_https(mut self) -> Result<ResponseLazy, Error> {
        enforce_timeout(self.timeout_at, move || {
            self.request.url.host = ensure_ascii_host(self.request.url.host)?;
            let bytes = self.request.as_bytes();

            log::trace!("Setting up TLS parameters for {}.", self.request.url.host);
            let dns_name = &self.request.url.host;
            /*
            let mut builder = TlsConnector::builder();
            ...
            let sess = match builder.build() {
            */
            let sess = match TlsConnector::new() {
                Ok(sess) => sess,
                Err(err) => return Err(Error::IoError(io::Error::new(io::ErrorKind::Other, err))),
            };

            log::trace!("Establishing TCP connection to {}.", self.request.url.host);
            let tcp = self.connect()?;

            // Send request
            log::trace!("Establishing TLS session to {}.", self.request.url.host);
            let mut tls = match sess.connect(dns_name, tcp) {
                Ok(tls) => tls,
                Err(err) => return Err(Error::IoError(io::Error::new(io::ErrorKind::Other, err))),
            };
            log::trace!("Writing HTTPS request to {}.", self.request.url.host);
            let _ = tls.get_ref().set_write_timeout(self.timeout()?);
            tls.write_all(&bytes)?;

            // Receive request
            log::trace!("Reading HTTPS response from {}.", self.request.url.host);
            let response = ResponseLazy::from_stream(
                HttpStream::create_secured(tls, self.timeout_at),
                self.request.config.max_headers_size,
                self.request.config.max_status_line_len,
            )?;
            handle_redirects(self, response)
        })
    }

    /// Sends the [`Request`](struct.Request.html), consumes this
    /// connection, and returns a [`Response`](struct.Response.html).
    pub(crate) fn send(mut self) -> Result<ResponseLazy, Error> {
        enforce_timeout(self.timeout_at, move || {
            self.request.url.host = ensure_ascii_host(self.request.url.host)?;
            let bytes = self.request.as_bytes();

            log::trace!("Establishing TCP connection to {}.", self.request.url.host);
            let mut tcp = self.connect()?;

            // Send request
            log::trace!("Writing HTTP request.");
            let _ = tcp.set_write_timeout(self.timeout()?);
            tcp.write_all(&bytes)?;

            // Receive response
            log::trace!("Reading HTTP response.");
            let stream = HttpStream::create_unsecured(tcp, self.timeout_at);
            let response = ResponseLazy::from_stream(
                stream,
                self.request.config.max_headers_size,
                self.request.config.max_status_line_len,
            )?;
            handle_redirects(self, response)
        })
    }

    fn connect(&self) -> Result<TcpStream, Error> {
        let tcp_connect = |host: &str, port: u32| -> Result<TcpStream, Error> {
            let host = format!("{}:{}", host, port);
            let addrs = host.to_socket_addrs().map_err(Error::IoError)?;
            let addrs_count = addrs.len();

            // Try all resolved addresses. Return the first one to which we could connect. If all
            // failed return the last error encountered.
            for (i, addr) in addrs.enumerate() {
                let stream = if let Some(timeout) = self.timeout()? {
                    TcpStream::connect_timeout(&addr, timeout)
                } else {
                    TcpStream::connect(addr)
                };
                if stream.is_ok() || i == addrs_count - 1 {
                    return stream.map_err(Error::from);
                }
            }

            Err(Error::AddressNotFound)
        };

        #[cfg(feature = "proxy")]
        match self.request.config.proxy {
            Some(ref proxy) => {
                // do proxy things
                let mut tcp = tcp_connect(&proxy.server, proxy.port)?;

                write!(tcp, "{}", proxy.connect(&self.request)).unwrap();
                tcp.flush()?;

                let mut proxy_response = Vec::new();

                loop {
                    let mut buf = vec![0; 256];
                    let total = tcp.read(&mut buf)?;
                    proxy_response.append(&mut buf);
                    if total < 256 {
                        break;
                    }
                }

                crate::Proxy::verify_response(&proxy_response)?;

                Ok(tcp)
            }
            None => tcp_connect(&self.request.url.host, self.request.url.port.port()),
        }

        #[cfg(not(feature = "proxy"))]
        tcp_connect(&self.request.url.host, self.request.url.port.port())
    }
}

fn handle_redirects(
    connection: Connection,
    mut response: ResponseLazy,
) -> Result<ResponseLazy, Error> {
    let status_code = response.status_code;
    let url = response.headers.get("location");
    match get_redirect(connection, status_code, url) {
        NextHop::Redirect(connection) => {
            let connection = connection?;
            if connection.request.url.https {
                #[cfg(not(any(
                    feature = "rustls",
                    feature = "openssl",
                    feature = "native-tls"
                )))]
                return Err(Error::HttpsFeatureNotEnabled);
                #[cfg(any(feature = "rustls", feature = "openssl", feature = "native-tls"))]
                return connection.send_https();
            } else {
                connection.send()
            }
        }
        NextHop::Destination(connection) => {
            let dst_url = connection.request.url;
            dst_url.write_base_url_to(&mut response.url).unwrap();
            dst_url.write_resource_to(&mut response.url).unwrap();
            Ok(response)
        }
    }
}

enum NextHop {
    Redirect(Result<Connection, Error>),
    Destination(Connection),
}

fn get_redirect(mut connection: Connection, status_code: i32, url: Option<&String>) -> NextHop {
    match status_code {
        301 | 302 | 303 | 307 => {
            let url = match url {
                Some(url) => url,
                None => return NextHop::Redirect(Err(Error::RedirectLocationMissing)),
            };
            log::debug!("Redirecting ({}) to: {}", status_code, url);

            match connection.request.redirect_to(url.as_str()) {
                Ok(()) => {
                    if status_code == 303 {
                        match connection.request.config.method {
                            Method::Post | Method::Put | Method::Delete => {
                                connection.request.config.method = Method::Get;
                            }
                            _ => {}
                        }
                    }

                    NextHop::Redirect(Ok(connection))
                }
                Err(err) => NextHop::Redirect(Err(err)),
            }
        }
        _ => NextHop::Destination(connection),
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

/// Enforce the timeout by running the function in a new thread and
/// parking the current one with a timeout.
///
/// While minreq does use timeouts (somewhat) properly, some
/// interfaces such as [ToSocketAddrs] don't allow for specifying the
/// timeout. Hence this.
fn enforce_timeout<F, R>(timeout_at: Option<Instant>, f: F) -> Result<R, Error>
where
    F: 'static + Send + FnOnce() -> Result<R, Error>,
    R: 'static + Send,
{
    use std::sync::mpsc::{channel, RecvTimeoutError};

    match timeout_at {
        Some(deadline) => {
            let (sender, receiver) = channel();
            let thread = std::thread::spawn(move || {
                let result = f();
                let _ = sender.send(());
                result
            });
            if let Some(timeout_duration) = deadline.checked_duration_since(Instant::now()) {
                match receiver.recv_timeout(timeout_duration) {
                    Ok(()) => thread.join().unwrap(),
                    Err(err) => match err {
                        RecvTimeoutError::Timeout => Err(Error::IoError(timeout_err())),
                        RecvTimeoutError::Disconnected => {
                            Err(Error::Other("request connection paniced"))
                        }
                    },
                }
            } else {
                Err(Error::IoError(timeout_err()))
            }
        }
        None => f(),
    }
}
