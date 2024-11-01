use std::{io::Write, net::TcpStream};

pub fn handle_connection(mut stream: TcpStream) -> anyhow::Result<()> {
    stream.write_all(b"+PONG\r\n")?;

    Ok(())
}
