use std::io;
use std::{net::SocketAddr};
use std::error::Error;
use async_h1::{server, client};
use async_std::stream::StreamExt;
use async_std::{task, net::{TcpStream, TcpListener}};
use async_trait::async_trait;
use rustic::http_types::{Request, Url};
use rustic::{Route, http_types, Endpoint, Response, StatusCode, Server};

#[async_trait]
pub trait Proxy {
    async fn proxy_to(&mut self, url: &str) -> &mut Self;
}

#[async_trait]
impl<'a> Proxy for Route<'a> {

    async fn proxy_to(&mut self, url: &str) -> &mut Self {
        let ep = |req: rustic::Request| async {
            Ok(Response::new(StatusCode::Ok))
        };

        self.method(http_types::Method::Head, ep);
        self.method(http_types::Method::Options, ep);
        self.method(http_types::Method::Get, ep);
        self.method(http_types::Method::Post, ep);
        self.method(http_types::Method::Put, ep);
        self.method(http_types::Method::Patch, ep);
        self.method(http_types::Method::Delete, ep);
        self
    }
}

#[async_trait]
pub trait ProxyAt {
    async fn proxy_at<'a>(&'a mut self, url: &str) -> io::Result<()>;
}

#[async_trait]
impl ProxyAt for Server {
    async fn proxy_at<'a>(&'a mut self, url: &str) -> io::Result<()> {
        let listener = TcpListener::bind(("127.0.0.1", 8081)).await?;
        let addr = format!("http://{}", listener.local_addr()?);
        println!("listening on {}", addr);

        // For each incoming TCP connection, spawn a task and call `accept`.
        let mut incoming = listener.incoming();
        while let Some(stream) = incoming.next().await {
            let stream = stream?;
            task::spawn(async {
                if let Err(err) = accept(stream).await {
                    eprintln!("{}", err);
                }
            });
        }

        Ok(())
    }
}

async fn accept(stream: TcpStream) -> http_types::Result<()> {
    println!("starting new connection from {}", stream.peer_addr()?);

    async_h1::accept(stream.clone(), |req| async move {

        let res = proxy_request(req).await;
        Ok(res)
    }).await?;

    Ok(())
}

async fn proxy_request(request: Request) -> http_types::Response {
    let stream = async_std::net::TcpStream::connect("127.0.0.1:8082").await.unwrap();

    let mut res = async_h1::client::connect(stream, request).await.unwrap();
    res.insert_header("x-proxyed", "true");
    res
}
