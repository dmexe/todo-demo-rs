use hyper::Request;
use serde::de::DeserializeOwned;
use serde_json;
use futures::{Future, Stream};

use result::Error;
use common::FuturesExt;

pub trait RequestExt {
  fn json<T>(self) -> Box<Future<Item = T, Error = Error>>
  where
    T: DeserializeOwned + 'static;
}

impl RequestExt for Request {
  fn json<T>(self) -> Box<Future<Item = T, Error = Error>>
  where
    T: DeserializeOwned + 'static,
  {
    self
      .body()
      .concat2()
      .map_err(Error::from)
      .and_then(|chunk| serde_json::from_slice(&chunk).map_err(Error::from))
      .into_boxed()
  }
}
