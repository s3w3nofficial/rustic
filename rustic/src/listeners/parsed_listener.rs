use async_std::io;

use crate::server::Server;

use super::{tcp_listener::TcpListener, Listener};

pub enum ParsedListener {
    Tcp(TcpListener),
}

#[async_trait::async_trait]
impl Listener for ParsedListener {
    async fn bind(&mut self, server: Server) -> io::Result<()> {
        match self {
            Self::Tcp(t) => t.bind(server).await,
        }
    }

    async fn accept(&mut self) -> io::Result<()> {
        match self {
            Self::Tcp(t) => t.accept().await,
        }
    }
}
