use hyper::{Error as HyperError, Get, Post, Request, Response, StatusCode};
use hyper::server::{Http, NewService, Service};
use futures::{future, Future};
use std::io;
use std::error::Error as StdError;

use db::TodosRepo;
use result::Error;
use common::{FuturesExt, ResponseExt};

use super::todos_controller::TodosController;

#[derive(Clone)]
pub struct Server {
  todos_repo: TodosRepo,
}

impl NewService for Server {
  type Request = Request;
  type Response = Response;
  type Error = HyperError;
  type Instance = Server;

  fn new_service(&self) -> io::Result<Self::Instance> {
    Ok(self.clone())
  }
}

impl Service for Server {
  type Request = Request;
  type Response = Response;
  type Error = HyperError;
  type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

  fn call(&self, req: Self::Request) -> Self::Future {
    self.handle(req).then(handle_api_err).into_boxed()
  }
}

impl Server {
  pub fn new(todos_repo: TodosRepo) -> Self {
    Server { todos_repo }
  }

  pub fn listen(self, http_port: u16) {
    let addr = ([0, 0, 0, 0], http_port).into();
    let listener = Http::new()
      .bind(&addr, self)
      .expect("cannot bind to address");
    info!(
      "listen on http://{}",
      listener.local_addr().expect("cannot lookup local address")
    );

    listener.run().expect("cannot handle requests");
  }

  fn handle(&self, req: Request) -> Box<Future<Item = Response, Error = Error>> {
    let todos_repo = self.todos_repo.clone();

    match (req.method(), req.path()) {
      (&Post, "/todos/create") => TodosController::new(todos_repo).call_create(req),
      (&Post, "/todos/update") => TodosController::new(todos_repo).call_update(req),
      (&Post, "/todos/query") => TodosController::new(todos_repo).call_query(req),
      (&Get, "/health") => {
        let body = json!({"ok": true});
        future::ok(Response::new().json(&body)).into_boxed()
      }
      _ => {
        warn!("not found {} {}", req.method(), req.path());
        future::ok(Response::new().not_found()).into_boxed()
      }
    }
  }
}

fn handle_api_err(result: Result<Response, Error>) -> Result<Response, HyperError> {
  let err = match result {
    Err(err) => err,
    Ok(ok) => return Ok(ok),
  };

  let mut resp = Response::new();
  match err {
    Error::JsonParse(_) => resp.set_status(StatusCode::BadRequest),
    Error::RecordNotFound => resp.set_status(StatusCode::NotFound),
    Error::Validation(_) => resp.set_status(StatusCode::PreconditionFailed),
    _ => resp.set_status(StatusCode::InternalServerError),
  };

  let body = json!({"error": err.to_string(), "description": err.description()});

  Ok(resp.json(&body))
}

#[cfg(test)]
mod tests {
  use super::*;
  use spectral::prelude::*;
  use hyper::{Body, Uri};
  use serde_json;
  use serde_json::Value as JsonValue;
  use std::str::FromStr;
  use futures::Stream;

  use db;
  use config::Config;
  use http::assertions::*;

  #[test]
  fn should_be_healthy() {
    let svc = create_server();
    let resp = get(&svc, "/health");

    assert_that(&resp).is_ok().has_json();
  }

  #[test]
  fn should_handle_todos_create() {
    let svc = create_server();
    let resp = post(&svc, "/todos/create", json!({"text": "foo"}));

    assert_that(&resp).is_ok().has_json();

    let todo = json(resp);
    assert_that(&todo.get("id")).is_some();
    assert_that(&todo.get("text"))
      .is_some()
      .is_equal_to(&JsonValue::String("foo".to_string()));
    assert_that(&todo.get("done"))
      .is_some()
      .is_equal_to(&JsonValue::Bool(false));
    assert_that(&todo.get("created_at")).is_some();
    assert_that(&todo.get("updated_at")).is_some();
  }

  #[test]
  fn should_update_todo() {
    let svc = create_server();

    let resp = post(&svc, "/todos/create", json!({"text": "foo"}));
    assert_that(&resp).is_ok().has_json();

    let todo = json(resp);
    let id: i64 = match todo["id"] {
      JsonValue::Number(ref n) => n.as_i64().expect("id must be i64"),
      ref err => panic!("unexpected value {}, expecting number", err),
    };

    let resp = post(&svc, "/todos/update", json!({"id": id, "done": true}));
    assert_that(&resp).is_ok().has_json();

    let todo = json(resp);
    assert_that(&todo.get("id")).is_some();
    assert_that(&todo.get("text")).is_some();
    assert_that(&todo.get("done"))
      .is_some()
      .is_equal_to(&JsonValue::Bool(true));
    assert_that(&todo.get("created_at")).is_some();
    assert_that(&todo.get("updated_at")).is_some();
  }

  #[test]
  fn should_query_todos() {
    let svc = create_server();

    let resp = post(&svc, "/todos/create", json!({"text": "foo"}));
    assert_that(&resp).is_ok().has_json();

    let resp = post(&svc, "/todos/query", json!({}));
    assert_that(&resp).is_ok().has_json();

    let query = json(resp);
    assert_that(&query.get("next")).is_some();
    assert_that(&query.get("items")).is_some();
  }

  #[test]
  fn should_handle_not_found_error() {
    let svc = create_server();

    let resp = post(&svc, "/not/found", json!({}));
    assert_that(&resp).has_status(StatusCode::NotFound);
  }

  #[test]
  fn should_handle_json_error() {
    let svc = create_server();

    let resp = post(&svc, "/todos/create", json!({}));
    assert_that(&resp)
      .has_status(StatusCode::BadRequest)
      .has_json();
  }

  fn json(resp: Response) -> JsonValue {
    let chunk = resp.body().concat2().wait().unwrap();
    serde_json::from_slice(&chunk).unwrap()
  }

  fn post(svc: &Server, path: &str, body: JsonValue) -> Response {
    let mut req: Request<Body> = Request::new(Post, Uri::from_str(path).unwrap());
    let body = serde_json::to_string(&body).unwrap();
    req.set_body(body);

    svc.call(req).wait().unwrap()
  }

  fn get(svc: &Server, path: &str) -> Response {
    let req = Request::new(Get, Uri::from_str(path).unwrap());
    svc.call(req).wait().unwrap()
  }

  fn create_server() -> Server {
    let cfg = Config::default();
    let cpu_pool = cfg.create_cpu_pool();
    let conn_pool = db::connection_pool(&cfg.database_url, cfg.pool_size);
    let todos_repo = db::TodosRepo::new(conn_pool, cpu_pool);

    Server::new(todos_repo)
  }
}
