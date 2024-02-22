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
        eprintln!("In parse_next, input = {:?}", bytes);

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
    pub fn to_str(&self) -> &str {
        match self {
            Resp::SimpleString(s) => s,
            Resp::BulkString(s) => s,
            Resp::Array(_) => "",
        }
    }
}
