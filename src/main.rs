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

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let api_key = match args.len() {
        0 | 1 => panic!("No API key in arguments! Usage: dengbot <TOKEN>"),
        x => args[x - 1].clone()
    };

    TermLogger::init(LevelFilter::Debug, Config::default()).expect("Could not initialise logger");

    let dengs = dengstorage::read_dengs().unwrap_or_else(|_| dengstorage::create_storage());

    // TODO: proper seed time
    let seed_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time has gone backwards");

    let current_day = ::std::ops::Range { start: seed_time, end: seed_time + Duration::from_secs(86400) };
    debug!("Day range: {:?}", current_day);

    let mut handler = denghandler::DengHandler::new(dengs, current_day);

    match slack::RtmClient::login_and_run(&api_key, &mut handler) {
        Ok(_) => debug!("Gracefully closed connection"),
        Err(e) => error!("Ungraceful termination due to error: {}", e)
    }
}