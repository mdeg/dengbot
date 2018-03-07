use std::sync::mpsc::Receiver;
use std::thread;
use daycycle::*;
use deng::Deng;
use std::sync::{Arc, Mutex};
use dengstorage;

pub struct Runner {
    dengs: Vec<Deng>,
    rx: Receiver<HandleableMessages>,
    tx: ::slack::Sender,
    info: ::slackinfo::SlackInfo,
}

// TODO: rename
pub enum HandleableMessages {
    Deng(String),
    NonDeng(String),
    PrintScoreboard,
}

impl Runner {
    pub fn new(dengs: Vec<Deng>, rx: Receiver<HandleableMessages>,
               tx: ::slack::Sender, info: ::slackinfo::SlackInfo) -> Self {
        Runner {
            dengs,
            rx,
            tx,
            info,
        }
    }

    pub fn run(&mut self) {
        // Start the day immediately
        let day_cycle = Runner::start_day();

        loop {
            match self.rx.recv().expect("Receiver channel broken!") {
                // TODO: clean this up
                HandleableMessages::Deng(user_id) => {
                    let day = day_cycle.lock().unwrap();
                    let has_denged_today = day.has_denged_today(&user_id);
                    let deng = Deng::new_success(user_id, day.first_deng(), has_denged_today);

                    self.dengs.push(deng);
                    dengstorage::store_deng("./dengs", &self.dengs).expect("Could not store deng!");
                }
                HandleableMessages::NonDeng(user_id) => {
                    let deng = Deng::new_fail(user_id);

                    self.dengs.push(deng);
                    dengstorage::store_deng("./dengs", &self.dengs).expect("Could not store deng!");
                }
                HandleableMessages::PrintScoreboard => {
                    debug!("Sending scoreboard printout");
                    if let Err(e) = ::send::send_raw_msg(&self.tx, &self.info.meta_channel_id) {
                        error!("{}", e);
                    }
                }
            };
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
