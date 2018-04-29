use std::sync::mpsc::Receiver;
use std::thread;
use daycycle::*;
use std::sync::{Arc, Mutex};
use storage;
use diesel::PgConnection;
use types::*;

pub struct Runner {
    day_cycle: Arc<Mutex<DayCycle>>,
    tx: ::slack::Sender,
    info: ::slackinfo::SlackInfo,
    db_conn: PgConnection
}

impl Runner {
    pub fn new(db_conn: PgConnection, tx: ::slack::Sender, info: ::slackinfo::SlackInfo) -> Self {
        Runner {
            day_cycle: Runner::start_day_cycle(),
            tx,
            info,
            db_conn
        }
    }

    pub fn run(&mut self, rx: &Receiver<Broadcast>) {
        match rx.recv().expect("Receiver channel broken!") {
            Broadcast::Deng(user_id) => self.handle_deng(user_id),
            Broadcast::NonDeng(user_id) => {
                self.handle_non_deng(user_id);
                self.handle_request_display_scoreboard();
            },
            Broadcast::PrintScoreboard => self.handle_request_display_scoreboard()
        };
    }

    fn handle_deng(&mut self, user_id: String) {
        let (first_deng, has_denged_today) = {
            let day = self.day_cycle.lock().unwrap();
            (day.first_deng(), day.has_denged_today(&user_id))
        };

        storage::store_success(&self.db_conn, user_id, first_deng, has_denged_today);
    }

    fn handle_non_deng(&mut self, user_id: String) {
        storage::store_failure(&self.db_conn, user_id);
    }

    fn handle_request_display_scoreboard(&mut self) {
        debug!("Sending scoreboard printout");
        let dengs = storage::load(&self.db_conn);
        if let Err(e) = ::send::send_scoreboard(&self.tx, &self.info, &dengs) {
            error!("Could not send scoreboard: {}", e);
        }
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
