use http_types::StatusCode;
use routefinder::{Captures, Router as MethodRouter};
use std::collections::HashMap;

use crate::{endpoint::DynEndpoint, request::Request, response::Response};

pub(crate) struct Router {
    method_map: HashMap<http_types::Method, MethodRouter<Box<DynEndpoint>>>,
    all_method_router: MethodRouter<Box<DynEndpoint>>,
}

pub(crate) struct Selection<'a> {
    pub(crate) endpoint: &'a DynEndpoint,
    pub(crate) params: Captures<'static, 'static>,
}

impl Router {
    pub(crate) fn new() -> Self {
        Router {
            method_map: HashMap::default(),
            all_method_router: MethodRouter::new(),
        }
    }

    pub(crate) fn add(&mut self, path: &str, method: http_types::Method, ep: Box<DynEndpoint>) {
        self.method_map
            .entry(method)
            .or_insert_with(MethodRouter::new)
            .add(path, ep)
            .unwrap()
    }

    pub(crate) fn route(&self, path: &str, method: http_types::Method) -> Selection<'_> {
        if let Some(m) = self
            .method_map
            .get(&method)
            .and_then(|r| r.best_match(path))
        {
            Selection {
                endpoint: m.handler(),
                params: m.captures().into_owned(),
            }
        } else if let Some(m) = self.all_method_router.best_match(path) {
            Selection {
                endpoint: m.handler(),
                params: m.captures().into_owned(),
            }
        } else if method == http_types::Method::Head {
            // If it is a HTTP HEAD request then check if there is a callback in the endpoints map
            // if not then fallback to the behavior of HTTP GET else proceed as usual

            self.route(path, http_types::Method::Get)
        } else if self
            .method_map
            .iter()
            .filter(|(k, _)| **k != method)
            .any(|(_, r)| r.best_match(path).is_some())
        {
            // If this `path` can be handled by a callback registered with a different HTTP method
            // should return 405 Method Not Allowed
            Selection {
                endpoint: &method_not_allowed,
                params: Captures::default(),
            }
        } else {
            Selection {
                endpoint: &not_found_endpoint,
                params: Captures::default(),
            }
        }
    }
}

async fn not_found_endpoint(_req: Request) -> crate::Result {
    Ok(Response::new(StatusCode::NotFound))
}

async fn method_not_allowed(_req: Request) -> crate::Result {
    Ok(Response::new(StatusCode::MethodNotAllowed))
}
