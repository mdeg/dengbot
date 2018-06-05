use types::Deng;
use diesel::{RunQueryDsl, PgConnection, QueryResult};
use std::time::SystemTime;

table! {
    dengs (id) {
        id -> Int4,
        ts -> Timestamp,
        user_id -> Varchar,
        successful -> Bool,
        days_first_deng -> Bool,
        users_first_deng -> Bool,
    }
}

#[derive(Debug, Insertable)]
#[table_name="dengs"]
pub struct NewDeng {
    pub ts: SystemTime,
    pub user_id: String,
    pub successful: bool,
    pub days_first_deng: bool,
    pub users_first_deng: bool,
}

impl NewDeng {
    pub fn new_success(user_id: String, days_first_deng: bool, users_first_deng: bool) -> Self {
        NewDeng {
            ts: SystemTime::now(),
            user_id,
            successful: true,
            days_first_deng,
            users_first_deng,
        }
    }

    pub fn new_failure(user_id: String) -> Self {
        NewDeng {
            ts: SystemTime::now(),
            user_id,
            successful: false,
            days_first_deng: false,
            users_first_deng: false,
        }
    }
}

pub fn store_failure(conn: &PgConnection, user_id: String) -> QueryResult<Deng> {

    let deng = NewDeng::new_failure(user_id);
    ::diesel::insert_into(dengs::table)
        .values(&deng)
        .get_result(conn)
}

pub fn store_success(conn: &PgConnection,
                     user_id: String,
                     days_first_deng: bool,
                     users_first_deng: bool) -> QueryResult<Deng> {

    let deng = NewDeng::new_success(user_id, days_first_deng, users_first_deng);
    ::diesel::insert_into(dengs::table)
        .values(&deng)
        .get_result(conn)
}

pub fn load(conn: &PgConnection) -> QueryResult<Vec<Deng>> {
    dengs::table.load::<Deng>(conn)
}