use std::any::Any;

use async_trait::async_trait;
use http_types::{Result, StatusCode};
use kv_log_macro::{error, info};

use crate::{Middleware, Next, Request, Response, Server};

pub struct AuthMiddleware<ImplScheme: Scheme> {
    pub(crate) scheme: ImplScheme,
}

impl<ImplScheme: Scheme> AuthMiddleware<ImplScheme> {
    pub fn new(scheme: ImplScheme) -> Self {
        Self { scheme }
    }
}

pub trait WithHttpAuth {
    fn with_basic_auth(
        &mut self,
        verify_password: fn(username: &str, password: &str) -> bool,
    ) -> &mut Self;

    fn with_token_auth(&mut self, verify_token: fn(token: &str) -> bool) -> &mut Self;
}

impl WithHttpAuth for Server {
    fn with_basic_auth(
        &mut self,
        verify_password: fn(username: &str, password: &str) -> bool,
    ) -> &mut Self {
        self.with(AuthMiddleware::new(BasicAuthScheme::new(verify_password)));
        self
    }

    fn with_token_auth(&mut self, verify_token: fn(token: &str) -> bool) -> &mut Self {
        self.with(AuthMiddleware::new(BearerAuthScheme::new(verify_token)));
        self
    }
}

fn get_basic_auth_unuathorized_response() -> Response {
    let mut res = Response::new(StatusCode::Unauthorized);
    res.insert_header("WWW-Authenticate", "Basic");
    res
}

fn get_basic_auth_forbiden_response() -> Response {
    let mut res = Response::new(StatusCode::Forbidden);
    res.insert_header("WWW-Authenticate", "Basic");
    res
}

#[async_trait]
impl<ImplScheme> Middleware for AuthMiddleware<ImplScheme>
where
    ImplScheme: Scheme + Send + Sync + 'static,
{
    async fn handle(&self, req: Request, next: Next<'_>) -> crate::Result {
        let auth_header = req.header(ImplScheme::header_name());
        if auth_header.is_none() {
            info!("no auth header, proceeding");
            return Ok(get_basic_auth_unuathorized_response());
        }
        let value: Vec<_> = auth_header.unwrap().into_iter().collect();

        if value.is_empty() {
            info!("empty auth header, proceeding");
            return Ok(get_basic_auth_unuathorized_response());
        }

        if value.len() > 1 && ImplScheme::should_401_on_multiple_values() {
            error!("multiple auth headers, bailing");
            return Ok(get_basic_auth_unuathorized_response());
        }

        for value in value {
            let value = value.as_str();
            if !value.starts_with(ImplScheme::scheme_name()) {
                continue;
            }
            let auth_param = &value[ImplScheme::scheme_name().len()..];

            info!("saw auth header, attempting to auth");

            return match self.scheme.authenticate(auth_param).await? {
                Some(_bool) => Ok(next.run(req).await),
                _ => return Ok(get_basic_auth_forbiden_response()),
            };
        }

        Ok(get_basic_auth_unuathorized_response())
    }
}

#[async_trait]
pub trait Scheme {
    type Request: Any + Send + Sync;

    async fn authenticate(&self, auth_param: &str) -> Result<Option<bool>>;

    fn should_401_on_multiple_values() -> bool {
        true
    }

    fn should_403_on_bad_auth() -> bool {
        true
    }

    fn header_name() -> &'static str {
        "Authorization"
    }

    fn scheme_name() -> &'static str;
}

pub struct BasicAuthScheme {
    pub(crate) verify_password: fn(username: &str, password: &str) -> bool,
}

impl BasicAuthScheme {
    pub fn new(verify_password: fn(username: &str, password: &str) -> bool) -> Self {
        Self {
            verify_password: verify_password,
        }
    }
}

pub struct BasicAuthRequest {
    pub username: String,
    pub password: String,
}

#[async_trait]
impl Scheme for BasicAuthScheme {
    type Request = BasicAuthRequest;

    async fn authenticate(&self, auth_param: &str) -> Result<Option<bool>> {
        let bytes = base64::decode(auth_param);
        if bytes.is_err() {
            // This is invalid. Fail the request.
            return Err(http_types::Error::from_str(
                StatusCode::Unauthorized,
                "Basic auth param must be valid base64.",
            ));
        }

        let as_utf8 = String::from_utf8(bytes.unwrap());
        if as_utf8.is_err() {
            // You know the drill.
            return Err(http_types::Error::from_str(
                StatusCode::Unauthorized,
                "Basic auth param base64 must contain valid utf-8.",
            ));
        }

        let as_utf8 = as_utf8.unwrap();
        let parts: Vec<_> = as_utf8.split(':').collect();

        if parts.len() < 2 {
            return Err(http_types::Error::from_str(
                StatusCode::Unauthorized,
                "Basic auth must contain both username and password",
            ));
        }

        let (username, password) = (parts[0], parts[1]);

        if (self.verify_password)(username, password) {
            return Ok(Some(true));
        }

        return Err(http_types::Error::from_str(
            StatusCode::Unauthorized,
            "username and password doesn't match",
        ));
    }

    fn scheme_name() -> &'static str {
        "Basic "
    }
}

pub struct BearerAuthScheme {
    pub(crate) verify_token: fn(token: &str) -> bool,
}

impl BearerAuthScheme {
    pub fn new(verify_token: fn(token: &str) -> bool) -> Self {
        Self {
            verify_token: verify_token,
        }
    }
}

pub struct BearerAuthRequest {
    pub token: String,
}

#[async_trait]
impl Scheme for BearerAuthScheme {
    type Request = BearerAuthRequest;

    async fn authenticate(&self, auth_param: &str) -> Result<Option<bool>> {
        let bytes = base64::decode(auth_param);
        if bytes.is_err() {
            // This is invalid. Fail the request.
            return Err(http_types::Error::from_str(
                StatusCode::Unauthorized,
                "Basic auth param must be valid base64.",
            ));
        }

        let as_utf8 = String::from_utf8(bytes.unwrap());
        if as_utf8.is_err() {
            // You know the drill.
            return Err(http_types::Error::from_str(
                StatusCode::Unauthorized,
                "Basic auth param base64 must contain valid utf-8.",
            ));
        }

        let as_utf8 = as_utf8.unwrap();
        let parts: Vec<_> = as_utf8.split(':').collect();

        if parts.len() < 1 {
            return Err(http_types::Error::from_str(
                StatusCode::Unauthorized,
                "Bearer auth must contain token",
            ));
        }

        let token = parts[0];

        if (self.verify_token)(token) {
            return Ok(Some(true));
        }

        return Err(http_types::Error::from_str(
            StatusCode::Unauthorized,
            "token is expired or invalid",
        ));
    }

    fn scheme_name() -> &'static str {
        "Bearer "
    }
}
