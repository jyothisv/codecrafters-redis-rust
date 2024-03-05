use anyhow::anyhow;
use std::env::args;

pub struct Config {
    pub port: u32,
}

impl Config {
    pub fn new() -> anyhow::Result<Self> {
        let mut port = 6379;

        let mut args = args().skip(1);

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--port" => {
                    port = args
                        .next()
                        .ok_or(anyhow!("The port not specified"))?
                        .parse::<u32>()?
                }
                _ => unimplemented!(),
            }
        }

        Ok(Self { port })
    }
}
