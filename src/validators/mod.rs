mod todos_validator;

use result::Result;

pub trait Validator<T> {
  fn validated(self) -> Result<T>;
}
