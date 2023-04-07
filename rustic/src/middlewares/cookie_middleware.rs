use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use http_types::{cookies::{CookieJar, Delta}, Cookie, headers};

use crate::{Middleware, Request, Next, response::CookieEvent};

#[derive(Default)]
pub struct CookieMiddleware;

impl CookieMiddleware {

    pub(crate) fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl Middleware for CookieMiddleware {

    async fn handle(&self, mut request: Request, next: Next<'_>) -> crate::Result {

        let cookie_jar = if let Some(cookie_data) = request.ext::<CookieData>() {
            cookie_data.content.clone()
        } else {
            let cookie_data = CookieData::from_request(&request);
            let content = cookie_data.content.clone();
            request.set_ext(cookie_data);
            content
        };

        let mut res = next.run(request).await;

        if res.cookie_events.is_empty() {
            return Ok(res);
        }

        let jar = &mut *cookie_jar.write().unwrap();

        for cookie in res.cookie_events.drain(..) {
            match cookie {
                CookieEvent::Added(cookie) => jar.add(cookie.clone()),
                CookieEvent::Removed(cookie) => jar.remove(cookie.clone()),
            }
        }

        for cookie in jar.delta() {
            let encoded_cookie = cookie.encoded().to_string();
            res.append_header(headers::SET_COOKIE, encoded_cookie);
        }

        Ok(res)
    }
}

pub struct CookieData {
    pub(crate) content: Arc<RwLock<LazyJar>>,
}

impl CookieData {
    pub(crate) fn from_request(req: &Request) -> Self {
        let jar = if let Some(cookie_headers) = req.header(&headers::COOKIE) {
            let mut jar = CookieJar::new();
            for cookie_header in cookie_headers {
                for pair in cookie_header.as_str().split(';') {
                    if let Ok(cookie) = Cookie::parse_encoded(String::from(pair)) {
                        jar.add_original(cookie);
                    }
                }
            }

            LazyJar(Some(jar))
        } else {
            LazyJar::default()
        };

        CookieData {
            content: Arc::new(RwLock::new(jar)),
        }
    }
}

#[derive(Default)]
pub(crate) struct LazyJar(Option<CookieJar>);

impl LazyJar {
    fn add(&mut self, cookie: Cookie<'static>) {
        self.get_jar().add(cookie)
    }

    fn remove(&mut self, cookie: Cookie<'static>) {
        self.get_jar().remove(cookie)
    }

    fn delta(&mut self) -> Delta<'_> {
        self.get_jar().delta()
    }

    pub(crate) fn get(&self, name: &str) -> Option<&Cookie<'static>> {
        if let Some(jar) = &self.0 {
            return jar.get(name);
        }
        None
    }

    fn get_jar(&mut self) -> &mut CookieJar {
        if self.0.is_none() {
            self.0 = Some(CookieJar::new());
        }

        self.0.as_mut().unwrap()
    }
}