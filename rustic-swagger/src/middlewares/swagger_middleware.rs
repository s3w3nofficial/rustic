use std::sync::Arc;

use http_types::Mime;
use rustic::{Server, Response, StatusCode, Request};
use utoipa::openapi::OpenApi;
use utoipa_swagger_ui::Config;

pub trait WithSwagger {
    fn with_swagger(&mut self, op: OpenApi) -> &mut Self;
}

impl WithSwagger for Server {

    fn with_swagger(&mut self, op: OpenApi) -> &mut Self {

        self.at("/api-docs/openapi.json")
            .get(move |_| {
                let op_clone = op.clone();
                async move {
                    let body = serde_json::to_string(&op_clone).unwrap_or_else(|_| "{}".to_string());
                    let mut response = Response::new(StatusCode::Ok);
                    response.set_body(rustic::Body::from_string(body));
                    Ok(response)
                }
            });
        
        self.at("/swagger-ui/*")
            .get(|request: Request| async move {

                let config = Arc::new(Config::from("/api-docs/openapi.json"));
                let path = request.url().path().to_string();
                let tail = path.strip_prefix("/swagger-ui/").unwrap();

                match utoipa_swagger_ui::serve(tail, config) {
                    Ok(swagger_file) => swagger_file
                        .map(|file| {
                            let mut response = Response::new(StatusCode::Ok);
                            response.set_body(file.bytes.to_vec());
                            response.set_content_type(file.content_type.parse::<Mime>()?);

                            Ok(response)
                        })
                        .unwrap_or_else(|| Ok(Response::new(StatusCode::NotFound))),
                    Err(_error) => Ok(Response::new(StatusCode::InternalServerError)),
                }
            });
    
        self
    }
}