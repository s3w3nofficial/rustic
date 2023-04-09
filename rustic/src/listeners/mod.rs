mod parsed_listener;
mod tcp_listener;
pub mod to_listener;

use async_std::io;
use async_trait::async_trait;

use crate::Server;

#[async_trait]
pub trait Listener: Send + 'static {
    async fn bind(&mut self, app: Server) -> io::Result<()>;

    async fn accept(&mut self) -> io::Result<()>;
}

#[async_trait]
impl<L> Listener for Box<L>
where
    L: Listener,
{
    async fn bind(&mut self, app: Server) -> io::Result<()> {
        self.as_mut().bind(app).await
    }

    async fn accept(&mut self) -> io::Result<()> {
        self.as_mut().accept().await
    }
}

pub(crate) fn is_transient_error(e: &io::Error) -> bool {
    use io::ErrorKind::*;

    matches!(
        e.kind(),
        ConnectionRefused | ConnectionAborted | ConnectionReset
    )
}
