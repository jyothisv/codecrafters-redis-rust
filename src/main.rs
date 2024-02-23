mod commands;
mod resp;

use commands::CommandHandler;
use resp::Resp;

use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    let store: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let store = Arc::clone(&store);
                thread::spawn(move || handle_client(stream, store));
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
    Ok(())
}

fn handle_client(
    mut stream: TcpStream,
    store: Arc<Mutex<HashMap<String, String>>>,
) -> anyhow::Result<()> {
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
