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

/// A builder for client-side TLS connections.
#[derive(Clone)]
pub struct TlsConnector(imp::TlsConnector);

impl TlsConnector {
    /// Returns a new connector with default settings.
    pub fn new() -> Result<TlsConnector> {
        let connector = imp::TlsConnector::new()?;
        Ok(TlsConnector(connector))
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
    is_sync::<TlsConnector>();
    is_send::<TlsConnector>();
    is_sync::<TlsStream<TcpStream>>();
    is_send::<TlsStream<TcpStream>>();
}

mod imp {
    use openssl::error::ErrorStack;
    use openssl::ssl::{
        self, MidHandshakeSslStream, SslConnector, SslContextBuilder, SslMethod, SslOptions,
        SslStream,
    };
    use openssl::x509::{X509VerifyResult, X509};
    use std::fmt;
    use std::io;
    use std::{error, fs};

    fn load_android_root_certs(connector: &mut SslContextBuilder) -> Result<(), Error> {
        if let Ok(dir) = fs::read_dir("/system/etc/security/cacerts") {
            let certs = dir
                .filter_map(|r| r.ok())
                .filter_map(|e| fs::read(e.path()).ok())
                .filter_map(|b| X509::from_pem(&b).ok());
            for cert in certs {
                if let Err(err) = connector.cert_store_mut().add_cert(cert) {
                    log::debug!("load_android_root_certs error: {:?}", err);
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
    }

    impl TlsConnector {
        pub fn new() -> Result<TlsConnector, Error> {
            let mut connector = SslConnector::builder(SslMethod::tls())?;

            #[cfg(feature = "openssl-probe")]
            {
                let probe = openssl_probe::probe();
                connector
                    .load_verify_locations(probe.cert_file.as_deref(), probe.cert_dir.as_deref())?;
            }

            #[cfg(not(have_min_max_version))]
            connector.set_options(SslOptions::NO_SSLV2 | SslOptions::NO_SSLV3);
            #[cfg(have_min_max_version)]
            connector.set_min_proto_version(Some(openssl::ssl::SslVersion::TLS1))?;

            if cfg!(target_os = "android") {
                load_android_root_certs(&mut connector)?;
            }

            Ok(TlsConnector {
                connector: connector.build(),
            })
        }

        pub fn connect<S>(&self, domain: &str, stream: S) -> Result<TlsStream<S>, HandshakeError>
        where
            S: io::Read + io::Write,
        {
            let s = self
                .connector
                .configure()?
                .use_server_name_indication(true)
                .verify_hostname(true)
                .connect(domain, stream)?;
            Ok(TlsStream(s))
        }
    }

    pub struct TlsStream<S>(SslStream<S>);

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
