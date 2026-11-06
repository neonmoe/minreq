use std::fmt::{self, Write};

use crate::Error;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Port {
    ImplicitHttp,
    ImplicitHttps,
    Explicit(u16),
}

impl Port {
    pub(crate) fn port(self) -> u16 {
        match self {
            Port::ImplicitHttp => 80,
            Port::ImplicitHttps => 443,
            Port::Explicit(port) => port,
        }
    }
}

/// URL split into its parts. See [RFC 3986 section
/// 3](https://datatracker.ietf.org/doc/html/rfc3986#section-3). Note that the
/// userinfo component is not allowed since [RFC
/// 7230](https://datatracker.ietf.org/doc/html/rfc7230#section-2.7.1).
///
/// ```text
/// scheme "://" host [ ":" port ] path [ "?" query ] [ "#" fragment ]
/// ```
#[derive(Clone, PartialEq)]
pub(crate) struct HttpUrl {
    /// If scheme is "https", true, if "http", false.
    pub(crate) https: bool,
    /// `host`
    pub(crate) host: String,
    /// `[":" port]`
    pub(crate) port: Port,
    /// `path ["?" query]` including the `?`.
    pub(crate) path_and_query: String,
    /// `["#" fragment]` without the `#`.
    pub(crate) fragment: Option<String>,
}

impl HttpUrl {
    pub(crate) fn parse(url: &str, redirected_from: Option<&HttpUrl>) -> Result<HttpUrl, Error> {
        enum UrlParseStatus {
            Host,
            Port,
            PathAndQuery,
            Fragment,
        }

        let (url, https) = if let Some(after_protocol) = url.strip_prefix("http://") {
            (after_protocol, false)
        } else if let Some(after_protocol) = url.strip_prefix("https://") {
            (after_protocol, true)
        } else {
            return Err(Error::InvalidProtocol);
        };

        let mut host = String::new();
        let mut port = String::new();
        let mut resource = String::new(); // At first this is the path and query, after # this becomes fragment.
        let mut path_and_query = None;
        let mut status = UrlParseStatus::Host;
        for c in url.chars() {
            match status {
                UrlParseStatus::Host => {
                    match c {
                        '/' | '?' => {
                            // Tolerate typos like: www.example.com?some=params
                            status = UrlParseStatus::PathAndQuery;
                            resource.push(c);
                        }
                        ':' => status = UrlParseStatus::Port,
                        '#' => {
                            status = UrlParseStatus::Fragment;
                            path_and_query = Some(String::new());
                        }
                        _ => host.push(c),
                    }
                }
                UrlParseStatus::Port => match c {
                    '#' => {
                        status = UrlParseStatus::Fragment;
                        path_and_query = Some(String::new());
                    }
                    '/' | '?' => {
                        status = UrlParseStatus::PathAndQuery;
                        resource.push(c);
                    }
                    _ => port.push(c),
                },
                UrlParseStatus::PathAndQuery if c == '#' => {
                    status = UrlParseStatus::Fragment;
                    path_and_query = Some(resource);
                    resource = String::new();
                }
                #[cfg(not(feature = "urlencoding"))]
                UrlParseStatus::PathAndQuery | UrlParseStatus::Fragment => resource.push(c),
                #[cfg(feature = "urlencoding")]
                UrlParseStatus::PathAndQuery | UrlParseStatus::Fragment => match c {
                    // All URL-'safe' characters, plus URL 'special
                    // characters' like &, #, =, / ,?
                    '0'..='9'
                    | 'A'..='Z'
                    | 'a'..='z'
                    | '-'
                    | '.'
                    | '_'
                    | '~'
                    | '&'
                    | '#'
                    | '='
                    | '/'
                    | '?' => {
                        resource.push(c);
                    }
                    // There is probably a simpler way to do this, but this
                    // method avoids any heap allocations (except extending
                    // `resource`)
                    _ => {
                        // Any UTF-8 character can fit in 4 bytes
                        let mut utf8_buf = [0u8; 4];
                        // Bytes fill buffer from the front
                        c.encode_utf8(&mut utf8_buf);
                        // Slice disregards the unused portion of the buffer
                        utf8_buf[..c.len_utf8()].iter().for_each(|byte| {
                            // Convert byte to URL escape, e.g. %21 for b'!'
                            let rem = *byte % 16;
                            let right_char = to_hex_digit(rem);
                            let left_char = to_hex_digit((*byte - rem) >> 4);
                            resource.push('%');
                            resource.push(left_char);
                            resource.push(right_char);
                        });
                    }
                },
            }
        }
        let (mut path_and_query, mut fragment) = if let Some(path_and_query) = path_and_query {
            (path_and_query, Some(resource))
        } else {
            (resource, None)
        };

        // If a redirected resource does not have a fragment, but the original
        // URL did, the fragment should be preserved over redirections. See RFC
        // 7231 section 7.1.2.
        if fragment.is_none() {
            if let Some(old_fragment) = redirected_from.and_then(|url| url.fragment.clone()) {
                fragment = Some(old_fragment);
            }
        }

        // Ensure the resource is *something*
        if path_and_query.is_empty() {
            path_and_query.push('/');
        }

        // Set appropriate port
        let port = if port.is_empty() {
            if https {
                Port::ImplicitHttps
            } else {
                Port::ImplicitHttp
            }
        } else {
            let invalid_port = || {
                Error::IoError(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "invalid URL port",
                ))
            };
            if !port.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(invalid_port());
            }
            Port::Explicit(port.parse::<u16>().map_err(|_| invalid_port())?)
        };

        Ok(HttpUrl {
            https,
            host,
            port,
            path_and_query,
            fragment,
        })
    }

    /// Writes the `scheme "://" host [ ":" port ]` part to the destination.
    pub(crate) fn write_base_url_to<W: Write>(&self, dst: &mut W) -> fmt::Result {
        write!(
            dst,
            "http{s}://{host}",
            s = if self.https { "s" } else { "" },
            host = self.host,
        )?;
        if let Port::Explicit(port) = self.port {
            write!(dst, ":{}", port)?;
        }
        Ok(())
    }

    /// Writes the `path [ "?" query ] [ "#" fragment ]` part to the destination.
    pub(crate) fn write_resource_to<W: Write>(&self, dst: &mut W) -> fmt::Result {
        write!(
            dst,
            "{path_and_query}{maybe_hash}{maybe_fragment}",
            path_and_query = self.path_and_query,
            maybe_hash = if self.fragment.is_some() { "#" } else { "" },
            maybe_fragment = self.fragment.as_deref().unwrap_or(""),
        )
    }
}

// https://github.com/kornelski/rust_urlencoding/blob/a4df8027ab34a86a63f1be727965cf101556403f/src/enc.rs#L130-L136
// Converts a UTF-8 byte to a single hexadecimal character
#[cfg(feature = "urlencoding")]
fn to_hex_digit(digit: u8) -> char {
    match digit {
        0..=9 => (b'0' + digit) as char,
        10..=255 => (b'A' - 10 + digit) as char,
    }
}

#[cfg(test)]
mod port_tests {
    use super::*;

    #[test]
    fn port_boundaries_and_defaults() {
        for (url, expected) in [
            ("http://example.test/", 80),
            ("https://example.test/", 443),
            ("https://example.test:/", 443),
            ("http://example.test:0/", 0),
            ("http://example.test:65535/", 65535),
            ("http://example.test:00080/", 80),
        ] {
            assert_eq!(HttpUrl::parse(url, None).unwrap().port.port(), expected);
        }
    }

    #[test]
    fn authority_fragments_preserve_valid_ports() {
        for (url, port) in [
            ("http://example.test#section", 80),
            ("http://example.test:80#section", 80),
            ("https://example.test:65535#section", 65535),
            ("https://example.test:#section", 443),
        ] {
            let parsed = HttpUrl::parse(url, None).unwrap();
            assert_eq!(parsed.port.port(), port);
            assert_eq!(parsed.path_and_query, "/");
            assert_eq!(parsed.fragment.as_deref(), Some("section"));
        }
        assert!(HttpUrl::parse("http://example.test:65536#section", None).is_err());
    }

    #[test]
    fn invalid_explicit_ports_are_rejected() {
        for port in [
            "65536",
            "65616",
            "4294967295",
            "4294967296",
            "-1",
            "+80",
            "abc",
        ] {
            let url = format!("http://example.test:{port}/");
            assert!(matches!(
                HttpUrl::parse(&url, None),
                Err(Error::IoError(error)) if error.kind() == std::io::ErrorKind::InvalidInput
            ));
        }
    }
}
