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
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::ops::Range;
use rand::{Rng, thread_rng};
use std::fmt::{self, Debug, Formatter};
use chrono::{DateTime, Utc, NaiveDateTime};

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
    day: Range<Duration>,
    denged_today: Vec<String>,
}

impl DayCycle {
    pub fn new() -> Self {
        DayCycle {
            day: Self::generate_day(Self::now()),
            denged_today: vec![],
        }
    }

    pub fn new_day(&mut self) {
        self.denged_today.clear();

        // Start a new day, but ensure the end is in the future
        while self.has_ended() {
            self.day = Self::generate_day(self.day.end);
        }

        info!("Starting new day @ {:?}", self.day);
    }

    pub fn has_ended(&self) -> bool {
        self.day.end < Self::now()
    }

    fn generate_day(start: Duration) -> Range<Duration> {
        // In 24 hours time, with up to an hour variance either way
        let mut rng = thread_rng();
        let hours = 24.0 + rng.gen_range(-1.0, 1.0);

        Range {
            start,
            end: start + Duration::from_secs(hours as u64 * 60 * 60),
        }
    }

    fn now() -> Duration {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time has gone backwards")
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
        let start_dt = NaiveDateTime::from_timestamp(self.day.start.as_secs() as i64, 0);
        let local_start_dt: DateTime<Utc> = DateTime::from_utc(start_dt, Utc);

        let end_dt = NaiveDateTime::from_timestamp(self.day.end.as_secs() as i64, 0);
        let local_end_dt: DateTime<Utc> = DateTime::from_utc(end_dt, Utc);

        write!(f, "day starts @ {}. Day ends @ {}", local_start_dt.to_rfc2822(), local_end_dt.to_rfc2822())
    }
}