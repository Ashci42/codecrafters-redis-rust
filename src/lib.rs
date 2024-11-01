use std::io::{BufRead, Write};

pub fn handle_connection(mut stream: std::net::TcpStream) -> anyhow::Result<()> {
    let read_stream = stream.try_clone()?;
    let buf_reader = std::io::BufReader::new(&read_stream);
    for command in buf_reader.lines() {
        if command.is_ok() && command.unwrap() == "PING" {
            stream.write_all(b"+PONG\r\n")?;
        }
    }

    Ok(())
}
