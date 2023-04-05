use http_types::{Error};
use response::Response;

use crate::server::Server;

pub mod listener;
pub mod server;

pub mod router;
pub mod endpoint;
pub mod request;
mod response;
mod route;
pub mod middleware;

#[must_use]
pub fn new() -> Server {
    Server::new()
}

pub type Result<T = Response> = std::result::Result<T, Error>;