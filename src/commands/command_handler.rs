use crate::Command;
use crate::{resp::ToResp, store::Store};

pub struct CommandHandler {
    store: Store,
}

impl CommandHandler {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub fn handle_command(&mut self, cmd: Command) -> anyhow::Result<String> {
        match cmd {
            Command::Ping => self.handle_ping(),
            Command::Echo(arg) => self.handle_echo(&arg),
            Command::Set { key, value, expiry } => self.handle_set(&key, &value, expiry),
            Command::Get(key) => self.handle_get(&key),
        }
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
