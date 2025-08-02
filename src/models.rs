use chrono::{DateTime, Duration, TimeDelta, Utc};
#[derive(Debug)]
pub struct Session {
    pub id: i32,
    pub client_id: i32,
    pub start_timestamp: String,       // stored in RFC339
    pub end_timestamp: Option<String>, // stored in RFC339
    pub note: Option<String>,
    pub offset_minutes: i32, // can be negative or positive
}
impl Session {
    pub fn get_timedelta(&self) -> TimeDelta {
        let start: DateTime<Utc> = self
            .start_timestamp
            .parse::<DateTime<Utc>>()
            .expect("Invalid start timestamp");

        let end = match &self.end_timestamp {
            Some(end_str) => end_str
                .parse::<DateTime<Utc>>()
                .expect("Failed to parse ending timestamp"),
            None => Utc::now(),
        };
        end - start + Duration::minutes(self.offset_minutes.into())
    }
}

#[derive(Debug)]
pub struct Client {
    pub id: i32,
    pub name: String,
    pub note: Option<String>,
}
