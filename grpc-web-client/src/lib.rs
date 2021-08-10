mod call;

use crate::call::Encoding::Base64;
use bytes::Bytes;
pub use call::Encoding;
use call::GrpcWebCall;
use core::fmt;
use core::task::{Context, Poll};
use futures::{Future, Stream};
use http::{request::Request, response::Response, HeaderMap, HeaderValue};
use http_body::Body;
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
            .version(reqwest::Version::HTTP_11); // TODO: Shouldn't this be HTTP_2 at least?

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

        eprintln!("Request Body:\n{:#?}", &body);

        let req = req.body(hyper::Body::from(body));

        eprintln!("{:#?}", &req);

        let response = req.send().await.unwrap();

        eprintln!("{:#?}", &response);

        let mut res = Response::builder().status(response.status());
        let enc = Encoding::from_content_type(response.headers());

        for kv in response.headers() {
            res = res.header(kv.0, HeaderValue::from(kv.1));
        }

        let body = GrpcWebCall::client_response(ReadableStreamBody::new(response), enc);

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

struct ReadableStreamBody {
    stream: Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>>>>,
}

impl ReadableStreamBody {
    fn new(inner: reqwest::Response) -> Self {
        ReadableStreamBody {
            stream: Box::pin(inner.bytes_stream()),
        }
    }
}

impl Body for ReadableStreamBody {
    type Data = Bytes;
    type Error = reqwest::Error;

    fn poll_data(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        self.stream.as_mut().poll_next(cx)
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        Poll::Ready(Ok(None))
    }

    fn is_end_stream(&self) -> bool {
        false
    }
}

// WARNING: these are required to satisfy the Body and Error traits, make sure about thread-safety.

unsafe impl Sync for ReadableStreamBody {}
unsafe impl Send for ReadableStreamBody {}

unsafe impl Sync for ClientError {}
unsafe impl Send for ClientError {}
