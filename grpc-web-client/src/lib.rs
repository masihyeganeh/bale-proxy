mod call;

use crate::call::Encoding::Base64;
use bytes::Bytes;
pub use call::Encoding;
use call::GrpcWebCall;
use core::{
    fmt,
    task::{Context, Poll},
};
use futures::Future;
use http::{request::Request, response::Response, HeaderValue};
use std::{error::Error, pin::Pin};
use tonic::{body::BoxBody, client::GrpcService};

#[derive(Debug, Clone, PartialEq)]
pub enum ClientError {
    Err,
    FetchFailed(Bytes),
}

impl Error for ClientError {}
impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone)]
pub struct Client {
    base_uri: String,
    encoding: Encoding,
}

impl Client {
    pub fn new(base_uri: String) -> Self {
        Client {
            base_uri,
            encoding: Encoding::None,
        }
    }

    pub fn new_with_encoding(base_uri: String, encoding: Encoding) -> Self {
        Client { base_uri, encoding }
    }

    async fn request(self, rpc: Request<BoxBody>) -> Result<Response<BoxBody>, ClientError> {
        let mut uri = rpc.uri().to_string();
        uri.insert_str(0, &self.base_uri);

        let mut req = reqwest::Client::new()
            .post(uri)
            .version(reqwest::Version::HTTP_11);

        for (k, v) in rpc.headers().iter() {
            if k.as_str() == "content-type" {
                continue;
            }
            req = req.header(k.as_str(), v.to_str().unwrap());
        }
        req = req.header(
            "user-agent",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:90.0) Gecko/20100101 Firefox/90.0",
        );
        req = req.header("x-user-agent", "grpc-web-javascript/0.1");
        req = req.header("x-grpc-web", "1");
        req = req.header("content-type", self.encoding.to_content_type());

        let mut body = hyper::body::to_bytes(rpc.into_body()).await.unwrap();
        if self.encoding == Base64 {
            body = Bytes::from(base64::encode(body))
        }

        let req = req.body(hyper::Body::from(body));

        eprintln!("{:#?}", &req);

        let response = req.send().await.unwrap();

        eprintln!("{:#?}", &response);

        let mut res = Response::builder().status(response.status());
        let enc = Encoding::from_content_type(response.headers());

        for kv in response.headers() {
            res = res.header(kv.0, HeaderValue::from(kv.1));
        }

        let body =
            GrpcWebCall::client_response(hyper::Body::from(response.bytes().await.unwrap()), enc);

        Ok(res.body(BoxBody::new(body)).unwrap())
    }
}

impl GrpcService<BoxBody> for Client {
    type ResponseBody = BoxBody;
    type Error = ClientError;
    type Future = Pin<Box<dyn Future<Output = Result<Response<BoxBody>, ClientError>>>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, rpc: Request<BoxBody>) -> Self::Future {
        Box::pin(self.clone().request(rpc))
    }
}
