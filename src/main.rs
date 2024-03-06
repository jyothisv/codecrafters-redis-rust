mod commands;
mod config;
mod resp;
mod store;

pub use commands::Command;
use commands::CommandHandler;
use config::Config;

use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

use std::sync::OnceLock;

pub static CONFIG: OnceLock<Config> = OnceLock::new();

fn main() -> anyhow::Result<()> {
    let config = Config::new()?;

    CONFIG.get_or_init(|| config);

    let address = format!("127.0.0.1:{}", CONFIG.get().unwrap().port);

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

        let command = std::str::from_utf8(&buf)?.parse()?;

        let response = command_handler.handle_command(command)?;

        stream.write_all(response.as_bytes())?;
    }
}
