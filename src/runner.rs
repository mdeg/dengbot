use std::sync::mpsc::Receiver;
use std::thread;
use daycycle::*;
use storage;
use diesel::pg::PgConnection;
use types::*;
use std::sync::mpsc;
use slackinfo::SlackInfo;
use denghandler::DengHandler;
use slack;
use hyper;
use std::time::Duration;
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

        let handler = DengHandler::new(tx.clone(), info_tx.clone());
        self.launch_client(handler, String::from(api_key));

        self.launch_command_listener(info_rx, listen_port);
        self.run(rx);
    }

    fn launch_command_listener(&self, info_rx: Receiver<SlackInfo>, listen_port: &str) {
        let addr = format!("0.0.0.0:{}", listen_port).parse().expect("Listen port is not valid");
        let pool = self.db_conn_pool.clone();

        info!("Starting command listener on {}", &addr);

        // Wait for the client thread to connect to the server and give us our info
        let info = info_rx.recv().expect("Client died without sending us Slack info!");

        thread::spawn(move || {
            let server = hyper::server::Http::new()
                .bind(&addr, move || {
                    // TODO: this should not unwrap
                    let db_conn = pool.get().unwrap();
                    Ok(command::CommandListener::new(info.clone(), db_conn))
                })
                .expect("Could not create command listener server");

            match server.run() {
                Ok(()) => info!("Command server ended gracefully"),
                Err(e) => error!("Command server died: {}", e)
            }
        });
    }

    fn launch_client(&self, mut handler: DengHandler, key: String) {
        thread::spawn(move || {
            loop {
                info!("Connecting to Slack...");

                match slack::RtmClient::login(&key) {
                    Ok(client) => {
                        info!("Login succeeded. Running Slack client...");
                        match client.run(&mut handler) {
                            Ok(_) => info!("Gracefully closed connection"),
                            Err(e) => error!("Ungraceful termination due to error: {}", e)
                        }
                    },
                    Err(e) => error!("Could not log in to Slack client: {}", e)
                }

                // Sleep for 10 seconds before attempting to reconnect
                warn!("Connection to Slack has been lost. Attempting reconnect in 10 seconds...");
                thread::sleep(Duration::from_secs(10));
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