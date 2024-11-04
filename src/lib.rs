mod command;

use bytes::Bytes;
use command::{AsyncCommandReader, AsyncSequenceReader, Command, NextSequenceError, Resp};
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWriteExt};

impl<T> AsyncSequenceReader for tokio::io::Lines<T>
where
    T: AsyncBufRead + Unpin,
{
    async fn next_sequence(&mut self) -> Result<Option<String>, NextSequenceError> {
        self.next_line().await.map_err(|_| NextSequenceError)
    }
}

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
    let mut command_reader = AsyncCommandReader::new(&mut lines);
    while let Some(command) = command_reader.next_command().await? {
        handle_command(&mut write_half, command).await?;
    }

    Ok(())
}

async fn handle_command<W>(writer: &mut W, command: Command) -> anyhow::Result<()>
where
    W: AsyncWriteExt + Unpin,
{
    match command {
        Command::Ping => handle_ping(writer).await?,
        Command::Echo(echo) => handle_echo(writer, echo).await?,
    }

    Ok(())
}

async fn handle_ping<W>(writer: &mut W) -> anyhow::Result<()>
where
    W: AsyncWriteExt + Unpin,
{
    let response = Resp::SimpleString("PONG".into());
    let response: Bytes = response.into();

    writer.write_all(&response).await?;

    Ok(())
}

async fn handle_echo<W>(writer: &mut W, echo: String) -> anyhow::Result<()>
where
    W: AsyncWriteExt + Unpin,
{
    let response = Resp::BulkString(echo);
    let response: Bytes = response.into();

    writer.write_all(&response).await?;

    Ok(())
}
