#[tokio::main]
async fn main() -> anyhow::Result<()> {
    redis_starter_rust::run("127.0.0.1:6379").await
}
