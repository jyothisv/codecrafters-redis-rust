mod commands;
mod config;
mod resp;
mod store;

use commands::CommandHandler;
use config::Config;
use resp::Resp;

use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

fn main() -> anyhow::Result<()> {
    let config = Config::new()?;

    let address = format!("127.0.0.1:{}", config.port);

    let listener = TcpListener::bind(address).unwrap();

    let store = store::Store::default();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let store = store.clone();
                thread::spawn(move || handle_client(stream, store));
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
    Ok(())
}

fn handle_client(mut stream: TcpStream, store: store::Store) -> anyhow::Result<()> {
    let mut buf = [0u8; 512];
    let mut command_handler = CommandHandler::new(store);

    loop {
        let bytes_read = stream.read(&mut buf)?;

        if bytes_read == 0 {
            return Ok(());
        }

        let str = std::str::from_utf8(&buf)?;

        let (parsed, _) = Resp::parse_command(str)?;

        let response = command_handler.handle_command(parsed)?;

        stream.write_all(response.as_bytes())?;
    }
}
