use std::sync::mpsc::Receiver;
use std::thread;
use daycycle::*;
use deng::Deng;
use std::sync::{Arc, Mutex};
use dengstorage;
use diesel::PgConnection;

pub struct Runner {
    dengs: Vec<Deng>,
    day_cycle: Arc<Mutex<DayCycle>>,
    tx: ::slack::Sender,
    info: ::slackinfo::SlackInfo,
    db_conn: PgConnection
}

// TODO: rename
pub enum Broadcast {
    Deng(String),
    NonDeng(String),
    PrintScoreboard,
}

impl Runner {
    pub fn new(db_conn: PgConnection, tx: ::slack::Sender, info: ::slackinfo::SlackInfo) -> Self {

        let dengs = ::dengstorage::load(&db_conn);

        // Start the day immediately
        let day_cycle = Runner::start_day();

        Runner {
            dengs,
            day_cycle,
            tx,
            info,
            db_conn
        }
    }

    pub fn run(&mut self, rx: &Receiver<Broadcast>) {
        match rx.recv().expect("Receiver channel broken!") {
            Broadcast::Deng(user_id) => self.handle_deng(user_id),
            Broadcast::NonDeng(user_id) => self.handle_non_deng(user_id),
            Broadcast::PrintScoreboard => self.handle_scoreboard()
        };
    }

    fn handle_deng(&mut self, user_id: String) {
        let (first_deng, has_denged_today) = {
            let day = self.day_cycle.lock().unwrap();
            (day.first_deng(), day.has_denged_today(&user_id))
        };

        let deng = dengstorage::store_success(&self.db_conn, user_id,
                                              first_deng, has_denged_today);
        self.dengs.push(deng);
    }

    fn handle_non_deng(&mut self, user_id: String) {
        let deng = dengstorage::store_failure(&self.db_conn, user_id);
        self.dengs.push(deng);
    }

    fn handle_scoreboard(&mut self) {
        debug!("Sending scoreboard printout");
        if let Err(e) = ::send::send_raw_msg(&self.tx, &self.info.meta_channel_id) {
            error!("{}", e);
        }
    }

    // TODO: rewrite this
    fn start_day() -> Arc<Mutex<DayCycle>> {
        let day = Arc::new(Mutex::new(DayCycle::start()));

        let handle = day.clone();
        thread::spawn(move || loop {
            let sleep_time = {
                handle.lock()
                    .expect("Could not modify day cycle")
                    .time_to_end()
            };
            thread::sleep(sleep_time);
            *handle.lock().unwrap() = DayCycle::start();
        });

        debug!("Launched time reset thread");

        day
    }
}
