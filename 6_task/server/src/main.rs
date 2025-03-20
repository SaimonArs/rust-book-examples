extern crate postgres;
extern crate chrono;
use std::env; 
use server::{db_client::*, listener::*};
fn main() {

    let postgres_url = env::var("DATABASE_URL").unwrap();
    let addr = env::var("ADDRESS").unwrap();
    let port = env::var("PORT").unwrap();

    let db = Database::build(postgres_url);
    let mut listener = Listener::build(db, addr, port);

    listener.start();
}

