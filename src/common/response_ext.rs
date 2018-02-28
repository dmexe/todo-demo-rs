use serde::Serialize;
use serde_json;
use hyper::{Response, StatusCode};
use hyper::header::{ContentLength, ContentType};

pub trait ResponseExt {
  fn json<T>(self, value: &T) -> Response
  where
    T: Serialize + 'static;

  fn not_found(self) -> Response;
}

impl ResponseExt for Response {
  fn json<T>(self, value: &T) -> Response
  where
    T: Serialize + 'static,
  {
    let body = serde_json::to_string(&value).expect("json serialization cannot be fail");
    let len = body.len();

    self
      .with_body(body)
      .with_header(ContentType::json())
      .with_header(ContentLength(len as u64))
  }

  fn not_found(self) -> Response {
    self
      .with_status(StatusCode::NotFound)
      .with_header(ContentLength(0))
  }
}
