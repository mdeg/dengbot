extern crate regex;
extern crate slack;

use slack::*;
use std::sync::mpsc::Sender;

pub struct DengHandler {
    tx: Sender<::Broadcast>,
    info: Option<::SlackInfo>
}

impl EventHandler for DengHandler {
    fn on_event(&mut self, _cli: &RtmClient, event: Event) {
        debug!("Event received: {:?}", event);

        if let Event::Message(result) = event {
            if let Message::Standard(message) = *result {
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

        self.info = Some(::SlackInfo::new(cli.start_response()));
    }
}

impl DengHandler {
    pub fn new(tx: Sender<::Broadcast>) -> Self {
        DengHandler {
            tx,
            info: None
        }
    }

    pub fn handle_message(&mut self, message: slack::api::MessageStandard) -> Result<(), &'static str> {
        let text = message.text.ok_or("No text in message")?;
        let user = message.user.ok_or("No user in message")?;

        debug!("Message from {}: {}", user, text);

        if let Some(channel_id) = message.channel {
            if channel_id == self.info.as_ref().unwrap().listen_channel_id {
                let msg = {
                    if &text == "deng" {
                        ::Broadcast::Deng(user)
                    } else {
                        ::Broadcast::NonDeng(user)
                    }
                };

                if let Err(e) = self.tx.send(msg) {
                    error!("Failed to send message to main thread: {}", e);
                }
            }
        }

        Ok(())
    }
}
