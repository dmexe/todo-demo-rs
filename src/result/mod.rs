use diesel::result::Error as DieselError;
use r2d2::Error as ConnectionPoolError;
use std::error::Error as StdError;
use std::fmt;
use std::result::Result as StdResult;
use serde_json::Error as SerdeJsonError;
use hyper::Error as HyperError;

#[derive(Debug)]
pub enum Error {
  /// Indicates mysql errors.
  MySql(DieselError),
  /// Indicates a db's connection errors.
  MySqlConnection(ConnectionPoolError),
  /// Indicates that record not found in database.
  RecordNotFound,
  /// Indicates a json parsing error
  JsonParse(SerdeJsonError),
  /// Indicates http server error
  HttpServer(HyperError),
  /// Indicates invalid input data
  Validation(String),
}

#[allow(dead_code)]
pub type Result<T> = StdResult<T, Error>;

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      Error::MySql(ref err) => write!(f, "Error::MySql {}", err),
      Error::MySqlConnection(ref err) => write!(f, "Error::MySqlConnection {}", err),
      Error::RecordNotFound => f.write_str("Error::RecordNotFound"),
      Error::JsonParse(ref err) => write!(f, "Error::JsonParse {}", err),
      Error::HttpServer(ref err) => write!(f, "Error::HttpServer {}", err),
      Error::Validation(ref err) => write!(f, "Error::Validation {}", err),
    }
  }
}

impl StdError for Error {
  fn description(&self) -> &str {
    match *self {
      Error::MySql(ref err) => err.description(),
      Error::MySqlConnection(ref err) => err.description(),
      Error::RecordNotFound => "record not found in database",
      Error::JsonParse(ref err) => err.description(),
      Error::HttpServer(ref err) => err.description(),
      Error::Validation(_) => "input data validation error",
    }
  }

  fn cause(&self) -> Option<&StdError> {
    match *self {
      Error::MySql(ref err) => Some(err),
      Error::MySqlConnection(ref err) => Some(err),
      Error::JsonParse(ref err) => Some(err),
      Error::HttpServer(ref err) => Some(err),
      _ => None,
    }
  }
}

impl From<DieselError> for Error {
  fn from(err: DieselError) -> Self {
    match err {
      DieselError::NotFound => Error::RecordNotFound,
      _ => Error::MySql(err),
    }
  }
}

impl From<ConnectionPoolError> for Error {
  fn from(err: ConnectionPoolError) -> Self {
    Error::MySqlConnection(err)
  }
}

impl From<SerdeJsonError> for Error {
  fn from(err: SerdeJsonError) -> Self {
    Error::JsonParse(err)
  }
}

impl From<HyperError> for Error {
  fn from(err: HyperError) -> Self {
    Error::HttpServer(err)
  }
}
