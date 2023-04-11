use async_trait::async_trait;
use rustic::http_types::{Request};
use rustic::http_types;
use rustic::{Route};

#[async_trait]
pub trait Proxy {
    async fn proxy_to(&mut self, url: &str) -> &mut Self;
}

#[async_trait]
impl<'a> Proxy for Route<'a> {

    async fn proxy_to(&mut self, url: &str) -> &mut Self {

        let url_owned  = url.to_owned();

        let ep = move |req: rustic::Request| {
            let url_clone = url_owned.clone();
            async move {
                let response = proxy_request(req.get_underlying_request(), url_clone.as_str()).await;
                Ok(response)
            }
        };

        self.method(http_types::Method::Head, ep.clone());
        self.method(http_types::Method::Options, ep.clone());
        self.method(http_types::Method::Get, ep.clone());
        self.method(http_types::Method::Post, ep.clone());
        self.method(http_types::Method::Put, ep.clone());
        self.method(http_types::Method::Patch, ep.clone());
        self.method(http_types::Method::Delete, ep.clone());
        self
    }
}

async fn proxy_request(request: Request, url: &str) -> http_types::Response {
    let stream = async_std::net::TcpStream::connect(url).await.unwrap();

    let mut res = async_h1::client::connect(stream, request).await.unwrap();
    res.insert_header("x-proxyed", "true");
    res
}