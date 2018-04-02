#[macro_use] extern crate log;
#[macro_use] extern crate diesel;
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
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};
use diesel::Connection;


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
    let listen_port = environ.get_command_listener_port();
    let db_url = environ.get_storage_location();

    let db_conn = diesel::pg::PgConnection::establish(&db_url)
        .expect(&format!("Error connecting to {}", db_url));

    let (tx, rx) = mpsc::channel();

    launch_command_listener(tx.clone(), listen_port);
    let (info, sender_tx) = launch_client(tx.clone(), &api_key);

    let mut runner = Runner::new(db_conn, sender_tx, info);
    loop {
        runner.run(&rx);
    }
}

fn launch_command_listener(tx: Sender<HandleableMessages>, listen_port: String) {
    thread::spawn(move || {
        let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), listen_port.parse::<u16>().unwrap());
        let listener = TcpListener::bind(addr).expect("Could not create command listener");
        for stream in listener.incoming() {
            match stream {
                Ok(recv) => {
                    debug!("Received command contents from Slack: {:?}", recv);
                    tx.send(HandleableMessages::PrintScoreboard);
                },
                Err(e) => panic!("Command listener server has died: {}", e)
            }
        }
    });
}

fn launch_client(tx: Sender<HandleableMessages>, api_key: &str) -> (SlackInfo, ::slack::Sender) {
    debug!("Launching client");

    let client = match slack::RtmClient::login(&api_key) {
        Ok(client) => client,
        Err(e) => panic!("Could not connect to Slack client: {}", e),
    };

    let info = SlackInfo::new(client.start_response());
    let sender_tx = client.sender().clone();

    thread::spawn(move || {
        let mut handler = denghandler::DengHandler::new(tx);
        match client.run(&mut handler) {
            Ok(_) => debug!("Gracefully closed connection"),
            Err(e) => error!("Ungraceful termination due to error: {}", e),
        }
    });

    (info, sender_tx)
}