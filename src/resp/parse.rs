use anyhow::anyhow;

use super::Resp;

pub struct Parser<'a> {
    input: &'a [u8],
    idx: usize,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input: input.as_bytes(),
            idx: 0,
        }
    }

    pub fn parse(&mut self) -> anyhow::Result<Resp> {
        let first_char = self.input[self.idx] as char;

        match first_char {
            '+' => self.parse_simple_string(),
            '$' => self.parse_bulk_string(),
            ':' => self.parse_int(),
            '*' => self.parse_array(),
            _ => Err(anyhow!("Unexpected character")),
        }
    }

    fn swallow_char(&mut self, char: char, idx: usize) -> anyhow::Result<usize> {
        let next_char = self.input[idx] as char;

        if next_char == char {
            Ok(idx + 1)
        } else {
            Err(anyhow!("Expected Some({}), got {:?}", char, next_char))
        }
    }

    fn swallow_crlf(&mut self, idx: usize) -> anyhow::Result<usize> {
        let idx = self.swallow_char('\r', idx)?;
        self.swallow_char('\n', idx)
    }

    fn peek_til_crlf(&self) -> anyhow::Result<usize> {
        let char_idx = self.input[self.idx..]
            .windows(2)
            .position(|x| String::from_utf8_lossy(x) == "\r\n")
            .ok_or(anyhow!("Runaway input"))?;

        // We need to add the initial offset
        Ok(char_idx + self.idx)
    }

    fn parse_bulk_string(&mut self) -> anyhow::Result<Resp> {
        let idx = self.swallow_char('$', self.idx)?;

        let last_idx = self.peek_til_crlf()?;

        let len = self.input[idx..last_idx]
            .iter()
            .map(|&x| x as char)
            .collect::<String>()
            .parse::<usize>()?;

        let str_start = last_idx + 2;
        let str_end = str_start + len;

        let str = self.input[str_start..str_end]
            .iter()
            .map(|&x| x as char)
            .collect::<String>();

        if str.len() != len {
            return Err(anyhow!("Malformed bulk string".to_string()));
        }

        let idx = self.swallow_crlf(str_end)?;
        self.idx = idx;

        Ok(Resp::BulkString(str))
    }

    fn parse_simple_string(&mut self) -> anyhow::Result<Resp> {
        let str_start = self.swallow_char('+', self.idx)?;

        let str_end = self.peek_til_crlf()?;

        let str = self.input[str_start..str_end]
            .iter()
            .map(|&x| x as char)
            .collect::<String>();

        self.idx = str_end + 2;
        Ok(Resp::SimpleString(str))
    }

    fn parse_int(&mut self) -> anyhow::Result<Resp> {
        let int_start = self.swallow_char(':', self.idx)?;

        let int_end = self.peek_til_crlf()?;

        let int = self.input[int_start..int_end]
            .iter()
            .map(|&x| x as char)
            .collect::<String>()
            .parse::<i64>()
            .map_err(|x| anyhow!("ParseIntError: {x}"))?;

        self.idx = int_end + 2;

        Ok(Resp::Int(int))
    }

    fn parse_array(&mut self) -> anyhow::Result<Resp> {
        let idx = self.swallow_char('*', self.idx)?;

        let crlf_idx = self.peek_til_crlf()?;

        let len = self.input[idx..crlf_idx]
            .iter()
            .map(|&x| x as char)
            .collect::<String>()
            .parse::<usize>()?;

        let mut result = vec![];

        // For rolling back in case of any error
        let last_idx = self.idx;
        let mut error = None;

        self.idx = crlf_idx + 2;
        for _ in 0..len {
            match self.parse() {
                Ok(v) => result.push(v),
                err => {
                    error = Some(err);
                    break;
                }
            }
        }

        if error.is_some() {
            self.idx = last_idx;
            return error.unwrap();
        }

        Ok(Resp::Array(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_string() -> anyhow::Result<()> {
        let s = "lisp";
        let encoded = format!("+{s}\r\n");
        let mut parser = Parser::new(&encoded);
        let decoded = parser.parse()?.into_string();
        assert_eq!(decoded, s);
        Ok(())
    }

    #[test]
    fn parse_empty_bulk_string() -> anyhow::Result<()> {
        let s = "";
        let encoded = format!("${}\r\n{}\r\n", s.len(), s);
        let mut parser = Parser::new(&encoded);
        let decoded = parser.parse()?.into_string();
        assert_eq!(decoded, s);
        Ok(())
    }

    #[test]
    fn parse_bulk_string() -> anyhow::Result<()> {
        let s = "This is a bulk string!";
        let encoded = format!("${}\r\n{}\r\n", s.len(), s);
        let mut parser = Parser::new(&encoded);
        let decoded = parser.parse()?.into_string();
        assert_eq!(decoded, s);
        Ok(())
    }

    #[test]
    fn parse_array() -> anyhow::Result<()> {
        let s = "*2\r\n$5\r\nhello\r\n$6\r\nworld!\r\n";
        let expected_out = vec!["hello", "world!"];

        let mut parser = Parser::new(s);

        let decoded = parser.parse()?;

        if let Resp::Array(vec) = decoded {
            let decoded: Vec<_> = vec.into_iter().map(|x| x.into_string()).collect();
            assert_eq!(decoded, expected_out);
        } else {
            panic!("Expected an array!");
        }
        Ok(())
    }
}
