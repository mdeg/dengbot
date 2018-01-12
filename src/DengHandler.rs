extern crate slack;
extern crate regex;

use slack::*;
use deng::Deng;
use dengstorage;
use std::ops::Range;
use std::time::Duration;
use constants::{DAILY_DENG_POINT_VALUE, FIRST_DENG_POINT_VALUE, USERS_FIRST_DENG_POINT_VALUE};

pub struct DengHandler {
    pub dengs: Vec<Deng>,
    pub current_day: Range<::std::time::Duration>,


}

impl EventHandler for DengHandler {

    fn on_event(&mut self, _cli: &RtmClient, event: Event) {
        debug!("Event received: {:?}", event);

        if let Event::Message(result) = event {
            if let slack::Message::Standard(message) = *result {
                if let Err(e) = self.handle_message(message) {
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

        let deng_channel_id = cli.start_response().channels.unwrap().iter()
            .filter_map(|channel| channel.name == "dengs");

        match cli.start_response() {

        }
    }
}

impl DengHandler {
    pub fn handle_message(&mut self, message: slack::api::MessageStandard) -> Result<(), &'static str> {

        let text = message.text.ok_or("No text in message")?;
        let user = message.user.ok_or("No user in message")?;

        debug!("Message from {}: {}", user, text);

        match regex::Regex::new(r"^deng$").unwrap().is_match(&text) {
            true => self.handle_deng(user),
            false => self.handle_non_deng(user)
        };

        Ok(())
    }

    fn handle_deng(&mut self, user_id: String) {
        debug!("Deng received from {}", user_id);

        let time = ::std::time::SystemTime::now()
            .duration_since(::std::time::UNIX_EPOCH)
            .expect("Time has gone backwards");

        let is_first_deng_of_day = self.is_first_deng_of_day();
        let is_users_first_deng_of_day = self.is_users_first_deng_of_day(&user_id);

        let deng = Deng { ts: time.as_secs(), user_id, is_first_deng_of_day, is_users_first_deng_of_day };

        let value = self.calculate_deng_value(&deng);
        //TODO: increment user values

        self.dengs.push(deng);
        dengstorage::store_deng(&self.dengs).expect("Could not store deng!");
    }

    fn is_users_first_deng_of_day(&self, user_id: &str) -> bool {
        self.dengs.iter().rev()
            .take_while(|deng| Duration::from_secs(deng.ts) > self.current_day.start)
            .all(|deng| deng.user_id != user_id)
    }

    fn is_first_deng_of_day(&self) -> bool {
        self.dengs.iter().rev()
            .take_while(|deng| Duration::from_secs(deng.ts) > self.current_day.start)
            .count() == 0
    }

    fn calculate_deng_value(&self, deng: &Deng) -> i32 {
        let points = DAILY_DENG_POINT_VALUE;

        if deng.is_first_deng_of_day {
            points + FIRST_DENG_POINT_VALUE;
        }

        if deng.is_users_first_deng_of_day {
            points + USERS_FIRST_DENG_POINT_VALUE;
        }

        points
    }

    fn handle_non_deng(&self, user: String) {
        debug!("Deng failed!");
    }

}