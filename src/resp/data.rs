#[derive(Debug)]
pub enum Resp {
    SimpleString(String),
    BulkString(String),
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

    pub fn serialize(&self) -> String {
        match self {
            Resp::SimpleString(s) => format!("+{}\r\n", s),

            Resp::BulkString(s) => {
                let len = s.len();
                format!("${}\r\n{}\r\n", len, s)
            }

            Resp::Int(n) => {
                format!(":{}\r\n", n)
            }

            Resp::Array(vec) => {
                let mut s = format!("*{}\r\n", vec.len());
                for resp in vec {
                    s.push_str(&resp.serialize());
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

// #[cfg(test)]
// mod tests {
//     use super::Resp;

//     #[test]
//     fn parse_simple_string() -> anyhow::Result<()> {
//         let s = "lisp";
//         let encoded = format!("+{s}\r\n");
//         let decoded = Resp::parse_next(&encoded)?;
//         let decoded = decoded.0.into_string();
//         assert_eq!(decoded, s);
//         Ok(())
//     }

//     #[test]
//     fn parse_empty_bulk_string() -> anyhow::Result<()> {
//         let s = "";
//         let encoded = format!("${}\r\n{}\r\n", s.len(), s);
//         let decoded = Resp::parse_next(&encoded)?;
//         let decoded = decoded.0.into_string();
//         assert_eq!(decoded, s);
//         Ok(())
//     }

//     #[test]
//     fn parse_bulk_string() -> anyhow::Result<()> {
//         let s = "This is a bulk string!";
//         let encoded = format!("${}\r\n{}\r\n", s.len(), s);
//         let decoded = Resp::parse_next(&encoded)?;
//         let decoded = decoded.0.into_string();
//         assert_eq!(decoded, s);
//         Ok(())
//     }

//     #[test]
//     fn parse_array() -> anyhow::Result<()> {
//         let s = "*2\r\n$5\r\nhello\r\n$6\r\nworld!\r\n";
//         let expected_out = vec!["hello", "world!"];

//         let decoded = Resp::parse_next(s)?.0;

//         if let Resp::Array(vec) = decoded {
//             let decoded: Vec<_> = vec.into_iter().map(|x| x.into_string()).collect();
//             assert_eq!(decoded, expected_out);
//         } else {
//             panic!("Expected an array!");
//         }
//         Ok(())
//     }
// }
