extern crate regex;

use slackinfo::SlackInfo;
use slack;
use slack::*;
use types::Broadcast;
use std::sync::mpsc::Sender;

pub struct DengHandler {
    tx: Sender<Broadcast>,
    info: Option<SlackInfo>
}

impl EventHandler for DengHandler {
    fn on_event(&mut self, _cli: &RtmClient, event: Event) {
        info!("Event received: {:?}", event);

        if let Event::Message(result) = event {
            if let Message::Standard(message) = *result {
                if let Err(e) = self.handle_message(message) {
                    error!("Could not process message: {}", e);
                }
            }
        }
    }

    fn on_close(&mut self, _cli: &RtmClient) {
        info!("Connection closed")
    }

    fn on_connect(&mut self, cli: &RtmClient) {
        info!("Connected to server");
        self.info = Some(SlackInfo::from_start_response(cli.start_response()));
    }
}

impl DengHandler {
    pub fn new(tx: Sender<Broadcast>) -> Self {
        DengHandler { tx, info: None }
    }

    pub fn handle_message(&mut self, message: slack::api::MessageStandard) -> Result<(), String> {
        let text = message.text.ok_or(String::from("No text in message"))?;
        let user = message.user.ok_or(String::from("No user in message"))?;

        debug!("Message from {}: {}", user, text);

        if let Some(channel_id) = message.channel {
        if channel_id == self.info.as_ref().unwrap().listen_channel_id {
                let msg = match text.as_str() {
                    "deng" => Broadcast::Deng(user),
                    _ => Broadcast::NonDeng(user)
                };

                self.tx.send(msg).map_err(|e| String::from(format!("{}", e)))?;
            }
        }

        Ok(())
    }
}
