use rustic::{Redirect, WithLogging};
use rustic_proxy::{Proxy};

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    femme::start();

    let mut app = rustic::new();

    app.with_logging();

    app.at("/").get(|_| async { Ok("Hello, world!") });

    app.at("/forward").get(|_| async { 
        Ok(Redirect::new("https://www.youtube.com/watch?v=dQw4w9WgXcQ"))
    });

    app.at("/proxy").proxy_to("127.0.0.1:8081").await;

    app.listen("127.0.0.1:8080").await?;

    Ok(())
}
