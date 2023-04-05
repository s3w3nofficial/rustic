use routefinder::Captures;

pub struct Request {
    pub(crate) req: http_types::Request,
    pub(crate) route_params: Vec<Captures<'static, 'static>>,
}

impl Request {
    pub(crate) fn new(
        req: http_types::Request,
        route_params: Vec<Captures<'static, 'static>>,
    ) -> Self {
        Self {
            req,
            route_params,
        }
    }

    pub async fn body_json<T: serde::de::DeserializeOwned>(&mut self) -> crate::Result<T> {
        let res = self.req.body_json().await?;
        Ok(res)
    }
}