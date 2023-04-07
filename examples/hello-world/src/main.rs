use serde::{Deserialize, Serialize};
use rustic::{Body, Request, Error};

#[derive(Deserialize, Serialize)]
struct Cat {
    name: String,
}

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    femme::start();

    let mut app = rustic::new();

    app.at("/").get(|_| async { Ok("Hello, world!") });
    app.at("/submit").post(submit);

    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

pub async fn submit(mut req: Request) -> Result<Body, Error> {
    let cat: Cat = req.body_json().await?;
    println!("cat name: {}", cat.name);

    let cat = Cat {
        name: "chashu".into(),
    };

    Body::from_json(&cat)
}