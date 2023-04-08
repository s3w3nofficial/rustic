use std::sync::Arc;

use async_trait::async_trait;
use futures_core::Future;

use crate::{request::Request, middleware::{Middleware, Next}, response::Response};

#[async_trait]
pub trait Endpoint: Send + Sync + 'static {

    async fn call(&self, req: Request) -> crate::Result;
}

pub(crate) type DynEndpoint = dyn Endpoint;

#[async_trait]
impl<F, Fut, Res> Endpoint for F
where
    F: Send + Sync + 'static + Fn(Request) -> Fut,
    Fut: Future<Output = http_types::Result<Res>> + Send + 'static,
    Res: Into<Response> + 'static,
{
    async fn call(&self, req: Request) -> crate::Result {
        let fut = (self)(req);
        let res = fut.await?;
        Ok(res.into())
    }
}

pub(crate) struct MiddlewareEndpoint<E> {
    endpoint: E,
    middleware: Vec<Arc<dyn Middleware>>,
}

impl<E: Clone> Clone for MiddlewareEndpoint<E> {
    fn clone(&self) -> Self {
        Self {
            endpoint: self.endpoint.clone(),
            middleware: self.middleware.clone(),
        }
    }
}

impl<E> MiddlewareEndpoint<E>
where
    E: Endpoint,
{
    pub(crate) fn wrap_with_middleware(
        ep: E,
        middleware: &[Arc<dyn Middleware>],
    ) -> Box<dyn Endpoint + Send + Sync + 'static> {
        if middleware.is_empty() {
            Box::new(ep)
        } else {
            Box::new(Self {
                endpoint: ep,
                middleware: middleware.to_vec(),
            })
        }
    }
}

#[async_trait]
impl<E> Endpoint for MiddlewareEndpoint<E>
where
    E: Endpoint,
{
    async fn call(&self, req: Request) -> crate::Result {
        let next = Next {
            endpoint: &self.endpoint,
            next_middleware: &self.middleware,
        };
        Ok(next.run(req).await)
    }
}

#[async_trait]
impl Endpoint for Box<dyn Endpoint> {
    async fn call(&self, request: Request) -> crate::Result {
        self.as_ref().call(request).await
    }
}