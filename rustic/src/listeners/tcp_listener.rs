use std::os::fd::AsRawFd;

use async_std::net::{self, SocketAddr, TcpStream};
use async_std::stream::StreamExt;
use async_std::{io, task};
use kv_log_macro::error;

use super::{is_transient_error, Listener};
use crate::server::Server;

pub struct TcpListener {
    addrs: Option<Vec<SocketAddr>>,
    listener: Option<net::TcpListener>,
    server: Option<Server>,
}

impl TcpListener {
    pub fn from_addrs(addrs: Vec<SocketAddr>) -> Self {
        Self {
            addrs: Some(addrs),
            listener: None,
            server: None,
        }
    }
}

fn handle_tcp(app: Server, stream: TcpStream) {
    task::spawn(async move {
        let local_addr = stream.local_addr().ok();
        let peer_addr = stream.peer_addr().ok();

        let fut = async_h1::accept(stream, |mut req| async {
            req.set_local_addr(local_addr);
            req.set_peer_addr(peer_addr);
            app.respond(req).await
        });

        if let Err(error) = fut.await {
            error!("async-h1 error", { error: error.to_string() });
        }
    });
}

#[async_trait::async_trait]
impl Listener for TcpListener {
    async fn bind(&mut self, server: Server) -> io::Result<()> {
        self.server = Some(server);

        if self.listener.is_none() {
            let addrs = self
                .addrs
                .take()
                .expect("`bind` should only be called once");

            let listener = net::TcpListener::bind(addrs.as_slice()).await?;
            let raw_fd = listener.as_raw_fd();

            let optval: libc::c_int = 1;
            unsafe {
                libc::setsockopt(raw_fd, libc::SOL_SOCKET, libc::SO_REUSEADDR, &optval as *const _ as *const _, std::mem::size_of_val(&optval) as u32);
            }

            // unsafe {
            //     let optval: libc::c_int = 1;
            //     libc::setsockopt(
            //         raw_fd,
            //         libc::SOL_SOCKET,
            //         libc::SO_REUSEADDR,
            //         &optval as *const _ as *const libc::c_void,
            //         std::mem::size_of_val(&optval) as libc::socklen_t,
            //     );
            // }

            self.listener = Some(listener);
        }

        Ok(())
    }

    async fn accept(&mut self) -> io::Result<()> {
        let server = self
            .server
            .take()
            .expect("`Listener::bind` must be called before `Listener::accept`");

        let listener = self
            .listener
            .take()
            .expect("`Listener::bind` must be called before `Listener::accept`");

        let mut incoming = listener.incoming();

        while let Some(stream) = incoming.next().await {
            match stream {
                Err(ref e) if is_transient_error(e) => continue,
                Err(error) => {
                    let delay = std::time::Duration::from_millis(500);
                    error!("Error: {}. Pausing for {:?}.", error, delay);
                    task::sleep(delay).await;
                    continue;
                }

                Ok(stream) => {
                    handle_tcp(server.clone(), stream);
                }
            };
        }

        Ok(())
    }
}
