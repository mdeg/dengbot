use std::time::SystemTime;

#[derive(Debug, Queryable)]
pub struct Deng {
    pub id: i32,
    pub ts: SystemTime,
    pub user_id: String,
    pub successful: bool,
    pub days_first_deng: bool,
    pub users_first_deng: bool,
}

// TODO: move this
impl Deng {
    const DAILY_DENG_POINT_VALUE: i32 = 1;
    const FIRST_DENG_POINT_VALUE: i32 = 1;
    const USERS_FIRST_DENG_POINT_VALUE: i32 = 1;

    pub fn calculate_value(&self) -> i32 {
        let mut points = Self::DAILY_DENG_POINT_VALUE;

        if self.days_first_deng {
            points += Self::FIRST_DENG_POINT_VALUE;
        }
        if self.users_first_deng {
            points += Self::USERS_FIRST_DENG_POINT_VALUE;
        }

        points
    }
}