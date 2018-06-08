extern crate regex;

use slack;
use slack::*;
use types::{Broadcast, SlackInfo};
use std::sync::mpsc::Sender;

pub struct DengHandler {
    tx: Sender<Broadcast>,
    info_tx: Sender<SlackInfo>,
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

        let info = SlackInfo::from_start_response(cli.start_response());
        self.info = Some(info.clone());

        if let Err(e) = self.info_tx.send(info) {
            debug!("Could not broadcast on connect Slack info message: {}", e)
        }
    }
}

impl DengHandler {
    pub fn new(tx: Sender<Broadcast>, info_tx: Sender<SlackInfo>) -> Self {
        DengHandler { tx, info_tx, info: None }
    }

    pub fn handle_message(&mut self, message: slack::api::MessageStandard) -> Result<(), String> {
        let text = message.text.ok_or(String::from("No text in message"))?;
        let user = message.user.ok_or(String::from("No user in message"))?;

        debug!("Message from {}: {}", user, text);

        if let Some(channel_id) = message.channel {
            let listen_channel_id = &self.info.as_ref()
                .ok_or(String::from("Info has not been initialised yet"))?
                .listen_channel_id;

            if channel_id == *listen_channel_id {
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
