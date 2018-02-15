extern crate slack;
extern crate regex;

use slack::*;
use slackinfo::SlackInfo;
use deng::Deng;
use dengstorage;
use std::ops::Range;
use std::collections::HashMap;
use std::time::Duration;
use std::sync::{Arc, Mutex};

pub struct DengHandler {
    pub dengs: Vec<Deng>,
    pub current_day: Arc<Mutex<Range<::std::time::Duration>>>,
    pub info: Option<SlackInfo>
}

impl EventHandler for DengHandler {

    fn on_event(&mut self, cli: &RtmClient, event: Event) {
        debug!("Event received: {:?}", event);

        if let Event::Message(result) = event {
            if let slack::Message::Standard(message) = *result {
                if let Err(e) = self.handle_message(cli, message) {
                    error!("Could not process message: {}", e);
                }
            }
        }
    }

    fn on_close(&mut self, _cli: &RtmClient) {
        debug!("Connection closed")
    }

    fn on_connect(&mut self, cli: &RtmClient) {
        debug!("Connected to server");

        self.info = Some(SlackInfo::new(cli.start_response().clone()));
    }
}

impl DengHandler {

    pub fn new(dengs: Vec<Deng>, current_day: Arc<Mutex<Range<::std::time::Duration>>>) -> Self {
        DengHandler {
            dengs,
            current_day,
            info: None
        }
    }

    pub fn handle_message(&mut self, cli: &RtmClient, message: slack::api::MessageStandard) -> Result<(), &'static str> {

        let text = message.text.ok_or("No text in message")?;
        let user = message.user.ok_or("No user in message")?;

        let (listen_channel_id, meta_channel_id) = {
            let info = self.info.as_ref().expect("No slack info found");
            (info.listen_channel_id.clone(), info.meta_channel_id.clone())
        };

        debug!("Message from {}: {}", user, text);

        if let Some(channel_id) = message.channel {
            if channel_id == listen_channel_id {
                if &text == "deng" {
                    self.handle_deng(user);
                } else {
                    self.handle_non_deng(user)
                }
            }
        }

        cli.sender().send_message(&meta_channel_id, &self.format_scoreboard());

        Ok(())
    }

    // TODO: neater way to handle deng/nondengs
    fn handle_non_deng(&mut self, user_id: String) {
        info!("Non-deng received from {}", user_id);

        let deng = Deng::new_fail(user_id);

        self.store(deng);
    }

    fn store(&mut self, deng: Deng) {
        self.dengs.push(deng);
        dengstorage::store_deng("./dengs", &self.dengs).expect("Could not store deng!");
    }

    fn handle_deng(&mut self, user_id: String) {
        info!("Deng received from {}", user_id);

        let user_first_deng = self.is_users_first_deng_of_day(&user_id);
        let deng = Deng::new_success(user_id, self.is_first_deng_of_day(), user_first_deng);

        self.store(deng);
    }

    fn format_scoreboard(&self) -> String {
        let mut ordered_scores = (&self.dengs).iter()
            .filter(|deng| deng.successful)
            .fold(HashMap::new(), |mut map, deng| {
                *map.entry(&deng.user_id).or_insert(0) += deng.calculate_value();
                map
            })
            .into_iter()
            .collect::<Vec<_>>();

        if ordered_scores.is_empty() {
            info!("No scores found");
            return String::new();
        }

        ordered_scores.sort_by(|first, second| second.1.cmp(&first.1));

        trace!("Raw ordered score list: {:?}", ordered_scores);

        // TODO: do without clone
        let user_list = self.info.as_ref().expect("No slack info found").users.clone();

        ordered_scores
            .into_iter()
            .map(|(user_id, score)| {
                let default = String::from("Unknown");

                let name = &user_list.iter()
                    .find(|user| match user.id {
                        Some(ref id) => id == user_id,
                        None => false
                    })
                    .map(|user| user.name.as_ref().unwrap_or(&default))
                    .unwrap();

                format!("{}\t\t{}", name, score)
            })
            .scan(String::new(), |state, line| Some(format!("{}\n{}", *state, &line)))
            .next()
            .expect("Could not format the scoreboard")
    }

    fn is_users_first_deng_of_day(&self, user_id: &str) -> bool {
        self.dengs.iter()
            .rev()
            .take_while(|deng| Duration::from_secs(deng.ts) > self.current_day.lock().unwrap().start)
            .all(|deng| deng.user_id != user_id)
    }

    fn is_first_deng_of_day(&self) -> bool {
        self.dengs.iter()
            .rev()
            .take_while(|deng| Duration::from_secs(deng.ts) > self.current_day.lock().unwrap().start)
            .count() == 0
    }
}