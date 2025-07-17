#[derive(Debug)]
pub struct Session {
    pub id: i32,
    pub client_id: i32,
    pub start_timestamp: String,       // stored in RFC339
    pub end_timestamp: Option<String>, // stored in RFC339
    pub note: Option<String>,
}

#[derive(Debug)]
pub struct Client {
    pub id: i32,
    pub name: String,
    pub note: Option<String>,
}
