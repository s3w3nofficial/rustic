use rustic::WithHttpAuth;

fn verify_password(username: &str, password: &str) -> bool {
    if username == "user" && password == "pass" {
        return true;
    }

    false
}

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    femme::start();

    let mut app = rustic::new();

    app.with_basic_auth(verify_password);

    app.at("/").get(|_| async { Ok("Hello, world!") });

    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

