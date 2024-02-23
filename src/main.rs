mod commands;
mod resp;

use resp::Resp;

use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || handle_client(stream));
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
    Ok(())
}

fn handle_client(mut stream: TcpStream) -> anyhow::Result<()> {
    let mut buf = [0u8; 512];

    loop {
        let bytes_read = stream.read(&mut buf)?;

        if bytes_read == 0 {
            return Ok(());
        }

        let str = std::str::from_utf8(&buf)?;

        let (parsed, _) = Resp::parse_command(str)?;

        let response = commands::handle_command(parsed)?;

        stream.write_all(response.as_bytes())?;
    }
}
