use futures::future::Future;
use futures;
use storage;
use slack_hook;
use hyper;
use slackinfo::SlackInfo;
use types::DbConnection;

pub struct CommandListener {
    info: SlackInfo,
    db_conn: DbConnection,
    hook_client: slack_hook::Slack
}

impl CommandListener {
    pub fn new(info: SlackInfo, db_conn: DbConnection) -> Self {
        let client = slack_hook::Slack::new(dotenv!("WEBHOOK_URL"))
            .expect("Could not build hook client!");

        Self {
            info,
            db_conn,
            hook_client: client
        }
    }

    pub fn handle_scoreboard(&self) {
        info!("Sending scoreboard printout");

        let dengs = storage::load(&self.db_conn);
        if let Err(e) = ::send::build_scoreboard_message(&self.hook_client, &self.info, &dengs) {
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

        debug!("Contents of command request received from Slack: {:#?}", req);

        self.handle_scoreboard();

        Box::new(futures::future::ok(
            hyper::Response::new().with_status(hyper::StatusCode::Ok)
        ))
    }
}