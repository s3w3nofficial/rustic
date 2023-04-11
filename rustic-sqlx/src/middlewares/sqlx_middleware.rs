use async_trait::async_trait;
use rustic::{Middleware, Next, Request, Server};
use sqlx::{Database, Pool};

pub struct SQLxMiddleware<DB>
where
    DB: Database
{
    _pool: Pool<DB>,
}

impl<DB> SQLxMiddleware<DB> 
where 
    DB: Database
{
    pub async fn new(connection_string: &'_ str) -> std::result::Result<Self, sqlx::Error> {
        let pool: Pool<DB> = Pool::connect(connection_string).await?;
        Ok(Self {_pool: pool})
    }
}

#[async_trait]
impl<DB> Middleware for SQLxMiddleware<DB> 
where
    DB: Database
{
    async fn handle(&self, request: Request, next: Next<'_>) -> rustic::Result {
        return Ok(next.run(request).await);
    }
}

#[async_trait]
pub trait WithSQLx {
    async fn with_sqlx<DB>(&mut self, connection_str: &'_ str) -> &mut Self
        where
            DB: Database;
}

#[async_trait]
impl WithSQLx for Server {

    async fn with_sqlx<DB>(&mut self, connection_str: &'_ str) -> &mut Self
    where
        DB: Database
    {
        let middleware = SQLxMiddleware::<DB>::new(connection_str).await;
        let handled = match middleware {
            Ok(middleware) => middleware,
            Err(_middleware) => panic!("Unknow error!"),
        };

        self.with(handled);
        self
    }
}