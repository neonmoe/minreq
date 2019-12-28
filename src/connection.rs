use crate::{Error, Method, Request};
#[cfg(not(target_arch = "wasm32"))]
use crate::ResponseLazy;
#[cfg(not(target_arch = "wasm32"))]
#[cfg(feature = "https")]
use rustls::{self, ClientConfig, ClientSession, StreamOwned};
use std::env;
#[cfg(not(target_arch = "wasm32"))]
use std::io::{self, BufReader, BufWriter, Read, Write};
#[cfg(not(target_arch = "wasm32"))]
use std::net::{TcpStream, ToSocketAddrs};
#[cfg(feature = "https")]
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;
#[cfg(feature = "https")]
#[cfg(not(target_arch = "wasm32"))]
use webpki::DNSNameRef;
#[cfg(feature = "https")]
#[cfg(not(target_arch = "wasm32"))]
use webpki_roots::TLS_SERVER_ROOTS;

#[cfg(target_arch = "wasm32")]
const HEADERS_LIST: [&str; 58] = [
    "Access-Control-Allow-Origin,",
    "Access-Control-Allow-Credentials,",
    "Access-Control-Expose-Headers,",
    "Access-Control-Max-Age,",
    "Access-Control-Allow-Methods,",
    "Access-Control-Allow-Headers",
    "Accept-Patch",
    "Accept-Ranges",
    "Age",
    "Allow",
    "Alt-Svc",
    "Cache-Control",
    "Connection",
    "Content-Disposition",
    "Content-Encoding",
    "Content-Language",
    "Content-Length",
    "Content-Location",
    "Content-MD5",
    "Content-Range",
    "Content-Type",
    "Date",
    "Delta-Base",
    "ETag",
    "Expires",
    "IM",
    "Last-Modified",
    "Link",
    "Location",
    "P3P",
    "Pragma",
    "Proxy-Authenticate",
    "Public-Key-Pins",
    "Retry-After",
    "Server",
    "Set-Cookie",
    "Strict-Transport-Security",
    "Trailer",
    "Transfer-Encoding",
    "Tk",
    "Upgrade",
    "Vary",
    "Via",
    "Warning",
    "WWW-Authenticate",
    "Content-Security-Policy,",
    "X-Content-Security-Policy,",
    "X-WebKit-CSP",
    "Refresh",
    "Status",
    "Timing-Allow-Origin",
    "X-Content-Duration",
    "X-Content-Type-Options",
    "X-Powered-By",
    "X-Request-ID,",
    "X-Correlation-ID",
    "X-UA-Compatible",
    "X-XSS-Protection"
];

#[cfg(feature = "https")]
#[cfg(not(target_arch = "wasm32"))]
lazy_static::lazy_static! {
    static ref CONFIG: Arc<ClientConfig> = {
        let mut config = ClientConfig::new();
        config
            .root_store
            .add_server_trust_anchors(&TLS_SERVER_ROOTS);
        Arc::new(config)
    };
}

#[cfg(not(target_arch = "wasm32"))]
type UnsecuredStream = BufReader<TcpStream>;
#[cfg(feature = "https")]
#[cfg(not(target_arch = "wasm32"))]
type SecuredStream = StreamOwned<ClientSession, TcpStream>;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) enum HttpStream {
    Unsecured(UnsecuredStream),
    #[cfg(feature = "https")]
    Secured(Box<SecuredStream>),
}

#[cfg(not(target_arch = "wasm32"))]
impl HttpStream {
    fn create_unsecured(reader: UnsecuredStream) -> HttpStream {
        HttpStream::Unsecured(reader)
    }

    #[cfg(feature = "https")]
    fn create_secured(reader: SecuredStream) -> HttpStream {
        HttpStream::Secured(Box::new(reader))
    }
}

#[cfg(not(target_arch = "wasm32"))]
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
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn send_https(mut self) -> Result<ResponseLazy, Error> {
        self.request.host = ensure_ascii_host(self.request.host)?;
        let bytes = self.request.as_bytes();

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
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn send(mut self) -> Result<ResponseLazy, Error> {
        self.request.host = ensure_ascii_host(self.request.host)?;
        let bytes = self.request.as_bytes();

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

    /// Sends the [`Request`](struct.Request.html), consumes this
    /// connection, and returns a [`Response`](struct.Response.html).
    #[cfg(target_arch = "wasm32")]
    pub(crate) async fn send(mut self) -> Result<crate::Response, Error> {
        use wasm_bindgen::prelude::*;

        #[wasm_bindgen]
        extern "C" {
            #[wasm_bindgen(js_namespace = console)]
            fn log(s: &str);
        }
        macro_rules! println {
            // Note that this is using the `log` function imported above during
            // `bare_bones`
            ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
        }

        use crate::Response;
        use web_sys::{RequestInit};
        use wasm_bindgen::{JsValue, JsCast};
        use wasm_bindgen_futures::JsFuture;
        use std::collections::HashMap;
        type WebSysRequest = web_sys::Request;
        type WebSysResponse = web_sys::Response;

        self.request.host = ensure_ascii_host(self.request.host)?;

        // set the method
        let mut opts = RequestInit::new();
        opts.method(match self.request.method {
            Method::Get => "GET",
            Method::Head => "HEAD",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Delete => "DELETE",
            Method::Connect => "CONNECT",
            Method::Options => "OPTIONS",
            Method::Trace => "TRACE",
            Method::Patch => "PATCH",
            Method::Custom(ref s) => s,
        });
        
        // set the body
        if let Some(body) = &self.request.body {
            opts.body(Some(&JsValue::from_str(&String::from_utf8_lossy(&body))));
        }

        // set the url
        let request: WebSysRequest = match WebSysRequest::new_with_str_and_init(
            &self.request.get_full_url(),
            &opts,
        ) {
            Ok(request) => request,
            Err(e) => {
                println!("{:?}", e);
                return Err(Error::Other("can't create request"))
            },
        };

        // set the headers
        for (header_name, header_value) in self.request.headers {
            if let Err(e) = request.headers().set(&header_name, &header_value) {
                println!("{:?}", e);
                return Err(Error::Other("can't set header"))
            }
        }
        
        // send the request
        let window = match web_sys::window() {
            Some(window) => window,
            None => return Err(Error::Other("program should be run in a browser (require a window)"))
        };
        let resp = match JsFuture::from(window.fetch_with_request(&request)).await {
            Ok(resp) => resp,
            Err(e) => {
                println!("{:?}", e);
                return Err(Error::Other("can't send request"))
            }
        };

        // get the response
        if !resp.is_instance_of::<WebSysResponse>() {
            return Err(Error::Other("can't convert response into a Response object"));
        }
        let resp: WebSysResponse = resp.dyn_into().unwrap();

        // get the body
        let mut bodyvec: Vec<u8> = Vec::new();
        let body = JsFuture::from(match resp.text() {
            Ok(body) => body,
            Err(e) => {
                println!("{:?}", e);
                return Err(Error::Other("can't read response body"))
            }
        });
        let body = match body.await {
            Ok(body) => body,
            Err(e) => {
                println!("{:?}", e);
                return Err(Error::Other("can't read response body (invalid utf8?)"))
            }
        };
        let body = match body.as_string() {
            Some(body) => body,
            None => return Err(Error::Other("can't read response body. body is not a string or is invalid utf8"))
        };
        for byte in body.as_bytes() {
            bodyvec.push(*byte);
        }

        // get headers
        let headers = resp.headers();
        let mut final_headers = HashMap::new();
        for header in HEADERS_LIST.iter() {
            if let Ok(Some(value)) = headers.get(header) {
                final_headers.insert(header.to_string(), value);
            }
        }

        Ok(Response::new(resp.status() as i32, resp.status_text(), final_headers, bodyvec))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn handle_redirects(connection: Connection, response: ResponseLazy) -> Result<ResponseLazy, Error> {
    let status_code = response.status_code;
    let url = response.headers.get("location");
    if let Some(request) = get_redirect(connection, status_code, url) {
        request?.send_lazy()
    } else {
        Ok(response)
    }
}

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(not(target_arch = "wasm32"))]
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
            Ok(result)
        }
    }
}
