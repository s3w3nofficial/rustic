use rustic::{CorsMiddleware, Origin};

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    femme::start();

    let mut app = rustic::new();

    let allow_origin = String::from("http://localhost:8000");

    let cors = CorsMiddleware::new()
        .allow_credentials(true)
        .allow_origin(Origin::Exact(allow_origin));

    app.with(cors);

    app.at("/").get(|_| async { Ok("Hello, world!") });

    app.listen("127.0.0.1:8080").await?;
    Ok(())
}


