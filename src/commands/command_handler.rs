use super::response::Response;
use crate::store::Store;
use crate::{Command, CONFIG};
use anyhow::anyhow;

pub struct CommandHandler {
    store: Store,
}

impl CommandHandler {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub fn handle_command(&mut self, cmd: Command) -> anyhow::Result<Vec<u8>> {
        let response = match cmd {
            Command::Ping => self.handle_ping(),
            Command::Echo(arg) => self.handle_echo(&arg),
            Command::Set { key, value, expiry } => self.handle_set(&key, &value, expiry),
            Command::Get(key) => self.handle_get(&key),
            Command::Info(key) => self.handle_info(key.as_deref()),
            Command::ReplconfPort(port) => self.handle_replconf_port(port),
            Command::ReplconfCapa(capabilities) => self.handle_replconf_capa(capabilities),
            _ => todo!(),
        }?;

        Ok(response.serialize())
    }

    fn handle_ping(&self) -> anyhow::Result<Response> {
        Ok(Response::Pong)
    }

    fn handle_echo(&self, arg: &str) -> anyhow::Result<Response> {
        Ok(Response::BulkString(arg.to_owned()))
    }

    fn handle_set(
        &mut self,
        key: &str,
        value: &str,
        expiry: Option<u64>,
    ) -> anyhow::Result<Response> {
        self.store.insert(key, value, expiry)?;

        Ok(Response::OK)
    }

    fn handle_get(&self, key: &str) -> anyhow::Result<Response> {
        let item = self.store.get(key);

        Ok(item.map_or(Response::Null, |x| Response::BulkString(x)))
    }

    fn handle_info(&self, _key: Option<&str>) -> anyhow::Result<Response> {
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
        );

        Ok(Response::BulkString(result))
    }

    fn handle_replconf_port(&self, port: u32) -> anyhow::Result<Response> {
        Ok(Response::OK)
    }

    fn handle_replconf_capa(&self, capabilities: Vec<String>) -> anyhow::Result<Response> {
        Ok(Response::OK)
    }
}
