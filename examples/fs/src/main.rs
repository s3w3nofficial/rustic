#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    femme::start();

    let mut app = rustic::new();

    app.at("/").serve_file("src/index.html").expect("file could not be served!");
    app.at("/static/*").serve_dir("src/static/").expect("directory could not be served");

    app.listen("127.0.0.1:8080").await?;
    Ok(())
}