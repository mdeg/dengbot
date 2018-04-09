pub enum Broadcast {
    Deng(String),
    NonDeng(String),
    PrintScoreboard,
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

