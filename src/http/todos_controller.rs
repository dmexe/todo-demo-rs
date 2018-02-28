use hyper::{Request, Response};
use futures::Future;

use result::Error;
use db::{NewTodo, QueryTodos, TodosRepo, UpdateTodo};
use common::{FuturesExt, RequestExt, ResponseExt};
use validators::Validator;

type BoxFuture<T> = Box<Future<Item = T, Error = Error>>;

pub struct TodosController {
  todos_repo: TodosRepo,
}

impl TodosController {
  pub fn new(todos_repo: TodosRepo) -> Self {
    TodosController { todos_repo }
  }

  pub fn call_create(&self, req: Request) -> BoxFuture<Response> {
    let repo = self.todos_repo.clone();

    req
      .json::<NewTodo>()
      .and_then(|it| it.validated())
      .and_then(move |it| repo.insert(it))
      .inspect(|it| info!("created {:?}", it))
      .map(|it| Response::new().json(&it))
      .into_boxed()
  }

  pub fn call_query(&self, req: Request) -> BoxFuture<Response> {
    let repo = self.todos_repo.clone();

    req
      .json::<QueryTodos>()
      .and_then(|it| it.validated())
      .and_then(move |it| repo.query(it))
      .map(|it| Response::new().json(&it))
      .into_boxed()
  }

  pub fn call_update(&self, req: Request) -> BoxFuture<Response> {
    let repo = self.todos_repo.clone();

    req
      .json::<UpdateTodo>()
      .and_then(|it| it.validated())
      .and_then(move |it| repo.update(it))
      .inspect(|it| info!("updated {:?}", it))
      .map(|it| Response::new().json(&it))
      .into_boxed()
  }
}
