use crate::Command;
use anyhow::anyhow;

#[derive(Debug)]
pub enum Resp {
    SimpleString(String),
    BulkString(String),
    Int(i64),
    Array(Vec<Resp>),
}

impl Resp {
    fn parse_int(bytes: &str) -> anyhow::Result<(Self, &str)> {
        let idx = bytes
            .find("\r\n")
            .ok_or(anyhow!("Non-terminated simple string"))?;

        Ok((Resp::Int(bytes[..idx].parse::<i64>()?), &bytes[idx + 2..]))
    }

    fn parse_simple_string(bytes: &str) -> anyhow::Result<(Self, &str)> {
        let idx = bytes
            .find("\r\n")
            .ok_or(anyhow!("Non-terminated simple string"))?;

        Ok((
            Resp::SimpleString(bytes[..idx].to_string()),
            &bytes[idx + 2..],
        ))
    }

    fn parse_bulk_string(bytes: &str) -> anyhow::Result<(Self, &str)> {
        let idx = bytes
            .find("\r\n")
            .ok_or(anyhow!("Non-terminated bulk string"))?;

        let len = bytes[..idx].parse::<usize>()?;

        let str_start = idx + 2;
        let str_end = str_start + len;
        let str = &bytes[str_start..str_end];
        if str.len() != len {
            return Err(anyhow!("Malformed bulk string".to_string()));
        }

        let (head, rest) = bytes[str_end..].split_at(2);

        if head != "\r\n" {
            return Err(anyhow!("Malformed bulk string".to_string()));
        }

        Ok((Resp::BulkString(str.to_string()), rest))
    }

    fn parse_array(bytes: &str) -> anyhow::Result<(Self, &str)> {
        let idx = bytes.find("\r\n").ok_or(anyhow!("Non-terminated array"))?;

        let len = bytes[..idx].parse::<usize>()?;

        let mut result = vec![];

        // Strip off the number and the CRLF
        let mut rest = &bytes[idx + 2..];

        for _ in 0..len {
            let res = Self::parse_next(rest)?;
            rest = res.1;
            result.push(res.0);
        }

        Ok((Resp::Array(result), rest))
    }

    fn parse_next(bytes: &str) -> anyhow::Result<(Self, &str)> {
        let (head, rest) = bytes.split_at(1);

        match head {
            "+" => Self::parse_simple_string(rest),
            "$" => Self::parse_bulk_string(rest),
            ":" => Self::parse_int(rest),
            "*" => Self::parse_array(rest),
            _ => Err(anyhow!("Unexpected data type")),
        }
    }

    pub fn parse_command(bytes: &str) -> anyhow::Result<Command> {
        let (head, rest) = bytes.split_at(1);

        if head != "*" {
            return Err(anyhow!("Expected an array"));
        }

        if let (Resp::Array(array), _) = Self::parse_array(rest)? {
            let mut cmd_tokens = array.into_iter().map(|x| x.into_string());

            let cmd_name = cmd_tokens
                .next()
                .ok_or(anyhow!("No command specified"))?
                .to_lowercase();

            let command = match cmd_name.as_str() {
                "ping" => Command::Ping,
                "echo" => {
                    let arg = cmd_tokens.next().ok_or(anyhow!("No argument to echo"))?;
                    Command::Echo(arg)
                }
                "set" => {
                    let key = cmd_tokens.next().ok_or(anyhow!("No key specified"))?;
                    let value = cmd_tokens.next().ok_or(anyhow!("No value specified"))?;

                    let expiry = cmd_tokens
                        .next()
                        .and_then(|px| {
                            if px.to_lowercase() == "px" {
                                cmd_tokens.next()
                            } else {
                                None
                            }
                        })
                        .and_then(|expiry| expiry.as_str().parse::<u64>().ok());

                    Command::Set { key, value, expiry }
                }
                "get" => {
                    let key = cmd_tokens.next().ok_or(anyhow!("No key specified"))?;
                    Command::Get(key)
                }
                _ => unimplemented!(),
            };

            Ok(command)
        } else {
            Err(anyhow!("Expected an array"))
        }
    }

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

#[cfg(test)]
mod tests {
    use super::Resp;

    #[test]
    fn parse_simple_string() -> anyhow::Result<()> {
        let s = "lisp";
        let encoded = format!("+{s}\r\n");
        let decoded = Resp::parse_next(&encoded)?;
        let decoded = decoded.0.to_string();
        assert_eq!(decoded, s);
        Ok(())
    }

    #[test]
    fn parse_empty_bulk_string() -> anyhow::Result<()> {
        let s = "";
        let encoded = format!("${}\r\n{}\r\n", s.len(), s);
        let decoded = Resp::parse_next(&encoded)?;
        let decoded = decoded.0.to_string();
        assert_eq!(decoded, s);
        Ok(())
    }

    #[test]
    fn parse_bulk_string() -> anyhow::Result<()> {
        let s = "This is a bulk string!";
        let encoded = format!("${}\r\n{}\r\n", s.len(), s);
        let decoded = Resp::parse_next(&encoded)?;
        let decoded = decoded.0.to_string();
        assert_eq!(decoded, s);
        Ok(())
    }

    #[test]
    fn parse_array() -> anyhow::Result<()> {
        let s = "*2\r\n$5\r\nhello\r\n$6\r\nworld!\r\n";
        let expected_out = vec!["hello", "world!"];

        let decoded = Resp::parse_next(s)?.0;

        if let Resp::Array(vec) = decoded {
            let decoded: Vec<_> = vec.into_iter().map(|x| x.to_string().to_owned()).collect();
            assert_eq!(decoded, expected_out);
        } else {
            panic!("Expected an array!");
        }
        Ok(())
    }
}
