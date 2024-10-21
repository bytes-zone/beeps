use crate::conn::Conn;

pub async fn handler(Conn(_): Conn) -> &'static str {
    "OK"
}
