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
    pub fn get_timedelta(&self) -> Option<TimeDelta> {
        let start: DateTime<Utc> = self
            .start_timestamp
            .parse::<DateTime<Utc>>()
            .expect("Invalid start timestamp");
        let end_str:&String = self.end_timestamp.as_ref()?;
        let end = end_str.parse::<DateTime<Utc>>().ok()?;
        Some(end-start + Duration::minutes(self.offset_minutes.into()))
    }
} 

#[derive(Debug)]
pub struct Client {
    pub id: i32,
    pub name: String,
    pub note: Option<String>,
}
