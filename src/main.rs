pub mod config;
pub mod padm_client;
pub mod server;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Path to the config file
    #[arg(short, long)]
    config: String,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let config = config::config::load_config_from_file(&args.config)
        .unwrap();

    server::server::run(&config).await
}
