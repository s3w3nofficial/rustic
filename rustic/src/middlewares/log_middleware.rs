use async_trait::async_trait;
use kv_log_macro::{info, warn, error};

use crate::{Middleware, Request, Next, Server};


pub struct LogMiddleware {

}

impl LogMiddleware {

    fn new() -> Self {
        Self {

        }
    }
}

#[async_trait]
impl Middleware for LogMiddleware {

    async fn handle(&self, request: Request, next: Next<'_>) -> crate::Result {

        let path = request.url().path().to_owned();
        let method = request.method().to_string();
        info!("<-- Request received", {
            method: method,
            path: path,
        });
        let start = std::time::Instant::now();
        let response = next.run(request).await;
        let status = response.status();
        if status.is_server_error() {
            if let Some(error) = response.error() {
                error!("Internal error --> Response sent", {
                    message: format!("{:?}", error),
                    error_type: error.type_name(),
                    method: method,
                    path: path,
                    status: format!("{} - {}", status as u16, status.canonical_reason()),
                    duration: format!("{:?}", start.elapsed()),
                });
            } else {
                error!("Internal error --> Response sent", {
                    method: method,
                    path: path,
                    status: format!("{} - {}", status as u16, status.canonical_reason()),
                    duration: format!("{:?}", start.elapsed()),
                });
            }
        } else if status.is_client_error() {
            if let Some(error) = response.error() {
                warn!("Client error --> Response sent", {
                    message: format!("{:?}", error),
                    error_type: error.type_name(),
                    method: method,
                    path: path,
                    status: format!("{} - {}", status as u16, status.canonical_reason()),
                    duration: format!("{:?}", start.elapsed()),
                });
            } else {
                warn!("Client error --> Response sent", {
                    method: method,
                    path: path,
                    status: format!("{} - {}", status as u16, status.canonical_reason()),
                    duration: format!("{:?}", start.elapsed()),
                });
            }
        } else {
            info!("--> Response sent", {
                method: method,
                path: path,
                status: format!("{} - {}", status as u16, status.canonical_reason()),
                duration: format!("{:?}", start.elapsed()),
            });
        }
        Ok(response)
    }
}

pub trait WithLogging {
    fn with_logging(&mut self) -> &mut Self;
}

impl WithLogging for Server {

    fn with_logging(&mut self) -> &mut Self {
        self.with(LogMiddleware::new());
        self
    }
}