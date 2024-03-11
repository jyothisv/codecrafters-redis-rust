use bytes::Bytes;

use crate::resp::{Resp, ToResp};

pub enum Response {
    OK,
    Pong,
    Null,
    SimpleString(String),
    BulkString(String),
    File(Bytes),
    Seq(Vec<Response>),
}

impl Response {
    pub fn serialize(&self) -> Vec<u8> {
        match self {
            Response::OK => "OK".as_simple_string().serialize(),
            Response::Pong => "PONG".as_simple_string().serialize(),
            Response::Null => "$-1\r\n".as_bytes().to_vec(),
            Response::SimpleString(s) => s.as_simple_string().serialize(),
            Response::BulkString(s) => s.as_bulk_string().serialize(),
            Response::File(s) => Resp::File(s.to_owned()).serialize(),
            Response::Seq(seq) => {
                let mut result = vec![];

                for resp in seq {
                    result.extend(resp.serialize())
                }

                result
            }
        }
    }
}
