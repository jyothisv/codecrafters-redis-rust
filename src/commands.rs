use anyhow::anyhow;
use std::str::FromStr;

mod command_handler;

pub use command_handler::CommandHandler;

use crate::resp::{Parser, Resp};

pub enum Command {
    Ping,
    Echo(String),
    Set {
        key: String,
        value: String,
        expiry: Option<u64>,
    },
    Get(String),
}

impl FromStr for Command {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parser = Parser::new(s);

        if let Resp::Array(array) = parser.parse()? {
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
}
