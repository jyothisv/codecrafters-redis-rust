use anyhow::anyhow;

#[derive(Debug)]
pub enum Resp {
    SimpleString(String),
    BulkString(String),
    Array(Vec<Resp>),
}

impl Resp {
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
            "*" => Self::parse_array(rest),
            _ => Err(anyhow!("Unexpected data type")),
        }
    }

    pub fn parse_command(bytes: &str) -> anyhow::Result<(Self, &str)> {
        let (head, rest) = bytes.split_at(1);

        if head != "*" {
            return Err(anyhow!("Expected an array"));
        }

        Self::parse_array(rest)
    }
}

impl Resp {
    pub fn to_str(&self) -> String {
        match self {
            Resp::SimpleString(s) => s.to_owned(),
            Resp::BulkString(s) => s.to_owned(),
            Resp::Array(_) => "".to_owned(),
        }
    }

    pub fn serialize(&self) -> String {
        match self {
            Resp::SimpleString(s) => format!("+{}\r\n", s),
            Resp::BulkString(s) => {
                let len = s.len();
                format!("${}\r\n{}\r\n", len, s)
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
        let decoded = decoded.0.to_str();
        assert_eq!(decoded, s);
        Ok(())
    }

    #[test]
    fn parse_empty_bulk_string() -> anyhow::Result<()> {
        let s = "";
        let encoded = format!("${}\r\n{}\r\n", s.len(), s);
        let decoded = Resp::parse_next(&encoded)?;
        let decoded = decoded.0.to_str();
        assert_eq!(decoded, s);
        Ok(())
    }

    #[test]
    fn parse_bulk_string() -> anyhow::Result<()> {
        let s = "This is a bulk string!";
        let encoded = format!("${}\r\n{}\r\n", s.len(), s);
        let decoded = Resp::parse_next(&encoded)?;
        let decoded = decoded.0.to_str();
        assert_eq!(decoded, s);
        Ok(())
    }

    #[test]
    fn parse_array() -> anyhow::Result<()> {
        let s = "*2\r\n$5\r\nhello\r\n$6\r\nworld!\r\n";
        let expected_out = vec!["hello", "world!"];

        let decoded = Resp::parse_next(s)?.0;

        if let Resp::Array(vec) = decoded {
            let decoded: Vec<_> = vec.into_iter().map(|x| x.to_str().to_owned()).collect();
            assert_eq!(decoded, expected_out);
        } else {
            panic!("Expected an array!");
        }
        Ok(())
    }
}
