mod config;
mod padm_client;
mod server;

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

    let config = config::load_config_from_file(&args.config).unwrap();

    let env = env_logger::Env::default()
        .filter_or("MY_LOG_LEVEL", config.log_level())
        .write_style_or("MY_LOG_LEVEL", config.log_level());
    env_logger::init_from_env(env);

    server::run(config).await
}
