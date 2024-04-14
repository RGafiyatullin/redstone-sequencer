use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use http::{Request, Response};
use reth_rpc::{Claims, JwtSecret};
use tower::{Layer, Service};

#[derive(Debug, Clone)]
pub struct EngineAuthLayer(Arc<JwtSecret>);

impl EngineAuthLayer {
    pub fn new(jwt_secret: JwtSecret) -> Self {
        Self(Arc::new(jwt_secret))
    }
}

impl<S> Layer<S> for EngineAuthLayer {
    type Service = AddJwtHeader<S>;
    fn layer(&self, inner: S) -> Self::Service {
        AddJwtHeader {
            inner,
            secret: Arc::clone(&self.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AddJwtHeader<S> {
    inner: S,
    secret: Arc<JwtSecret>,
}

impl<S, B> Service<Request<B>> for AddJwtHeader<S>
where
    S: Service<Request<B>, Response = Response<B>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<B>) -> Self::Future {
        let now = SystemTime::now();
        let iat = now.duration_since(UNIX_EPOCH).unwrap().as_secs();
        let token = self
            .secret
            .encode(&Claims { iat, exp: None })
            .expect("FIXME");
        req.headers_mut().insert(
            http::header::AUTHORIZATION,
            format!("Bearer {}", token).try_into().expect("FIXME"),
        );

        self.inner.call(req)
    }
}
