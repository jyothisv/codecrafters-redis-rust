use crate::{
    resp::{Resp, ToResp},
    store::Store,
};

pub struct CommandHandler {
    store: Store,
}

impl CommandHandler {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub fn handle_command(&mut self, cmd: Resp) -> anyhow::Result<String> {
        if let Resp::Array(array) = cmd {
            // let cmd_tokens: Vec<_> = array.into_iter().map(|x| x.to_str()).collect();

            let cmd_name = array[0].to_str().to_lowercase();
            let cmd_args: Vec<_> = array[1..].iter().map(|x| x.to_str()).collect();

            return match cmd_name.as_str() {
                "ping" => self.handle_ping(),
                "echo" => {
                    let arg = &cmd_args[0];
                    self.handle_echo(arg)
                }
                "set" => {
                    let num_opt_args = cmd_args.len();

                    let mut expiry = None;

                    let key = &cmd_args[0];
                    let value = &cmd_args[1];

                    if num_opt_args >= 4 {
                        let cmd_opt_name = cmd_args[2].to_lowercase();

                        if cmd_opt_name == "px" {
                            expiry = cmd_args[3].parse::<u64>().ok();
                        }
                    }
                    self.handle_set(key, value, expiry)
                }

                "get" => {
                    let arg = array[1].to_str();
                    self.handle_get(&arg)
                }
                _ => unimplemented!(),
            };
        }
        todo!()
    }

    fn handle_ping(&self) -> anyhow::Result<String> {
        Ok("PONG".as_simple_string().serialize())
    }

    fn handle_echo(&self, arg: &str) -> anyhow::Result<String> {
        Ok(arg.as_bulk_string().serialize())
    }

    fn handle_set(
        &mut self,
        key: &str,
        value: &str,
        expiry: Option<u64>,
    ) -> anyhow::Result<String> {
        self.store.insert(key, value, expiry)?;

        Ok("OK".as_simple_string().serialize())
    }

    fn handle_get(&self, key: &str) -> anyhow::Result<String> {
        let item = self.store.get(key);

        Ok(item.map_or("$-1\r\n".to_owned(), |x| x.as_bulk_string().serialize()))
    }
}
