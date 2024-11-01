use redis_starter_rust::handle_connection;

fn main() -> anyhow::Result<()> {
    let listener = std::net::TcpListener::bind("127.0.0.1:6379")?;

    for stream in listener.incoming() {
        let stream = stream?;
        handle_connection(stream)?;
    }

    Ok(())
}
