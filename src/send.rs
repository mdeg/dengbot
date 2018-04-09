use types::Deng;
use std::collections::HashMap;

// TODO: move this into the forked slack crate
pub fn send_raw_msg(sender: &::slack::Sender, channel_id: &str) -> Result<(), ::slack::Error> {

    // TODO: serialize Slack stuff to json
//    let attachment = ::slack::api::MessageStandardAttachment {
//        text: Some(String::from("test text")),
//        title: Some(String::from("test title")),
//        ...
//    };

    let extra = format!(r#""attachments": [{{"text": "test attachment", "title": "Slack API Documentation"}}]"#);

    let data = format!(r#"{{"id": {},"type": "message", "channel": "{}", "text": "z", {}}}"#,
            sender.get_msg_uid(),
            channel_id,
            extra);

    debug!("Raw data to send: {}", data);

    sender.send(&data)
}

pub fn format_scoreboard(dengs: &[Deng], user_list: &[::slack::User]) -> String {
    let mut ordered_scores = dengs
        .iter()
        .filter(|deng| deng.successful)
        .fold(HashMap::new(), |mut map, deng| {
            // TODO: calculate value
            *map.entry(&deng.user_id).or_insert(0) += 1;
            map
        })
        .into_iter()
        .collect::<Vec<_>>();

    if ordered_scores.is_empty() {
        info!("No scoreboard info found - returning default.");
        return String::from("No scores yet!");
    }

    ordered_scores.sort_by(|first, second| second.1.cmp(&first.1));

    trace!("Raw ordered score list: {:?}", ordered_scores);

    ordered_scores
        .into_iter()
        .map(|(user_id, score)| {
            let default = String::from("Unknown");

            let name = &user_list
                .iter()
                .find(|user| match user.id {
                    Some(ref id) => id == user_id,
                    None => false,
                })
                .map(|user| user.name.as_ref().unwrap_or(&default))
                .unwrap();

            format!("{}\t\t{}", name, score)
        })
        .scan(String::new(), |state, line| {
            Some(format!("{}\n{}", *state, &line))
        })
        .next()
        .expect("Could not format the scoreboard")
}
