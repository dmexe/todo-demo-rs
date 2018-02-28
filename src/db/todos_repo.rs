use chrono::{NaiveDateTime, Utc};
use diesel;
use futures_cpupool::{CpuFuture, CpuPool};
use diesel::prelude::*;

use result::Error;

use super::functions::last_insert_id;
use super::schema::todos;
use super::ConnectionPool;
use super::Paginated;

/// Todo item model, mapping to `todos` table
#[derive(Queryable, Debug, Clone, Serialize)]
pub struct Todo {
  pub id: i64,
  pub text: String,
  pub done: bool,
  pub created_at: NaiveDateTime,
  pub updated_at: NaiveDateTime,
}

/// Model for a new todo item that contains only fields required for todo item creation
#[derive(Debug, Clone, Deserialize)]
pub struct NewTodo {
  pub text: String,
}

/// Query parameters
#[derive(Clone, Debug, Deserialize)]
pub struct QueryTodos {
  pub next: Option<i64>,
  pub limit: Option<u8>,
  pub text: Option<String>,
}

/// Complete todo parameters
#[derive(Clone, Debug, Deserialize)]
pub struct UpdateTodo {
  pub id: i64,
  pub text: Option<String>,
  pub done: Option<bool>,
}

/// Todo's repository
#[derive(Clone)]
pub struct TodosRepo {
  conn_pool: ConnectionPool,
  cpu_pool: CpuPool,
}

impl TodosRepo {
  pub fn new(conn_pool: ConnectionPool, cpu_pool: CpuPool) -> Self {
    TodosRepo {
      conn_pool,
      cpu_pool,
    }
  }

  /// Create a new todo after that query and return it from db.
  pub fn insert(&self, new_todo: NewTodo) -> CpuFuture<Todo, Error> {
    let TodosRepo {
      conn_pool,
      cpu_pool,
    } = self.clone();

    cpu_pool.spawn_fn(move || {
      let conn = conn_pool.get().map_err(Error::from)?;
      let time = Utc::now().naive_utc();

      diesel::insert_into(todos::table)
        .values(&(
          todos::text.eq(new_todo.text.as_str()),
          todos::done.eq(false),
          todos::created_at.eq(&time),
          todos::updated_at.eq(&time),
        ))
        .execute(&*conn)
        .map_err(Error::from)?;

      let todo_id = diesel::select(last_insert_id)
        .first::<i64>(&*conn)
        .map_err(Error::from)?;

      todos::table
        .filter(todos::id.eq(todo_id))
        .first::<Todo>(&*conn)
        .map_err(Error::from)
    })
  }

  /// Query todo items, return paginated result
  pub fn query(&self, query: QueryTodos) -> CpuFuture<Paginated<Todo>, Error> {
    let TodosRepo {
      conn_pool,
      cpu_pool,
    } = self.clone();

    cpu_pool.spawn_fn(move || {
      let conn = conn_pool.get().map_err(Error::from)?;
      let mut stmt = todos::table.order(todos::id.desc()).into_boxed();

      match query.next {
        Some(next) if next > 0 => stmt = stmt.filter(todos::id.lt(next)),
        _ => {}
      }

      match query.limit {
        Some(limit) if limit <= 10 && limit > 0 => stmt = stmt.limit(i64::from(limit)),
        _ => stmt = stmt.limit(10),
      }

      match query.text {
        Some(ref text) if !text.is_empty() => {
          let pattern = format!("%{}%", text);
          stmt = stmt.filter(todos::text.like(pattern));
        }
        _ => {}
      }

      let items = stmt.load::<Todo>(&*conn).map_err(Error::from)?;
      let next = items.last().map(|it| it.id);

      Ok(Paginated { next, items })
    })
  }

  /// Find a single todo item
  #[allow(dead_code)]
  pub fn find(&self, id: i64) -> CpuFuture<Todo, Error> {
    let TodosRepo {
      conn_pool,
      cpu_pool,
    } = self.clone();

    cpu_pool.spawn_fn(move || {
      let conn = conn_pool.get().map_err(Error::from)?;

      todos::table
        .filter(todos::id.eq(id))
        .first::<Todo>(&*conn)
        .map_err(Error::from)
    })
  }

  /// Update completion status and/or text for a single todo item
  pub fn update(&self, update: UpdateTodo) -> CpuFuture<Todo, Error> {
    let TodosRepo {
      conn_pool,
      cpu_pool,
    } = self.clone();

    cpu_pool.spawn_fn(move || {
      let conn = conn_pool.get().map_err(Error::from)?;

      let mut todo = todos::table
        .filter(todos::id.eq(update.id))
        .first::<Todo>(&*conn)
        .map_err(Error::from)?;

      todo = Todo {
        updated_at: Utc::now().naive_utc(),
        ..todo
      };

      if let Some(text) = update.text {
        todo.text = text;
      }

      if let Some(done) = update.done {
        todo.done = done;
      }

      diesel::update(todos::table.filter(todos::id.eq(todo.id)))
        .set((
          todos::text.eq(todo.text.as_str()),
          todos::done.eq(todo.done),
          todos::updated_at.eq(&todo.updated_at),
        ))
        .execute(&*conn)
        .map_err(Error::from)
        .map(|_| todo)
    })
  }

  #[cfg(test)]
  fn truncate(&self) -> Result<(), Error> {
    let conn = self.conn_pool.get().map_err(Error::from)?;
    diesel::delete(todos::table)
      .execute(&*conn)
      .map_err(Error::from)?;

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use spectral::prelude::*;
  use futures::Future;

  use config::Config;
  use db::connection_pool;

  #[test]
  fn should_insert_new_todo() {
    let todos_repo = create_repo();
    let new_todo = NewTodo {
      text: "foo".to_string(),
    };
    let result = todos_repo.insert(new_todo).wait();

    let todo = assert_that(&result).is_ok().subject;

    assert_that(&todo.done).is_false();
    assert_that(&todo.text).is_equal_to("foo".to_string());
  }

  #[test]
  fn should_query_todos() {
    let todos_repo = create_repo();
    todos_repo.truncate().unwrap();

    todos_repo
      .insert(NewTodo {
        text: "foo".to_string(),
      })
      .wait()
      .unwrap();
    todos_repo
      .insert(NewTodo {
        text: "bar".to_string(),
      })
      .wait()
      .unwrap();

    let query = QueryTodos {
      next: None,
      limit: None,
      text: None,
    };
    let todos = todos_repo.query(query).wait().unwrap();

    assert_that(&todos.next).is_some();
    assert_that(&todos.items).has_length(2);

    let text: Vec<_> = todos.items.iter().map(|it| it.text.as_str()).collect();
    assert_that(&text).is_equal_to(vec!["bar", "foo"]);
  }

  #[test]
  fn should_update_todo() {
    let todos_repo = create_repo();
    let todo = todos_repo
      .insert(NewTodo {
        text: "foo".to_string(),
      })
      .wait()
      .unwrap();

    assert_that(&todo.done).is_false();

    {
      let update = UpdateTodo {
        id: todo.id,
        text: None,
        done: Some(true),
      };
      let todo = todos_repo.update(update).wait().unwrap();
      assert_that(&todo.done).is_true();
      assert_that(&todo.text).is_equal_to("foo".to_string());
    }

    {
      let update = UpdateTodo {
        id: todo.id,
        text: Some("bar".to_string()),
        done: None,
      };
      let todo = todos_repo.update(update).wait().unwrap();
      assert_that(&todo.done).is_true();
      assert_that(&todo.text).is_equal_to("bar".to_string());
    }
  }

  fn create_repo() -> TodosRepo {
    let cfg = Config::default();
    let conn_pool = connection_pool(&cfg.database_url, cfg.pool_size);
    let cpu_pool = cfg.create_cpu_pool();
    TodosRepo::new(conn_pool, cpu_pool)
  }
}
