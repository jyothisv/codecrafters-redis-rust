use anyhow::anyhow;
use std::str::FromStr;

mod command_handler;
mod response;

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
    Info(Option<String>),
    ReplConf(ReplConf),
    Psync {
        replica_id: String,
        offset: i32,
    },
}

pub enum ReplConf {
    ListeningPort(u32),
    Capa(Vec<String>),
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
                "info" => {
                    let role = cmd_tokens.next();
                    Command::Info(role)
                }
                "replconf" => {
                    let subcmd = cmd_tokens
                        .next()
                        .ok_or(anyhow!("No subcommand specified"))?
                        .to_lowercase();

                    let conf = match subcmd.as_str() {
                        "listening-port" => {
                            let port = cmd_tokens
                                .next()
                                .ok_or(anyhow!("No listening port specified"))?
                                .as_str()
                                .parse::<u32>()?;

                            ReplConf::ListeningPort(port)
                        }
                        "capa" => {
                            let capa = cmd_tokens.next().ok_or(anyhow!("Missing capability"))?;
                            let mut capas = vec![capa];

                            while let Some(capa_cmd) = cmd_tokens.next() {
                                let capa_cmd = capa_cmd.to_lowercase();

                                if capa_cmd != "capa" {
                                    return Err(anyhow!("Expected `capa', found {}", capa_cmd));
                                }

                                let capa =
                                    cmd_tokens.next().ok_or(anyhow!("Missing capability"))?;
                                capas.push(capa);
                            }

                            ReplConf::Capa(capas)
                        }
                        _ => todo!(),
                    };

                    Command::ReplConf(conf)
                }
                _ => unimplemented!(),
            };

            Ok(command)
        } else {
            Err(anyhow!("Expected an array"))
        }
    }
}

impl Command {
    pub fn serialize(&self) -> String {
        let mut result: Vec<String> = vec![];

        match self {
            Self::Ping => result.push("Ping".to_owned()),

            Self::ReplConf(ReplConf::ListeningPort(port)) => {
                let port = port.to_string();
                result.extend(["Replconf".to_owned(), "listening-port".to_owned(), port]);
            }

            Self::ReplConf(ReplConf::Capa(capas)) => {
                result.push("Replconf".to_owned());
                let capas: Vec<String> = capas
                    .iter()
                    .flat_map(|capa| ["capa".to_owned(), capa.to_owned()])
                    .collect();

                result.extend(capas);
            }

            Self::Psync { replica_id, offset } => result.extend([
                "Psync".to_owned(),
                replica_id.to_owned(),
                offset.to_string(),
            ]),

            _ => unimplemented!(),
        }

        let resp: Resp = result.into();
        resp.serialize()
    }
}
