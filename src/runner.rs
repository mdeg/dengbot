use std::sync::mpsc::Receiver;
use std::thread;
use daycycle::*;
use storage;
use diesel::pg::PgConnection;
use types::*;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use slackinfo::SlackInfo;
use denghandler;
use slack;
use hyper;
use command;
use r2d2_diesel::ConnectionManager;
use r2d2::Pool as ConnectionPool;

pub struct Runner {
    day_cycle: DayCycle,
    db_conn_pool: ConnectionPool<ConnectionManager<PgConnection>>
}

impl Runner {
    pub fn new(db_conn_pool: ConnectionPool<ConnectionManager<PgConnection>>) -> Self {
        Runner {
            day_cycle: DayCycle::new(),
            db_conn_pool
        }
    }

    pub fn start(&mut self, api_key: &str, listen_port: &str) {
        let (tx, rx) = mpsc::channel();
        let (info_tx, info_rx) = mpsc::channel();

        self.launch_client(tx.clone(), info_tx.clone(), api_key);
        self.launch_command_listener(info_rx, listen_port);
        self.run(rx);
    }

    fn launch_command_listener(&self, info_rx: Receiver<SlackInfo>, listen_port: &str) {
        let addr = format!("0.0.0.0:{}", listen_port).parse().unwrap();
        let pool = self.db_conn_pool.clone();

        info!("Starting command listener on {}", &addr);

        // Wait for the client thread to connect to the server and give us our info
        let info = info_rx.recv().expect("Client died without sending us Slack info!");

        thread::spawn(move || {
            let server = hyper::server::Http::new()
                .bind(&addr, move || {
                    let db_conn = pool.get().unwrap();
                    Ok(command::CommandListener::new(info.clone(), db_conn))
                })
                .expect("Could not create hyper command listener server");

            server.run().unwrap();
        });
    }

    fn launch_client(&self, tx: Sender<Broadcast>, info_tx: Sender<SlackInfo>, api_key: &str) {
        let key = String::from(api_key);

        thread::spawn(move || {
            info!("Connecting to Slack server");

            let client = slack::RtmClient::login(&key).expect("Could not connect to Slack!");
            let mut handler = denghandler::DengHandler::new(tx, info_tx);

            info!("Running Slack client");

            match client.run(&mut handler) {
                Ok(_) => info!("Gracefully closed connection"),
                Err(e) => error!("Ungraceful termination due to error: {}", e)
            }
        });
    }

    fn run(&mut self, rx: Receiver<Broadcast>) {
        loop {
            match rx.recv().expect("Receiver channel broken!") {
                Broadcast::Deng(user_id) => self.handle_deng(user_id),
                Broadcast::NonDeng(user_id) => self.handle_non_deng(user_id)
            };
        }
    }

    fn handle_deng(&mut self, user_id: String) {
        if self.day_cycle.has_ended() {
            self.day_cycle.new_day();
        }

        let first_deng = self.day_cycle.first_deng();
        let denged_today = self.day_cycle.has_denged_today(&user_id);
        self.day_cycle.register_deng(&user_id);

        storage::store_success(&self.db_conn_pool.get().unwrap(), user_id, first_deng, denged_today);
    }

    fn handle_non_deng(&mut self, user_id: String) {
        storage::store_failure(&self.db_conn_pool.get().unwrap(), user_id);
    }
}