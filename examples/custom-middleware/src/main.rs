use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use async_trait::async_trait;
use rustic::{Middleware, Next, Request};

#[derive(Default)]
struct RequestCounterMiddleware {
    requests_counted: Arc<AtomicUsize>,
}

impl RequestCounterMiddleware {
    fn new(start: usize) -> Self {
        Self {
            requests_counted: Arc::new(AtomicUsize::new(start)),
        }
    }
}

struct RequestCount(usize);

#[async_trait]
impl Middleware for RequestCounterMiddleware {
    async fn handle(&self, mut req: Request, next: Next<'_>) -> rustic::Result {
        let count = self.requests_counted.fetch_add(1, Ordering::Relaxed);
        println!("request counter: {}", count);
        req.set_ext(RequestCount(count));

        let mut res = next.run(req).await;

        res.insert_header("request-number", count.to_string());
        Ok(res)
    }
}

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    femme::start();

    let mut app = rustic::new();

    app.with(RequestCounterMiddleware::new(0));

    app.at("/").get(|_| async { Ok("Hello, world!") });

    app.listen("127.0.0.1:8080").await?;
    Ok(())
}
