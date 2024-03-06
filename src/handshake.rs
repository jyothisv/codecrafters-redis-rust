use anyhow::anyhow;

use crate::{resp::Resp, Command, CONFIG};
use std::{
    io::{Read, Write},
    net::TcpStream,
};

pub fn do_handshake_with_master(stream: &mut TcpStream) -> anyhow::Result<()> {
    let mut buf = [0u8; 512];

    let ping: Resp = vec!["ping"].into();

    let _ = stream.write(ping.serialize().as_bytes())?;

    let _ = stream.read(&mut buf)?;

    let port = CONFIG
        .get()
        .ok_or(anyhow!("Unable to access the configuration"))?
        .port;

    let replconf: Command = Command::ReplconfPort(port);

    let _ = stream.write(replconf.serialize().as_bytes())?;

    let _ = stream.read(&mut buf)?;

    let replconf: Command = Command::ReplconfCapa("psync2".to_owned());

    let _ = stream.write(replconf.serialize().as_bytes())?;

    let _ = stream.read(&mut buf)?;

    let psync: Command = Command::Psync {
        replica_id: "?".to_owned(),
        offset: -1,
    };

    let _ = stream.write(psync.serialize().as_bytes())?;

    let _ = stream.read(&mut buf)?;

    Ok(())
}
