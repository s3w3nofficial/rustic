use async_trait::async_trait;
use http_types::{headers::{HeaderValue, HeaderValues, self}, StatusCode, Method};
use regex::Regex;
use std::hash::Hash;

use crate::{Middleware, Request, Next, Server};

#[derive(Clone, Hash)]
pub struct CorsMiddleware {
    allow_credentials: Option<HeaderValue>,
    allow_headers: HeaderValue,
    allow_methods: HeaderValue,
    allow_origin: Origin,
    expose_headers: Option<HeaderValue>,
    max_age: HeaderValue,
}

pub(crate) const DEFAULT_MAX_AGE: &str = "86400";
pub(crate) const DEFAULT_METHODS: &str = "GET, POST, OPTIONS";
pub(crate) const WILDCARD: &str = "*";

impl CorsMiddleware {

    #[must_use]
    pub fn new() -> Self {
        Self {
            allow_credentials: None,
            allow_headers: WILDCARD.parse().unwrap(),
            allow_methods: DEFAULT_METHODS.parse().unwrap(),
            allow_origin: Origin::Any,
            expose_headers: None,
            max_age: DEFAULT_MAX_AGE.parse().unwrap(),
        }
    }

    #[must_use]
    pub fn allow_credentials(mut self, allow_credentials: bool) -> Self {
        self.allow_credentials = match allow_credentials.to_string().parse() {
            Ok(header) => Some(header),
            Err(_) => None,
        };
        self
    }

    pub fn allow_headers<T: Into<HeaderValue>>(mut self, headers: T) -> Self {
        self.allow_headers = headers.into();
        self
    }

    pub fn max_age<T: Into<HeaderValue>>(mut self, max_age: T) -> Self {
        self.max_age = max_age.into();
        self
    }

    pub fn allow_methods<T: Into<HeaderValue>>(mut self, methods: T) -> Self {
        self.allow_methods = methods.into();
        self
    }

    pub fn allow_origin<T: Into<Origin>>(mut self, origin: T) -> Self {
        self.allow_origin = origin.into();
        self
    }

    pub fn expose_headers<T: Into<HeaderValue>>(mut self, headers: T) -> Self {
        self.expose_headers = Some(headers.into());
        self
    }

    fn build_preflight_response(&self, origins: &HeaderValues) -> http_types::Response {
        let mut response = http_types::Response::new(StatusCode::Ok);
        response.insert_header(headers::ACCESS_CONTROL_ALLOW_ORIGIN, origins);

        response.insert_header(
            headers::ACCESS_CONTROL_ALLOW_METHODS,
            self.allow_methods.clone(),
        );

        response.insert_header(
            headers::ACCESS_CONTROL_ALLOW_HEADERS,
            self.allow_headers.clone(),
        );

        response.insert_header(headers::ACCESS_CONTROL_MAX_AGE, self.max_age.clone());

        if let Some(allow_credentials) = self.allow_credentials.clone() {
            response.insert_header(headers::ACCESS_CONTROL_ALLOW_CREDENTIALS, allow_credentials);
        }

        if let Some(expose_headers) = self.expose_headers.clone() {
            response.insert_header(headers::ACCESS_CONTROL_EXPOSE_HEADERS, expose_headers);
        }

        response
    }

    /// Look at origin of request and determine `allow_origin`
    fn response_origin(&self, origin: &HeaderValue) -> HeaderValue {
        match self.allow_origin {
            Origin::Any => WILDCARD.parse().unwrap(),
            _ => origin.clone(),
        }
    }

    /// Determine if origin is appropriate
    fn is_valid_origin(&self, origin: &HeaderValue) -> bool {
        let origin = origin.as_str().to_string();

        match &self.allow_origin {
            Origin::Any => true,
            Origin::Exact(s) => s == &origin,
            Origin::List(list) => list.contains(&origin),
            Origin::Match(regex) => regex.is_match(&origin),
        }
    }
}

#[async_trait]
impl Middleware for CorsMiddleware {

    async fn handle(&self, request: Request, next: Next<'_>) -> crate::Result {
        let origins = request.header(&headers::ORIGIN).cloned();

        if origins.is_none() {
            // This is not a CORS request if there is no Origin header
            return Ok(next.run(request).await);
        }

        let origins = origins.unwrap();
        let origin = origins.last();

        if !self.is_valid_origin(origin) {
            return Ok(http_types::Response::new(StatusCode::Unauthorized).into());
        }

        // Return results immediately upon preflight request
        if request.method() == Method::Options {
            return Ok(self.build_preflight_response(&origins).into());
        }

        let mut response = next.run(request).await;

        response.insert_header(
            headers::ACCESS_CONTROL_ALLOW_ORIGIN,
            self.response_origin(origin),
        );

        if let Some(allow_credentials) = &self.allow_credentials {
            response.insert_header(
                headers::ACCESS_CONTROL_ALLOW_CREDENTIALS,
                allow_credentials.clone(),
            );
        }

        if let Some(expose_headers) = &self.expose_headers {
            response.insert_header(
                headers::ACCESS_CONTROL_EXPOSE_HEADERS,
                expose_headers.clone(),
            );
        }

        Ok(response)
    }
}

impl Default for CorsMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub enum Origin {
    Any,
    Exact(String),
    List(Vec<String>),
    Match(Regex),
}

impl From<String> for Origin {
    fn from(s: String) -> Self {
        if s == "*" {
            return Self::Any;
        }
        Self::Exact(s)
    }
}

impl From<&str> for Origin {
    fn from(s: &str) -> Self {
        Self::from(s.to_string())
    }
}

impl From<Vec<String>> for Origin {
    fn from(list: Vec<String>) -> Self {
        if list.len() == 1 {
            return Self::from(list[0].clone());
        }

        Self::List(list)
    }
}

impl From<Regex> for Origin {
    fn from(regex: Regex) -> Self {
        Self::Match(regex)
    }
}

impl From<Vec<&str>> for Origin {
    fn from(list: Vec<&str>) -> Self {
        Self::from(
            list.iter()
                .map(|s| (*s).to_string())
                .collect::<Vec<String>>(),
        )
    }
}

impl PartialEq for Origin {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Exact(this), Self::Exact(other)) => this == other,
            (Self::List(this), Self::List(other)) => this == other,
            (Self::Match(this), Self::Match(other)) => this.to_string() == other.to_string(),
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl Hash for Origin {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Any => core::mem::discriminant(self).hash(state),
            Self::Exact(s) => s.hash(state),
            Self::List(list) => list.hash(state),
            Self::Match(regex) => regex.to_string().hash(state),
        }
    }
}

pub trait WithCors {
    fn with_cors(&mut self, configure: impl Fn(CorsMiddleware) -> CorsMiddleware) -> &mut Self;
}

impl WithCors for Server {

    fn with_cors(&mut self, configure: impl Fn(CorsMiddleware) -> CorsMiddleware) -> &mut Self {
        let cors = CorsMiddleware::new();

        self.with((configure)(cors));
        self
    }
}