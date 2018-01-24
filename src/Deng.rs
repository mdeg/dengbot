#[derive(Debug, Serialize, Deserialize)]
pub struct Deng {
    pub ts: u64,
    pub user_id: String,
    pub successful: bool,
    pub is_first_deng_of_day: bool,
    pub is_users_first_deng_of_day: bool
}