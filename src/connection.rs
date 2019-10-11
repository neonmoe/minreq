use crate::{Error, Method, Request, ResponseLazy};
#[cfg(feature = "https")]
use rustls::{self, ClientConfig, ClientSession, StreamOwned};
use std::env;
use std::io::{self, BufReader, BufWriter, Read, Write};
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

type UnsecuredStream = BufReader<TcpStream>;
#[cfg(feature = "https")]
type SecuredStream = StreamOwned<ClientSession, TcpStream>;

pub(crate) enum HttpStream {
    Unsecured(UnsecuredStream),
    #[cfg(feature = "https")]
    Secured(Box<SecuredStream>),
}

impl HttpStream {
    fn create_unsecured(reader: UnsecuredStream) -> HttpStream {
        HttpStream::Unsecured(reader)
    }

    #[cfg(feature = "https")]
    fn create_secured(reader: SecuredStream) -> HttpStream {
        HttpStream::Secured(Box::new(reader))
    }
}

impl Read for HttpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            HttpStream::Unsecured(inner) => inner.read(buf),
            #[cfg(feature = "https")]
            HttpStream::Secured(inner) => inner.read(buf),
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
        let bytes = self.request.to_string().into_bytes();

        // Rustls setup
        let dns_name = &self.request.host;
        let dns_name = dns_name.split(':').next().unwrap();
        let dns_name = DNSNameRef::try_from_ascii_str(dns_name).unwrap();
        let sess = ClientSession::new(&CONFIG, dns_name);

        let tcp = match create_tcp_stream(&self.request.host, self.timeout) {
            Ok(tcp) => tcp,
            Err(err) => return Err(Error::IoError(err)),
        };

        // Send request
        let mut tls = StreamOwned::new(sess, tcp);
        if let Err(err) = tls.write(&bytes) {
            return Err(Error::IoError(err));
        }

        // Receive request
        let response = ResponseLazy::from_stream(HttpStream::create_secured(tls))?;
        handle_redirects(self, response)
    }

    /// Sends the [`Request`](struct.Request.html), consumes this
    /// connection, and returns a [`Response`](struct.Response.html).
    pub(crate) fn send(mut self) -> Result<ResponseLazy, Error> {
        self.request.host = ensure_ascii_host(self.request.host)?;
        let bytes = self.request.to_string().into_bytes();

        let tcp = match create_tcp_stream(&self.request.host, self.timeout) {
            Ok(tcp) => tcp,
            Err(err) => return Err(Error::IoError(err)),
        };

        // Send request
        let mut stream = BufWriter::new(tcp);
        if let Err(err) = stream.write_all(&bytes) {
            return Err(Error::IoError(err));
        }

        // Receive response
        let tcp = match stream.into_inner() {
            Ok(tcp) => tcp,
            Err(_) => {
                return Err(Error::Other(
                    "IntoInnerError after writing the request into the TcpStream.",
                ));
            }
        };
        let stream = HttpStream::create_unsecured(BufReader::new(tcp));
        let response = ResponseLazy::from_stream(stream)?;
        handle_redirects(self, response)
    }
}

fn handle_redirects(connection: Connection, response: ResponseLazy) -> Result<ResponseLazy, Error> {
    let status_code = response.status_code;
    let url = response.headers.get("Location");
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
            println!("{}", result);
            Ok(result)
        }
    }
}
