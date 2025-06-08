//! A wrapper for the `openssl` crate's bindings to make the API similar to
//! rustls and native-tls as far as minreq is concerned.
//!
//! The entirety of this module was copied, pruned, and then pruned some more
//! from the OpenSSL related code in the `native-tls` crate. Thanks for your
//! work, native-tls developers! They have shared the code with the following
//! license:
//!
//! > Copyright (c) 2016 The rust-native-tls Developers
//! >
//! > Permission is hereby granted, free of charge, to any person obtaining a
//! > copy of this software and associated documentation files (the "Software"),
//! > to deal in the Software without restriction, including without limitation
//! > the rights to use, copy, modify, merge, publish, distribute, sublicense,
//! > and/or sell copies of the Software, and to permit persons to whom the
//! > Software is furnished to do so, subject to the following conditions:
//! >
//! > The above copyright notice and this permission notice shall be included in
//! > all copies or substantial portions of the Software.
//! >
//! > THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//! > IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//! > FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL
//! > THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//! > LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
//! > FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
//! > DEALINGS IN THE SOFTWARE.

use std::error;
use std::fmt;
use std::io;
use std::result;

/// A typedef of the result-type returned by many methods.
pub type Result<T> = result::Result<T, Error>;

/// An error returned from the TLS implementation.
#[derive(Debug)]
pub struct Error(imp::Error);

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        error::Error::source(&self.0)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, fmt)
    }
}

impl From<imp::Error> for Error {
    fn from(err: imp::Error) -> Error {
        Error(err)
    }
}

/// A cryptographic identity.
///
/// An identity is an X509 certificate along with its corresponding private key and chain of certificates to a trusted
/// root.
#[derive(Clone)]
pub struct Identity(imp::Identity);

/// An X509 certificate.
#[derive(Clone)]
pub struct Certificate(imp::Certificate);

/// An error returned from `ClientBuilder::handshake`.
#[derive(Debug)]
pub enum HandshakeError {
    /// A fatal error.
    Failure(Error),

    /// A stream interrupted midway through the handshake process due to a
    /// `WouldBlock` error.
    ///
    /// Note that this is not a fatal error and it should be safe to call
    /// `handshake` at a later time once the stream is ready to perform I/O
    /// again.
    WouldBlock,
}

impl error::Error for HandshakeError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            HandshakeError::Failure(ref e) => Some(e),
            HandshakeError::WouldBlock => None,
        }
    }
}

impl fmt::Display for HandshakeError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            HandshakeError::Failure(ref e) => fmt::Display::fmt(e, fmt),
            HandshakeError::WouldBlock => fmt.write_str("the handshake process was interrupted"),
        }
    }
}

impl From<imp::HandshakeError> for HandshakeError {
    fn from(e: imp::HandshakeError) -> HandshakeError {
        match e {
            imp::HandshakeError::Failure(e) => HandshakeError::Failure(Error(e)),
            imp::HandshakeError::WouldBlock => HandshakeError::WouldBlock,
        }
    }
}

/// SSL/TLS protocol versions.
#[derive(Debug, Copy, Clone)]
pub enum Protocol {
    /// The SSL 3.0 protocol.
    ///
    /// # Warning
    ///
    /// SSL 3.0 has severe security flaws, and should not be used unless absolutely necessary. If
    /// you are not sure if you need to enable this protocol, you should not.
    Sslv3,
    /// The TLS 1.0 protocol.
    Tlsv10,
    /// The TLS 1.1 protocol.
    Tlsv11,
    /// The TLS 1.2 protocol.
    Tlsv12,
    #[doc(hidden)]
    __NonExhaustive,
}

/// A builder for `TlsConnector`s.
pub struct TlsConnectorBuilder {
    identity: Option<Identity>,
    min_protocol: Option<Protocol>,
    max_protocol: Option<Protocol>,
    root_certificates: Vec<Certificate>,
    accept_invalid_certs: bool,
    accept_invalid_hostnames: bool,
    use_sni: bool,
    disable_built_in_roots: bool,
}

impl TlsConnectorBuilder {
    /// Creates a new `TlsConnector`.
    pub fn build(&self) -> Result<TlsConnector> {
        let connector = imp::TlsConnector::new(self)?;
        Ok(TlsConnector(connector))
    }
}

/// A builder for client-side TLS connections.
#[derive(Clone, Debug)]
pub struct TlsConnector(imp::TlsConnector);

impl TlsConnector {
    /// Returns a new connector with default settings.
    pub fn new() -> Result<TlsConnector> {
        TlsConnector::builder().build()
    }

    /// Returns a new builder for a `TlsConnector`.
    pub fn builder() -> TlsConnectorBuilder {
        TlsConnectorBuilder {
            identity: None,
            min_protocol: Some(Protocol::Tlsv10),
            max_protocol: None,
            root_certificates: vec![],
            use_sni: true,
            accept_invalid_certs: false,
            accept_invalid_hostnames: false,
            disable_built_in_roots: false,
        }
    }

    /// Initiates a TLS handshake.
    ///
    /// The provided domain will be used for both SNI and certificate hostname
    /// validation.
    ///
    /// If the socket is nonblocking and a `WouldBlock` error is returned during
    /// the handshake, a `HandshakeError::WouldBlock` error will be returned
    /// which can be used to restart the handshake when the socket is ready
    /// again.
    ///
    /// The domain is ignored if both SNI and hostname verification are
    /// disabled.
    pub fn connect<S>(
        &self,
        domain: &str,
        stream: S,
    ) -> result::Result<TlsStream<S>, HandshakeError>
    where
        S: io::Read + io::Write,
    {
        let s = self.0.connect(domain, stream)?;
        Ok(TlsStream(s))
    }
}

/// A stream managing a TLS session.
pub struct TlsStream<S>(imp::TlsStream<S>);

impl<S: fmt::Debug> fmt::Debug for TlsStream<S> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.0, fmt)
    }
}

impl<S> TlsStream<S> {
    /// Returns a shared reference to the inner stream.
    pub fn get_ref(&self) -> &S {
        self.0.get_ref()
    }

    /// Returns a mutable reference to the inner stream.
    #[allow(dead_code)]
    pub fn get_mut(&mut self) -> &mut S {
        self.0.get_mut()
    }
}

impl<S: io::Read + io::Write> io::Read for TlsStream<S> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl<S: io::Read + io::Write> io::Write for TlsStream<S> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

fn _check_kinds() {
    use std::net::TcpStream;

    fn is_sync<T: Sync>() {}
    fn is_send<T: Send>() {}
    is_sync::<Error>();
    is_send::<Error>();
    is_sync::<TlsConnectorBuilder>();
    is_send::<TlsConnectorBuilder>();
    is_sync::<TlsConnector>();
    is_send::<TlsConnector>();
    is_sync::<TlsStream<TcpStream>>();
    is_send::<TlsStream<TcpStream>>();
}

mod imp {
    use openssl::error::ErrorStack;
    use openssl::pkey::PKey;
    use openssl::ssl::{
        self, MidHandshakeSslStream, SslConnector, SslContextBuilder, SslMethod, SslVerifyMode,
    };
    use openssl::x509::{store::X509StoreBuilder, X509VerifyResult, X509};
    use std::error;
    use std::fmt;
    use std::io;

    use super::{Protocol, TlsConnectorBuilder};
    use openssl::pkey::Private;

    #[cfg(have_min_max_version)]
    fn supported_protocols(
        min: Option<Protocol>,
        max: Option<Protocol>,
        ctx: &mut SslContextBuilder,
    ) -> Result<(), ErrorStack> {
        use openssl::ssl::SslVersion;
        fn cvt(p: Protocol) -> SslVersion {
            match p {
                Protocol::Sslv3 => SslVersion::SSL3,
                Protocol::Tlsv10 => SslVersion::TLS1,
                Protocol::Tlsv11 => SslVersion::TLS1_1,
                Protocol::Tlsv12 => SslVersion::TLS1_2,
                Protocol::__NonExhaustive => unreachable!(),
            }
        }

        ctx.set_min_proto_version(min.map(cvt))?;
        ctx.set_max_proto_version(max.map(cvt))?;

        Ok(())
    }

    #[cfg(not(have_min_max_version))]
    fn supported_protocols(
        min: Option<Protocol>,
        max: Option<Protocol>,
        ctx: &mut SslContextBuilder,
    ) -> Result<(), ErrorStack> {
        use openssl::ssl::SslOptions;

        let no_ssl_mask = SslOptions::NO_SSLV2
            | SslOptions::NO_SSLV3
            | SslOptions::NO_TLSV1
            | SslOptions::NO_TLSV1_1
            | SslOptions::NO_TLSV1_2;

        ctx.clear_options(no_ssl_mask);
        let mut options = SslOptions::empty();
        options |= match min {
            None => SslOptions::empty(),
            Some(Protocol::Sslv3) => SslOptions::NO_SSLV2,
            Some(Protocol::Tlsv10) => SslOptions::NO_SSLV2 | SslOptions::NO_SSLV3,
            Some(Protocol::Tlsv11) => {
                SslOptions::NO_SSLV2 | SslOptions::NO_SSLV3 | SslOptions::NO_TLSV1
            }
            Some(Protocol::Tlsv12) => {
                SslOptions::NO_SSLV2
                    | SslOptions::NO_SSLV3
                    | SslOptions::NO_TLSV1
                    | SslOptions::NO_TLSV1_1
            }
            Some(Protocol::__NonExhaustive) => unreachable!(),
        };
        options |= match max {
            None | Some(Protocol::Tlsv12) => SslOptions::empty(),
            Some(Protocol::Tlsv11) => SslOptions::NO_TLSV1_2,
            Some(Protocol::Tlsv10) => SslOptions::NO_TLSV1_1 | SslOptions::NO_TLSV1_2,
            Some(Protocol::Sslv3) => {
                SslOptions::NO_TLSV1 | SslOptions::NO_TLSV1_1 | SslOptions::NO_TLSV1_2
            }
            Some(Protocol::__NonExhaustive) => unreachable!(),
        };

        ctx.set_options(options);

        Ok(())
    }

    #[cfg(target_os = "android")]
    fn load_android_root_certs(connector: &mut SslContextBuilder) -> Result<(), Error> {
        use std::fs;

        if let Ok(dir) = fs::read_dir("/system/etc/security/cacerts") {
            let certs = dir
                .filter_map(|r| r.ok())
                .filter_map(|e| fs::read(e.path()).ok())
                .filter_map(|b| X509::from_pem(&b).ok());
            for cert in certs {
                if let Err(err) = connector.cert_store_mut().add_cert(cert) {
                    debug!("load_android_root_certs error: {:?}", err);
                }
            }
        }

        Ok(())
    }

    #[derive(Debug)]
    pub enum Error {
        Normal(ErrorStack),
        Ssl(ssl::Error, X509VerifyResult),
    }

    impl error::Error for Error {
        fn source(&self) -> Option<&(dyn error::Error + 'static)> {
            match *self {
                Error::Normal(ref e) => error::Error::source(e),
                Error::Ssl(ref e, _) => error::Error::source(e),
            }
        }
    }

    impl fmt::Display for Error {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                Error::Normal(ref e) => fmt::Display::fmt(e, fmt),
                Error::Ssl(ref e, X509VerifyResult::OK) => fmt::Display::fmt(e, fmt),
                Error::Ssl(ref e, v) => write!(fmt, "{} ({})", e, v),
            }
        }
    }

    impl From<ErrorStack> for Error {
        fn from(err: ErrorStack) -> Error {
            Error::Normal(err)
        }
    }

    #[derive(Clone)]
    pub struct Identity {
        pkey: PKey<Private>,
        cert: X509,
        chain: Vec<X509>,
    }

    #[derive(Clone)]
    pub struct Certificate(X509);

    pub struct MidHandshakeTlsStream<S>(MidHandshakeSslStream<S>);

    impl<S> fmt::Debug for MidHandshakeTlsStream<S>
    where
        S: fmt::Debug,
    {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            fmt::Debug::fmt(&self.0, fmt)
        }
    }

    pub enum HandshakeError {
        Failure(Error),
        WouldBlock,
    }

    impl<S> From<ssl::HandshakeError<S>> for HandshakeError {
        fn from(e: ssl::HandshakeError<S>) -> HandshakeError {
            match e {
                ssl::HandshakeError::SetupFailure(e) => HandshakeError::Failure(e.into()),
                ssl::HandshakeError::Failure(e) => {
                    let v = e.ssl().verify_result();
                    HandshakeError::Failure(Error::Ssl(e.into_error(), v))
                }
                ssl::HandshakeError::WouldBlock(_) => HandshakeError::WouldBlock,
            }
        }
    }

    impl From<ErrorStack> for HandshakeError {
        fn from(e: ErrorStack) -> HandshakeError {
            HandshakeError::Failure(e.into())
        }
    }

    #[derive(Clone)]
    pub struct TlsConnector {
        connector: SslConnector,
        use_sni: bool,
        accept_invalid_hostnames: bool,
        accept_invalid_certs: bool,
    }

    impl TlsConnector {
        pub fn new(builder: &TlsConnectorBuilder) -> Result<TlsConnector, Error> {
            let mut connector = SslConnector::builder(SslMethod::tls())?;

            #[cfg(feature = "openssl-probe")]
            {
                let probe = openssl_probe::probe();
                connector
                    .load_verify_locations(probe.cert_file.as_deref(), probe.cert_dir.as_deref())?;
            }

            if let Some(ref identity) = builder.identity {
                connector.set_certificate(&identity.0.cert)?;
                connector.set_private_key(&identity.0.pkey)?;
                for cert in identity.0.chain.iter().rev() {
                    connector.add_extra_chain_cert(cert.to_owned())?;
                }
            }
            supported_protocols(builder.min_protocol, builder.max_protocol, &mut connector)?;

            if builder.disable_built_in_roots {
                connector.set_cert_store(X509StoreBuilder::new()?.build());
            }

            for cert in &builder.root_certificates {
                if let Err(err) = connector.cert_store_mut().add_cert((cert.0).0.clone()) {
                    log::debug!("add_cert error: {:?}", err);
                }
            }

            #[cfg(target_os = "android")]
            load_android_root_certs(&mut connector)?;

            Ok(TlsConnector {
                connector: connector.build(),
                use_sni: builder.use_sni,
                accept_invalid_hostnames: builder.accept_invalid_hostnames,
                accept_invalid_certs: builder.accept_invalid_certs,
            })
        }

        pub fn connect<S>(&self, domain: &str, stream: S) -> Result<TlsStream<S>, HandshakeError>
        where
            S: io::Read + io::Write,
        {
            let mut ssl = self
                .connector
                .configure()?
                .use_server_name_indication(self.use_sni)
                .verify_hostname(!self.accept_invalid_hostnames);
            if self.accept_invalid_certs {
                ssl.set_verify(SslVerifyMode::NONE);
            }

            let s = ssl.connect(domain, stream)?;
            Ok(TlsStream(s))
        }
    }

    impl fmt::Debug for TlsConnector {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            fmt.debug_struct("TlsConnector")
                // n.b. SslConnector is a newtype on SslContext which implements a noop Debug so it's omitted
                .field("use_sni", &self.use_sni)
                .field("accept_invalid_hostnames", &self.accept_invalid_hostnames)
                .field("accept_invalid_certs", &self.accept_invalid_certs)
                .finish()
        }
    }

    pub struct TlsStream<S>(ssl::SslStream<S>);

    impl<S: fmt::Debug> fmt::Debug for TlsStream<S> {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            fmt::Debug::fmt(&self.0, fmt)
        }
    }

    impl<S> TlsStream<S> {
        pub fn get_ref(&self) -> &S {
            self.0.get_ref()
        }

        pub fn get_mut(&mut self) -> &mut S {
            self.0.get_mut()
        }
    }

    impl<S: io::Read + io::Write> io::Read for TlsStream<S> {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            self.0.read(buf)
        }
    }

    impl<S: io::Read + io::Write> io::Write for TlsStream<S> {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            self.0.flush()
        }
    }
}
