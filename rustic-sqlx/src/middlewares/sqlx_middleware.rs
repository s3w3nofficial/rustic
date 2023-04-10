use async_trait::async_trait;
use rustic::{Middleware, Next, Request, Server};

pub struct SQLxMiddleware;

impl SQLxMiddleware {
    fn new() -> Self {
        SQLxMiddleware {

        }
    }
}

#[async_trait]
impl Middleware for SQLxMiddleware {

    async fn handle(&self, request: Request, next: Next<'_>) -> rustic::Result {
        return Ok(next.run(request).await);
    }
}

pub trait WithSQLx {
    fn with_sqlx(&mut self) -> &mut Self;
}

impl WithSQLx for Server {

    fn with_sqlx(&mut self) -> &mut Self {
        self.with(SQLxMiddleware::new());
        self
    }
}