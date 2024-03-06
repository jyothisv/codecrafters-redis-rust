use crate::resp::{Resp, ToResp};
use std::{io::Write, net::TcpStream};

pub fn do_handshake_with_master(stream: &mut TcpStream) -> anyhow::Result<()> {
    let ping = "ping".as_bulk_string();
    let ping = Resp::Array(vec![ping]);

    let _ = stream.write(ping.serialize().as_bytes())?;

    Ok(())
}
