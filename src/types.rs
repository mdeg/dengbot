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
        if self.successful && self.users_first_deng { value += 1 }
        if self.days_first_deng { value += 1 }
        value
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