use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::ops::Range;
use rand::{Rng, thread_rng};
use std::fmt;
use std::fmt::{Debug, Formatter};
use chrono::{DateTime, Utc, NaiveDateTime};

pub struct DayCycle {
    day: Range<Duration>,
    denged_today: Vec<String>,
}

impl DayCycle {
    pub fn new() -> Self {
        DayCycle {
            day: Self::generate_day(),
            denged_today: vec![],
        }
    }

    pub fn new_day(&mut self) {
        self.day = Self::generate_day();
        self.denged_today.clear();
    }

    fn generate_day() -> Range<Duration> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time has gone backwards");

        let mut rng = thread_rng();
        let hours = 24.0 + rng.gen_range(-1.0, 1.0);

        Range {
            start: now,
            end: now + Duration::from_secs(hours as u64 * 60 * 60),
        }
    }

    pub fn time_to_end(&self) -> Duration {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time has gone backwards");

        self.day.end - now
    }

    pub fn has_denged_today(&self, user_id: &str) -> bool {
        self.denged_today.iter().any(|id| id.as_str() == user_id)
    }

    pub fn first_deng(&self) -> bool {
        self.denged_today.is_empty()
    }
}

impl Debug for DayCycle {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let start_dt = NaiveDateTime::from_timestamp(self.day.start.as_secs() as i64, 0);
        let local_start_dt: DateTime<Utc> = DateTime::from_utc(start_dt, Utc);

        let end_dt = NaiveDateTime::from_timestamp(self.day.end.as_secs() as i64, 0);
        let local_end_dt: DateTime<Utc> = DateTime::from_utc(end_dt, Utc);

        write!(f, "day starts @ {}. Day ends @ {}", local_start_dt.to_rfc2822(), local_end_dt.to_rfc2822())
    }
}