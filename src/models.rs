#[derive(Debug)]
pub struct Session {
    pub id: i32,
    pub client_id: i32,
    pub start_timestamp: String, // stored in RFC339
    pub elapsed_seconds: i64,
    pub note: Option<String>,
}

#[derive(Debug)]
pub struct Client {
    pub id: i32,
    pub name: String,
    pub note: Option<String>,
}
