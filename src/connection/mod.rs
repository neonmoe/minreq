mod http_connection;
mod https_connection;

use std::io::Error;
use http::Response;

pub use self::http_connection::HTTPConnection;
pub use self::https_connection::HTTPSConnection;

pub trait Connection {
    fn send(self) -> Result<Response, Error>;
}
