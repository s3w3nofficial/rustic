use http_types::StatusCode;
use http_types::headers::LOCATION;

use crate::{Response, Endpoint, Request};

#[derive(Debug, Clone)]
pub struct Redirect<T: AsRef<str>> {
    status: StatusCode,
    location: T,
}

impl<T: AsRef<str>> Redirect<T> {
    pub fn new(location: T) -> Self {
        Self {
            status: StatusCode::Found,
            location,
        }
    }

    pub fn permanent(location: T) -> Self {
        Self {
            status: StatusCode::PermanentRedirect,
            location,
        }
    }

    pub fn temporary(location: T) -> Self {
        Self {
            status: StatusCode::TemporaryRedirect,
            location,
        }
    }

    pub fn see_other(location: T) -> Self {
        Self {
            status: StatusCode::SeeOther,
            location,
        }
    }
}

#[async_trait::async_trait]
impl<T> Endpoint for Redirect<T>
where
    T: AsRef<str> + Send + Sync + 'static,
{
    async fn call(&self, _req: Request) -> crate::Result<Response> {
        Ok(self.into())
    }
}

impl<T: AsRef<str>> From<Redirect<T>> for Response {
    fn from(redirect: Redirect<T>) -> Self {
        let mut response = Response::new(redirect.status);
        response.append_header(LOCATION, redirect.location.as_ref());

        response
    }
}

impl<T: AsRef<str>> From<&Redirect<T>> for Response {
    fn from(redirect: &Redirect<T>) -> Response {
        let mut response = Response::new(redirect.status);
        response.append_header(LOCATION, redirect.location.as_ref());

        response
    }
}