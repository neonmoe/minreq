use crate::error::Error;
use crate::ParsedRequest;

/// Kind of proxy connection (Basic, Digest, etc)
#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) enum ProxyKind {
    Basic,
}

/// Proxy configuration. Only HTTP CONNECT proxies are supported (no SOCKS or
/// HTTPS).
///
/// When credentials are provided, the Basic authentication type is used for
/// Proxy-Authorization.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Proxy {
    pub(crate) server: String,
    pub(crate) port: u32,
    pub(crate) user: Option<String>,
    pub(crate) password: Option<String>,
    pub(crate) kind: ProxyKind,
}

impl Proxy {
    fn parse_creds(creds: &str) -> (Option<String>, Option<String>) {
        if let Some((user, pass)) = creds.split_once(':') {
            (Some(user.to_string()), Some(pass.to_string()))
        } else {
            (Some(creds.to_string()), None)
        }
    }

    fn parse_address(host: &str) -> Result<(String, Option<u32>), Error> {
        if let Some((host, port)) = host.split_once(':') {
            let port = port.parse::<u32>().map_err(|_| Error::BadProxy)?;
            Ok((host.to_string(), Some(port)))
        } else {
            Ok((host.to_string(), None))
        }
    }

    /// Creates a new Proxy configuration.
    ///
    /// Supported proxy format is:
    ///
    /// ```plaintext
    /// [http://][user[:password]@]host[:port]
    /// ```
    ///
    /// The default port is 8080, to be changed to 1080 in minreq 3.0.
    ///
    /// # Example
    ///
    /// ```
    /// let proxy = minreq::Proxy::new("user:password@localhost:1080").unwrap();
    /// let request = minreq::post("http://example.com").with_proxy(proxy);
    /// ```
    ///
    pub fn new<S: AsRef<str>>(proxy: S) -> Result<Self, Error> {
        let proxy = proxy.as_ref();
        let authority = if let Some((proto, auth)) = proxy.split_once("://") {
            if proto != "http" {
                return Err(Error::BadProxy);
            }
            auth
        } else {
            proxy
        };

        let ((user, password), host) = if let Some((userinfo, host)) = authority.rsplit_once('@') {
            (Proxy::parse_creds(userinfo), host)
        } else {
            ((None, None), authority)
        };

        let (host, port) = Proxy::parse_address(host)?;

        Ok(Self {
            server: host,
            user,
            password,
            port: port.unwrap_or(8080),
            kind: ProxyKind::Basic,
        })
    }

    pub(crate) fn connect(&self, proxied_req: &ParsedRequest) -> String {
        let authorization = if let Some(user) = &self.user {
            match self.kind {
                ProxyKind::Basic => {
                    let creds = if let Some(password) = &self.password {
                        base64::encode(format!("{user}:{password}"))
                    } else {
                        base64::encode(user)
                    };
                    format!("Proxy-Authorization: Basic {}\r\n", creds)
                }
            }
        } else {
            String::new()
        };
        let host = &proxied_req.host;
        let port = proxied_req.port.port();
        format!("CONNECT {host}:{port} HTTP/1.1\r\n{authorization}\r\n")
    }

    pub(crate) fn verify_response(response: &[u8]) -> Result<(), Error> {
        let response_string = String::from_utf8_lossy(response);
        let top_line = response_string.lines().next().ok_or(Error::ProxyConnect)?;
        let status_code = top_line.split_whitespace().nth(1).ok_or(Error::BadProxy)?;

        match status_code {
            "200" => Ok(()),
            "401" | "407" => Err(Error::InvalidProxyCreds),
            _ => Err(Error::BadProxy),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Proxy;

    #[test]
    fn parse_proxy() {
        let proxy = Proxy::new("user:p@ssw0rd@localhost:9999").unwrap();
        assert_eq!(proxy.user, Some(String::from("user")));
        assert_eq!(proxy.password, Some(String::from("p@ssw0rd")));
        assert_eq!(proxy.server, String::from("localhost"));
        assert_eq!(proxy.port, 9999);
    }

    #[test]
    fn parse_regular_proxy_with_protocol() {
        let proxy = Proxy::new("http://localhost:1080").unwrap();
        assert_eq!(proxy.user, None);
        assert_eq!(proxy.password, None);
        assert_eq!(proxy.server, String::from("localhost"));
        assert_eq!(proxy.port, 1080);
    }
}
