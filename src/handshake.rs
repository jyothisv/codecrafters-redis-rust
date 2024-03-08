use anyhow::anyhow;

use crate::{Command, CONFIG};
use std::{
    io::{Read, Write},
    net::TcpStream,
};

pub fn do_handshake_with_master(stream: &mut TcpStream) -> anyhow::Result<()> {
    let mut buf = [0u8; 512];

    let ping: Command = Command::Ping;

    stream.write_all(ping.serialize().as_bytes())?;

    let _ = stream.read(&mut buf)?;

    let port = CONFIG
        .get()
        .ok_or(anyhow!("Unable to access the configuration"))?
        .port;

    let replconf: Command = Command::ReplconfPort(port);

    stream.write_all(replconf.serialize().as_bytes())?;

    let _ = stream.read(&mut buf)?;

    let replconf: Command = Command::ReplconfCapa(vec!["psync2".to_owned()]);

    stream.write_all(replconf.serialize().as_bytes())?;

    let _ = stream.read(&mut buf)?;

    let psync: Command = Command::Psync {
        replica_id: "?".to_owned(),
        offset: -1,
    };

    stream.write_all(psync.serialize().as_bytes())?;

    let _ = stream.read(&mut buf)?;

    Ok(())
}
