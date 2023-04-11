use rustic::{Request, StatusCode, Response, Redirect};
use rustic_proxy::{Proxy, ProxyAt};

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    femme::start();

    let mut app = rustic::new();

    app.at("/").get(|_| async { Ok("Hello, world!") });

    app.at("/forward").get(|_| async { 
        Ok(Redirect::new("https://www.youtube.com/watch?v=dQw4w9WgXcQ"))
    });

    //app.at("proxy_to").proxy_to("abcd");
    app.proxy_at("127.0.0.1:8081").await.unwrap();

    app.listen("127.0.0.1:8080").await?;

    Ok(())
}
