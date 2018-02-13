extern crate slack;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate log;
extern crate simplelog;

mod constants;
mod denghandler;
mod deng;
mod dengstorage;
mod slackinfo;

use simplelog::*;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::fs::File;
use std::sync::{Arc, Mutex};

fn main() {

    let api_key = match std::env::var(constants::TOKEN_ENV_VAR) {
        Ok(token) => token,
        Err(_) => {
            println!("Could not find environment variable {}. Falling back to arguments", constants::TOKEN_ENV_VAR);
            let args: Vec<String> = std::env::args().collect();
            match args.len() {
                0 | 1 => panic!("No API key in arguments! Usage: dengbot <TOKEN>"),
                x => args[x - 1].clone()
            }
        }
    };

    // TODO: clean this up
    let environment = std::env::var(constants::RUN_MODE_ENV_VAR).unwrap_or("local".to_string());
    let dengs = match environment.as_ref() {
        "server" => {
            init_server_logging();
            debug!("Starting in server environment");
            dengstorage::read_dengs_server().unwrap_or_else(|_| dengstorage::create_storage())
        },
        _ => {
            init_local_logging();
            debug!("Starting in local enviroment");
            dengstorage::read_dengs_local().unwrap_or_else(|_| dengstorage::create_storage())
        }
    };

    // Start the day immediately
    let current_day = Arc::new(Mutex::new(calculate_new_day()));
    // Set up a thread to reset the day
    let time_handle = current_day.clone();
    std::thread::spawn(move || {
        loop {
            let sleep_time = {
                let mut current_day = time_handle.lock().unwrap();
                current_day.end - current_day.start
            };
            std::thread::sleep(sleep_time);
            *time_handle.lock().unwrap() = calculate_new_day();
        }
    });

    let mut handler = denghandler::DengHandler::new(dengs, current_day);

    trace!("Launching client");
    match slack::RtmClient::login_and_run(&api_key, &mut handler) {
        Ok(_) => debug!("Gracefully closed connection"),
        Err(e) => error!("Ungraceful termination due to error: {}", e)
    }
}

fn init_server_logging() {
    let log_file = File::create("/tmp/dengbot.log").expect("Could not create log file");
    WriteLogger::init(LevelFilter::Trace, Config::default(), log_file).expect("Could not initialise write logger");
}

fn init_local_logging() {
    let log_file = File::create("./dengbot.log").expect("Could not create log file");
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Debug, Config::default()).expect("Could not initialise terminal logger"),
            WriteLogger::new(LevelFilter::Trace, Config::default(), log_file)
        ]
    ).expect("Could not initialise combined logger");
}

fn calculate_new_day() -> std::ops::Range<Duration> {
    let day_start = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time has gone backwards");
    // TODO: add randomness?
    let day_period = std::ops::Range { start: day_start, end: day_start + Duration::from_secs(86400) };
    // TODO: convert these times into local timezone for readability
    debug!("Today starts @ {:?}. Next day starts @ {:?}", day_period.start.as_secs(), day_period.end.as_secs());
    day_period
}