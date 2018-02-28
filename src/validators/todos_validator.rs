use db::{NewTodo, QueryTodos, UpdateTodo};
use result::{Error, Result};
use super::Validator;

struct TodoText(Option<String>);
struct TodoId(i64);

impl Validator<TodoText> for TodoText {
  fn validated(self) -> Result<Self> {
    if let Some(text) = self.0.clone() {
      if text.is_empty() {
        return Err(Error::Validation("todo's text cannot be empty".to_string()));
      }

      if text.len() > 255 {
        return Err(Error::Validation(format!(
          "todo's text must be less then 255, got {}",
          text.len()
        )));
      }
    }

    Ok(self)
  }
}

impl Validator<TodoId> for TodoId {
  fn validated(self) -> Result<Self> {
    if self.0 <= 0 {
      return Err(Error::Validation(format!(
        "todo's id cannot be negative, got {}",
        self.0
      )));
    }

    Ok(self)
  }
}

impl Validator<NewTodo> for NewTodo {
  fn validated(self) -> Result<Self> {
    TodoText(Some(self.text.clone())).validated()?;
    Ok(self)
  }
}

impl Validator<UpdateTodo> for UpdateTodo {
  fn validated(self) -> Result<Self> {
    TodoText(self.text.clone()).validated()?;
    TodoId(self.id).validated()?;
    Ok(self)
  }
}

impl Validator<QueryTodos> for QueryTodos {
  fn validated(self) -> Result<Self> {
    TodoText(self.text.clone()).validated()?;
    Ok(self)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use spectral::prelude::*;
  use std::iter;

  #[test]
  fn should_validate_new_todo() {
    let subject = NewTodo {
      text: "text".to_string(),
    };
    assert_that(&subject.validated()).is_ok();

    let subject = NewTodo {
      text: "".to_string(),
    };
    assert_that(&subject.validated()).is_err();

    let subject = NewTodo {
      text: iter::repeat("x").take(256).collect::<String>(),
    };
    assert_that(&subject.validated()).is_err();
  }

  #[test]
  fn should_validate_update_todo() {
    let subject = UpdateTodo {
      id: 1,
      text: Some("text".to_string()),
      done: Some(false),
    };
    {
      let subject = subject.clone();
      assert_that(&subject.validated()).is_ok();
    }

    {
      let subject = UpdateTodo { id: -1, ..subject };
      assert_that(&subject.validated()).is_err();
    }

    {
      let subject = UpdateTodo {
        text: Some("".to_string()),
        ..subject
      };
      assert_that(&subject.validated()).is_err();
    }

    {
      let subject = UpdateTodo {
        text: Some(iter::repeat("x").take(256).collect::<String>()),
        ..subject
      };
      assert_that(&subject.validated()).is_err();
    }
  }

  #[test]
  fn should_validate_query_todo() {
    let subject = QueryTodos {
      next: None,
      limit: None,
      text: Some("text".to_string()),
    };
    {
      let subject = subject.clone();
      assert_that(&subject.validated()).is_ok();
    }

    {
      let subject = QueryTodos {
        text: Some("".to_string()),
        ..subject
      };
      assert_that(&subject.validated()).is_err();
    }

    {
      let subject = QueryTodos {
        text: Some(iter::repeat("x").take(256).collect::<String>()),
        ..subject
      };
      assert_that(&subject.validated()).is_err();
    }
  }
}
