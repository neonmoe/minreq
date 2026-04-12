//! TLS connection handling functionality when using the `rustls` crate for
//! handling TLS.

use rustls::pki_types::ServerName;
#[cfg(all(feature = "rustls-webpki", not(feature = "rustls-platform-verifier")))]
use rustls::RootCertStore;
use rustls::{self, ClientConfig, ClientConnection, StreamOwned};
#[cfg(feature = "rustls-platform-verifier")]
use rustls_platform_verifier::BuilderVerifierExt;
use std::convert::TryFrom;
use std::io::{self, Write};
use std::net::TcpStream;
use std::sync::{Arc, LazyLock};
#[cfg(all(feature = "rustls-webpki", not(feature = "rustls-platform-verifier")))]
use webpki_roots::TLS_SERVER_ROOTS;

use crate::Error;

use super::{Connection, HttpStream};

pub type SecuredStream = StreamOwned<ClientConnection, TcpStream>;

// This code requires the `rustls` feature to be enabled, so minreq's MSRV
// policy doesn't apply here. LazyLock was added in 1.80, which should be old
// enough for rustls users, given that it's old enough for rustls's
// at-least-9-rust-versions-old MSRV policy.
#[allow(clippy::incompatible_msrv)]
static CONFIG: LazyLock<Result<Arc<ClientConfig>, rustls::Error>> = LazyLock::new(|| {
    let config = ClientConfig::builder();

    #[cfg(all(feature = "rustls-webpki", not(feature = "rustls-platform-verifier")))]
    let config = config.with_root_certificates(RootCertStore {
        roots: TLS_SERVER_ROOTS.to_vec(),
    });

    #[cfg(feature = "rustls-platform-verifier")]
    let config = config.with_platform_verifier()?;

    let config = config.with_no_client_auth();
    Ok(Arc::new(config))
});

pub fn create_secured_stream(conn: &Connection) -> Result<HttpStream, Error> {
    // Rustls setup
    #[cfg(feature = "log")]
    log::trace!("Setting up TLS parameters for {}.", conn.request.url.host);
    let dns_name = match ServerName::try_from(conn.request.url.host.clone()) {
        Ok(result) => result,
        Err(err) => return Err(Error::IoError(io::Error::new(io::ErrorKind::Other, err))),
    };
    let config = CONFIG.clone().map_err(Error::RustlsCreateConnection)?;
    let sess = ClientConnection::new(config, dns_name).map_err(Error::RustlsCreateConnection)?;

    // Connect
    #[cfg(feature = "log")]
    log::trace!("Establishing TCP connection to {}.", conn.request.url.host);
    let tcp = conn.connect()?;

    // Send request
    #[cfg(feature = "log")]
    log::trace!("Establishing TLS session to {}.", conn.request.url.host);
    let mut tls = StreamOwned::new(sess, tcp); // I don't think this actually does any communication.
    #[cfg(feature = "log")]
    log::trace!("Writing HTTPS request to {}.", conn.request.url.host);
    let _ = tls.get_ref().set_write_timeout(conn.timeout()?);
    tls.write_all(&conn.request.as_bytes())?;

    Ok(HttpStream::create_secured(tls, conn.timeout_at))
}
