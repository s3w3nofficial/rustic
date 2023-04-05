use http_types::{Error, Body, StatusCode};
use std::fmt::{Debug};

pub struct Response {
    pub(crate) res: http_types::Response,
    pub(crate) error: Option<Error>,
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
        }
    }

    pub fn set_body(&mut self, body: impl Into<Body>) {
        self.res.set_body(body);
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
        }
    }
}

impl From<http_types::Response> for Response {
    fn from(res: http_types::Response) -> Self {
        Self {
            res,
            error: None,
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
