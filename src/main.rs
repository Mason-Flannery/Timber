mod db;
mod models;

fn main() {
    println!("Hello, world!");
    let conn = db::init_db(); // make sure the database exists
}