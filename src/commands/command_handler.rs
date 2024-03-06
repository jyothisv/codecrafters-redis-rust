use crate::{resp::ToResp, store::Store};
use crate::{Command, CONFIG};
use anyhow::anyhow;

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
            Command::Info(key) => self.handle_info(key.as_deref()),
            _ => todo!(),
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

    fn handle_info(&self, _key: Option<&str>) -> anyhow::Result<String> {
        let config = CONFIG.get().ok_or(anyhow!("Unable to get config"))?;
        let role = if config.master.is_none() {
            "master"
        } else {
            "slave"
        };

        let master_replid = &config.master_replid;
        let master_repl_offset = config.master_repl_offset;

        let result = format!(
            "# Replication\nrole:{}\nmaster_replid:{}\nmaster_repl_offset:{}",
            role, master_replid, master_repl_offset
        )
        .as_bulk_string()
        .serialize();
        Ok(result)
    }
}
