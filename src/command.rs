extern crate url;

use futures::future::Future;
use futures;
use futures::Stream;
use serde_json;
use storage;
use hyper;
use std::collections::HashMap;
use hyper::{Response, Request, server::Service, StatusCode};
use slackinfo::SlackInfo;
use types::{DbConnection, Error};

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

    fn build_payload(&self) -> Result<String, Error> {
        let dengs = storage::load(&self.db_conn).map_err(|e| Error::from(e))?;
        let payload = ::send::build_scoreboard_message(&self.info, &dengs)?;
        serde_json::to_string(&payload).map_err(|e| Error::from(e))
    }
}

impl Service for CommandListener {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        // TODO: build payload AFTER verifying token
        let payload = self.build_payload().unwrap();

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
                                        .with_body(payload))
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