pub use http_types::{Body, Error, Status, StatusCode, Cookie};

mod listeners;
mod server;
mod router;
mod endpoint;
mod request;
mod response;
mod middleware;
mod route;
mod middlewares;

pub use endpoint::Endpoint;
pub use middleware::{Middleware, Next};
pub use request::Request;
pub use response::Response;
pub use route::Route;
pub use server::Server;

#[must_use]
pub fn new() -> Server {
    Server::new()
}

pub type Result<T = Response> = std::result::Result<T, Error>;