use constants::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Deng {
    pub ts: u64,
    pub user_id: String,
    pub successful: bool,
    pub days_first_deng: bool,
    pub users_first_deng: bool
}

impl Deng {
    pub fn new_success(user_id: String, days_first_deng: bool, users_first_deng: bool) -> Self {
        let time = ::std::time::SystemTime::now()
            .duration_since(::std::time::UNIX_EPOCH)
            .expect("Time has gone backwards");

        Deng {
            ts: time.as_secs(),
            user_id,
            successful: true,
            days_first_deng,
            users_first_deng
        }
    }

    pub fn new_fail(user_id: String) -> Self {
        let time = ::std::time::SystemTime::now()
            .duration_since(::std::time::UNIX_EPOCH)
            .expect("Time has gone backwards");

        Deng {
            ts: time.as_secs(),
            user_id,
            successful: false,
            days_first_deng: false,
            users_first_deng: false
        }
    }

    pub fn calculate_value(&self) -> i32 {
        let points = DAILY_DENG_POINT_VALUE;

        if self.days_first_deng {
            points + FIRST_DENG_POINT_VALUE;
        }

        if self.users_first_deng {
            points + USERS_FIRST_DENG_POINT_VALUE;
        }

        points
    }
}