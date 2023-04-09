use crate::{Body, Endpoint, Request, Response, Result, StatusCode};
use std::io;
use std::path::Path;

use async_std::path::PathBuf as AsyncPathBuf;
use async_trait::async_trait;
use kv_log_macro::warn;

pub(crate) struct ServeFile {
    path: AsyncPathBuf,
}

impl ServeFile {
    pub(crate) fn init(path: impl AsRef<Path>) -> io::Result<Self> {
        let file = path.as_ref().to_owned().canonicalize()?;
        Ok(Self {
            path: AsyncPathBuf::from(file),
        })
    }
}

#[async_trait]
impl Endpoint for ServeFile {
    async fn call(&self, _: Request) -> Result {
        match Body::from_file(&self.path).await {
            Ok(body) => {
                let mut res = Response::new(StatusCode::Ok);
                res.set_body(body);

                Ok(res)
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                warn!("File not found: {:?}", &self.path);
                Ok(Response::new(StatusCode::NotFound))
            }
            Err(e) => Err(e.into()),
        }
    }
}
