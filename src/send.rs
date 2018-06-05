use types::Deng;
use std::collections::HashMap;
use slackinfo;
use slack;
use slack_hook;
use slack_hook::{Attachment, AttachmentBuilder, PayloadBuilder};

pub fn build_scoreboard_message(hook_client: &slack_hook::Slack,
                                info: &slackinfo::SlackInfo,
                                dengs: &[Deng]) -> Result<(), slack_hook::Error> {

    let msg = match dengs.len() {
        0 => {
            info!("No scoreboard info found - returning default.");

            PayloadBuilder::new()
                .text("No scores yet!")
                .build()?
        },
        _ => {
            let attachments = create_scoreboard_attachments(dengs, &info.users)
                .into_iter()
                .filter_map(|attachment| match attachment {
                        Ok(attach) => Some(attach),
                        Err(e) => {
                            error!("Could not build attachment: {}", e);
                            None
                        }
                })
                .collect();

            PayloadBuilder::new()
                .text(":jewdave: *Deng Champions* :jewdave:")
                .attachments(attachments)
                .build()?
        }
    };

    hook_client.send(&msg)
}

fn create_scoreboard_attachments(dengs: &[Deng],
                         user_list: &[slack::User]) -> Vec<Result<Attachment, String>> {
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

            let username = profile.display_name
                .as_ref()
                .ok_or("Could not find username")?;

            let full_name = profile.real_name
                .as_ref()
                .ok_or("Could not find username")?;

            let hex_color = format!("#{}", user.color.as_ref().unwrap_or(&String::from("000000")));

            let formatted_msg = match username.len() {
                0 => format!("*{}*\t\t\t*{}*", score, full_name),
                _ => format!("*{}*\t\t\t*{}* ({})", score, username, full_name)
            };

            AttachmentBuilder::new(formatted_msg)
                .color(hex_color.as_str())
                .build()
                .map_err(|e| format!("Could not build attachment: {}", e))
        })
        .collect()
}
