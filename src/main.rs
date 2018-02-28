extern crate chrono;
#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate env_logger;
extern crate futures;
extern crate futures_cpupool;
extern crate hyper;
#[macro_use]
extern crate log;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate url;

#[cfg(test)]
extern crate spectral;

mod result;
mod common;
mod config;
mod db;
mod http;
mod validators;

use config::Config;
use dotenv::dotenv;

fn main() {
  dotenv().ok();
  env_logger::init();

  let cfg = Config::default();
  info!("using {:?}", cfg);

  let cpu_pool = cfg.create_cpu_pool();
  let conn_pool = db::connection_pool(&cfg.database_url, cfg.pool_size);
  let todos_repo = db::TodosRepo::new(conn_pool, cpu_pool);

  http::Server::new(todos_repo).listen(cfg.http_port);
}
