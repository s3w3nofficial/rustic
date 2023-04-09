use async_std::path::PathBuf as AsyncPathBuf;
use async_trait::async_trait;
use http_types::{Body, StatusCode};
use kv_log_macro::{info, warn};
use std::{
    ffi::OsStr,
    io,
    path::{Path, PathBuf},
};

use crate::{Endpoint, Request, Response};

pub(crate) struct ServeDir {
    prefix: String,
    dir: PathBuf,
}

impl ServeDir {
    pub(crate) fn new(prefix: String, dir: PathBuf) -> Self {
        Self { prefix, dir }
    }
}

#[async_trait]
impl Endpoint for ServeDir {
    async fn call(&self, req: Request) -> crate::Result {
        let path = req.url().path();
        let path = path
            .strip_prefix(&self.prefix.trim_end_matches('*'))
            .unwrap();
        let path = path.trim_start_matches('/');
        let mut file_path = self.dir.clone();
        for p in Path::new(path) {
            if p == OsStr::new(".") {
                continue;
            } else if p == OsStr::new("..") {
                file_path.pop();
            } else {
                file_path.push(&p);
            }
        }

        info!("Requested file: {:?}", file_path);

        let file_path = AsyncPathBuf::from(file_path);
        if !file_path.starts_with(&self.dir) {
            warn!("Unauthorized attempt to read: {:?}", file_path);
            Ok(Response::new(StatusCode::Forbidden))
        } else {
            match Body::from_file(&file_path).await {
                Ok(body) => {
                    let mut res = Response::new(StatusCode::Ok);
                    res.set_body(body);

                    Ok(res)
                }
                Err(e) if e.kind() == io::ErrorKind::NotFound => {
                    warn!("File not found: {:?}", &file_path);
                    Ok(Response::new(StatusCode::NotFound))
                }
                Err(e) => Err(e.into()),
            }
        }
    }
}
