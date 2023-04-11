use std::{io, path::Path, sync::Arc};

use kv_log_macro::info;

use crate::{
    endpoint::{Endpoint, MiddlewareEndpoint},
    fs::{ServeDir, ServeFile},
    middleware::Middleware,
    router::Router,
};

pub struct Route<'a> {
    router: &'a mut Router,
    path: String,
    middleware: Vec<Arc<dyn Middleware>>,
}

impl<'a> Route<'a> {
    pub(crate) fn new(router: &'a mut Router, path: String) -> Route<'a> {
        Route {
            router,
            path,
            middleware: Vec::new(),
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

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn serve_dir(&mut self, dir: impl AsRef<Path>) -> io::Result<()> {
        let dir = dir.as_ref().to_owned().canonicalize()?;
        let prefix = self.path().to_string();
        self.get(ServeDir::new(prefix, dir));
        Ok(())
    }

    pub fn serve_file(&mut self, file: impl AsRef<Path>) -> io::Result<()> {
        self.get(ServeFile::init(file)?);
        Ok(())
    }

    pub fn method(&mut self, method: http_types::Method, ep: impl Endpoint) -> &mut Self {
        self.router.add(
            &self.path,
            method,
            MiddlewareEndpoint::wrap_with_middleware(ep, &self.middleware),
        );
        self
    }

    pub fn head(&mut self, ep: impl Endpoint) -> &mut Self {
        self.method(http_types::Method::Head, ep);
        self
    }

    pub fn options(&mut self, ep: impl Endpoint) -> &mut Self {
        self.method(http_types::Method::Options, ep);
        self
    }

    pub fn get(&mut self, ep: impl Endpoint) -> &mut Self {
        self.method(http_types::Method::Get, ep);
        self
    }

    pub fn post(&mut self, ep: impl Endpoint) -> &mut Self {
        self.method(http_types::Method::Post, ep);
        self
    }

    pub fn put(&mut self, ep: impl Endpoint) -> &mut Self {
        self.method(http_types::Method::Put, ep);
        self
    }

    pub fn patch(&mut self, ep: impl Endpoint) -> &mut Self {
        self.method(http_types::Method::Patch, ep);
        self
    }

    pub fn delete(&mut self, ep: impl Endpoint) -> &mut Self {
        self.method(http_types::Method::Delete, ep);
        self
    }

    pub fn with<M>(&mut self, middleware: M) -> &mut Self
    where
        M: Middleware,
    {
        info!(
            "Adding middleware {} to route {:?}",
            middleware.name(),
            self.path
        );
        self.middleware.push(Arc::new(middleware));
        self
    }
}
