use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

pub async fn run(addr: &str) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            handle_connection(stream).await?;

            Ok::<(), anyhow::Error>(())
        });
    }
}

async fn handle_connection(mut stream: tokio::net::TcpStream) -> anyhow::Result<()> {
    let (read_half, mut write_half) = stream.split();
    let buf_reader = tokio::io::BufReader::new(read_half);
    let mut lines = buf_reader.lines();
    while let Some(command) = lines.next_line().await? {
        handle_command(&mut write_half, &command).await?;
    }

    Ok(())
}

async fn handle_command<W>(writer: &mut W, command: &str) -> anyhow::Result<()>
where
    W: AsyncWriteExt + Unpin,
{
    if command == "PING" {
        writer.write_all(b"+PONG\r\n").await?;
    }

    Ok(())
}
