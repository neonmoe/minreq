extern crate curl;
#[cfg(test)]
extern crate tiny_http;

mod requests;
mod http;
#[cfg(test)]
mod tests;

pub use requests::*;
pub use http::*;
