use crate::resp::{Resp, ToResp};

pub fn handle_command(cmd: Resp) -> anyhow::Result<String> {
    if let Resp::Array(array) = cmd {
        // let cmd_tokens: Vec<_> = array.into_iter().map(|x| x.to_str()).collect();

        let cmd_name = array[0].to_str().to_lowercase();
        let cmd_args = &array[1..];

        return match cmd_name.as_str() {
            "ping" => handle_ping(cmd_args),
            "echo" => handle_echo(cmd_args),
            _ => unimplemented!(),
        };
    }
    todo!()
}

fn handle_ping(_: &[Resp]) -> anyhow::Result<String> {
    Ok("PONG".as_simple_string().serialize())
}

fn handle_echo(args: &[Resp]) -> anyhow::Result<String> {
    Ok(args[0].serialize())
}
