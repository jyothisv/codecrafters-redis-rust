use crate::resp::Resp;

pub fn handle_command(cmd: Resp) -> anyhow::Result<String> {
    if let Resp::Array(array) = cmd {
        // let cmd_tokens: Vec<_> = array.into_iter().map(|x| x.to_str()).collect();

        let cmd_name = array[0].to_str().to_lowercase();
        let cmd_args = &array[1..];

        return match cmd_name.as_str() {
            "ping" => handle_ping(),
            "echo" => handle_echo(&cmd_args[0]),
            _ => unimplemented!(),
        };
    }
    todo!()
}

fn handle_ping() -> anyhow::Result<String> {
    Ok(Resp::into_simple_string("PONG").serialize())
}

fn handle_echo(arg: &Resp) -> anyhow::Result<String> {
    Ok(arg.serialize())
}
