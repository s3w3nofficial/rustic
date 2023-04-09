use rustic::{Body, Error, Request, WithLogging};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct Cat {
    name: String,
}

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    femme::start();

    let mut app = rustic::new();

    app.with_logging();

    app.at("/").get(|_| async { Ok("Hello, world!") });
    app.at("/posts/:id").get(|req: Request| async move {
        let post_id = req.param("id")?.parse().unwrap_or(0);
        Ok(format!("post id: {}", post_id))
    });
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
