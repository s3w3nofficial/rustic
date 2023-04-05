use std::sync::Arc;

use crate::{router::Router, endpoint::{Endpoint, MiddlewareEndpoint}, middleware::Middleware};

pub struct Route<'a> {
    router: &'a mut Router,
    path: String,
    middleware: Vec<Arc<dyn Middleware>>,
}

impl <'a> Route<'a> {
    pub(crate) fn new(router: &'a mut Router, path: String) -> Route<'a> {
        Route {
            router,
            path,
            middleware: Vec::new()
        }
    }

    pub fn at<'b>(&'b mut self, path: &str) -> Route<'b> {
        let mut p = self.path.clone();

        if !p.ends_with('/') && !path.starts_with('/') {
            p.push('/');
        }

        if path != "/" {
            p.push_str(path);
        }

        Route {
            router: self.router,
            path: p,
            middleware: self.middleware.clone(),
        }
    }

    pub fn method(&mut self, method: http_types::Method, ep: impl Endpoint) -> &mut Self {
        self.router.add(
            &self.path,
            method,
            MiddlewareEndpoint::wrap_with_middleware(ep, &self.middleware),
        );
        self
    }

    pub fn get(&mut self, ep: impl Endpoint) -> &mut Self {
        self.method(http_types::Method::Get, ep);
        self
    }
}