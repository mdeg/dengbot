use std::sync::mpsc::Receiver;
use std::thread;
use daycycle::*;
use std::sync::{Arc, Mutex};
use storage;
use diesel::Connection;
use diesel::pg::PgConnection;
use types::*;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use slackinfo::SlackInfo;
use denghandler;
use slack;
use hyper;
use command;

pub struct Runner {
    day_cycle: Arc<Mutex<DayCycle>>,
    db_conn: PgConnection
}

impl Runner {
    pub fn new(db_conn: PgConnection) -> Self {
        Runner {
            day_cycle: Runner::start_day_cycle(),
            db_conn
        }
    }

    pub fn start(&self, api_key: &str, listen_port: &str) -> Receiver<Broadcast> {
        let (tx, rx) = mpsc::channel();
        let info = self.launch_client(tx.clone(), api_key);
        self.launch_command_listener(info.clone(), listen_port);
        rx
    }

    fn launch_command_listener(&self, info: Arc<SlackInfo>, listen_port: &str) {
        // TODO: better URL parsing - get URL from system
        let addr = format!("192.168.1.72:{}", listen_port).parse().unwrap();

        thread::spawn(move || {
            let server = hyper::server::Http::new()
                .bind(&addr, move || {
                    // TODO: implement r2d2 and use a connection pool to avoid duplicating these connections
                    let db_url = dotenv!("DB_URL");
                    let db_conn = PgConnection::establish(&db_url)
                        .expect(&format!("Error connecting to {}", db_url));
                    
                    Ok(command::CommandListener::new(info.clone(), db_conn))
                })
                .unwrap();

            server.run().unwrap();
        });
    }

    fn launch_client(&self, tx: Sender<Broadcast>, api_key: &str) -> Arc<SlackInfo> {
        debug!("Launching Slack client");

        let client = match slack::RtmClient::login(&api_key) {
            Ok(client) => client,
            Err(e) => panic!("Could not connect to Slack client: {}", e),
        };

        let info = Arc::new(SlackInfo::from_start_response(client.start_response()));

        thread::spawn(move || {
            let mut handler = denghandler::DengHandler::new(tx);
            debug!("Connecting to Slack server");
            match client.run(&mut handler) {
                Ok(_) => debug!("Gracefully closed connection"),
                Err(e) => error!("Ungraceful termination due to error: {}", e)
            }
        });

        info
    }

    pub fn run(&mut self, rx: &Receiver<Broadcast>) {
        match rx.recv().expect("Receiver channel broken!") {
            Broadcast::Deng(user_id) => self.handle_deng(user_id),
            Broadcast::NonDeng(user_id) => self.handle_non_deng(user_id)
        };
    }

    fn handle_deng(&mut self, user_id: String) {
        let (first_deng, has_denged_today) = {
            let mut day = self.day_cycle.lock().unwrap();
            let (first_deng, denged_today) = (day.first_deng(), day.has_denged_today(&user_id));
            day.register_deng(&user_id);
            (first_deng, denged_today)
        };

        storage::store_success(&self.db_conn, user_id, first_deng, has_denged_today);
    }

    fn handle_non_deng(&mut self, user_id: String) {
        storage::store_failure(&self.db_conn, user_id);
    }

    fn start_day_cycle() -> Arc<Mutex<DayCycle>> {
        let day = Arc::new(Mutex::new(DayCycle::new()));
        let day_obj_handle = day.clone();

        thread::spawn(move || {
            debug!("Launched time reset thread");
            loop {
                let sleep_time = {
                    let day = &mut *day_obj_handle.lock().expect("Could not modify day cycle");
                    // Generate a new day
                    day.new_day();
                    debug!("Starting new day: {:?}", day);
                    day.time_to_end()
                };
                thread::sleep(sleep_time);
            }
        });

        day
    }
}
