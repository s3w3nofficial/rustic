use rustic::HttpAuth;

fn verify_token(_token: &str) -> bool {
    true
}

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    femme::start();

    let mut app = rustic::new();

    app.with_token_auth(verify_token);

    app.at("/").get(|_| async { Ok("Hello, world!") });

    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

