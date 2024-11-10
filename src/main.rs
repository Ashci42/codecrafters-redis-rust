use std::path::PathBuf;

use clap::Parser;
use redis_starter_rust::Config;

#[derive(Parser)]
struct Cli {
    #[arg(long = "dir")]
    rdb_dir: Option<PathBuf>,
    #[arg(long = "dbfilename")]
    rdb_file_name: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let rdb_dir = cli.rdb_dir.as_deref();
    let rdb_file_name = cli.rdb_file_name.as_deref();
    let config = Config::new("127.0.0.1:6379", rdb_dir, rdb_file_name);

    redis_starter_rust::run(&config).await
}
