use anyhow::anyhow;
use std::env::args;

pub struct HostAddr {
    pub host: String,
    pub port: u32,
}

pub struct Config {
    pub port: u32,
    pub master: Option<HostAddr>,
}

impl Config {
    pub fn new() -> anyhow::Result<Self> {
        let mut port = 6379;
        let mut master = None;

        let mut args = args().skip(1);

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--port" => {
                    port = args
                        .next()
                        .ok_or(anyhow!("The port not specified"))?
                        .parse::<u32>()?
                }
                "--replicaof" => {
                    let host = args
                        .next()
                        .ok_or(anyhow!("The master host not specified"))?;

                    let port = args
                        .next()
                        .ok_or(anyhow!("The port not specified"))?
                        .parse::<u32>()?;

                    master = Some(HostAddr { host, port })
                }
                _ => unimplemented!(),
            }
        }

        Ok(Self { port, master })
    }
}
