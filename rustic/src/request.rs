use http_types::Cookie;
use routefinder::Captures;

use crate::middlewares::CookieData;

pub struct Request {
    pub(crate) req: http_types::Request,
    pub(crate) route_params: Vec<Captures<'static, 'static>>,
}

impl Request {
    pub(crate) fn new(
        req: http_types::Request,
        route_params: Vec<Captures<'static, 'static>>,
    ) -> Self {
        Self {
            req,
            route_params,
        }
    }

    pub fn header(
        &self,
        key: impl Into<http_types::headers::HeaderName>,
    ) -> Option<&http_types::headers::HeaderValues> {
        self.req.header(key)
    }

    pub fn ext<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.req.ext().get()
    }

    pub fn set_ext<T: Send + Sync + 'static>(&mut self, val: T) -> Option<T> {
        self.req.ext_mut().insert(val)
    }

    pub async fn body_json<T: serde::de::DeserializeOwned>(&mut self) -> crate::Result<T> {
        let res = self.req.body_json().await?;
        Ok(res)
    }

    #[must_use]
    pub fn cookie(&self, name: &str) -> Option<Cookie<'static>> {
        self.ext::<CookieData>()
            .and_then(|cookie_data| cookie_data.content.read().unwrap().get(name).cloned())
    }
}