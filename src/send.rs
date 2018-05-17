use types::Deng;
use std::collections::HashMap;
use slack_hook::{Attachment, AttachmentBuilder, PayloadBuilder};

pub fn send_scoreboard(hook_client: &::slack_hook::Slack,
                       info: &::slackinfo::SlackInfo,
                       dengs: &[Deng]) -> Result<(), ::slack_hook::Error> {

    let msg = match dengs.len() {
        0 => {
            info!("No scoreboard info found - returning default.");

            PayloadBuilder::new()
                .text("No scores yet!")
                .build()
                .unwrap()
        },
        _ => {
            match format_scoreboard(dengs, &info.users) {
                Ok(msg) => {
                    PayloadBuilder::new()
                        .text("DENG CHAMPS")
                        .attachments(msg)
                        .build()
                        .unwrap()
                },
                Err(error) => {
                    error!("Could not send scoreboard: {}", error);
                    // TODO: cascade errors correctly
                    return Ok(())
                }
            }
        }
    };

    hook_client.send(&msg)
}

pub fn format_scoreboard(dengs: &[Deng],
                         user_list: &[::slack::User]) -> Result<Vec<Attachment>, &'static str> {
    let mut ordered_scores = dengs
        .iter()
        .filter(|deng| deng.successful)
        .fold(HashMap::new(), |mut map, deng| {
            *map.entry(&deng.user_id).or_insert(0) += deng.value();
            map
        })
        .into_iter()
        .collect::<Vec<_>>();

    ordered_scores.sort_by(|first, second| second.1.cmp(&first.1));

    trace!("Raw ordered score list: {:?}", ordered_scores);

    ordered_scores.into_iter()
        .map(|(user_id, score)| {
            let user = &user_list
                .iter()
                .find(|user| match user.id {
                    Some(ref id) => id == user_id,
                    None => false,
                })
                .ok_or("Could not find matching user! Bot may need to reconnect to regenerate user list")?;

            let profile = user.profile
                .as_ref()
                .ok_or("Could not find user profile")?;

            let name = profile.real_name
                .as_ref()
                .ok_or("Could not find username")?;

            let avatar = profile.image_72
                .as_ref()
                .ok_or("Could not find avatar")?;

            let formatted = format!("*{}*\t\t\t{}", name, score);

            Ok(AttachmentBuilder::new(formatted)
                .image_url(avatar.as_str())
                .build()
                .unwrap())
        })
        .collect()
}
