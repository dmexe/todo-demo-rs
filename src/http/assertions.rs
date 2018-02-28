use spectral::{AssertionFailure, Spec};
use hyper::{Response, StatusCode};
use hyper::header::ContentType;

pub trait ResponseAssertions
where
  Self: Sized,
{
  fn has_json(&self) -> &Self;
  fn has_status(&self, expected: StatusCode) -> &Self;
  fn is_ok(&self) -> &Self {
    self.has_status(StatusCode::Ok);
    self
  }
}

impl<'s> ResponseAssertions for Spec<'s, Response> {
  fn has_json(&self) -> &Self {
    let subject: &Response = self.subject;

    match subject.headers().get::<ContentType>() {
      Some(content_type) if content_type != &ContentType::json() => {
        AssertionFailure::from_spec(self)
          .with_expected(format!(
            "response with content type {}",
            ContentType::json()
          ))
          .with_actual(format!("<{}>", content_type))
          .fail();
      }
      None => {
        AssertionFailure::from_spec(self)
          .with_expected(format!(
            "response with content type {}",
            ContentType::json()
          ))
          .with_actual("<None>".to_string())
          .fail();
      }
      _ => {}
    }

    self
  }

  fn has_status(&self, expected: StatusCode) -> &Self {
    let subject: &Response = self.subject;

    if subject.status() != expected {
      AssertionFailure::from_spec(self)
        .with_expected(format!("response with status code {}", expected))
        .with_actual(format!("<{}>", subject.status()))
        .fail();
    }

    self
  }
}
