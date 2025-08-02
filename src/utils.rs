use chrono::{DateTime, Datelike, Duration, NaiveDate, TimeZone, Utc, Weekday};
use rusqlite::Connection;

use crate::{cli::UserInput, db};

pub fn split_minutes(total_minutes: u32) -> (u32, u32) {
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;
    (hours, minutes)
}

pub fn current_day_range() -> (DateTime<Utc>, DateTime<Utc>) {
    let now = chrono::Utc::now();
    let start = chrono::Utc
        .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
        .unwrap(); // Get time at start of day
    let end = start + chrono::Duration::days(1); // Get time at end of day

    (start, end)
}

pub fn current_week_range() -> (DateTime<Utc>, DateTime<Utc>) {
    let today = Utc::now().date_naive();
    let weekday = today.weekday();

    // Compute how many days we need to subtract to get to the most recent Saturday
    let days_back = match weekday {
        Weekday::Sat => 0,
        _ => (weekday.num_days_from_sunday() + 1) % 7, // Add a day -- we want this relative to saturday.
    };
    let last_saturday = today - Duration::days(days_back as i64);
    let next_friday = last_saturday + Duration::days(6);

    let start = Utc.from_utc_datetime(&last_saturday.and_hms_opt(0, 0, 0).unwrap());
    let end = Utc.from_utc_datetime(&next_friday.and_hms_opt(23, 59, 59).unwrap());
    (start, end)
}

pub fn current_month_range() -> (DateTime<Utc>, DateTime<Utc>) {
    let today = Utc::now().date_naive();

    // Start of the current month
    let start_date = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();

    // First day of next month
    let (next_year, next_month) = if today.month() == 12 {
        (today.year() + 1, 1)
    } else {
        (today.year(), today.month() + 1)
    };
    let first_day_next_month = NaiveDate::from_ymd_opt(next_year, next_month, 1).unwrap();

    // Last day of this month = day before first of next month
    let end_date = first_day_next_month - Duration::days(1);

    let start = Utc.from_utc_datetime(&start_date.and_hms_opt(0, 0, 0).unwrap());
    let end = Utc.from_utc_datetime(&end_date.and_hms_opt(23, 59, 59).unwrap());

    (start, end)
}

pub fn handle_user_client_input(conn: &Connection, input: Option<UserInput>) -> Option<i32> {
    match input {
        Some(UserInput::ById(id)) => Some(id),
        Some(UserInput::ByName(name)) => db::get_client_id_by_name(conn, name).ok().flatten(),
        None => None,
    }
}
