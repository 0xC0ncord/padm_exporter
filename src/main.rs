use anyhow::Result;
use clap::Parser;

mod client;
mod config;
mod metrics;
mod server;
mod target;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Path to the config file
    #[arg(short, long)]
    config: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let config = config::load_from_file(&args.config)?;

    unsafe {
        std::env::set_var(
            "RUST_LOG",
            std::env::var("RUST_LOG").unwrap_or(config.log_level().to_string()),
        );
    }
    env_logger::init();

    server::run(config).await
}
