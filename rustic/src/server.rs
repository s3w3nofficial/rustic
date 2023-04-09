use std::sync::Arc;

use async_std::io;
use kv_log_macro::info;

use crate::{
    listeners::{to_listener::ToListener, Listener},
    middleware::{Middleware, Next},
    middlewares,
    request::Request,
    route::Route,
    router::{Router, Selection},
};

pub struct Server {
    router: Arc<Router>,
    middleware: Arc<Vec<Arc<dyn Middleware>>>,
}

impl Server {
    #[must_use]
    pub fn new() -> Self {
        Self {
            router: Arc::new(Router::new()),
            middleware: Arc::new(vec![Arc::new(middlewares::CookieMiddleware::new())]),
        }
    }

    pub async fn listen<L: ToListener>(self, listener: L) -> io::Result<()> {
        let mut listener = listener.to_listener()?;
        listener.bind(self).await?;
        listener.accept().await?;
        Ok(())
    }

    pub fn at<'a>(&'a mut self, path: &str) -> Route<'a> {
        let router = Arc::get_mut(&mut self.router)
            .expect("Registering routes is not possible after the Server has started");
        Route::new(router, path.to_owned())
    }

    pub fn with<M>(&mut self, middleware: M) -> &mut Self
    where
        M: Middleware,
    {
        info!("Adding middleware {}", middleware.name());
        let m = Arc::get_mut(&mut self.middleware)
            .expect("Registering middleware is not possible after the Server has started");
        m.push(Arc::new(middleware));
        self
    }

    pub async fn respond<Req, Res>(&self, req: Req) -> http_types::Result<Res>
    where
        Req: Into<http_types::Request>,
        Res: From<http_types::Response>,
    {
        let req = req.into();
        let Self { router, middleware } = self.clone();

        let method = req.method().to_owned();
        let Selection { endpoint, params } = router.route(req.url().path(), method);
        let route_params = vec![params];
        let req = Request::new(req, route_params);

        let next = Next {
            endpoint,
            next_middleware: &middleware,
        };

        let res = next.run(req).await;
        let res: http_types::Response = res.into();
        Ok(res.into())
    }
}

impl Clone for Server {
    fn clone(&self) -> Self {
        Self {
            router: self.router.clone(),
            middleware: self.middleware.clone(),
        }
    }
}
