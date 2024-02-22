use crate::resp::Resp;

pub fn handle_command(cmd: Resp) -> anyhow::Result<String> {
    if let Resp::Array(array) = cmd {
        let cmd_name = array[0].to_str().to_lowercase();
        let cmd_args: Vec<_> = array[1..].iter().map(|x| x.to_str()).collect();

        return match cmd_name.as_str() {
            "ping" => handle_ping(cmd_args),
            "echo" => handle_echo(cmd_args),
            _ => unimplemented!(),
        };
    }
    todo!()
}

fn handle_ping(_: Vec<&str>) -> anyhow::Result<String> {
    Ok("PONG".to_string())
}

fn handle_echo(args: Vec<&str>) -> anyhow::Result<String> {
    todo!()
}
