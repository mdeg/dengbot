#[macro_use] extern crate log;
#[macro_use] extern crate diesel;
extern crate simplelog;
extern crate slack;
extern crate dotenv;
#[macro_use] extern crate dotenv_codegen;

mod denghandler;
mod deng;
mod dengstorage;
mod slackinfo;
mod daycycle;
mod runner;
mod send;

use runner::*;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;
use slackinfo::SlackInfo;
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};
use diesel::Connection;
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

fn init_logger(path: &str) {
    let log_file = File::create(path).expect("Could not create log file");
    CombinedLogger::init(vec![
        TermLogger::new(LevelFilter::Debug, Config::default())
            .expect("Could not initialise terminal logger"),
        WriteLogger::new(LevelFilter::Trace, Config::default(), log_file),
    ]).expect("Could not initialise combined logger");
}

fn launch_command_listener(tx: Sender<Broadcast>, listen_port: &str) {
    let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), listen_port.parse::<u16>().unwrap());
    thread::spawn(move || {
        let listener = TcpListener::bind(addr).expect("Could not create command listener");
        for stream in listener.incoming() {
            match stream {
                Ok(recv) => {
                    debug!("Received command contents from Slack: {:?}", recv);
                    tx.send(Broadcast::PrintScoreboard);
                },
                Err(e) => panic!("Command listener server has died: {}", e)
            }
        }
    });
}

fn launch_client(tx: Sender<Broadcast>, api_key: &str) -> (SlackInfo, ::slack::Sender) {
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