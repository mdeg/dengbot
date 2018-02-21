#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
extern crate simplelog;
extern crate slack;

mod denghandler;
mod deng;
mod dengstorage;
mod slackinfo;
mod environment;
mod daycycle;
mod runner;
mod send;

use environment::*;
use runner::*;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;
use slackinfo::SlackInfo;

pub static TOKEN_ENV_VAR: &'static str = "DENGBOT_TOKEN";
pub static RUN_MODE_ENV_VAR: &'static str = "DENGBOT_RUN_MODE";
pub static HEROKU_PORT_ENV_VAR: &'static str = "PORT";

fn main() {
    let api_key = match std::env::var(TOKEN_ENV_VAR) {
        Ok(token) => token,
        Err(_) => {
            println!("Could not find environment variable {}. Falling back to arguments", TOKEN_ENV_VAR);
            let args: Vec<String> = std::env::args().collect();
            match args.len() {
                0 | 1 => panic!("No API key in arguments! Usage: dengbot <TOKEN>"),
                x => args[x - 1].clone(),
            }
        }
    };

    // Initialise for correct environment
    let env = std::env::var(RUN_MODE_ENV_VAR).unwrap_or_else(|_| "local".to_string());
    let environ: Box<Init> = match env.as_ref() {
        "server" => Box::new(ServerEnvironment),
        "local" | _ => Box::new(LocalEnvironment),
    };
    environ.init_logger();
    environ.announce();
    // TODO: separate deng reads from storage initialisation
    let dengs = environ.init_storage();

    let (tx, rx) = mpsc::channel();

    let (info, sender_tx) = launch_client(tx.clone(), api_key);

    let mut runner = Runner::new(dengs, rx, sender_tx, info);
    runner.run();
}

fn launch_client(tx: Sender<HandleableMessages>, api_key: &str) -> (SlackInfo, ::slack::Sender) {
    debug!("Launching client");

    let client = slack::RtmClient::login(&api_key)

        match  {
        Ok(client) => client,
        Err(e) => panic!("Could not connect to Slack client: {}", e),
    };

    thread::spawn(move || {
        let mut handler = denghandler::DengHandler::new(tx);
        match client.run(&mut handler) {
            Ok(_) => debug!("Gracefully closed connection"),
            Err(e) => error!("Ungraceful termination due to error: {}", e),
        }
    });

    let info = SlackInfo::new(client.start_response());
    let sender_tx = client.sender().clone();

    (info, sender_tx)
}