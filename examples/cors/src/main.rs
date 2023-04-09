use rustic::{Origin, WithCors};

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    femme::start();

    let mut app = rustic::new();

    app.with_cors(|cors| {
        cors.allow_credentials(true)
            .allow_origin(Origin::from("http://localhost:8000"))
    });

    app.at("/").get(|_| async { Ok("Hello, world!") });

    app.listen("127.0.0.1:8080").await?;
    Ok(())
}
