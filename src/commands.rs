use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::resp::{Resp, ToResp};

pub struct CommandHandler {
    store: Arc<Mutex<HashMap<String, String>>>,
}

impl CommandHandler {
    pub fn new(store: Arc<Mutex<HashMap<String, String>>>) -> Self {
        Self { store }
    }

    pub fn handle_command(&mut self, cmd: Resp) -> anyhow::Result<String> {
        if let Resp::Array(array) = cmd {
            // let cmd_tokens: Vec<_> = array.into_iter().map(|x| x.to_str()).collect();

            let cmd_name = array[0].to_str().to_lowercase();
            let cmd_args: Vec<_> = array[1..].iter().map(|x| x.to_str()).collect();

            return match cmd_name.as_str() {
                "ping" => self.handle_ping(),
                "echo" => self.handle_echo(&cmd_args[0]),
                "set" => self.handle_set(&cmd_args[0], &cmd_args[1]),
                "get" => self.handle_get(&cmd_args[0]),
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

    fn handle_set(&mut self, key: &str, val: &str) -> anyhow::Result<String> {
        let mut m = self.store.lock().unwrap();
        m.insert(key.to_owned(), val.to_owned());

        println!("{:?}", m);

        Ok("OK".as_simple_string().serialize())
    }

    fn handle_get(&self, key: &str) -> anyhow::Result<String> {
        let m = self.store.lock().unwrap();

        if let Some(x) = m.get(key) {
            return Ok(x.as_bulk_string().serialize());
        }

        Ok("$-1\r\n".to_owned())
    }
}
