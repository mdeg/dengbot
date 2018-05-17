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

mod denghandler;
mod storage;
mod types;
mod slackinfo;
mod daycycle;
mod runner;
mod send;

use runner::*;
use std::sync::mpsc;
use types::Broadcast;
use std::sync::mpsc::Sender;
use std::thread;
use std::sync::Arc;
use slackinfo::SlackInfo;
use diesel::Connection;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::fs::File;
use simplelog::*;
use futures::future::Future;

fn main() {

    dotenv().ok();
    let api_key = dotenv!("SLACK_API_KEY");
    let db_url = dotenv!("DB_URL");
    let listen_port = dotenv!("LISTEN_PORT");
    let log_path = dotenv!("LOG_PATH");

    init_logger(&log_path);

    debug!("Starting up dengbot");

    let db_conn = PgConnection::establish(&db_url)
            .expect(&format!("Error connecting to {}", db_url));

    debug!("Connected to database");

    let (tx, rx) = mpsc::channel();

    let (info, sender_tx) = launch_client(tx.clone(), &api_key);

    launch_command_listener(info.clone(), listen_port);

    let mut runner = Runner::new(db_conn, sender_tx);
    loop {
        runner.run(&rx);
    }
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

struct CommandListener {
    info: Arc<SlackInfo>,
    db_conn: PgConnection,
    hook_client: slack_hook::Slack
}

impl CommandListener {
    fn new(info: Arc<SlackInfo>) -> Self {
        let db_url = dotenv!("DB_URL");
        // TODO: use a connection pool for this
        let db_conn = PgConnection::establish(&db_url)
            .expect(&format!("Error connecting to {}", db_url));

        Self {
            info,
            db_conn,
            hook_client: slack_hook::Slack::new(dotenv!("WEBHOOK_URL")).unwrap()
        }
    }

    pub fn handle_scoreboard(&self) {
        debug!("Sending scoreboard printout");
        let dengs = storage::load(&self.db_conn);
        if let Err(e) = ::send::send_scoreboard(&self.hook_client, &self.info, &dengs) {
            error!("Could not send scoreboard: {}", e);
        }
    }
}

impl hyper::server::Service for CommandListener {
    type Request = hyper::Request;
    type Response = hyper::Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        self.handle_scoreboard();

        Box::new(futures::future::ok(
            hyper::Response::new().with_status(hyper::StatusCode::Ok)
        ))
    }
}

fn launch_command_listener(info: Arc<SlackInfo>, listen_port: &str) {
    // TODO: better URL parsing - get URL from system
    let addr = format!("192.168.1.72:{}", listen_port).parse().unwrap();
    thread::spawn(move || {
        let server = hyper::server::Http::new()
            .bind(&addr, move || Ok(CommandListener::new(info.clone())))
            .unwrap();

        server.run().unwrap();
    });
}

fn launch_client(tx: Sender<Broadcast>, api_key: &str) -> (Arc<SlackInfo>, ::slack::Sender) {
    debug!("Launching Slack client");

    let client = match slack::RtmClient::login(&api_key) {
        Ok(client) => client,
        Err(e) => panic!("Could not connect to Slack client: {}", e),
    };

    let info = Arc::new(SlackInfo::from_start_response(client.start_response()));
    let sender_tx = client.sender().clone();

    thread::spawn(move || {
        let mut handler = denghandler::DengHandler::new(tx);
        debug!("Connecting to Slack server");
        match client.run(&mut handler) {
            Ok(_) => debug!("Gracefully closed connection"),
            Err(e) => error!("Ungraceful termination due to error: {}", e)
        }
    });

    (info, sender_tx)
}