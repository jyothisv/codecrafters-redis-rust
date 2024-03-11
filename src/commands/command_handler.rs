use super::response::Response;
use crate::store::Store;
use crate::{Command, CONFIG};
use anyhow::anyhow;

pub const EMPTY_RDB: &[u8] = &[
    0x52, 0x45, 0x44, 0x49, 0x53, 0x30, 0x30, 0x31, 0x31, 0xfa, 0x09, 0x72, 0x65, 0x64, 0x69, 0x73,
    0x2d, 0x76, 0x65, 0x72, 0x05, 0x37, 0x2e, 0x32, 0x2e, 0x30, 0xfa, 0x0a, 0x72, 0x65, 0x64, 0x69,
    0x73, 0x2d, 0x62, 0x69, 0x74, 0x73, 0xc0, 0x40, 0xfa, 0x05, 0x63, 0x74, 0x69, 0x6d, 0x65, 0xc2,
    0x6d, 0x08, 0xbc, 0x65, 0xfa, 0x08, 0x75, 0x73, 0x65, 0x64, 0x2d, 0x6d, 0x65, 0x6d, 0xc2, 0xb0,
    0xc4, 0x10, 0x00, 0xfa, 0x08, 0x61, 0x6f, 0x66, 0x2d, 0x62, 0x61, 0x73, 0x65, 0xc0, 0x00, 0xff,
    0xf0, 0x6e, 0x3b, 0xfe, 0xc0, 0xff, 0x5a, 0xa2,
];

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
            Command::ReplConf(conf) => self.handle_replconf(conf),
            Command::Psync { replica_id, offset } => self.handle_psync(replica_id, offset),
        }?;

        Ok(response.serialize().as_bytes().to_vec())
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

        Ok(item.map_or(Response::Null, Response::BulkString))
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

    fn handle_replconf(&self, _conf: super::ReplConf) -> anyhow::Result<Response> {
        Ok(Response::OK)
    }

    fn handle_psync(&self, _replica_id: String, _offset: i32) -> anyhow::Result<Response> {
        let config = CONFIG.get().ok_or(anyhow!("Unable to get config"))?;
        let repl_id = &config.master_replid;
        let offset = config.master_repl_offset;

        let rdb_file = Response::File(String::from_utf8_lossy(EMPTY_RDB).into());

        let response = vec![
            Response::SimpleString(format!("FULLRESYNC {} {}", repl_id, offset)),
            rdb_file,
        ];

        Ok(Response::Seq(response))
    }
}
