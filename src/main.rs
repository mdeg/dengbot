extern crate slack;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate log;

mod constants;
mod denghandler;
mod deng;
mod dengstorage;

use std::time::{SystemTime, UNIX_EPOCH, Duration};

fn main() {
    let message = slack::api::MessageStandard { attachments: None, bot_id: None, channel: None,
        edited: None, event_ts: None, reply_broadcast: None, source_team: None,
        user: Some(String::from("123")), text: Some(String::from("deng")), thread_ts: None, ts:
        None,
    team: None, ty: None };

    let dengs = match dengstorage::read_dengs() {
        Ok(d) => d,
        Err(_) => {
            dengstorage::create_storage();
            vec!()
        }
    };

    let seed_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time has gone backwards");

    debug!("Starting the day at {:?}", seed_time);

    let current_day = ::std::ops::Range { start: seed_time, end: seed_time + Duration::from_secs(86400) };

    let mut handler = denghandler::DengHandler { dengs, current_day };
    handler.handle_message(message);

//    match slack::RtmClient::login_and_run(constants::TOKEN, &mut denghandler::DengHandler) {
//        Ok(_) => debug!("Gracefully closed connection"),
//        Err(e) => error!("Ungraceful termination due to error: {}", e)
//    }
}

