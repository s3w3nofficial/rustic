use std::sync::Arc;

use async_trait::async_trait;

use crate::{request::Request, endpoint::DynEndpoint, response::Response};

#[async_trait]
pub trait Middleware: Send + Sync + 'static {
    async fn handle(&self, request: Request, next: Next<'_>) -> crate::Result;

    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}

pub struct Next<'a> {
    pub(crate) endpoint: &'a DynEndpoint,
    pub(crate) next_middleware: &'a [Arc<dyn Middleware>],
}

impl Next<'_> {
    /// Asynchronously execute the remaining middleware chain.
    pub async fn run(mut self, req: Request) -> Response {
        if let Some((current, next)) = self.next_middleware.split_first() {
            self.next_middleware = next;
            match current.handle(req, self).await {
                Ok(request) => request,
                Err(err) => err.into(),
            }
        } else {
            match self.endpoint.call(req).await {
                Ok(request) => request,
                Err(err) => err.into(),
            }
        }
    }
}