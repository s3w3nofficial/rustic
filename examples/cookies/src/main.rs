use rustic::{Request, Response, StatusCode, Cookie};

async fn retrieve_cookie(req: Request) -> rustic::Result<String> {
    Ok(format!("hello cookies: {:?}", req.cookie("hello").unwrap()))
}

async fn insert_cookie(_req: Request) -> rustic::Result {
    let mut res = Response::new(StatusCode::Ok);
    res.insert_cookie(Cookie::new("hello", "world"));
    Ok(res)
}

async fn remove_cookie(_req: Request) -> rustic::Result {
    let mut res = Response::new(StatusCode::Ok);
    res.remove_cookie(Cookie::named("hello"));
    Ok(res)
}

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    let mut app = rustic::new();

    app.at("/").get(retrieve_cookie);
    app.at("/set").post(insert_cookie);
    app.at("/remove").delete(remove_cookie);

    app.listen("127.0.0.1:8080").await?;

    Ok(())
}
