use std::sync::mpsc::{self, Receiver};
use std::thread;
use storage;
use diesel::pg::PgConnection;
use types::*;
use denghandler::DengHandler;
use slack;
use hyper;
use command;
use r2d2_diesel::ConnectionManager;
use r2d2::Pool as ConnectionPool;
use std::time::Duration;
use rand::{Rng, thread_rng};
use std::fmt::{self, Debug, Formatter};
use chrono::{self, FixedOffset, DateTime, Datelike, Timelike};

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

        self.launch_command_listener(&info_rx, listen_port);
        self.run(&rx);
    }

    fn launch_command_listener(&self, info_rx: &Receiver<SlackInfo>, listen_port: &str) {
        let addr = format!("0.0.0.0:{}", listen_port).parse().expect("Listen port is not valid");
        let pool = self.db_conn_pool.clone();

        info!("Starting command listener on {}", &addr);

        // Wait for the client thread to connect to the server and give us our info
        let info = info_rx.recv().expect("Client died without sending us Slack info!");

        thread::spawn(move || {
            loop {
                let (info_in, pool_in) = (info.clone(), pool.clone());
                let server = hyper::server::Http::new()
                    .bind(&addr, move || {
                        match pool_in.get() {
                            Ok(db_conn) => Ok(command::CommandListener::new(info_in.clone(), db_conn)),
                            Err(e) => Err(::std::io::Error::new(::std::io::ErrorKind::TimedOut, e))
                        }
                    });

                match server {
                    Ok(serv) => {
                        match serv.run() {
                            Ok(()) => info!("Command server ended gracefully"),
                            Err(e) => error!("Command server died: {}", e)
                        }
                    },
                    Err(e) => error!("Could not create server: {}", e)
                }

                // Sleep for 10 seconds before attempting to reconnect
                warn!("Command server has been lost. Attempting reconnect in 10 seconds...");
                thread::sleep(Duration::from_secs(10));
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
                warn!("Connection to Slack has died. Attempting to restablish in 10 seconds...");
                thread::sleep(Duration::from_secs(10));
            }
        });
    }

    fn run(&mut self, rx: &Receiver<Broadcast>) {
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

        match &self.db_conn_pool.get() {
            Ok(conn) => {
                if let Err(e) = storage::store_success(conn, user_id, first_deng, denged_today) {
                    error!("Could not store successful deng: {}", e);
                }
            },
            Err(e) => error!("Could not get connection to DB: {}", e)
        }
    }

    fn handle_non_deng(&mut self, user_id: String) {
        match &self.db_conn_pool.get() {
            Ok(conn) => {
                if let Err(e) = storage::store_failure(conn, user_id) {
                    error!("Could not store failed deng: {}", e);
                }
            },
            Err(e) => error!("Could not get connection to DB: {}", e)
        }
    }
}

pub struct DayCycle {
    start: DateTime<FixedOffset>,
    end: DateTime<FixedOffset>,
    denged_today: Vec<String>,
}

impl DayCycle {
    pub fn new() -> Self {
        let start = Self::generate_day();
        let end = Self::calculate_end(start);

        info!("Starting new day @ {:?} and ending @ {:?}", start, end);

        DayCycle {
            start,
            end,
            denged_today: vec![],
        }
    }

    pub fn new_day(&mut self) {
        self.denged_today.clear();

        self.start = Self::generate_day();
        self.end = Self::calculate_end(self.start);

        info!("Starting new day @ {:?} and ending @ {:?}", self.start, self.end);
    }

    pub fn has_ended(&self) -> bool {
        self.end < Self::now()
    }

    // End in one days' time
    // Add in up to 15 minutes' fuzz to prevent gaming the system
    fn calculate_end(start: DateTime<FixedOffset>) -> DateTime<FixedOffset> {
        let mut rng = thread_rng();
        let hours = 24.0 + rng.gen_range(0.0, 0.25);
        let seconds = hours * 60.0 * 60.0;
        start + chrono::Duration::seconds(seconds as i64)
    }

    fn generate_day() -> DateTime<FixedOffset> {
        let mut time = Self::now();
        // Rewind to start the day yesterday if we are currently in the morning
        if time.hour() < 9 {
            time = time.with_day(time.day() - 1).unwrap();
        }
        // Set to 9am
        time.with_hour(9).unwrap()
            .with_minute(0).unwrap()
            .with_second(0).unwrap()
    }

    // Returns Perth time (GMT+8)
    fn now() -> DateTime<FixedOffset> {
        let offset = FixedOffset::east(8 * 3600);
        chrono::Utc::now().with_timezone(&offset)
    }

    pub fn has_denged_today(&self, user_id: &str) -> bool {
        !self.denged_today.iter().any(|id| id.as_str() == user_id)
    }

    pub fn first_deng(&self) -> bool {
        self.denged_today.is_empty()
    }

    pub fn register_deng(&mut self, user_id: &str) {
        self.denged_today.push(String::from(user_id));
    }
}

impl Debug for DayCycle {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "day starts @ {}. Day ends @ {}", self.start.to_rfc2822(), self.end.to_rfc2822())
    }
}