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
    let args: Vec<String> = std::env::args().collect();
    let api_key = match args.len() {
        0 | 1 => panic!("No API key in arguments! Usage: dengbot <TOKEN>"),
        x => args[x - 1].clone()
    };

    let log_file = File::create("dengbot.log").expect("Could not initialise write logger");
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Debug, Config::default()).expect("Could not initialise terminal logger"),
            WriteLogger::new(LevelFilter::Trace, Config::default(), log_file)
        ]
    ).expect("Could not initialise combined logger");

    let dengs = dengstorage::read_dengs().unwrap_or_else(|_| dengstorage::create_storage());

    // Start the day immediately
    let current_day = Arc::new(Mutex::new(calculate_new_day()));
    // Set up a thread to reset the day
    let time_handle = current_day.clone();
    ::std::thread::spawn(move || {
        loop {
            let sleep_time = {
                let mut current_day = time_handle.lock().unwrap();
                current_day.start - current_day.end
            };
            ::std::thread::sleep(sleep_time);
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

fn calculate_new_day() -> ::std::ops::Range<Duration> {
    let day_start = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time has gone backwards");
    // TODO: add randomness?
    let day_period = ::std::ops::Range { start: day_start, end: day_start + Duration::from_secs(86400) };
    debug!("Day reset: start {:?} end {:?}", day_period.start, day_period.end);
    day_period
}