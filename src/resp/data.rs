use bytes::Bytes;

#[derive(Debug)]
pub enum Resp {
    SimpleString(String),
    BulkString(String),
    File(Bytes),
    Int(i64),
    Array(Vec<Resp>),
}

impl Resp {
    pub fn into_string(self) -> String {
        match self {
            Resp::SimpleString(s) => s,
            Resp::BulkString(s) => s,
            _ => panic!("Should only be called on strings"),
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        match self {
            Resp::SimpleString(s) => format!("+{}\r\n", s).as_bytes().to_vec(),

            Resp::BulkString(s) => {
                let len = s.len();
                format!("${}\r\n{}\r\n", len, s).as_bytes().to_vec()
            }

            Resp::File(s) => {
                let len = s.len();
                let mut vec = format!("${}\r\n", len).as_bytes().to_vec();
                vec.extend(s);
                vec
            }

            Resp::Int(n) => format!(":{}\r\n", n).as_bytes().to_vec(),

            Resp::Array(vec) => {
                let mut s = format!("*{}\r\n", vec.len()).as_bytes().to_vec();
                for resp in vec {
                    s.extend(&resp.serialize());
                }
                s
            }
        }
    }
}

impl From<Vec<Resp>> for Resp {
    fn from(value: Vec<Resp>) -> Self {
        Resp::Array(value)
    }
}

impl From<Vec<&str>> for Resp {
    fn from(values: Vec<&str>) -> Self {
        let resp_vec: Vec<_> = values.iter().map(|s| s.as_bulk_string()).collect();

        Self::Array(resp_vec)
    }
}

impl From<Vec<String>> for Resp {
    fn from(values: Vec<String>) -> Self {
        let resp_vec: Vec<_> = values.into_iter().map(Resp::BulkString).collect();

        Self::Array(resp_vec)
    }
}

pub trait ToResp {
    fn as_simple_string(&self) -> Resp;
    fn as_bulk_string(&self) -> Resp;
}

impl ToResp for str {
    fn as_simple_string(&self) -> Resp {
        Resp::SimpleString(self.to_string())
    }

    fn as_bulk_string(&self) -> Resp {
        Resp::BulkString(self.to_string())
    }
}
