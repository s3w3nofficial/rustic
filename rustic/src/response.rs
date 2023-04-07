use http_types::{
    Error, 
    Body, 
    StatusCode, 
    Cookie, 
    headers::{HeaderName, ToHeaderValues}
};
use std::fmt::{Debug};

pub(crate) enum CookieEvent {
    Added(Cookie<'static>),
    Removed(Cookie<'static>),
}

pub struct Response {
    pub(crate) res: http_types::Response,
    pub(crate) error: Option<Error>,
    pub(crate) cookie_events: Vec<CookieEvent>,
}

impl Response {
    #[must_use]
    pub fn new<S>(status: S) -> Self
    where
        S: TryInto<StatusCode>,
        S::Error: Debug,
    {
        let res = http_types::Response::new(status);
        Self {
            res,
            error: None,
            cookie_events: vec![],
        }
    }

    pub fn insert_header(&mut self, key: impl Into<HeaderName>, value: impl ToHeaderValues) {
        self.res.insert_header(key, value);
    }

    pub fn append_header(&mut self, key: impl Into<HeaderName>, value: impl ToHeaderValues) {
        self.res.append_header(key, value);
    }

    pub fn set_body(&mut self, body: impl Into<Body>) {
        self.res.set_body(body);
    }

    pub fn insert_cookie(&mut self, cookie: Cookie<'static>) {
        self.cookie_events.push(CookieEvent::Added(cookie));
    }

    pub fn remove_cookie(&mut self, cookie: Cookie<'static>) {
        self.cookie_events.push(CookieEvent::Removed(cookie));
    }
}

impl From<Response> for http_types::Response {
    fn from(response: Response) -> http_types::Response {
        response.res
    }
}

impl From<http_types::Body> for Response {
    fn from(body: http_types::Body) -> Self {
        let mut res = Response::new(200);
        res.set_body(body);
        res
    }
}

impl From<Error> for Response {
    fn from(err: Error) -> Self {
        Self {
            res: http_types::Response::new(err.status()),
            error: Some(err),
            cookie_events: vec![],
        }
    }
}

impl From<http_types::Response> for Response {
    fn from(res: http_types::Response) -> Self {
        Self {
            res,
            error: None,
            cookie_events: vec![],
        }
    }
}

impl From<StatusCode> for Response {
    fn from(status: StatusCode) -> Self {
        let res: http_types::Response = status.into();
        res.into()
    }
}

impl From<String> for Response {
    fn from(s: String) -> Self {
        Body::from_string(s).into()
    }
}

impl<'a> From<&'a str> for Response {
    fn from(s: &'a str) -> Self {
        Body::from_string(String::from(s)).into()
    }
}
