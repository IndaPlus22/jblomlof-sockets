
use std::net::TcpStream;


pub struct User {
    pub username: String,
    pub id: usize,
    pub client: TcpStream,
}

impl User {
    pub fn new(username: &str, id: usize,client: TcpStream) -> User {
        User {
            username: username.to_string(),
            id,
            client
        }
    }
}