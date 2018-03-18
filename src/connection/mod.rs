mod http_connection;

use std::io::Error;
use http::Response;

pub use self::http_connection::HTTPConnection;

pub trait Connection {
    fn send(self) -> Result<Response, Error>;
}
