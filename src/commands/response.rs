use crate::resp::ToResp;

pub enum Response {
    OK,
    Pong,
    BulkString(String),
    Null,
}

impl Response {
    pub fn serialize(&self) -> Vec<u8> {
        match self {
            Response::OK => "OK".as_simple_string().serialize(),
            Response::Pong => "PONG".as_simple_string().serialize(),
            Response::BulkString(s) => s.as_bulk_string().serialize(),
            Response::Null => "$-1\r\n".to_owned(),
        }
        .as_bytes()
        .to_vec()
    }
}
