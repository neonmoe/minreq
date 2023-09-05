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
        if let Some((user, pass)) = split_once(creds, ":") {
            (Some(user.to_string()), Some(pass.to_string()))
        } else {
            (Some(creds.to_string()), None)
        }
    }

    fn parse_address(host: &str) -> Result<(String, Option<u32>), Error> {
        if let Some((host, port)) = split_once(host, ":") {
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
        let authority = if let Some((proto, auth)) = split_once(proxy, "://") {
            if proto != "http" {
                return Err(Error::BadProxy);
            }
            auth
        } else {
            proxy
        };

        let ((user, password), host) = if let Some((userinfo, host)) = rsplit_once(authority, "@") {
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
                        base64::encode(format!("{}:{}", user, password))
                    } else {
                        base64::encode(user)
                    };
                    format!("Proxy-Authorization: Basic {}\r\n", creds)
                }
            }
        } else {
            String::new()
        };
        let host = &proxied_req.url.host;
        let port = proxied_req.url.port.port();
        format!(
            "CONNECT {}:{} HTTP/1.1\r\n{}\r\n",
            host, port, authorization
        )
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

#[allow(clippy::manual_split_once)]
/// Replacement for str::split_once until MSRV is at least 1.52.0.
fn split_once<'a>(string: &'a str, pattern: &str) -> Option<(&'a str, &'a str)> {
    let mut parts = string.splitn(2, pattern);
    let first = parts.next()?;
    let second = parts.next()?;
    Some((first, second))
}

#[allow(clippy::manual_split_once)]
/// Replacement for str::rsplit_once until MSRV is at least 1.52.0.
fn rsplit_once<'a>(string: &'a str, pattern: &str) -> Option<(&'a str, &'a str)> {
    let mut parts = string.rsplitn(2, pattern);
    let second = parts.next()?;
    let first = parts.next()?;
    Some((first, second))
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
