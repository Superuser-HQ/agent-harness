use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "superagents", version, about = "Multi-agent harness")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Start the agent runtime
    Start {
        /// Config file path
        #[arg(short, long, default_value = "config.toml")]
        config: String,
    },
    /// Show runtime health
    Health,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Command::Start { config } => {
            tracing::info!("Starting superagents runtime with config: {}", config);
            // TODO: load config, init memory, start cortex, start session tree
            todo!("runtime not yet implemented")
        }
        Command::Health => {
            println!("superagents: ok");
            Ok(())
        }
    }
}
