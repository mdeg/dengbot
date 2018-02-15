extern crate slack;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate log;
extern crate simplelog;

mod constants;
mod denghandler;
mod deng;
mod dengstorage;
mod slackinfo;
mod environment;

use environment::*;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
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

    // Initialise for correct environment
    let env = std::env::var(constants::RUN_MODE_ENV_VAR).unwrap_or("local".to_string());
    let environ: Box<Init> = match env.as_ref() {
        "server" => Box::new(ServerEnvironment),
        "local" | _ => Box::new(LocalEnvironment)
    };
    environ.init_logger();
    environ.announce();
    let dengs = environ.init_storage();

    // Start the day immediately
    let current_day = Arc::new(Mutex::new(calculate_new_day()));
    start_day_reset(current_day.clone());

    let mut handler = denghandler::DengHandler::new(dengs, current_day);

    debug!("Launching client");

    match slack::RtmClient::login_and_run(&api_key, &mut handler) {
        Ok(_) => debug!("Gracefully closed connection"),
        Err(e) => error!("Ungraceful termination due to error: {}", e)
    }
}

fn start_day_reset(current_day_handle: Arc<Mutex<std::ops::Range<Duration>>>) {
    std::thread::spawn(move || {
        loop {
            let sleep_time = {
                let mut current_day = current_day_handle.lock().unwrap();
                current_day.end - current_day.start
            };
            std::thread::sleep(sleep_time);
            *current_day_handle.lock().unwrap() = calculate_new_day();
        }
    });
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