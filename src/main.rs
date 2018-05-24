#[macro_use] extern crate log;
#[macro_use] extern crate diesel;
extern crate simplelog;
extern crate slack;
extern crate chrono;
extern crate rand;
extern crate dotenv;
#[macro_use] extern crate dotenv_codegen;
extern crate futures;
extern crate hyper;
extern crate slack_hook;
extern crate r2d2;
extern crate r2d2_diesel;

mod denghandler;
mod storage;
mod types;
mod slackinfo;
mod daycycle;
mod runner;
mod send;
mod command;

use runner::*;
use types::Broadcast;
use dotenv::dotenv;
use std::fs::File;
use simplelog::*;

fn main() {

    dotenv().ok();
    let api_key = dotenv!("SLACK_API_KEY");
    let db_url = dotenv!("DB_URL");
    let listen_port = dotenv!("LISTEN_PORT");
    let log_path = dotenv!("LOG_PATH");

    init_logger(&log_path);

    debug!("Starting up dengbot");

    let conn_mgr = r2d2_diesel::ConnectionManager::new(db_url);
    let db_conn_pool = r2d2::Pool::builder().build(conn_mgr).unwrap();

    debug!("Connected to database");

    let mut runner = Runner::new(db_conn_pool.clone());
    let rx = runner.start(&api_key, &listen_port);
    runner.run(&rx);
}

fn init_logger(path: &str) {
    let log_file = File::create(path).expect("Could not create log file");

    if let Some(term_logger) = TermLogger::new(LevelFilter::Debug, Config::default()) {
        CombinedLogger::init(vec![
            term_logger,
            WriteLogger::new(LevelFilter::Trace, Config::default(), log_file),
        ]).expect("Could not initialise combined logger");
    } else {
        if let Err(e) = WriteLogger::init(LevelFilter::Trace, Config::default(), log_file) {
            error!("Could not initialise write logger: {}", e);
        }
    }
}