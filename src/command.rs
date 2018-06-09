extern crate url;

use futures::{self, Stream, future::Future};
use serde_json;
use storage;
use std::collections::HashMap;
use hyper::{self, Response, Request, server::Service, StatusCode};
use types::{DbConnection, Deng, Error, SlackInfo};
use slack;
use slack_hook::{self, Attachment, AttachmentBuilder, PayloadBuilder};

const SLACK_TOKEN_PARAM_NAME: &'static str = "token";

pub struct CommandListener {
    info: SlackInfo,
    db_conn: DbConnection
}

impl CommandListener {
    pub fn new(info: SlackInfo, db_conn: DbConnection) -> Self {
        Self {
            info,
            db_conn
        }
    }

    fn build_scoreboard_message(&self) -> Result<String, Error> {
        let dengs = storage::load(&self.db_conn).map_err(|e| Error::from(e))?;
        let message = Self::build_scoreboard_payload(&self.info, &dengs)?;
        serde_json::to_string(&message).map_err(|e| Error::from(e))
    }

    fn build_scoreboard_payload(info: &SlackInfo, dengs: &[Deng]) -> Result<CommandResponse, Error> {
        match dengs.len() {
            0 => {
                info!("No scoreboard info found - returning default.");

                PayloadBuilder::new()
                    .text("No scores yet!")
                    .build()
                    .map(|payload| payload.into())
                    .map_err(|e| Error::from(e))
            },
            _ => {
                let attachments = Self::create_scoreboard_attachments(dengs, &info.users)
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
                    .build()
                    .map(|payload| payload.into())
                    .map_err(|e| Error::from(e))
            }
        }
    }

    fn create_scoreboard_attachments(dengs: &[Deng],
                                     user_list: &[slack::User]) -> Vec<Result<Attachment, Error>> {
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
                let user = &user_list.iter()
                    .find(|user| match user.id {
                        Some(ref id) => id == user_id,
                        None => false,
                    })
                    .ok_or(Error::from("Could not find matching user"))?;

                let profile = user.profile.as_ref()
                    .ok_or(Error::from("Could not find user profile"))?;

                let username = profile.display_name.as_ref()
                    .ok_or(Error::from("Could not find username"))?;

                let full_name = profile.real_name.as_ref()
                    .ok_or(Error::from("Could not find username"))?;

                let hex_color = format!("#{}", user.color.as_ref().unwrap_or(&String::from("000000")));

                let formatted_msg = match username.len() {
                    0 => format!("*{}*\t\t\t*{}*", score, full_name),
                    _ => format!("*{}*\t\t\t*{}* ({})", score, username, full_name)
                };

                AttachmentBuilder::new(formatted_msg)
                    .color(hex_color.as_str())
                    .build()
                    .map_err(|e| Error::from(e))
            })
            .collect()
    }
}

impl Service for CommandListener {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        // TODO: build payload AFTER verifying token
        let msg = match self.build_scoreboard_message() {
            Ok(msg) => msg,
            Err(e) => {
                error!("Could not build scoreboard message: {}", e);
                return Box::new(futures::future::ok(hyper::Response::new()
                    .with_status(StatusCode::InternalServerError)));
            }
        };

        Box::new(req.body().concat2().and_then(move |body_chunk| {
            match String::from_utf8(body_chunk.to_vec()) {
                Ok(body) => {
                    debug!("Received Slack command body: {}", body);

                    let params = url::form_urlencoded::parse(body.as_ref())
                        .into_owned()
                        .collect::<HashMap<String, String>>();

                    match params.get(SLACK_TOKEN_PARAM_NAME) {
                        Some(val) => {
                            info!("Received command from Slack. Verifying command token...");

                            if val == dotenv!("SLACK_COMMAND_VERIFICATION_KEY") {
                                info!("Successfully verified command token. Sending response.");

                                return futures::future::ok(
                                    hyper::Response::new()
                                        .with_status(StatusCode::Ok)
                                        .with_header(hyper::header::ContentType::json())
                                        .with_body(msg))
                            } else {
                                error!("Could not validate command token!");
                            }
                        },
                        None => error!("Could not find token parameter in Slack POST.")
                    }
                },
                Err(e) => error!("Could not create string from request body: {}", e)
            }

            // fall back to server error response
            futures::future::ok(hyper::Response::new()
                .with_status(StatusCode::InternalServerError))
        }))
    }
}

#[derive(Serialize)]
enum ResponseType {
    #[serde(rename = "ephemeral")]
    #[allow(dead_code)]
    Ephemeral,
    #[serde(rename = "in_channel")]
    InChannel
}

#[derive(Serialize)]
pub struct CommandResponse {
    response_type: ResponseType,
    #[serde(flatten)]
    payload: slack_hook::Payload
}

impl Into<CommandResponse> for slack_hook::Payload {
    fn into(self) -> CommandResponse {
        CommandResponse {
            response_type: ResponseType::InChannel,
            payload: self
        }
    }
}