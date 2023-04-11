pub use http_types::{Body, Cookie, Error, Status, StatusCode};

mod endpoint;
mod fs;
mod listeners;
mod middleware;
mod middlewares;
mod request;
mod response;
mod route;
mod router;
mod redirect;
mod server;

pub use endpoint::Endpoint;
pub use middleware::{Middleware, Next};
pub use middlewares::{
    AuthMiddleware, BasicAuthScheme, BearerAuthScheme, CorsMiddleware, Origin, WithCors,
    WithHttpAuth, WithLogging,
};
pub use request::Request;
pub use response::Response;
pub use route::Route;
pub use redirect::Redirect;
pub use server::Server;

pub use http_types;

#[must_use]
pub fn new() -> Server {
    Server::new()
}

pub type Result<T = Response> = std::result::Result<T, Error>;
