pub type DbConnection = ::r2d2::PooledConnection<::r2d2_diesel::ConnectionManager<::diesel::PgConnection>>;

pub enum Broadcast {
    Deng(String),
    NonDeng(String)
}

#[derive(Debug, Queryable)]
pub struct Deng {
    pub id: i32,
    pub ts: ::std::time::SystemTime,
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
