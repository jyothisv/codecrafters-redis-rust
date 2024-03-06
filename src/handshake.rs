use anyhow::anyhow;

use crate::{resp::Resp, Command, CONFIG};
use std::{io::Write, net::TcpStream};

pub fn do_handshake_with_master(stream: &mut TcpStream) -> anyhow::Result<()> {
    let ping: Resp = vec!["ping"].into();

    let _ = stream.write(ping.serialize().as_bytes())?;

    let master = CONFIG
        .get()
        .ok_or(anyhow!("Unable to access the configuration"))?
        .master
        .as_ref()
        .ok_or(anyhow!("No master in the configuration"))?;

    let replconf: Command = Command::ReplconfPort(master.port);

    let _ = stream.write(replconf.serialize().as_bytes())?;

    let replconf: Command = Command::ReplconfCapa("psync2".to_owned());

    let _ = stream.write(replconf.serialize().as_bytes())?;

    Ok(())
}
