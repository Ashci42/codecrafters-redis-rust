mod command;
mod config;
mod store;

pub use config::Config;

use std::{sync::Arc, time::Duration};

use bytes::Bytes;
use command::{
    AsyncCommandReader, AsyncSequenceReader, Command, NextSequenceError, Resp, SetArguments,
};
use store::Store;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWriteExt};

type StoreArc = Arc<tokio::sync::Mutex<Store>>;

impl<T> AsyncSequenceReader for tokio::io::Lines<T>
where
    T: AsyncBufRead + Unpin,
{
    async fn next_sequence(&mut self) -> Result<Option<String>, NextSequenceError> {
        self.next_line().await.map_err(|_| NextSequenceError)
    }
}

pub async fn run<'a>(config: &Config<'a>) -> anyhow::Result<()> {
    let mut store = Store::new();
    if let Some(rdb_dir) = config.rdb_dir {
        store.set_rdb_dir(rdb_dir.to_path_buf());
    }
    if let Some(rdb_file_name) = config.rdb_file_name {
        store.set_rdb_file_name(rdb_file_name.to_string());
    }

    let store = Arc::new(tokio::sync::Mutex::new(store));

    spawn_cleanup_thread(store.clone());

    let listener = tokio::net::TcpListener::bind(config.addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let store = store.clone();
        tokio::spawn(async move {
            handle_connection(stream, store).await?;

            Ok::<(), anyhow::Error>(())
        });
    }
}

async fn handle_connection(
    mut stream: tokio::net::TcpStream,
    mut store: StoreArc,
) -> anyhow::Result<()> {
    let (read_half, mut write_half) = stream.split();
    let buf_reader = tokio::io::BufReader::new(read_half);
    let mut lines = buf_reader.lines();
    let mut command_reader = AsyncCommandReader::new(&mut lines);
    while let Some(command) = command_reader.next_command().await? {
        handle_command(&mut write_half, command, &mut store).await?;
    }

    Ok(())
}

async fn handle_command<W>(
    writer: &mut W,
    command: Command,
    store: &mut StoreArc,
) -> anyhow::Result<()>
where
    W: AsyncWriteExt + Unpin,
{
    match command {
        Command::Ping => handle_ping(writer).await?,
        Command::Echo(echo) => handle_echo(writer, echo).await?,
        Command::Set(set_arguments) => handle_set(writer, set_arguments, store).await?,
        Command::Get(key) => handle_get(writer, key, store).await?,
        Command::ConfigGet(key) => handle_config_get(writer, key, store).await?,
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

async fn handle_set<W>(
    writer: &mut W,
    set_arguments: SetArguments,
    store: &mut StoreArc,
) -> anyhow::Result<()>
where
    W: AsyncWriteExt + Unpin,
{
    let mut store = store.lock().await;
    store.set(set_arguments.key, set_arguments.value, set_arguments.px);
    drop(store);

    let response = Resp::SimpleString("OK".into());
    let response: Bytes = response.into();

    writer.write_all(&response).await?;

    Ok(())
}

async fn handle_get<W>(writer: &mut W, key: String, store: &mut StoreArc) -> anyhow::Result<()>
where
    W: AsyncWriteExt + Unpin,
{
    let mut store = store.lock().await;
    let value = store.get(&key).cloned();
    drop(store);

    match value {
        Some(value) => {
            let response = Resp::BulkString(value);
            let response: Bytes = response.into();

            writer.write_all(&response).await?;
        }
        None => {
            let response = Resp::NullBulkString;
            let response: Bytes = response.into();

            writer.write_all(&response).await?;
        }
    };

    Ok(())
}

fn spawn_cleanup_thread(store: StoreArc) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1));

        loop {
            interval.tick().await;
            let mut store = store.lock().await;
            store.clean_expired_keys();
        }
    });
}

async fn handle_config_get<W>(
    writer: &mut W,
    key: String,
    store: &mut StoreArc,
) -> anyhow::Result<()>
where
    W: AsyncWriteExt + Unpin,
{
    let store = store.lock().await;
    let config_value = store
        .get_config(&key)
        .map(|config_value| config_value.to_string());
    drop(store);

    let resp_config_value = match config_value {
        Some(config_value) => Resp::BulkString(config_value),
        None => Resp::NullBulkString,
    };
    let response = Resp::Array(vec![Resp::BulkString(key), resp_config_value]);
    let response: Bytes = response.into();

    writer.write_all(&response).await?;

    Ok(())
}
