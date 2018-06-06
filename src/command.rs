extern crate serde_json;

use futures::future::Future;
use futures;
use futures::Stream;
use storage;
use hyper;
use hyper::{Response, Request, server::Service};
use slackinfo::SlackInfo;
use types::DbConnection;

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

    pub fn send_scoreboard(&self) {
        info!("Sending scoreboard printout");

        match storage::load(&self.db_conn) {
            Ok(dengs) => {
                if let Err(e) = ::send::build_scoreboard_message(&self.info, &dengs) {
                    error!("Could not send scoreboard: {}", e);
                }
            },
            Err(e) => error!("Could not load dengs from database: {}", e)
        }
    }
//
//    fn build_slash_command_failure_response(&self) -> hyper::Response {
//
//    }
}

impl Service for CommandListener {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        let dengs = storage::load(&self.db_conn).unwrap();
        let payload = ::send::build_scoreboard_message(&self.info, &dengs).unwrap();

        Box::new(req.body().concat2().and_then(move |body| {

            let bod = String::from_utf8(body.to_vec()).unwrap();
            debug!("Received Slack command body: {}", bod);

            let serialized = serde_json::to_string(&payload).unwrap();

            debug!("Command response payload: {:?}", serialized);

            let response = hyper::Response::new()
                .with_status(hyper::StatusCode::Ok)
                .with_header(hyper::header::ContentType::json())
                .with_body(serialized);

            futures::future::ok(response)
        }))
    }
}