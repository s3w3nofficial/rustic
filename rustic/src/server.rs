//use crate::listener::{Listener};

use async_std::io;
use http_types::{Response, StatusCode};

use crate::listener::{Listener, to_listener::ToListener};

pub struct Server {

}

impl Server {

    #[must_use]
    pub fn new() -> Self {
        Self {

        }
    }

    pub async fn listen<L: ToListener>(self, listener: L) -> io::Result<()> {
        let mut listener = listener.to_listener()?;
        listener.bind(self).await?;
        listener.accept().await?;
        Ok(())
    }

    pub async fn respond<Req, Res>(&self, _req: Req) -> http_types::Result<Res>
    where
        Req: Into<http_types::Request>,
        Res: From<http_types::Response>,
    {
        let mut res = Response::new(StatusCode::Ok);
        res.set_body("Hello, World!");

        Ok(res.into())
    }
}

impl Clone for Server {
    fn clone(&self) -> Self {
        Self {

        }
    }
}
