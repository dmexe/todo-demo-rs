use r2d2;
use r2d2_diesel;
use diesel;
use url::Url;
use std::time::Duration;

pub type ConnectionPool = r2d2::Pool<r2d2_diesel::ConnectionManager<diesel::MysqlConnection>>;

/// Create a new connection pool using given database url and pool's size.
pub fn connection_pool(database_url: &Url, pool_size: u32) -> ConnectionPool {
  let manager = r2d2_diesel::ConnectionManager::new(database_url.as_str());
  r2d2::Pool::builder()
    .connection_timeout(Duration::from_secs(1))
    .max_size(pool_size)
    .min_idle(Some(pool_size))
    .build(manager)
    .expect("cannot connect to mysql server")
}

#[cfg(test)]
mod tests {
  use super::*;
  use config::Config;
  use std::ops::Deref;

  #[test]
  fn should_create_a_new_connection_pool() {
    let cfg = Config::default();
    let pool = connection_pool(&cfg.database_url, cfg.pool_size);

    pool.get().unwrap().deref();
  }
}
