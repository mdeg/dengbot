use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::ops::Range;

pub struct DayCycle {
    day: Range<Duration>,
    denged_today: Vec<String>,
}

impl DayCycle {
    pub fn start() -> Self {
        let day_start = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time has gone backwards");

        // TODO: add randomness?
        let day = Range {
            start: day_start,
            end: day_start + Duration::from_secs(86400),
        };

        // TODO: convert these times into local timezone for readability (use chrono?)
        debug!(
            "Today starts @ {:?}. Next day starts @ {:?}",
            day.start.as_secs(),
            day.end.as_secs()
        );

        DayCycle {
            day,
            denged_today: vec![],
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
