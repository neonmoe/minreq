mod native_tls;

use native_tls::{TlsConnector, TlsStream};
use std::io::{self, Write};
use std::net::TcpStream;

use crate::Error;

use super::{Connection, HttpStream};

pub type SecuredStream = TlsStream<TcpStream>;

pub fn create_secured_stream(conn: &Connection) -> Result<HttpStream, Error> {
    // native-tls setup
    log::trace!("Setting up TLS parameters for {}.", conn.request.url.host);
    let dns_name = &conn.request.url.host;
    let sess = match TlsConnector::new() {
        Ok(sess) => sess,
        Err(err) => return Err(Error::IoError(io::Error::new(io::ErrorKind::Other, err))),
    };

    // Connect
    log::trace!("Establishing TCP connection to {}.", conn.request.url.host);
    let tcp = conn.connect()?;

    // Send request
    log::trace!("Establishing TLS session to {}.", conn.request.url.host);
    let mut tls = match sess.connect(dns_name, tcp) {
        Ok(tls) => tls,
        Err(err) => return Err(Error::IoError(io::Error::new(io::ErrorKind::Other, err))),
    };
    log::trace!("Writing HTTPS request to {}.", conn.request.url.host);
    let _ = tls.get_ref().set_write_timeout(conn.timeout()?);
    tls.write_all(&conn.request.as_bytes())?;

    Ok(HttpStream::create_secured(tls, conn.timeout_at))
}
