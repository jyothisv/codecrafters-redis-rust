use crate::resp::ToResp;

pub enum Response {
    OK,
    Pong,
    Null,
    SimpleString(String),
    BulkString(String),
    File(String),
    Seq(Vec<Response>),
}

impl Response {
    pub fn serialize(&self) -> String {
        match self {
            Response::OK => "OK".as_simple_string().serialize(),
            Response::Pong => "PONG".as_simple_string().serialize(),
            Response::Null => "$-1\r\n".to_owned(),
            Response::SimpleString(s) => s.as_simple_string().serialize(),
            Response::BulkString(s) => s.as_bulk_string().serialize(),
            Response::File(s) => s.as_bulk_string().serialize(),
            Response::Seq(seq) => seq.iter().map(|s| s.serialize()).collect(),
        }
    }
}
