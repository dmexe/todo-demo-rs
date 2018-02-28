use std::env;
use url::Url;
use futures_cpupool;

const HTTP_PORT_ENV: &str = "HTTP_PORT";
const DEFAULT_HTTP_PORT: &str = "3000";

const DATABASE_URL: &str = "DATABASE_URL";
const DEFAULT_DATABASE_URL: &str = "mysql://root@127.0.0.1:3306/todos";

const POOL_SIZE: &str = "POOL_SIZE";
const DEFAULT_POOL_SIZE: &str = "10";

/// An application's configuration variables.
#[derive(Debug, Clone)]
pub struct Config {
  /// http port which listening to
  pub http_port: u16,
  /// thread pool size
  pub pool_size: u32,
  /// database url which connecting to
  pub database_url: Url,
}

impl Config {
  /// Create a new cpu pool
  pub fn create_cpu_pool(&self) -> futures_cpupool::CpuPool {
    futures_cpupool::Builder::new()
      .pool_size(self.pool_size as usize)
      .name_prefix("cpu-")
      .create()
  }
}

impl Default for Config {
  /// Create a config from environment variables
  fn default() -> Self {
    let http_port: u16 = env::var(HTTP_PORT_ENV.to_string())
      .unwrap_or_else(|_| DEFAULT_HTTP_PORT.to_string())
      .parse()
      .expect("cannot parse http port");

    let database_url =
      env::var(DATABASE_URL.to_string()).unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_string());

    let database_url = Url::parse(database_url.as_str()).expect("cannot parse database url");

    let pool_size: u32 = env::var(POOL_SIZE.to_string())
      .unwrap_or_else(|_| DEFAULT_POOL_SIZE.to_string())
      .parse()
      .expect("cannot parse pool size");

    Config {
      http_port,
      pool_size,
      database_url,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use spectral::prelude::*;

  #[test]
  fn should_be_default_values() {
    let cfg = Config::default();
    assert_that(&cfg.http_port).is_equal_to(3000);
    assert_that(&cfg.database_url.as_str()).is_equal_to(DEFAULT_DATABASE_URL);
    assert_that(&cfg.pool_size).is_equal_to(10);
  }
}
