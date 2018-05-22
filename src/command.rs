use futures::future::Future;
use futures;
use diesel::PgConnection;
use std::sync::Arc;
use storage;
use slack_hook;
use hyper;
use slackinfo::SlackInfo;

pub struct CommandListener {
    info: Arc<SlackInfo>,
    db_conn: PgConnection,
    hook_client: slack_hook::Slack
}

impl CommandListener {
    pub fn new(info: Arc<SlackInfo>, db_conn: PgConnection) -> Self {
        Self {
            info,
            db_conn,
            hook_client: slack_hook::Slack::new(dotenv!("WEBHOOK_URL")).unwrap()
        }
    }

    pub fn handle_scoreboard(&self) {
        debug!("Sending scoreboard printout");
        let dengs = storage::load(&self.db_conn);
        if let Err(e) = ::send::send_scoreboard(&self.hook_client, &self.info, &dengs) {
            error!("Could not send scoreboard: {}", e);
        }
    }
}

impl hyper::server::Service for CommandListener {
    type Request = hyper::Request;
    type Response = hyper::Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {

        debug!("COMMAND REQUEST: {:#?}", req);

        self.handle_scoreboard();

        Box::new(futures::future::ok(
            hyper::Response::new().with_status(hyper::StatusCode::Ok)
        ))
    }
}