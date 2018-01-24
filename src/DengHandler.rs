extern crate slack;
extern crate regex;

use slack::*;
use deng::Deng;
use dengstorage;
use std::ops::Range;
use std::collections::HashMap;
use std::time::Duration;
use constants::*;

pub struct DengHandler {
    pub dengs: Vec<Deng>,
    pub current_day: Range<::std::time::Duration>,
    users: Option<HashMap<String, String>>,
    deng_channel_id: Option<String>
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

        let resp = cli.start_response().clone();

        self.deng_channel_id = resp.channels.unwrap()
            .into_iter()
            .filter_map(|channel| channel.name)
            .find(|name| name == "dengs");

        self.users = Some(resp.users
            .expect("No users")
            .into_iter()
            .map(|user| (user.id.expect("No user id"), user.name.expect("No user name")))
            .collect::<HashMap<_,_>>());
    }
}

impl DengHandler {

    pub fn new(dengs: Vec<Deng>, current_day: Range<::std::time::Duration>) -> Self {
        DengHandler {
            dengs,
            current_day,
            users: None,
            deng_channel_id: None
        }
    }

    pub fn handle_message(&mut self, message: slack::api::MessageStandard) -> Result<(), &'static str> {

        let text = message.text.ok_or("No text in message")?;
        let user = message.user.ok_or("No user in message")?;

        debug!("Message from {}: {}", user, text);

        let successful = regex::Regex::new(r"^deng$").expect("Bad deng regex").is_match(&text);
        self.handle_deng(user, successful);

        Ok(())
    }

    fn handle_deng(&mut self, user_id: String, successful: bool) {
        debug!("Deng received from {}", user_id);

        let time = ::std::time::SystemTime::now()
            .duration_since(::std::time::UNIX_EPOCH)
            .expect("Time has gone backwards");

        let is_first_deng_of_day = self.is_first_deng_of_day();
        let is_users_first_deng_of_day = self.is_users_first_deng_of_day(&user_id);

        let deng = Deng { ts: time.as_secs(), user_id, successful,
            is_first_deng_of_day, is_users_first_deng_of_day };

        self.dengs.push(deng);
        dengstorage::store_deng(&self.dengs).expect("Could not store deng!");

        println!("{}", self.format_scoreboard());
    }

    fn format_scoreboard(&self) -> String {
        let mut ordered_scores = (&self.dengs).iter()
            .fold(HashMap::new(), |mut map, ref deng| {
                *map.entry(&deng.user_id).or_insert(0) += self.calculate_deng_value(&deng);
                map
            })
            .into_iter()
            .collect::<Vec<_>>();

        ordered_scores.sort_by(|first, second| second.1.cmp(&first.1));

        debug!("Raw ordered score list: {:?}", ordered_scores);

        let user_list = self.users.clone().expect("No list of users found");

        ordered_scores
            .into_iter()
            .map(|(user_id, score)| {
                let default = String::from("Unknown");
                let name = user_list.get(user_id).unwrap_or(&default);
                format!("{}\t\t{}", name, score)
            })
            .scan(String::new(), |state, line| Some(format!("{}\n{}", *state, &line)))
            .next()
            .expect("Could not format the scoreboard")
    }

    fn is_users_first_deng_of_day(&self, user_id: &str) -> bool {
        self.dengs.iter()
            .rev()
            .take_while(|deng| Duration::from_secs(deng.ts) > self.current_day.start)
            .all(|deng| deng.user_id != user_id)
    }

    fn is_first_deng_of_day(&self) -> bool {
        self.dengs.iter()
            .rev()
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

}