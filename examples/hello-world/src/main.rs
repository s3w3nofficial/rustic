

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    let app = rustic::new();

    app.listen("127.0.0.1:8080").await?;
    Ok(())
}