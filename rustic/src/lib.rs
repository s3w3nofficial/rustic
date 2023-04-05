use crate::server::Server;

pub mod listener;
pub mod server;

#[must_use]
pub fn new() -> Server {
    Server::new()
}