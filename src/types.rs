use ::std;
use ::std::error::Error as StdError;

pub type DbConnection = ::r2d2::PooledConnection<::r2d2_diesel::ConnectionManager<::diesel::PgConnection>>;

pub enum Broadcast {
    Deng(String),
    NonDeng(String)
}

#[derive(Debug, Queryable)]
pub struct Deng {
    pub id: i32,
    pub ts: std::time::SystemTime,
    pub user_id: String,
    pub successful: bool,
    pub days_first_deng: bool,
    pub users_first_deng: bool,
}

impl Deng {
    pub fn value(&self) -> i32 {
        let mut value = 0;
        if self.successful {
            if self.users_first_deng {
                value += 1
            }
            if self.days_first_deng {
                value += 1
            }
        }
        value
    }
}

#[derive(Clone, Debug)]
pub struct SlackInfo {
    pub users: Vec<::slack::User>,
    pub listen_channel_id: String,
    pub meta_channel_id: String,
}

// Slack will send us up-to-date channel and user IDs on initial connection
// We need to store these and use them to store dengs and construct messages
// It should be considered fatal if any of these data items are not found
impl SlackInfo {
    pub fn from_start_response(resp: &::slack::api::rtm::StartResponse) -> Self {
        let mut channels = resp.channels
            .as_ref()
            .expect("No channel list returned")
            .iter();

        let listen_channel_id = channels
            .find(|channel| channel.name.as_ref().expect("No listen channel name found") == dotenv!("LISTEN_CHANNEL_NAME"))
            .expect("Could not find listen channel by that name")
            .id
            .clone()
            .expect("No ID associated with listen channel");

        debug!("Found listen channel ID: {}", listen_channel_id);

        let meta_channel_id = channels
            .find(|channel| {
                channel.name.as_ref().expect("No listen channel name found") == dotenv!("META_CHANNEL_NAME")
            })
            .expect("Could not find meta channel by that name")
            .id
            .clone()
            .expect("No ID associated with meta channel");

        debug!("Found meta channel ID: {}", meta_channel_id);

        let users = resp.users.clone().expect("No users returned on connection");

        debug!("Users: {:#?}", users);

        SlackInfo {
            users,
            listen_channel_id,
            meta_channel_id,
        }
    }
}

#[derive(Debug)]
pub struct Error {
    description: String,
    cause: Option<Box<StdError>>
}

impl StdError for Error {
    fn description(&self) -> &str {
        &self.description
    }

    fn cause(&self) -> Option<&StdError> {
        self.cause.as_ref().map(|e| &**e)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.cause {
            Some(ref cause) => write!(f, "{}\n{}", self.description, cause),
            None => write!(f, "{}", self.description)
        }
    }
}

impl From<&'static str> for Error {
    fn from(error: &'static str) -> Self {
        Error {
            description: String::from(error),
            cause: None
        }
    }
}

impl From<String> for Error {
    fn from(error: String) -> Self {
        Error {
            description: error,
            cause: None
        }
    }
}

impl From<::slack_hook::Error> for Error {
    fn from(error: ::slack_hook::Error) -> Self {
        Error {
            description: String::from(error.description()),
            cause: Some(Box::new(error))
        }
    }
}

impl From<::serde_json::Error> for Error {
    fn from(error: ::serde_json::Error) -> Self {
        Error {
            description: String::from(error.description()),
            cause: Some(Box::new(error))
        }
    }
}

impl From<::diesel::result::Error> for Error {
    fn from(error: ::diesel::result::Error) -> Self {
        Error {
            description: String::from(error.description()),
            cause: Some(Box::new(error))
        }
    }
}